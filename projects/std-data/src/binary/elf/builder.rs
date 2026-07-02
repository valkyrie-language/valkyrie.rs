use std::collections::BTreeMap;

use miette::{Result, miette};

use crate::binary::x86_64::{X64Encoder, X64Instruction, apply_fixups};

use super::{NativeElfImage, NativeElfWriter, SharedElfImage, SharedElfWriter, SharedObjectExport, layout::ElfSectionLayout};

/// 原生 ELF 镜像构建器：编码 `.text`、布局 `.rodata` 并写出可执行文件。
#[derive(Debug, Default)]
pub struct NativeElfImageBuilder {
    encoder: X64Encoder,
    rodata: Vec<u8>,
    rodata_labels: BTreeMap<String, u32>,
    text_ptr_fixups: Vec<(u32, String)>,
}

impl NativeElfImageBuilder {
    /// 创建构建器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加指令。
    pub fn push(&mut self, instruction: X64Instruction) {
        self.encoder.push(instruction);
    }

    /// 向 `.rodata` 追加数据并返回标签偏移。
    pub fn add_rodata(&mut self, label: &str, data: &[u8]) -> u32 {
        let offset = u32::try_from(self.rodata.len()).unwrap_or(0);
        self.rodata_labels.insert(label.to_string(), offset);
        self.rodata.extend_from_slice(data);
        offset
    }

    /// 在 `.rodata` 追加指向 `.text` 标签的 8 字节指针槽。
    pub fn add_rodata_text_ptr(&mut self, slot_label: &str, text_label: &str) -> u32 {
        let offset = u32::try_from(self.rodata.len()).unwrap_or(0);
        self.rodata_labels.insert(slot_label.to_string(), offset);
        self.rodata.extend_from_slice(&[0u8; 8]);
        self.text_ptr_fixups.push((offset, text_label.to_string()));
        offset
    }

    /// 在 `.rodata` 追加连续 qword 槽位，各指向 `.text` 标签；`base_label` 标记首槽。
    pub fn add_rodata_text_ptr_sequence(&mut self, base_label: &str, text_labels: &[String]) -> u32 {
        let offset = u32::try_from(self.rodata.len()).unwrap_or(0);
        self.rodata_labels.insert(base_label.to_string(), offset);
        for text_label in text_labels {
            let slot_offset = u32::try_from(self.rodata.len()).unwrap_or(0);
            self.rodata.extend_from_slice(&[0u8; 8]);
            self.text_ptr_fixups.push((slot_offset, text_label.clone()));
        }
        offset
    }

    /// 完成镜像并写出可执行文件字节。
    pub fn build_executable(self, entry_label: &str) -> Result<Vec<u8>> {
        let mut encoded = self.encoder.finish();
        let entry_point = *encoded.labels.get(entry_label).ok_or_else(|| miette!("缺少入口标签 `{entry_label}`"))?;
        let text_len = u32::try_from(encoded.text.len()).map_err(|_| miette!("`.text` 过大"))?;
        let rodata_len = u32::try_from(self.rodata.len()).map_err(|_| miette!("`.rodata` 过大"))?;
        let layout = ElfSectionLayout::compute(text_len, rodata_len);

        let rodata_labels = self.rodata_labels;
        let text_ptr_fixups = self.text_ptr_fixups;
        apply_fixups(&mut encoded, layout.text_vaddr, &rodata_labels, layout.rodata_vaddr, &[])?;
        let mut rodata = self.rodata;
        for (offset, text_label) in &text_ptr_fixups {
            let text_offset = encoded.labels.get(text_label).ok_or_else(|| miette!("缺少 `.text` 标签 `{text_label}`"))?;
            let value = u64::from(layout.text_vaddr + text_offset);
            let start = *offset as usize;
            rodata[start..start + 8].copy_from_slice(&value.to_le_bytes());
        }

        let image = NativeElfImage { text: encoded.text, rodata, entry_point };
        NativeElfWriter::write_executable(&image)
    }

    /// 完成镜像并写出 AArch64 `ET_DYN` 共享库（`.so`）。
    pub fn build_shared_object(self, exports: &[SharedObjectExport]) -> Result<Vec<u8>> {
        let text_ptr_fixups = self.text_ptr_fixups;
        let encoded = self.encoder.finish();
        let mut rodata = self.rodata;
        let text_labels = encoded.labels;
        let layout = ElfSectionLayout::compute(
            u32::try_from(encoded.text.len()).map_err(|_| miette!("`.text` 过大"))?,
            u32::try_from(rodata.len()).map_err(|_| miette!("`.rodata` 过大"))?,
        );
        for (offset, text_label) in &text_ptr_fixups {
            let text_offset = text_labels.get(text_label).ok_or_else(|| miette!("缺少 `.text` 标签 `{text_label}`"))?;
            let value = u64::from(layout.text_vaddr + text_offset);
            let start = *offset as usize;
            rodata[start..start + 8].copy_from_slice(&value.to_le_bytes());
        }
        let image = SharedElfImage { text: encoded.text, rodata, bss: Vec::new() };
        SharedElfWriter::write_aarch64(&image, exports)
    }

    fn apply_rodata_text_ptr_fixups(&self, rodata: &mut Vec<u8>, text_vaddr: u32, text_labels: &BTreeMap<String, u32>) -> Result<()> {
        for (offset, text_label) in &self.text_ptr_fixups {
            let text_offset = text_labels.get(text_label).ok_or_else(|| miette!("缺少 `.text` 标签 `{text_label}`"))?;
            let value = u64::from(text_vaddr + text_offset);
            let start = *offset as usize;
            rodata[start..start + 8].copy_from_slice(&value.to_le_bytes());
        }
        Ok(())
    }
}

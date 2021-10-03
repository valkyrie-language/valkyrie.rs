use std::collections::{BTreeMap, BTreeSet};

use miette::{Result, miette};

use crate::binary::x86_64::{X64Encoder, X64Instruction, apply_fixups};

use super::{NativeDllImport, NativePeImage, NativePeWriter, layout::NativeSectionLayout};
use crate::binary::pe::writer::IMAGE_BASE_X64;

/// 原生镜像构建器：编码 `.text`、布局 `.rdata/.idata` 并写出 `PE`。
#[derive(Debug, Default)]
pub struct NativeImageBuilder {
    encoder: X64Encoder,
    rdata: Vec<u8>,
    rdata_labels: BTreeMap<String, u32>,
    text_ptr_fixups: Vec<(u32, String)>,
    import_order: Vec<(String, String)>,
    import_slots: BTreeMap<(String, String), usize>,
}

impl NativeImageBuilder {
    /// 创建构建器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加指令。
    pub fn push(&mut self, instruction: X64Instruction) {
        self.encoder.push(instruction);
    }

    /// 注册导入并返回 IAT 槽索引。
    pub fn import(&mut self, dll: &str, function: &str) -> usize {
        let key = (dll.to_string(), function.to_string());
        if let Some(slot) = self.import_slots.get(&key) {
            return *slot;
        }
        let slot = self.import_order.len();
        self.import_order.push(key.clone());
        self.import_slots.insert(key, slot);
        slot
    }

    /// 向 `.rdata` 追加数据并返回标签偏移。
    pub fn add_rdata(&mut self, label: &str, data: &[u8]) -> u32 {
        let offset = u32::try_from(self.rdata.len()).unwrap_or(0);
        self.rdata_labels.insert(label.to_string(), offset);
        self.rdata.extend_from_slice(data);
        self.rdata.push(0);
        offset
    }

    /// 在 `.rdata` 追加指向 `.text` 标签的 8 字节指针槽。
    pub fn add_rdata_text_ptr(&mut self, slot_label: &str, text_label: &str) -> u32 {
        let offset = u32::try_from(self.rdata.len()).unwrap_or(0);
        self.rdata_labels.insert(slot_label.to_string(), offset);
        self.rdata.extend_from_slice(&[0u8; 8]);
        self.text_ptr_fixups.push((offset, text_label.to_string()));
        offset
    }

    /// 在 `.rdata` 追加连续 qword 槽位，各指向 `.text` 标签；`base_label` 标记首槽。
    pub fn add_rdata_text_ptr_sequence(&mut self, base_label: &str, text_labels: &[String]) -> u32 {
        let offset = u32::try_from(self.rdata.len()).unwrap_or(0);
        self.rdata_labels.insert(base_label.to_string(), offset);
        for text_label in text_labels {
            let slot_offset = u32::try_from(self.rdata.len()).unwrap_or(0);
            self.rdata.extend_from_slice(&[0u8; 8]);
            self.text_ptr_fixups.push((slot_offset, text_label.clone()));
        }
        offset
    }

    /// 完成镜像并写出可执行文件字节。
    pub fn build_executable(self, entry_label: &str) -> Result<Vec<u8>> {
        let mut encoded = self.encoder.finish();
        let entry_point = *encoded.labels.get(entry_label).ok_or_else(|| miette!("缺少入口标签 `{entry_label}`"))?;
        let imports = group_imports(&self.import_order);
        let idata = build_idata_section(&imports)?;

        let text_len = u32::try_from(encoded.text.len()).map_err(|_| miette!("`.text` 过大"))?;
        let rdata_len = u32::try_from(self.rdata.len()).map_err(|_| miette!("`.rdata` 过大"))?;
        let idata_len = u32::try_from(idata.bytes.len()).map_err(|_| miette!("`.idata` 过大"))?;
        let layout = NativeSectionLayout::compute(text_len, rdata_len, idata_len);

        let rdata_rva = layout.rdata.map(|section| section.rva).unwrap_or(0);
        let idata_rva = layout.idata.map(|section| section.rva).unwrap_or(0);

        let mut iat_rvas = Vec::with_capacity(self.import_order.len());
        for key in &self.import_order {
            let slot_offset = *idata.slot_offsets.get(key).ok_or_else(|| miette!("缺少导入槽 `{}!{}`", key.0, key.1))?;
            iat_rvas.push(idata_rva + slot_offset);
        }

        apply_fixups(&mut encoded, layout.text.rva, &self.rdata_labels, rdata_rva, &iat_rvas)?;
        let mut rdata = self.rdata;
        for (offset, text_label) in &self.text_ptr_fixups {
            let text_offset = encoded.labels.get(text_label).ok_or_else(|| miette!("缺少 `.text` 标签 `{text_label}`"))?;
            let value = IMAGE_BASE_X64 + u64::from(layout.text.rva + text_offset);
            let start = *offset as usize;
            rdata[start..start + 8].copy_from_slice(&value.to_le_bytes());
        }

        let image = NativePeImage { text: encoded.text, rdata, idata: idata.bytes, imports, entry_point };
        NativePeWriter::write_executable(&image)
    }
}

struct IdataSection {
    bytes: Vec<u8>,
    slot_offsets: BTreeMap<(String, String), u32>,
}

fn build_idata_section(imports: &[NativeDllImport]) -> Result<IdataSection> {
    if imports.is_empty() {
        return Ok(IdataSection { bytes: Vec::new(), slot_offsets: BTreeMap::new() });
    }

    let mut bytes = Vec::new();
    let descriptor_bytes = (imports.len() + 1) * 20;
    bytes.resize(descriptor_bytes, 0);

    let mut dll_name_offsets = Vec::new();
    let mut hint_name_offsets = Vec::new();
    for import in imports {
        dll_name_offsets.push(u32::try_from(bytes.len()).map_err(|_| miette!("`.idata` 过大"))?);
        let dll = normalize_dll_name(&import.dll);
        bytes.extend_from_slice(dll.as_bytes());
        bytes.push(0);
        align2(&mut bytes);
        for function in &import.functions {
            hint_name_offsets.push(u32::try_from(bytes.len()).map_err(|_| miette!("`.idata` 过大"))?);
            bytes.extend_from_slice(&0u16.to_le_bytes());
            bytes.extend_from_slice(function.as_bytes());
            bytes.push(0);
            align2(&mut bytes);
        }
    }

    let mut ilt_offsets = Vec::new();
    let mut hint_cursor = 0usize;
    for import in imports {
        ilt_offsets.push(u32::try_from(bytes.len()).map_err(|_| miette!("`.idata` 过大"))?);
        for _ in &import.functions {
            let hint = hint_name_offsets[hint_cursor];
            bytes.extend_from_slice(&(hint as u64).to_le_bytes());
            hint_cursor += 1;
        }
        bytes.extend_from_slice(&0u64.to_le_bytes());
    }

    let mut iat_offsets = Vec::new();
    let mut slot_offsets = BTreeMap::new();
    hint_cursor = 0;
    for import in imports {
        iat_offsets.push(u32::try_from(bytes.len()).map_err(|_| miette!("`.idata` 过大"))?);
        for function in &import.functions {
            let hint = hint_name_offsets[hint_cursor];
            let offset = u32::try_from(bytes.len()).map_err(|_| miette!("`.idata` 过大"))?;
            slot_offsets.insert((import.dll.clone(), function.clone()), offset);
            bytes.extend_from_slice(&(hint as u64).to_le_bytes());
            hint_cursor += 1;
        }
        bytes.extend_from_slice(&0u64.to_le_bytes());
    }

    for (dll_index, _) in imports.iter().enumerate() {
        write_descriptor(&mut bytes, dll_index * 20, ilt_offsets[dll_index], dll_name_offsets[dll_index], iat_offsets[dll_index]);
    }

    Ok(IdataSection { bytes, slot_offsets })
}

fn group_imports(order: &[(String, String)]) -> Vec<NativeDllImport> {
    let mut dll_order = Vec::new();
    let mut seen = BTreeSet::new();
    let mut grouped: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (dll, function) in order {
        if seen.insert(dll.clone()) {
            dll_order.push(dll.clone());
        }
        grouped.entry(dll.clone()).or_default().push(function.clone());
    }
    dll_order.into_iter().map(|dll| NativeDllImport { functions: grouped.remove(&dll).unwrap_or_default(), dll }).collect()
}

fn write_descriptor(bytes: &mut [u8], offset: usize, ilt_rva: u32, name_rva: u32, iat_rva: u32) {
    bytes[offset..offset + 4].copy_from_slice(&ilt_rva.to_le_bytes());
    bytes[offset + 12..offset + 16].copy_from_slice(&name_rva.to_le_bytes());
    bytes[offset + 16..offset + 20].copy_from_slice(&iat_rva.to_le_bytes());
}

fn normalize_dll_name(dll: &str) -> String {
    if dll.to_ascii_lowercase().ends_with(".dll") { dll.to_ascii_lowercase() } else { format!("{}.dll", dll.to_ascii_lowercase()) }
}

fn align2(bytes: &mut Vec<u8>) {
    if bytes.len() % 2 != 0 {
        bytes.push(0);
    }
}

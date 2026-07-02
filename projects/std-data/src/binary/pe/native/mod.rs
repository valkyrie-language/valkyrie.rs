mod builder;
mod layout;

pub use builder::NativeImageBuilder;

use miette::{Result, miette};

use self::layout::{NativeSectionLayout, SectionPlacement};
pub(crate) use super::writer::{FILE_ALIGNMENT, IMAGE_BASE_X64, PeBinaryWriter, SECTION_ALIGNMENT};

/// 单个 `DLL` 导入描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeDllImport {
    /// `DLL` 名称。
    pub dll: String,
    /// 导入函数名。
    pub functions: Vec<String>,
}

/// 待写入的原生 `PE` 镜像。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativePeImage {
    /// `.text` 节。
    pub text: Vec<u8>,
    /// `.rdata` 节。
    pub rdata: Vec<u8>,
    /// `.idata` 节（段内偏移尚未加 `idata_rva`）。
    pub idata: Vec<u8>,
    /// 导入目录描述。
    pub imports: Vec<NativeDllImport>,
    /// 入口点在 `.text` 内的偏移。
    pub entry_point: u32,
}

/// 原生 `PE` 可执行文件写入器。
pub struct NativePeWriter;

impl NativePeWriter {
    /// 将原生镜像写入 `PE32+` 控制台可执行文件。
    pub fn write_executable(image: &NativePeImage) -> Result<Vec<u8>> {
        if image.text.is_empty() {
            return Err(miette!("`.text` 节不能为空"));
        }

        let text_len = u32::try_from(image.text.len()).map_err(|_| miette!("`.text` 过大"))?;
        let rdata_len = u32::try_from(image.rdata.len()).map_err(|_| miette!("`.rdata` 过大"))?;
        let idata_len = u32::try_from(image.idata.len()).map_err(|_| miette!("`.idata` 过大"))?;
        let layout = NativeSectionLayout::compute(text_len, rdata_len, idata_len);

        let mut idata = image.idata.clone();
        let import_directory_size = if image.imports.is_empty() {
            0
        }
        else {
            u32::try_from((image.imports.len() + 1) * 20).map_err(|_| miette!("导入目录过大"))?
        };
        let import_directory_rva = layout.idata.map(|section| section.rva).unwrap_or(0);
        if let Some(idata_section) = layout.idata {
            patch_idata_rvas(&mut idata, idata_section.rva, image.imports.len());
        }

        let entry_rva = layout.text.rva + image.entry_point;
        let mut writer = PeBinaryWriter::new();
        write_dos_header(&mut writer);
        writer.write_u32(0x0000_4550);
        write_coff_header(&mut writer, layout.section_count);
        write_optional_header(
            &mut writer,
            layout.text.rva,
            layout.text.raw_size,
            layout.size_of_initialized_data,
            layout.size_of_headers,
            layout.size_of_image,
            entry_rva,
            import_directory_rva,
            import_directory_size,
        );
        write_section(&mut writer, b".text\0\0\0", &layout.text, 0x6000_0020);
        if let Some(rdata) = layout.rdata {
            write_section(&mut writer, b".rdata\0\0", &rdata, 0x4000_0040);
        }
        if let Some(idata_section) = layout.idata {
            write_section(&mut writer, b".idata\0\0", &idata_section, 0xC000_0040);
        }

        pad_to(&mut writer, layout.size_of_headers as usize);
        writer.write_bytes(&image.text);
        pad_to(&mut writer, (layout.text.raw_ptr + layout.text.raw_size) as usize);
        if let Some(rdata) = layout.rdata {
            writer.write_bytes(&image.rdata);
            pad_to(&mut writer, (rdata.raw_ptr + rdata.raw_size) as usize);
        }
        if let Some(idata_section) = layout.idata {
            writer.write_bytes(&idata);
            pad_to(&mut writer, (idata_section.raw_ptr + idata_section.raw_size) as usize);
        }

        Ok(writer.into_bytes())
    }
}

fn patch_idata_rvas(idata: &mut [u8], idata_rva: u32, import_count: usize) {
    let mut thunk_starts = Vec::with_capacity(import_count * 2);
    for index in 0..import_count {
        let offset = index * 20;
        let ilt = u32::from_le_bytes(idata[offset..offset + 4].try_into().unwrap());
        let name = u32::from_le_bytes(idata[offset + 12..offset + 16].try_into().unwrap());
        let iat = u32::from_le_bytes(idata[offset + 16..offset + 20].try_into().unwrap());
        thunk_starts.push(ilt);
        thunk_starts.push(iat);
        idata[offset..offset + 4].copy_from_slice(&(idata_rva + ilt).to_le_bytes());
        idata[offset + 12..offset + 16].copy_from_slice(&(idata_rva + name).to_le_bytes());
        idata[offset + 16..offset + 20].copy_from_slice(&(idata_rva + iat).to_le_bytes());
    }

    for start in thunk_starts {
        let mut cursor = start as usize;
        while cursor + 8 <= idata.len() {
            let value = u64::from_le_bytes(idata[cursor..cursor + 8].try_into().unwrap());
            if value == 0 {
                break;
            }
            idata[cursor..cursor + 8].copy_from_slice(&(idata_rva as u64 + value).to_le_bytes());
            cursor += 8;
        }
    }
}

fn write_dos_header(writer: &mut PeBinaryWriter) {
    writer.write_u16(0x5A4D);
    for _ in 0..29 {
        writer.write_u16(0);
    }
    writer.write_u32(0x40);
}

fn write_coff_header(writer: &mut PeBinaryWriter, section_count: u16) {
    writer.write_u16(0x8664);
    writer.write_u16(section_count);
    writer.write_u32(0);
    writer.write_u32(0);
    writer.write_u32(0);
    writer.write_u16(240);
    // EXECUTABLE_IMAGE | LARGE_ADDRESS_AWARE | RELOCS_STRIPPED
    writer.write_u16(0x0023);
}

fn write_optional_header(
    writer: &mut PeBinaryWriter,
    text_rva: u32,
    text_raw_size: u32,
    size_of_initialized_data: u32,
    size_of_headers: u32,
    size_of_image: u32,
    entry_rva: u32,
    import_rva: u32,
    import_size: u32,
) {
    writer.write_u16(0x020B);
    writer.write_u8(14);
    writer.write_u8(0);
    writer.write_u32(text_raw_size);
    writer.write_u32(size_of_initialized_data);
    writer.write_u32(0);
    writer.write_u32(entry_rva);
    writer.write_u32(text_rva);
    writer.write_u64(IMAGE_BASE_X64);
    writer.write_u32(SECTION_ALIGNMENT);
    writer.write_u32(FILE_ALIGNMENT);
    writer.write_u16(6);
    writer.write_u16(0);
    writer.write_u16(0);
    writer.write_u16(0);
    writer.write_u16(6);
    writer.write_u16(0);
    writer.write_u32(0);
    writer.write_u32(size_of_image);
    writer.write_u32(size_of_headers);
    writer.write_u32(0);
    writer.write_u16(3);
    // NX_COMPAT only (no DYNAMIC_BASE — 无 .reloc)
    writer.write_u16(0x0100);
    writer.write_u64(0x100000);
    writer.write_u64(0x1000);
    writer.write_u64(0x100000);
    writer.write_u64(0x1000);
    writer.write_u32(0);
    writer.write_u32(16);
    // Export
    writer.write_u32(0);
    writer.write_u32(0);
    // Import
    writer.write_u32(import_rva);
    writer.write_u32(import_size);
    for _ in 0..14 {
        writer.write_u32(0);
        writer.write_u32(0);
    }
}

fn write_section(writer: &mut PeBinaryWriter, name: &[u8; 8], section: &SectionPlacement, characteristics: u32) {
    writer.write_bytes(name);
    writer.write_u32(section.virtual_size);
    writer.write_u32(section.rva);
    writer.write_u32(section.raw_size);
    writer.write_u32(section.raw_ptr);
    writer.write_u32(0);
    writer.write_u32(0);
    writer.write_u16(0);
    writer.write_u16(0);
    writer.write_u32(characteristics);
}

fn pad_to(writer: &mut PeBinaryWriter, size: usize) {
    while writer.len() < size {
        writer.write_u8(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn writes_amd64_console_executable() {
        let image = NativePeImage { text: vec![0x31, 0xC0, 0xC3], rdata: Vec::new(), idata: Vec::new(), imports: Vec::new(), entry_point: 0 };
        let bytes = NativePeWriter::write_executable(&image).expect("write pe");
        assert_eq!(&bytes[0..2], b"MZ");
        assert_eq!(&bytes[0x40..0x44], b"PE\0\0");
        assert_eq!(u16::from_le_bytes([bytes[0x44], bytes[0x45]]), 0x8664);

        let pe_offset = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
        let magic = u16::from_le_bytes(bytes[pe_offset + 24..pe_offset + 26].try_into().unwrap());
        assert_eq!(magic, 0x020B);
        let section_count = u16::from_le_bytes(bytes[pe_offset + 6..pe_offset + 8].try_into().unwrap());
        assert_eq!(section_count, 1);
        let import_rva = u32::from_le_bytes(bytes[pe_offset + 24 + 120..pe_offset + 24 + 124].try_into().unwrap());
        let import_size = u32::from_le_bytes(bytes[pe_offset + 24 + 124..pe_offset + 24 + 128].try_into().unwrap());
        assert_eq!(import_rva, 0);
        assert_eq!(import_size, 0);
    }

    #[test]
    #[cfg(windows)]
    fn runs_stub_executable() {
        let image = NativePeImage { text: vec![0x31, 0xC0, 0xC3], rdata: Vec::new(), idata: Vec::new(), imports: Vec::new(), entry_point: 0 };
        let bytes = NativePeWriter::write_executable(&image).expect("write pe");
        let path = std::env::temp_dir().join(format!("valkyrie-native-stub-{}.exe", std::process::id()));
        std::fs::write(&path, &bytes).expect("write");
        let status = std::process::Command::new(&path).status().expect("run");
        let _ = std::fs::remove_file(&path);
        assert!(status.success(), "status={status:?}");
    }
}

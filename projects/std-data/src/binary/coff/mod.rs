#![doc = include_str!("readme.md")]

use miette::{miette, Result};
use serde::{Deserialize, Serialize};

use nyar::abstractions::{BinaryArch, ObjectKind};

/// `COFF` 机器类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoffMachine {
    /// 未指定。
    Unknown,
    /// `x86`
    I386,
    /// `x64`
    Amd64,
    /// `arm64`
    Arm64,
}

impl CoffMachine {
    /// 从共享架构转换。
    pub fn from_arch(arch: BinaryArch) -> Self {
        match arch {
            BinaryArch::X86 => Self::I386,
            BinaryArch::X64 => Self::Amd64,
            BinaryArch::Arm64 => Self::Arm64,
            BinaryArch::Any => Self::Unknown,
        }
    }
}

/// `COFF` 文件头。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoffHeader {
    /// 机器类型。
    pub machine: CoffMachine,
    /// 节数量。
    pub section_count: u16,
    /// 符号表偏移。
    pub symbol_table_offset: u32,
    /// 符号数量。
    pub symbol_count: u32,
    /// 特征位。
    pub characteristics: u16,
}

/// `COFF` 节。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoffSection {
    /// 节名。
    pub name: String,
    /// 原始数据。
    pub data: Vec<u8>,
    /// 重定位信息。
    pub relocations: Vec<CoffRelocation>,
    /// 特征位。
    pub characteristics: u32,
}

/// 重定位种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CoffRelocationKind {
    /// 绝对地址。
    Absolute,
    /// 相对地址。
    Relative,
    /// 段地址。
    SectionRelative,
}

/// `COFF` 重定位项。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoffRelocation {
    /// 节内偏移。
    pub offset: u32,
    /// 目标符号。
    pub symbol_name: String,
    /// 重定位种类。
    pub kind: CoffRelocationKind,
}

/// `COFF` 符号。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoffSymbol {
    /// 符号名。
    pub name: String,
    /// 节索引。
    pub section_index: i16,
    /// 符号值。
    pub value: u32,
    /// 存储类。
    pub storage_class: u8,
}

/// `COFF` 对象文件。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoffObject {
    /// 文件头。
    pub header: CoffHeader,
    /// 对象种类。
    pub object_kind: ObjectKind,
    /// 节表。
    pub sections: Vec<CoffSection>,
    /// 符号表。
    pub symbols: Vec<CoffSymbol>,
}

/// `COFF` 对象写出器。
pub struct CoffObjectWriter;

impl CoffObjectWriter {
    /// 将 `COFF` 对象编码为字节流。
    pub fn write(object: &CoffObject) -> Result<Vec<u8>> {
        let section_count = u16::try_from(object.sections.len()).map_err(|_| miette!("节数量超过 `COFF` 上限"))?;
        let symbol_count = u32::try_from(object.symbols.len()).map_err(|_| miette!("符号数量超过 `COFF` 上限"))?;
        let header_size = 20usize;
        let section_headers_size = object.sections.len() * 40usize;
        let mut raw_data_offset = header_size + section_headers_size;
        let mut section_raw_offsets = Vec::with_capacity(object.sections.len());

        for section in &object.sections {
            section_raw_offsets.push(u32::try_from(raw_data_offset).map_err(|_| miette!("`COFF` 偏移超过 32 位范围"))?);
            raw_data_offset += section.data.len();
        }

        let symbol_table_offset = u32::try_from(raw_data_offset).map_err(|_| miette!("符号表偏移超过 32 位范围"))?;
        let mut bytes = Vec::new();
        let mut string_table = CoffStringTable::new();

        bytes.extend_from_slice(&machine_to_u16(object.header.machine)?.to_le_bytes());
        bytes.extend_from_slice(&section_count.to_le_bytes());
        bytes.extend_from_slice(&0u32.to_le_bytes());
        bytes.extend_from_slice(&symbol_table_offset.to_le_bytes());
        bytes.extend_from_slice(&symbol_count.to_le_bytes());
        bytes.extend_from_slice(&0u16.to_le_bytes());
        bytes.extend_from_slice(&object.header.characteristics.to_le_bytes());

        for (index, section) in object.sections.iter().enumerate() {
            encode_name(&section.name, &mut bytes, &mut string_table)?;
            bytes.extend_from_slice(&0u32.to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
            bytes.extend_from_slice(&(u32::try_from(section.data.len()).map_err(|_| miette!("节数据过大"))?).to_le_bytes());
            bytes.extend_from_slice(&section_raw_offsets[index].to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
            bytes.extend_from_slice(&0u32.to_le_bytes());
            bytes.extend_from_slice(&0u16.to_le_bytes());
            bytes.extend_from_slice(&0u16.to_le_bytes());
            bytes.extend_from_slice(&section.characteristics.to_le_bytes());
        }

        for section in &object.sections {
            bytes.extend_from_slice(&section.data);
        }

        for symbol in &object.symbols {
            encode_name(&symbol.name, &mut bytes, &mut string_table)?;
            bytes.extend_from_slice(&symbol.value.to_le_bytes());
            bytes.extend_from_slice(&symbol.section_index.to_le_bytes());
            bytes.extend_from_slice(&0x0020u16.to_le_bytes());
            bytes.push(symbol.storage_class);
            bytes.push(0);
        }

        bytes.extend_from_slice(&string_table.to_bytes()?);
        Ok(bytes)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
struct CoffStringTable {
    bytes: Vec<u8>,
}

impl CoffStringTable {
    fn new() -> Self {
        Self { bytes: Vec::new() }
    }

    fn intern(&mut self, value: &str) -> Result<u32> {
        let offset = u32::try_from(self.bytes.len() + 4).map_err(|_| miette!("`COFF` 字符串表过大"))?;
        self.bytes.extend_from_slice(value.as_bytes());
        self.bytes.push(0);
        Ok(offset)
    }

    fn to_bytes(&self) -> Result<Vec<u8>> {
        let total_size = u32::try_from(self.bytes.len() + 4).map_err(|_| miette!("`COFF` 字符串表过大"))?;
        let mut out = Vec::with_capacity(self.bytes.len() + 4);
        out.extend_from_slice(&total_size.to_le_bytes());
        out.extend_from_slice(&self.bytes);
        Ok(out)
    }
}

fn machine_to_u16(machine: CoffMachine) -> Result<u16> {
    match machine {
        CoffMachine::Unknown => Err(miette!("`COFF` 机器类型不能为 `Unknown`")),
        CoffMachine::I386 => Ok(0x014C),
        CoffMachine::Amd64 => Ok(0x8664),
        CoffMachine::Arm64 => Ok(0xAA64),
    }
}

fn encode_name(name: &str, out: &mut Vec<u8>, string_table: &mut CoffStringTable) -> Result<()> {
    if name.len() <= 8 && name.is_ascii() {
        let mut bytes = [0u8; 8];
        bytes[..name.len()].copy_from_slice(name.as_bytes());
        out.extend_from_slice(&bytes);
        return Ok(());
    }

    out.extend_from_slice(&0u32.to_le_bytes());
    let offset = string_table.intern(name)?;
    out.extend_from_slice(&offset.to_le_bytes());
    Ok(())
}

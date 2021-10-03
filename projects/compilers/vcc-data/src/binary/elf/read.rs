use std::fmt;

use super::layout::ELF_EHDR_SIZE;

/// `ELF64` 可执行文件解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Elf64ParseError {
    /// 镜像过短。
    TooShort,
    /// `ELF` 魔数错误。
    BadMagic,
    /// 不是 64 位 `ELF`。
    NotElf64,
    /// 文件类型字段。
    FileType(u16),
    /// 机器类型字段。
    Machine(u16),
}

impl fmt::Display for Elf64ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooShort => write!(f, "ELF 镜像过短"),
            Self::BadMagic => write!(f, "ELF 镜像缺少 `\\x7fELF` 魔数"),
            Self::NotElf64 => write!(f, "ELF 镜像不是 64 位"),
            Self::FileType(kind) => write!(f, "ELF 镜像类型 0x{kind:04X}"),
            Self::Machine(machine) => write!(f, "ELF 镜像机器类型 0x{machine:04X}"),
        }
    }
}

impl std::error::Error for Elf64ParseError {}

/// 解析后的 `ELF64` 头摘要。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Elf64Header {
    /// `e_type`。
    pub file_type: u16,
    /// `e_machine`。
    pub machine: u16,
}

/// 解析 `ELF64` 可执行文件头。
pub fn parse_elf64(bytes: &[u8]) -> Result<Elf64Header, Elf64ParseError> {
    if bytes.len() < ELF_EHDR_SIZE as usize {
        return Err(Elf64ParseError::TooShort);
    }
    if &bytes[0..4] != b"\x7fELF" {
        return Err(Elf64ParseError::BadMagic);
    }
    if bytes[4] != 2 {
        return Err(Elf64ParseError::NotElf64);
    }
    let file_type = u16::from_le_bytes(bytes[16..18].try_into().expect("slice"));
    let machine = u16::from_le_bytes(bytes[18..20].try_into().expect("slice"));
    Ok(Elf64Header { file_type, machine })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::elf::{NativeElfImage, NativeElfWriter};

    #[test]
    fn parses_native_elf_writer_output() {
        let image = NativeElfImage { text: vec![0x31, 0xC0, 0xC3], rodata: Vec::new(), entry_point: 0 };
        let bytes = NativeElfWriter::write_executable(&image).expect("write elf");
        let header = parse_elf64(&bytes).expect("parse elf");
        assert_eq!(header.file_type, 2);
        assert_eq!(header.machine, 62);
    }
}

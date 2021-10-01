#![doc = include_str!("readme.md")]

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// `WASM` 二进制读写错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmBinaryError {
    /// 输入在中途结束。
    UnexpectedEof,
    /// 魔数不正确。
    InvalidMagic([u8; 4]),
    /// 只支持版本 `1`。
    UnsupportedVersion(u32),
    /// `LEB128` 编码不合法。
    InvalidLeb128,
    /// 自定义段名称无效。
    InvalidUtf8Name,
}

impl Display for WasmBinaryError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnexpectedEof => write!(f, "WASM 二进制在读取过程中意外结束"),
            Self::InvalidMagic(magic) => {
                write!(f, "无效的 WASM 魔数：{:02X} {:02X} {:02X} {:02X}", magic[0], magic[1], magic[2], magic[3])
            }
            Self::UnsupportedVersion(version) => write!(f, "暂不支持的 WASM 版本：{version}"),
            Self::InvalidLeb128 => write!(f, "无效的 WASM `LEB128` 编码"),
            Self::InvalidUtf8Name => write!(f, "WASM 自定义段名称不是有效 UTF-8"),
        }
    }
}

impl std::error::Error for WasmBinaryError {}

/// `WASM` 自定义段模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WasmCustomSection {
    /// 段名。
    pub name: String,
    /// 段内容。
    pub bytes: Vec<u8>,
}

/// `WASM` 段模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WasmSection {
    /// 段 `id`。
    pub id: u8,
    /// 自定义段名，仅 `id == 0` 时有值。
    pub name: Option<String>,
    /// 去掉自定义段名称后的原始段内容。
    pub bytes: Vec<u8>,
}

/// `WebAssembly` 二进制模块模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WasmBinaryModule {
    /// 模块版本，当前默认 `1`。
    pub version: u32,
    /// 模块段列表。
    pub sections: Vec<WasmSection>,
}

impl WasmBinaryModule {
    /// 创建一个新的空二进制模块。
    pub fn new() -> Self {
        Self { version: 1, sections: Vec::new() }
    }

    /// 追加一个自定义段。
    pub fn push_custom_section(&mut self, name: impl Into<String>, bytes: Vec<u8>) {
        self.sections.push(WasmSection { id: 0, name: Some(name.into()), bytes });
    }

    /// 返回所有自定义段视图。
    pub fn custom_sections(&self) -> Vec<WasmCustomSection> {
        self.sections
            .iter()
            .filter_map(|section| {
                if section.id == 0 {
                    section.name.clone().map(|name| WasmCustomSection { name, bytes: section.bytes.clone() })
                }
                else {
                    None
                }
            })
            .collect()
    }

    /// 从二进制数据解析 `WASM` 模块。
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, WasmBinaryError> {
        let mut reader = WasmReader::new(bytes);
        let magic = reader.read_exact_array::<4>()?;
        if magic != WASM_MAGIC {
            return Err(WasmBinaryError::InvalidMagic(magic));
        }

        let version = reader.read_u32_le()?;
        if version != 1 {
            return Err(WasmBinaryError::UnsupportedVersion(version));
        }

        let mut sections = Vec::new();
        while !reader.is_eof() {
            let id = reader.read_u8()?;
            let payload_len = reader.read_uleb128()? as usize;
            let payload = reader.read_bytes(payload_len)?;
            if id == 0 {
                let mut payload_reader = WasmReader::new(payload);
                let name_len = payload_reader.read_uleb128()? as usize;
                let name_bytes = payload_reader.read_bytes(name_len)?;
                let name = String::from_utf8(name_bytes.to_vec()).map_err(|_| WasmBinaryError::InvalidUtf8Name)?;
                let remaining = payload_reader.read_remaining().to_vec();
                sections.push(WasmSection { id, name: Some(name), bytes: remaining });
            }
            else {
                sections.push(WasmSection { id, name: None, bytes: payload.to_vec() });
            }
        }

        Ok(Self { version, sections })
    }

    /// 将模块重新编码为 `WASM` 二进制。
    pub fn to_bytes(&self) -> Result<Vec<u8>, WasmBinaryError> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&WASM_MAGIC);
        bytes.extend_from_slice(&self.version.to_le_bytes());
        for section in &self.sections {
            bytes.push(section.id);
            let mut payload = Vec::new();
            if section.id == 0 {
                let name = section.name.clone().unwrap_or_default();
                write_uleb128(name.len() as u32, &mut payload);
                payload.extend_from_slice(name.as_bytes());
                payload.extend_from_slice(&section.bytes);
            }
            else {
                payload.extend_from_slice(&section.bytes);
            }
            write_uleb128(payload.len() as u32, &mut bytes);
            bytes.extend_from_slice(&payload);
        }
        Ok(bytes)
    }
}

struct WasmReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> WasmReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.bytes.len()
    }

    fn read_u8(&mut self) -> Result<u8, WasmBinaryError> {
        let value = *self.bytes.get(self.offset).ok_or(WasmBinaryError::UnexpectedEof)?;
        self.offset += 1;
        Ok(value)
    }

    fn read_u32_le(&mut self) -> Result<u32, WasmBinaryError> {
        let bytes = self.read_exact_array::<4>()?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_exact_array<const N: usize>(&mut self) -> Result<[u8; N], WasmBinaryError> {
        let bytes = self.read_bytes(N)?;
        let mut array = [0u8; N];
        array.copy_from_slice(bytes);
        Ok(array)
    }

    fn read_bytes(&mut self, length: usize) -> Result<&'a [u8], WasmBinaryError> {
        let end = self.offset.checked_add(length).ok_or(WasmBinaryError::UnexpectedEof)?;
        let bytes = self.bytes.get(self.offset..end).ok_or(WasmBinaryError::UnexpectedEof)?;
        self.offset = end;
        Ok(bytes)
    }

    fn read_uleb128(&mut self) -> Result<u32, WasmBinaryError> {
        let mut result = 0u32;
        let mut shift = 0u32;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift > 28 {
                return Err(WasmBinaryError::InvalidLeb128);
            }
        }
    }

    fn read_remaining(&mut self) -> &'a [u8] {
        let bytes = &self.bytes[self.offset..];
        self.offset = self.bytes.len();
        bytes
    }
}

fn write_uleb128(mut value: u32, output: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        output.push(byte);
        if value == 0 {
            break;
        }
    }
}

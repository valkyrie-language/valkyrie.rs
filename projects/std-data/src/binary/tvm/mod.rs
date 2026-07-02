//! TextVM `.tvm` 正则引擎产物格式。

use std::fmt::{Display, Formatter};

/// 魔术标识 `TVM\0`。
pub const TVM_MAGIC: [u8; 4] = *b"TVM\0";

/// TVM 头部总大小（魔术 4 字节 + 结构体 28 字节）。
pub const TVM_HEADER_SIZE: usize = 32;

/// TVM 操作类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
#[repr(u8)]
pub enum TvmOperation {
    /// 无操作。
    #[default]
    None = 0,
    /// 检查字符串是否存在。
    Exists = 1,
    /// 查找匹配位置。
    Find = 2,
    /// 查找并替换。
    Replace = 3,
}

impl TvmOperation {
    /// 从原始字节值解析操作类型。
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::None),
            1 => Some(Self::Exists),
            2 => Some(Self::Find),
            3 => Some(Self::Replace),
            _ => None,
        }
    }
}

/// TVM 头部标志常量。
pub mod TvmFlags {
    /// 表示正则表达式为纯字面量（无需编译 DFA）。
    pub const IS_LITERAL: u16 = 1;
    /// 表示存在字面量前缀，可用于快速过滤。
    pub const HAS_PREFIX: u16 = 2;
    /// 表示存在捕获组。
    pub const HAS_CAPTURE: u16 = 4;
    /// 表示使用 NFA（非确定性有限自动机）执行。
    pub const IS_NFA: u16 = 8;
}

/// TVM 二进制文件头部结构（32 字节）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TvmHeader {
    /// 版本号。
    pub version: u32,
    /// 文本编码类型（对应 `TextEncoding` 枚举值）。
    pub encoding: u8,
    /// 操作类型（对应 `TvmOperation` 枚举值）。
    pub operation: u8,
    /// 标志位（参见 `TvmFlags`）。
    pub flags: u16,
    /// 最小匹配长度。
    pub min_match_len: u32,
    /// 字面量前缀偏移量。
    pub literal_prefix_offset: u32,
    /// DFA 表偏移量。
    pub dfa_table_offset: u32,
    /// DFA 状态数量。
    pub dfa_state_count: u32,
    /// 文件总大小。
    pub total_size: u32,
}

/// TVM 头部读写错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TvmHeaderError {
    /// 数据长度不足。
    BufferTooShort { expected: usize, actual: usize },
    /// 魔术标识不匹配。
    InvalidMagic([u8; 4]),
    /// 未知的操作类型。
    InvalidOperation(u8),
}

impl Display for TvmHeaderError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BufferTooShort { expected, actual } => {
                write!(f, "数据长度不足 {expected} 字节，实际 {actual} 字节")
            }
            Self::InvalidMagic(magic) => {
                write!(f, "无效的魔术标识：{:02X} {:02X} {:02X} {:02X}", magic[0], magic[1], magic[2], magic[3])
            }
            Self::InvalidOperation(op) => write!(f, "无效的 TVM 操作类型：{op}"),
        }
    }
}

impl std::error::Error for TvmHeaderError {}

impl TvmHeader {
    /// 头部总大小。
    pub const HEADER_SIZE: usize = TVM_HEADER_SIZE;

    /// 从字节切片中读取头部信息。
    pub fn decode(data: &[u8]) -> Result<Self, TvmHeaderError> {
        if data.len() < Self::HEADER_SIZE {
            return Err(TvmHeaderError::BufferTooShort { expected: Self::HEADER_SIZE, actual: data.len() });
        }

        let magic: [u8; 4] = data[..4].try_into().expect("slice length checked");
        if magic != TVM_MAGIC {
            return Err(TvmHeaderError::InvalidMagic(magic));
        }

        let body = &data[4..];
        let version = u32::from_le_bytes(body[0..4].try_into().expect("version"));
        let encoding = body[4];
        let operation = body[5];
        let flags = u16::from_le_bytes(body[6..8].try_into().expect("flags"));
        let min_match_len = u32::from_le_bytes(body[8..12].try_into().expect("min_match_len"));
        let literal_prefix_offset = u32::from_le_bytes(body[12..16].try_into().expect("literal_prefix_offset"));
        let dfa_table_offset = u32::from_le_bytes(body[16..20].try_into().expect("dfa_table_offset"));
        let dfa_state_count = u32::from_le_bytes(body[20..24].try_into().expect("dfa_state_count"));
        let total_size = u32::from_le_bytes(body[24..28].try_into().expect("total_size"));

        TvmOperation::from_u8(operation).ok_or(TvmHeaderError::InvalidOperation(operation))?;

        Ok(Self { version, encoding, operation, flags, min_match_len, literal_prefix_offset, dfa_table_offset, dfa_state_count, total_size })
    }

    /// 将头部信息写入字节切片。
    pub fn encode(&self, data: &mut [u8]) -> Result<(), TvmHeaderError> {
        if data.len() < Self::HEADER_SIZE {
            return Err(TvmHeaderError::BufferTooShort { expected: Self::HEADER_SIZE, actual: data.len() });
        }

        data[..4].copy_from_slice(&TVM_MAGIC);
        data[4..8].copy_from_slice(&self.version.to_le_bytes());
        data[8] = self.encoding;
        data[9] = self.operation;
        data[10..12].copy_from_slice(&self.flags.to_le_bytes());
        data[12..16].copy_from_slice(&self.min_match_len.to_le_bytes());
        data[16..20].copy_from_slice(&self.literal_prefix_offset.to_le_bytes());
        data[20..24].copy_from_slice(&self.dfa_table_offset.to_le_bytes());
        data[24..28].copy_from_slice(&self.dfa_state_count.to_le_bytes());
        data[28..32].copy_from_slice(&self.total_size.to_le_bytes());
        Ok(())
    }

    /// 将头部序列化为 32 字节数组。
    pub fn to_bytes(&self) -> [u8; Self::HEADER_SIZE] {
        let mut buf = [0u8; Self::HEADER_SIZE];
        self.encode(&mut buf).expect("fixed-size buffer");
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_round_trip() {
        let header = TvmHeader {
            version: 42,
            encoding: 1,
            operation: TvmOperation::Find as u8,
            flags: TvmFlags::HAS_PREFIX | TvmFlags::HAS_CAPTURE,
            min_match_len: 7,
            literal_prefix_offset: 32,
            dfa_table_offset: 128,
            dfa_state_count: 25,
            total_size: 512,
        };

        let encoded = header.to_bytes();
        let decoded = TvmHeader::decode(&encoded).expect("decode");
        assert_eq!(decoded, header);
    }

    #[test]
    fn bad_magic_is_rejected() {
        let mut buf = [0u8; TVM_HEADER_SIZE];
        buf[..4].copy_from_slice(b"BAD\0");
        assert!(matches!(TvmHeader::decode(&buf), Err(TvmHeaderError::InvalidMagic(_))));
    }
}

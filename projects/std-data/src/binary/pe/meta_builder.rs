//! `CLR` 元数据堆构建器。

use std::{collections::HashMap, fmt};

/// 元数据编码错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClrMetadataError {
    /// 字符串过长。
    StringTooLong,
    /// 表行数溢出。
    TableOverflow,
}

impl fmt::Display for ClrMetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::StringTooLong => write!(f, "字符串过长"),
            Self::TableOverflow => write!(f, "表行数溢出"),
        }
    }
}

impl std::error::Error for ClrMetadataError {}

/// `#Strings` 堆构建器。
pub struct StringsHeap {
    data: Vec<u8>,
    offsets: HashMap<String, u32>,
}

impl Default for StringsHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl StringsHeap {
    /// 创建一个新的空 `#Strings` 堆。
    pub fn new() -> Self {
        let mut heap = Self { data: Vec::new(), offsets: HashMap::new() };
        heap.data.push(0);
        heap
    }

    /// 添加字符串并返回堆偏移。
    pub fn add(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.offsets.get(s) {
            return offset;
        }
        let offset = self.data.len() as u32;
        self.data.extend_from_slice(s.as_bytes());
        self.data.push(0);
        self.offsets.insert(s.to_string(), offset);
        offset
    }

    /// 获取堆数据。
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

/// `#US` 堆构建器。
pub struct UserStringsHeap {
    data: Vec<u8>,
    offsets: HashMap<String, u32>,
}

impl Default for UserStringsHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl UserStringsHeap {
    /// 创建一个新的空 `#US` 堆。
    pub fn new() -> Self {
        let mut heap = Self { data: Vec::new(), offsets: HashMap::new() };
        heap.data.push(0);
        heap
    }

    /// 添加用户字符串并返回堆偏移。
    ///
    /// ECMA-335 II.24.2.4：`UTF-16LE` 之后一字节尾标志——当任一 UTF-16 码元
    /// 高位为 1（`≥ 0x8000`，含补充平面代理对）时写 `1`，否则写 `0`。
    pub fn add(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.offsets.get(s) {
            return offset;
        }
        let offset = self.data.len() as u32;
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let has_high_bit = utf16.iter().any(|&unit| unit >= 0x8000);
        let mut bytes = Vec::with_capacity(utf16.len() * 2 + 1);
        for c in utf16 {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        bytes.push(if has_high_bit { 1 } else { 0 });
        Self::write_compressed_uint(&mut self.data, bytes.len() as u32);
        self.data.extend_from_slice(&bytes);
        self.offsets.insert(s.to_string(), offset);
        offset
    }

    /// 获取堆数据。
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn write_compressed_uint(bytes: &mut Vec<u8>, value: u32) {
        if value < 0x80 {
            bytes.push(value as u8);
        }
        else if value < 0x4000 {
            bytes.push((0x80 | (value >> 8) as u8) as u8);
            bytes.push(value as u8);
        }
        else {
            bytes.push((0xC0 | (value >> 24) as u8) as u8);
            bytes.push((value >> 16) as u8);
            bytes.push((value >> 8) as u8);
            bytes.push(value as u8);
        }
    }
}

/// `#Blob` 堆构建器。
pub struct BlobHeap {
    data: Vec<u8>,
}

impl Default for BlobHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl BlobHeap {
    /// 创建一个新的空 `#Blob` 堆。
    pub fn new() -> Self {
        let mut heap = Self { data: Vec::new() };
        heap.data.push(0);
        heap
    }

    /// 添加 blob 并返回堆偏移。
    pub fn add(&mut self, bytes: &[u8]) -> u32 {
        let offset = self.data.len() as u32;
        Self::write_compressed_uint(&mut self.data, bytes.len() as u32);
        self.data.extend_from_slice(bytes);
        offset
    }

    /// 获取堆数据。
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    fn write_compressed_uint(bytes: &mut Vec<u8>, value: u32) {
        if value < 0x80 {
            bytes.push(value as u8);
        }
        else if value < 0x4000 {
            bytes.push((0x80 | (value >> 8) as u8) as u8);
            bytes.push(value as u8);
        }
        else {
            bytes.push((0xC0 | (value >> 24) as u8) as u8);
            bytes.push((value >> 16) as u8);
            bytes.push((value >> 8) as u8);
            bytes.push(value as u8);
        }
    }
}

/// `#GUID` 堆构建器。
pub struct GuidHeap {
    data: Vec<u8>,
}

impl Default for GuidHeap {
    fn default() -> Self {
        Self::new()
    }
}

impl GuidHeap {
    /// 创建一个新的 `#GUID` 堆。
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// 添加一个 `GUID`，返回 1-based 索引。
    pub fn add(&mut self, guid: [u8; 16]) -> u32 {
        let index = (self.data.len() as u32) / 16 + 1;
        self.data.extend_from_slice(&guid);
        index
    }

    /// 获取堆数据。
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

pub use super::read::TableKind;

/// `CLR` 元数据构建器。
pub struct ClrMetadataBuilder {
    /// `#Strings` 堆。
    pub strings: StringsHeap,
    /// `#US` 堆。
    pub user_strings: UserStringsHeap,
    /// `#Blob` 堆。
    pub blob: BlobHeap,
    /// `#GUID` 堆。
    pub guid: GuidHeap,
    /// 方法体在 `.text` 中的 `RVA`。
    pub method_rvas: Vec<u32>,
}

impl Default for ClrMetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClrMetadataBuilder {
    /// 创建一个新的元数据构建器。
    pub fn new() -> Self {
        Self {
            strings: StringsHeap::new(),
            user_strings: UserStringsHeap::new(),
            blob: BlobHeap::new(),
            guid: GuidHeap::new(),
            method_rvas: Vec::new(),
        }
    }
}

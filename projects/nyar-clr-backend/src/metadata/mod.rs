//! `CLR` 元数据编码器。
//!
//! 构建 `CLR` 元数据流（`#Strings`、`#US`、`#GUID`、`#Blob`、`#~`）。
//! 参考 `ECMA-335` II.24。

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
        // 索引 0 处为空字符串。
        heap.data.push(0);
        heap
    }

    /// 添加字符串，返回堆内偏移。
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

/// `#US` 堆（用户字符串）构建器。
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
        // 索引 0 处为空。
        heap.data.push(0);
        heap
    }

    /// 添加用户字符串，返回堆内偏移（token 的索引部分）。
    pub fn add(&mut self, s: &str) -> u32 {
        if let Some(&offset) = self.offsets.get(s) {
            return offset;
        }
        let offset = self.data.len() as u32;
        // 编码：压缩长度 + UTF-16LE 字节 + 终止字节。
        let utf16: Vec<u16> = s.encode_utf16().collect();
        let mut bytes = Vec::with_capacity(utf16.len() * 2 + 1);
        for c in utf16 {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        // 终止字节（高位指示是否有特殊字符）。
        bytes.push(0);
        // 写入压缩长度。
        Self::write_compressed_uint(&mut self.data, bytes.len() as u32);
        // 写入内容。
        self.data.extend_from_slice(&bytes);
        self.offsets.insert(s.to_string(), offset);
        offset
    }

    /// 获取堆数据。
    pub fn data(&self) -> &[u8] {
        &self.data
    }

    /// 写入压缩无符号整数（`ECMA-335` II.23.2）。
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

    /// 添加 blob，返回堆内偏移。
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

    /// 写入压缩无符号整数。
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
    ///
    /// `ECMA-335` II.24.2.5 规定 `#GUID` 堆使用 1-based 索引：
    /// 索引 0 表示 `null`（无 `GUID`），索引 1 对应字节 0-15（第一个 `GUID`）。
    /// 因此堆数据本身不需要 `null` 槽位。
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    /// 添加一个 `GUID`，返回堆索引（1-based）。
    ///
    /// 第一个 `GUID` 返回索引 1，对应字节 0-15。
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

/// 元数据表种类（`ECMA-335` II.22）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TableKind {
    /// Module 表（0x00）。
    Module = 0x00,
    /// TypeRef 表（0x01）。
    TypeRef = 0x01,
    /// TypeDef 表（0x02）。
    TypeDef = 0x02,
    /// Field 表（0x04）。
    Field = 0x04,
    /// MethodDef 表（0x06）。
    MethodDef = 0x06,
    /// Param 表（0x08）。
    Param = 0x08,
    /// MemberRef 表（0x0A）。
    MemberRef = 0x0A,
    /// StandAloneSig 表（0x11）。
    StandAloneSig = 0x11,
    /// Assembly 表（0x20）。
    Assembly = 0x20,
    /// AssemblyRef 表（0x23）。
    AssemblyRef = 0x23,
}

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
    /// 方法体偏移列表（相对于 .text 节起始）。
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

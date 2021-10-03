#![doc = include_str!("readme.md")]

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// `WASM` 指令解码层。
pub mod decode;
/// `WASM` 常用编码辅助。
pub mod encode;
/// `WASM` 指令解码模型。
pub mod instruction;
/// `WASM` 操作码助记符查表。
pub mod opcode;
/// `WASM` 字节流读取器。
pub mod reader;
/// `WASM` 段级解码层。
pub mod section;

/// 解码函数体指令序列。
pub use decode::decode_code_body;
/// 解码 `GC` 前缀指令。
pub use decode::decode_gc_instruction;
/// 解码单条 `WASM` 指令。
pub use decode::decode_instruction;
/// `WASM` 常用编码辅助。
pub use encode::{
    BLOCKTYPE_EMPTY, FIELD_IMMUTABLE, FIELD_MUTABLE, LIMITS_HAS_MAX, LIMITS_MIN_ONLY, OP_CALL, OP_CALL_INDIRECT, OP_DROP, OP_END, OP_I32_CONST,
    OP_I32_LOAD, OP_LOCAL_SET, TYPE_FORM_ARRAY, TYPE_FORM_FUNC, TYPE_FORM_STRUCT, VALTYPE_ANYREF, VALTYPE_EXTERNREF, VALTYPE_F64,
    VALTYPE_FUNCREF, VALTYPE_I32, VALTYPE_I64, VALTYPE_REF, VALTYPE_REF_NULL, encode_array_copy, encode_array_get, encode_array_len,
    encode_array_new_default, encode_array_new_fixed, encode_array_set, encode_arraytype, encode_arraytype_raw, encode_block_empty, encode_br,
    encode_br_if, encode_call, encode_call_indirect, encode_drop, encode_else, encode_end, encode_external_kind, encode_f64_const,
    encode_f64_load, encode_f64_store, encode_functype, encode_functype_raw, encode_gc_op, encode_global_get, encode_global_set,
    encode_heap_type, encode_i32_add, encode_i32_and, encode_i32_const, encode_i32_eqz, encode_i32_load, encode_i32_store, encode_i32_sub,
    encode_i64_const, encode_i64_load, encode_i64_store, encode_if_empty, encode_local_decl, encode_local_get, encode_local_i32_decl,
    encode_local_set, encode_local_tee, encode_loop_empty, encode_memarg, encode_memarg_immediates, encode_memory_copy, encode_memory_fill,
    encode_memory_grow, encode_memory_size, encode_nop, encode_ref_cast_array, encode_ref_cast_type_index, encode_ref_is_null,
    encode_ref_null_anyref, encode_ref_null_externref, encode_ref_null_valtype, encode_return, encode_sleb128_i64, encode_struct_get,
    encode_struct_new_default, encode_struct_set, encode_structtype, encode_structtype_raw, encode_uleb128, encode_unreachable,
    encode_value_type, write_sleb128_i32,
};
/// 解码后的单条 `WASM` 指令。
pub use instruction::DecodedInstruction;
/// 解码后的 `WASM` 指令操作数。
pub use instruction::DecodedOperand;
/// 前缀指令类别。
pub use instruction::WasmPrefix;
/// `WASM` 操作码枚举。
pub use opcode::{WasmGcOpcode, WasmMiscOpcode, WasmOpcode, fc_mnemonic, gc_mnemonic, opcode_mnemonic};
/// `WASM` 字节流读取器。
pub use reader::WasmByteReader;
/// `WASM` 导出项。
pub use section::WasmExport;
/// `WASM` 外部实体种类。
pub use section::WasmExternalKind;
/// `WASM` 代码段函数条目。
pub use section::WasmFunctionEntry;
/// `WASM` 堆类型。
pub use section::WasmHeapType;
/// `WASM` 导入项。
pub use section::WasmImport;
/// `WASM` 类型段条目。
pub use section::WasmTypeEntry;
/// `WASM` 类型段条目种类。
pub use section::WasmTypeKind;
/// `WASM` 值类型。
pub use section::WasmValueType;
/// 解析 `code` 段函数列表。
pub use section::parse_code_section;
/// 解析 `export` 段。
pub use section::parse_export_section;
/// 解析 `import` 段。
pub use section::parse_import_section;
/// 解析 `type` 段。
pub use section::parse_type_section;
/// 读取 `WASM` 堆类型。
pub use section::read_heap_type;
/// 读取 `WASM` 值类型。
pub use section::read_value_type;
/// 将堆类型映射为可读名称。
pub use section::wasm_heap_type_name;
/// 将值类型映射为可读名称。
pub use section::wasm_value_type_name;

const WASM_MAGIC: [u8; 4] = [0x00, 0x61, 0x73, 0x6D];

/// 自定义段 `id`。
pub const SECTION_CUSTOM: u8 = 0;
/// 类型段 `id`。
pub const SECTION_TYPE: u8 = 1;
/// 导入段 `id`。
pub const SECTION_IMPORT: u8 = 2;
/// 函数段 `id`。
pub const SECTION_FUNCTION: u8 = 3;
/// 表段 `id`。
pub const SECTION_TABLE: u8 = 4;
/// 内存段 `id`。
pub const SECTION_MEMORY: u8 = 5;
/// 全局段 `id`。
pub const SECTION_GLOBAL: u8 = 6;
/// 导出段 `id`。
pub const SECTION_EXPORT: u8 = 7;
/// 起始段 `id`。
pub const SECTION_START: u8 = 8;
/// 元素段 `id`。
pub const SECTION_ELEMENT: u8 = 9;
/// 代码段 `id`。
pub const SECTION_CODE: u8 = 10;
/// 数据段 `id`。
pub const SECTION_DATA: u8 = 11;
/// 数据计数段 `id`。
pub const SECTION_DATA_COUNT: u8 = 12;

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
                if section.id == 0 { section.name.clone().map(|name| WasmCustomSection { name, bytes: section.bytes.clone() }) } else { None }
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

/// 返回段 `id` 对应的可读名称。
///
/// 已知 `id` 返回 `"custom" / "type" / "import" / "function" / "table" / "memory" / "global" / "export" / "start" / "element" / "code" / "data" / "data_count"`，未知 `id` 返回 `"unknown"`。
pub fn section_name(id: u8) -> &'static str {
    match id {
        SECTION_CUSTOM => "custom",
        SECTION_TYPE => "type",
        SECTION_IMPORT => "import",
        SECTION_FUNCTION => "function",
        SECTION_TABLE => "table",
        SECTION_MEMORY => "memory",
        SECTION_GLOBAL => "global",
        SECTION_EXPORT => "export",
        SECTION_START => "start",
        SECTION_ELEMENT => "element",
        SECTION_CODE => "code",
        SECTION_DATA => "data",
        SECTION_DATA_COUNT => "data_count",
        _ => "unknown",
    }
}

/// 计算无符号 `LEB128` 编码占用的字节数。
///
/// `0` 占 `1` 字节，每增加 `7` 位有效位多一字节。
pub fn uleb128_size(value: u32) -> usize {
    let mut size = 1;
    let mut v = value >> 7;
    while v != 0 {
        size += 1;
        v >>= 7;
    }
    size
}

/// 跳过单个值类型。
///
/// 普通值类型占一字节；当读到 `0x63`（`ref null ht`）或 `0x64`（`ref ht`）时，
/// 额外消耗一个 `s33` 有符号 `LEB128` 作为 heap type。
pub fn skip_value_type(reader: &mut WasmByteReader) -> Result<(), WasmBinaryError> {
    let byte = reader.read_u8()?;
    if byte == 0x63 || byte == 0x64 {
        let _ = reader.read_sleb128_i64();
    }
    Ok(())
}

/// 计算指定段在文件中的绝对偏移。
///
/// 从 `magic(4) + version(4) = 8` 开始累加每段的 `id(1) + len_field(uleb128_size) + payload`。
/// 自定义段 `payload` 还包含段名开销（`uleb128_size(name_len) + name_len`）。
/// 命中 `target_id` 时返回该段 `payload` 起始偏移（已跳过 `id` 与 `len` 字段，含自定义段名开销）。
pub fn compute_section_offset(module: &WasmBinaryModule, target_id: u8) -> usize {
    let mut offset = 8;
    for section in &module.sections {
        let id_field_size = 1;
        let name_overhead =
            if section.id == SECTION_CUSTOM { section.name.as_ref().map(|n| n.len() + uleb128_size(n.len() as u32)).unwrap_or(0) } else { 0 };
        let payload_len = if section.id == SECTION_CUSTOM { name_overhead + section.bytes.len() } else { section.bytes.len() };
        let len_field_size = uleb128_size(payload_len as u32);

        if section.id == target_id {
            return offset + id_field_size + len_field_size + name_overhead;
        }

        offset += id_field_size + len_field_size + payload_len;
    }
    offset
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

/// 将无符号整数编码为 `ULEB128`。
pub fn write_uleb128_public(value: u32, output: &mut Vec<u8>) {
    write_uleb128(value, output);
}

/// 将有符号整数编码为 `SLEB128`（`i64`/`s33`）。
pub fn write_sleb128_i64(mut value: i64, output: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        output.push(if done { byte } else { byte | 0x80 });
        if done {
            break;
        }
    }
}

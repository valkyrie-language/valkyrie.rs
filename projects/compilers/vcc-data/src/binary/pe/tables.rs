//! 元数据表行结构化解码器。
//!
//! 将 `#~` 流中的表行与 `#Blob` 中的签名解码为结构化模型，取代旧实现中返回格式化字符串的做法。
//!
//! 工具函数 [`read_idx`] / [`read_u32_table`] / [`super::read::read_compressed_uint`]
//! 统一供表行遍历与签名解码使用，不再保留窄版本 `coded_index_size_def_or_ref`。

use super::read::read_compressed_uint;

/// `MemberRefParent` 编码索引解码结果。
///
/// `MemberRefParent` 使用 3 个 `tag` 位（ECMA-335 II.24.2.6）：
/// - 0: `TypeDef`
/// - 1: `TypeRef`
/// - 2: `ModuleRef`
/// - 3: `MethodDef`
/// - 4: `TypeSpec`
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberRefParent {
    /// `TypeDef` 表行号。
    TypeDef(u32),
    /// `TypeRef` 表行号。
    TypeRef(u32),
    /// `ModuleRef` 表行号。
    ModuleRef(u32),
    /// `MethodDef` 表行号。
    MethodDef(u32),
    /// `TypeSpec` 表行号。
    TypeSpec(u32),
    /// 未知 `tag`，保留原始 `tag` 与行号。
    Unknown {
        /// 原始 `tag` 值。
        tag: u8,
        /// 解码后的行号。
        row: u32,
    },
}

/// `ELEMENT_TYPE` 编码（ECMA-335 II.23.1.16）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ElementType {
    /// `void` (0x01)。
    Void,
    /// `bool` (0x02)。
    Bool,
    /// `char` (0x03)。
    Char,
    /// `int8` (0x04)。
    Int8,
    /// `uint8` (0x05)。
    UInt8,
    /// `int16` (0x06)。
    Int16,
    /// `uint16` (0x07)。
    UInt16,
    /// `int32` (0x08)。
    Int32,
    /// `uint32` (0x09)。
    UInt32,
    /// `int64` (0x0A)。
    Int64,
    /// `uint64` (0x0B)。
    UInt64,
    /// `float32` (0x0C)。
    Float32,
    /// `float64` (0x0D)。
    Float64,
    /// `string` (0x0E)。
    String,
    /// `IntPtr` (0x18)。
    IntPtr,
    /// `UIntPtr` (0x19)。
    UIntPtr,
    /// `object` (0x1C)。
    Object,
    /// 一维零基数组（0x1D `SZARRAY`），后跟元素类型。
    SzArray(Box<ElementType>),
    /// 复杂类型（`VALUETYPE` / `CLASS` / `VAR` / `MVAR` / `GENERICINST` 等），
    /// 需读取后续 `TypeDefOrRef` 编码索引，此处仅保留原始 `ELEMENT_TYPE` 值。
    Complex(u8),
    /// 未知 `ELEMENT_TYPE`，保留原始字节。
    Unknown(u8),
}

/// 方法签名解码结果。
///
/// 格式参考 `ECMA-335` II.23.2.1：
/// `CallingConvention(1) + ParamCount(compressed) + RetType + ParamType × N`
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSignature {
    /// 调用约定字节（`0x00` = `DEFAULT`）。
    pub calling_convention: u8,
    /// 参数数量。
    pub param_count: u32,
    /// 返回类型。
    pub return_type: ElementType,
    /// 各参数类型。
    pub param_types: Vec<ElementType>,
}

/// 从表数据中读取变长索引。
///
/// `size` 为 2 或 4 字节，按小端序读取并返回 `u32`。
pub fn read_idx(data: &[u8], offset: usize, size: usize) -> u32 {
    if size == 2 {
        u16::from_le_bytes([data[offset], data[offset + 1]]) as u32
    }
    else {
        u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
    }
}

/// 从表数据中读取 4 字节小端无符号整数。
pub fn read_u32_table(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

/// 解码 `MemberRefParent` 编码索引为结构化 [`MemberRefParent`]。
///
/// `coded` 为编码索引原始值，`idx_size` 为其字节大小（2 或 4）。
pub fn decode_member_ref_parent(coded: u32, _idx_size: usize) -> MemberRefParent {
    let tag = (coded & 0x07) as u8;
    let row = coded >> 3;
    match tag {
        0 => MemberRefParent::TypeDef(row),
        1 => MemberRefParent::TypeRef(row),
        2 => MemberRefParent::ModuleRef(row),
        3 => MemberRefParent::MethodDef(row),
        4 => MemberRefParent::TypeSpec(row),
        _ => MemberRefParent::Unknown { tag, row },
    }
}

/// 解码方法签名 `blob` 为结构化 [`MethodSignature`]。
///
/// 仅支持 `DEFAULT` 调用约定（`0x00`）；其他调用约定将返回带原始字节的结果，
/// 参数与返回类型不再继续解码。
pub fn decode_method_signature(blob: &[u8]) -> MethodSignature {
    if blob.is_empty() {
        return MethodSignature { calling_convention: 0, param_count: 0, return_type: ElementType::Unknown(0), param_types: Vec::new() };
    }
    let mut cursor = 0usize;
    let cc = blob[cursor];
    cursor += 1;
    if cc != 0x00 {
        return MethodSignature { calling_convention: cc, param_count: 0, return_type: ElementType::Unknown(cc), param_types: Vec::new() };
    }
    let (param_count, len_bytes) = read_compressed_uint(blob, cursor);
    cursor += len_bytes;
    let (return_type, ret_len) = decode_element_type(blob, cursor);
    cursor += ret_len;
    let mut params = Vec::with_capacity(param_count as usize);
    for _ in 0..param_count {
        if cursor >= blob.len() {
            break;
        }
        let (ty, ty_len) = decode_element_type(blob, cursor);
        cursor += ty_len;
        params.push(ty);
    }
    MethodSignature { calling_convention: cc, param_count, return_type, param_types: params }
}

/// 解码 `ELEMENT_TYPE` 编码，返回 `(类型, 占用字节数)`。
///
/// 参考 `ECMA-335` II.23.1.16。`SZARRAY` (0x1D) 会递归解码其元素类型。
pub fn decode_element_type(blob: &[u8], offset: usize) -> (ElementType, usize) {
    if offset >= blob.len() {
        return (ElementType::Unknown(0), 0);
    }
    let et = blob[offset];
    let mut consumed = 1usize;
    let ty = match et {
        0x01 => ElementType::Void,
        0x02 => ElementType::Bool,
        0x03 => ElementType::Char,
        0x04 => ElementType::Int8,
        0x05 => ElementType::UInt8,
        0x06 => ElementType::Int16,
        0x07 => ElementType::UInt16,
        0x08 => ElementType::Int32,
        0x09 => ElementType::UInt32,
        0x0A => ElementType::Int64,
        0x0B => ElementType::UInt64,
        0x0C => ElementType::Float32,
        0x0D => ElementType::Float64,
        0x0E => ElementType::String,
        0x18 => ElementType::IntPtr,
        0x19 => ElementType::UIntPtr,
        0x1C => ElementType::Object,
        0x1D => {
            // SZARRAY：后跟元素类型。
            let (inner, inner_len) = decode_element_type(blob, offset + 1);
            consumed += inner_len;
            ElementType::SzArray(Box::new(inner))
        }
        0x11 | 0x12 | 0x13 | 0x14 | 0x15 | 0x16 | 0x17 | 0x1B | 0x50 | 0x55 => {
            // VALUETYPE / CLASS / VAR / MVAR / GENERICINST 等复杂类型，
            // 需要读取后续 TypeDefOrRef 编码索引，此处仅给出标记。
            ElementType::Complex(et)
        }
        _ => ElementType::Unknown(et),
    };
    (ty, consumed)
}

/// 计算模块表（`Module`，0x00）单行字节数的便捷函数。
///
/// 仅供调用方在未持有完整 [`MetadataRoot`] 时按统一公式估算使用。
pub fn module_row_size(strings_idx: usize, guid_idx: usize) -> usize {
    2 + strings_idx + 3 * guid_idx
}

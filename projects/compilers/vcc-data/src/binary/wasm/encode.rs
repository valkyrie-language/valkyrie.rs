//! `WASM` 指令与类型段编码助手。
//!
//! 覆盖后端 emitter 当前需要的指令形状，提供语义 API，避免手写操作码字节。
//! 二进制字面量仅出现在本模块与 [`super::opcode`] / [`super::section`] 的 encode 实现中。

use super::{
    opcode::{WasmGcOpcode, WasmMiscOpcode, WasmOpcode},
    section::{WasmExternalKind, WasmHeapType, WasmValueType},
    write_sleb128_i64, write_uleb128,
};

/// 空块类型立即数（`blocktype` = `0x40`）。
pub const BLOCKTYPE_EMPTY: u8 = 0x40;

/// 类型段 `functype` 形式字节。
pub const TYPE_FORM_FUNC: u8 = 0x60;
/// 类型段 `structtype` 形式字节（wasm-gc）。
pub const TYPE_FORM_STRUCT: u8 = 0x5F;
/// 类型段 `arraytype` 形式字节（GC MVP；早期草稿曾用 `0x61`）。
pub const TYPE_FORM_ARRAY: u8 = 0x5E;

/// 字段可变标志：不可变。
pub const FIELD_IMMUTABLE: u8 = 0x00;
/// 字段可变标志：可变。
pub const FIELD_MUTABLE: u8 = 0x01;
/// `limits` 标志：带有上限。
pub const LIMITS_HAS_MAX: u8 = 0x01;
/// `limits` 标志：仅有下限。
pub const LIMITS_MIN_ONLY: u8 = 0x00;

/// 兼容旧调用方：`call` 操作码字节。
pub const OP_CALL: u8 = WasmOpcode::Call as u8;
/// 兼容旧调用方：`call_indirect` 操作码字节。
pub const OP_CALL_INDIRECT: u8 = WasmOpcode::CallIndirect as u8;
/// 兼容旧调用方：`drop` 操作码字节。
pub const OP_DROP: u8 = WasmOpcode::Drop as u8;
/// 兼容旧调用方：`local.set` 操作码字节。
pub const OP_LOCAL_SET: u8 = WasmOpcode::LocalSet as u8;
/// 兼容旧调用方：`i32.load` 操作码字节。
pub const OP_I32_LOAD: u8 = WasmOpcode::I32Load as u8;
/// 兼容旧调用方：`i32.const` 操作码字节。
pub const OP_I32_CONST: u8 = WasmOpcode::I32Const as u8;
/// 兼容旧调用方：`end` 操作码字节。
pub const OP_END: u8 = WasmOpcode::End as u8;
/// 兼容旧调用方：`i32` 值类型字节。
pub const VALTYPE_I32: u8 = 0x7F;
/// `i64` 值类型字节。
pub const VALTYPE_I64: u8 = 0x7E;
/// `f64` 值类型字节。
pub const VALTYPE_F64: u8 = 0x7C;
/// `funcref` 值类型字节。
pub const VALTYPE_FUNCREF: u8 = 0x70;
/// `anyref` 值类型字节。
pub const VALTYPE_ANYREF: u8 = 0x6E;
/// `externref` 值类型字节。
pub const VALTYPE_EXTERNREF: u8 = 0x6F;
/// `(ref ht)` 前缀字节。
pub const VALTYPE_REF: u8 = 0x64;
/// `(ref null ht)` 前缀字节。
pub const VALTYPE_REF_NULL: u8 = 0x63;

/// 将有符号 `i32` 编码为 `SLEB128`。
pub fn write_sleb128_i32(mut value: i32, output: &mut Vec<u8>) {
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

/// 将无符号 `u32` 编码为 `ULEB128`（公开给段编码器复用）。
#[inline]
pub fn encode_uleb128(value: u32, output: &mut Vec<u8>) {
    write_uleb128(value, output);
}

/// 将有符号 `i64` 编码为 `SLEB128`。
#[inline]
pub fn encode_sleb128_i64(value: i64, output: &mut Vec<u8>) {
    write_sleb128_i64(value, output);
}

/// 编码局部变量声明组：`count` 个 `i32`。
pub fn encode_local_i32_decl(count: u32, output: &mut Vec<u8>) {
    write_uleb128(count, output);
    WasmValueType::I32.write(output);
}

/// 编码局部变量声明组：`count` 个指定值类型。
pub fn encode_local_decl(count: u32, valtype: &WasmValueType, output: &mut Vec<u8>) {
    write_uleb128(count, output);
    valtype.write(output);
}

/// `i32.const`
pub fn encode_i32_const(value: i32, output: &mut Vec<u8>) {
    WasmOpcode::I32Const.encode(output);
    write_sleb128_i32(value, output);
}

/// `i64.const`
pub fn encode_i64_const(value: i64, output: &mut Vec<u8>) {
    WasmOpcode::I64Const.encode(output);
    write_sleb128_i64(value, output);
}

/// `f64.const`
pub fn encode_f64_const(value: f64, output: &mut Vec<u8>) {
    WasmOpcode::F64Const.encode(output);
    output.extend_from_slice(&value.to_le_bytes());
}

/// 内存参数：`align` + `offset`（不含操作码）。
pub fn encode_memarg_immediates(align: u32, offset: u32, output: &mut Vec<u8>) {
    write_uleb128(align, output);
    write_uleb128(offset, output);
}

/// 带操作码的 memarg 指令。
pub fn encode_memarg(opcode: WasmOpcode, align: u32, offset: u32, output: &mut Vec<u8>) {
    opcode.encode(output);
    encode_memarg_immediates(align, offset, output);
}

/// `i32.load`
pub fn encode_i32_load(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::I32Load, align, offset, output);
}

/// `i64.load`
pub fn encode_i64_load(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::I64Load, align, offset, output);
}

/// `f32.load`
pub fn encode_f32_load(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::F32Load, align, offset, output);
}

/// `f64.load`
pub fn encode_f64_load(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::F64Load, align, offset, output);
}

/// `i32.store`
pub fn encode_i32_store(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::I32Store, align, offset, output);
}

/// `i64.store`
pub fn encode_i64_store(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::I64Store, align, offset, output);
}

/// `f64.store`
pub fn encode_f64_store(align: u32, offset: u32, output: &mut Vec<u8>) {
    encode_memarg(WasmOpcode::F64Store, align, offset, output);
}

/// `local.get`
pub fn encode_local_get(index: u32, output: &mut Vec<u8>) {
    WasmOpcode::LocalGet.encode(output);
    write_uleb128(index, output);
}

/// `local.set`
pub fn encode_local_set(index: u32, output: &mut Vec<u8>) {
    WasmOpcode::LocalSet.encode(output);
    write_uleb128(index, output);
}

/// `local.tee`
pub fn encode_local_tee(index: u32, output: &mut Vec<u8>) {
    WasmOpcode::LocalTee.encode(output);
    write_uleb128(index, output);
}

/// `global.get`
pub fn encode_global_get(index: u32, output: &mut Vec<u8>) {
    WasmOpcode::GlobalGet.encode(output);
    write_uleb128(index, output);
}

/// `global.set`
pub fn encode_global_set(index: u32, output: &mut Vec<u8>) {
    WasmOpcode::GlobalSet.encode(output);
    write_uleb128(index, output);
}

/// `call`
pub fn encode_call(function_index: u32, output: &mut Vec<u8>) {
    WasmOpcode::Call.encode(output);
    write_uleb128(function_index, output);
}

/// `call_indirect`
pub fn encode_call_indirect(type_idx: u32, table_idx: u32, output: &mut Vec<u8>) {
    WasmOpcode::CallIndirect.encode(output);
    write_uleb128(type_idx, output);
    write_uleb128(table_idx, output);
}

/// `drop`
pub fn encode_drop(output: &mut Vec<u8>) {
    WasmOpcode::Drop.encode(output);
}

/// `end`
pub fn encode_end(output: &mut Vec<u8>) {
    WasmOpcode::End.encode(output);
}

/// `unreachable`
pub fn encode_unreachable(output: &mut Vec<u8>) {
    WasmOpcode::Unreachable.encode(output);
}

/// `return`
pub fn encode_return(output: &mut Vec<u8>) {
    WasmOpcode::Return.encode(output);
}

/// `nop`
pub fn encode_nop(output: &mut Vec<u8>) {
    WasmOpcode::Nop.encode(output);
}

/// `i32.eqz`
pub fn encode_i32_eqz(output: &mut Vec<u8>) {
    WasmOpcode::I32Eqz.encode(output);
}

/// `i32.add`
pub fn encode_i32_add(output: &mut Vec<u8>) {
    WasmOpcode::I32Add.encode(output);
}

/// `i32.sub`
pub fn encode_i32_sub(output: &mut Vec<u8>) {
    WasmOpcode::I32Sub.encode(output);
}

/// `i32.and`
pub fn encode_i32_and(output: &mut Vec<u8>) {
    WasmOpcode::I32And.encode(output);
}

/// `br`
pub fn encode_br(depth: u32, output: &mut Vec<u8>) {
    WasmOpcode::Br.encode(output);
    write_uleb128(depth, output);
}

/// `br_if`
pub fn encode_br_if(depth: u32, output: &mut Vec<u8>) {
    WasmOpcode::BrIf.encode(output);
    write_uleb128(depth, output);
}

/// `block` + 空块类型。
pub fn encode_block_empty(output: &mut Vec<u8>) {
    WasmOpcode::Block.encode(output);
    output.push(BLOCKTYPE_EMPTY);
}

/// `loop` + 空块类型。
pub fn encode_loop_empty(output: &mut Vec<u8>) {
    WasmOpcode::Loop.encode(output);
    output.push(BLOCKTYPE_EMPTY);
}

/// `if` + 空块类型。
pub fn encode_if_empty(output: &mut Vec<u8>) {
    WasmOpcode::If.encode(output);
    output.push(BLOCKTYPE_EMPTY);
}

/// `else`
pub fn encode_else(output: &mut Vec<u8>) {
    WasmOpcode::Else.encode(output);
}

/// `ref.null` + 引用类型立即数（缩写或 heap type 字节）。
pub fn encode_ref_null_valtype(valtype: &WasmValueType, output: &mut Vec<u8>) {
    WasmOpcode::RefNull.encode(output);
    valtype.write(output);
}

/// `ref.null anyref`（`0xD0 0x6E`）。
pub fn encode_ref_null_anyref(output: &mut Vec<u8>) {
    encode_ref_null_valtype(&WasmValueType::AnyRef, output);
}

/// `ref.null externref`
pub fn encode_ref_null_externref(output: &mut Vec<u8>) {
    encode_ref_null_valtype(&WasmValueType::ExternRef, output);
}

/// `ref.is_null`
pub fn encode_ref_is_null(output: &mut Vec<u8>) {
    WasmOpcode::RefIsNull.encode(output);
}

/// `memory.size`（memory index 0）。
pub fn encode_memory_size(output: &mut Vec<u8>) {
    WasmOpcode::MemorySize.encode(output);
    write_uleb128(0, output);
}

/// `memory.grow`（memory index 0）。
pub fn encode_memory_grow(output: &mut Vec<u8>) {
    WasmOpcode::MemoryGrow.encode(output);
    write_uleb128(0, output);
}

/// `memory.copy`：`0xFC 0x0A` + 两个 memory index（默认均为 0）。
pub fn encode_memory_copy(output: &mut Vec<u8>) {
    WasmMiscOpcode::MemoryCopy.encode(output);
    write_uleb128(0, output);
    write_uleb128(0, output);
}

/// `memory.fill`：`0xFC 0x0B` + memory index 0。
pub fn encode_memory_fill(output: &mut Vec<u8>) {
    WasmMiscOpcode::MemoryFill.encode(output);
    write_uleb128(0, output);
}

/// GC 前缀指令：仅前缀 + 子操作码。
pub fn encode_gc_op(op: WasmGcOpcode, output: &mut Vec<u8>) {
    op.encode(output);
}

/// `struct.new_default` + type index。
pub fn encode_struct_new_default(type_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::StructNewDefault.encode(output);
    write_uleb128(type_index, output);
}

/// `struct.get` + type index + field index。
pub fn encode_struct_get(type_index: u32, field_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::StructGet.encode(output);
    write_uleb128(type_index, output);
    write_uleb128(field_index, output);
}

/// `struct.set` + type index + field index。
pub fn encode_struct_set(type_index: u32, field_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::StructSet.encode(output);
    write_uleb128(type_index, output);
    write_uleb128(field_index, output);
}

/// `array.new_default` + type index。
pub fn encode_array_new_default(type_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::ArrayNewDefault.encode(output);
    write_uleb128(type_index, output);
}

/// `array.new_fixed` + type index + count。
pub fn encode_array_new_fixed(type_index: u32, count: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::ArrayNewFixed.encode(output);
    write_uleb128(type_index, output);
    write_uleb128(count, output);
}

/// `array.get` + type index。
pub fn encode_array_get(type_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::ArrayGet.encode(output);
    write_uleb128(type_index, output);
}

/// `array.set` + type index。
pub fn encode_array_set(type_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::ArraySet.encode(output);
    write_uleb128(type_index, output);
}

/// `array.copy` + destination/source type indices.
pub fn encode_array_copy(dst_type: u32, src_type: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::ArrayCopy.encode(output);
    write_uleb128(dst_type, output);
    write_uleb128(src_type, output);
}

/// `array.len`
pub fn encode_array_len(output: &mut Vec<u8>) {
    WasmGcOpcode::ArrayLen.encode(output);
}

/// `ref.cast`（非空）+ heap type 立即数（类型索引以 s33 编码）。
pub fn encode_ref_cast_type_index(type_index: u32, output: &mut Vec<u8>) {
    WasmGcOpcode::RefCast.encode(output);
    write_sleb128_i32(i32::try_from(type_index).unwrap_or(i32::MAX), output);
}

/// `ref.cast`（非空）+ 抽象堆类型 `array`（s33 = -0x16）。
///
/// 用于将 `anyref` local 收窄为 `arrayref`，以满足 `array.len` 等不带
/// 类型索引的数组指令对操作数类型的要求。栈：`[anyref] → [arrayref]`。
pub fn encode_ref_cast_array(output: &mut Vec<u8>) {
    WasmGcOpcode::RefCast.encode(output);
    write_sleb128_i32(-0x16, output);
}

/// 编码 `functype`：`0x60` + params + results。
pub fn encode_functype(params: &[WasmValueType], results: &[WasmValueType]) -> Vec<u8> {
    let mut bytes = vec![TYPE_FORM_FUNC];
    write_uleb128(u32::try_from(params.len()).unwrap_or(0), &mut bytes);
    for ty in params {
        ty.write(&mut bytes);
    }
    write_uleb128(u32::try_from(results.len()).unwrap_or(0), &mut bytes);
    for ty in results {
        ty.write(&mut bytes);
    }
    bytes
}

/// 编码 `functype`，参数/结果已是编码好的 valtype 字节序列。
pub fn encode_functype_raw(params: &[u8], results: &[u8]) -> Vec<u8> {
    let mut bytes = vec![TYPE_FORM_FUNC];
    write_uleb128(u32::try_from(params.len()).unwrap_or(0), &mut bytes);
    bytes.extend_from_slice(params);
    write_uleb128(u32::try_from(results.len()).unwrap_or(0), &mut bytes);
    bytes.extend_from_slice(results);
    bytes
}

/// 编码 `structtype`：全部字段默认可变。
pub fn encode_structtype(field_types: &[WasmValueType]) -> Vec<u8> {
    let mut bytes = vec![TYPE_FORM_STRUCT];
    write_uleb128(u32::try_from(field_types.len()).unwrap_or(0), &mut bytes);
    for field_type in field_types {
        field_type.write(&mut bytes);
        bytes.push(FIELD_MUTABLE);
    }
    bytes
}

/// 编码 `structtype`，字段已是编码好的 valtype 字节。
pub fn encode_structtype_raw(field_types: &[u8]) -> Vec<u8> {
    let mut bytes = vec![TYPE_FORM_STRUCT];
    write_uleb128(u32::try_from(field_types.len()).unwrap_or(0), &mut bytes);
    for &field_type in field_types {
        bytes.push(field_type);
        bytes.push(FIELD_MUTABLE);
    }
    bytes
}

/// 编码 `arraytype`（GC MVP `0x5E`），元素可变。
pub fn encode_arraytype(element: &WasmValueType) -> Vec<u8> {
    let mut bytes = vec![TYPE_FORM_ARRAY];
    element.write(&mut bytes);
    bytes.push(FIELD_MUTABLE);
    bytes
}

/// 编码 `arraytype`，元素已是编码好的 valtype 字节。
pub fn encode_arraytype_raw(element_type: u8) -> Vec<u8> {
    vec![TYPE_FORM_ARRAY, element_type, FIELD_MUTABLE]
}

/// 编码导出/导入 kind 字节。
pub fn encode_external_kind(kind: WasmExternalKind, output: &mut Vec<u8>) {
    output.push(kind.as_u8());
}

/// 编码堆类型（s33）。
pub fn encode_heap_type(ht: &WasmHeapType, output: &mut Vec<u8>) {
    ht.write(output);
}

/// 编码值类型。
pub fn encode_value_type(vt: &WasmValueType, output: &mut Vec<u8>) {
    vt.write(output);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::wasm::{WasmByteReader, read_value_type};

    #[test]
    fn encode_i32_const_bytes() {
        let mut bytes = Vec::new();
        encode_i32_const(42, &mut bytes);
        assert_eq!(bytes[0], WasmOpcode::I32Const.as_u8());
    }

    #[test]
    fn encode_anyref_roundtrips_read() {
        let mut bytes = Vec::new();
        WasmValueType::AnyRef.write(&mut bytes);
        let mut reader = WasmByteReader::new(&bytes);
        assert_eq!(read_value_type(&mut reader).unwrap(), WasmValueType::AnyRef);
    }

    #[test]
    fn encode_struct_new_default_prefix() {
        let mut bytes = Vec::new();
        encode_struct_new_default(3, &mut bytes);
        assert_eq!(bytes[0], WasmOpcode::PrefixGc.as_u8());
        assert_eq!(bytes[1], WasmGcOpcode::StructNewDefault.as_u8());
    }

    #[test]
    fn encode_memory_copy_matches_bulk_memory() {
        let mut bytes = Vec::new();
        encode_memory_copy(&mut bytes);
        assert_eq!(&bytes[..2], &[WasmOpcode::PrefixMisc.as_u8(), WasmMiscOpcode::MemoryCopy.as_u8()]);
    }

    #[test]
    fn encode_arraytype_uses_gc_mvp_form() {
        let bytes = encode_arraytype(&WasmValueType::I32);
        assert_eq!(bytes[0], TYPE_FORM_ARRAY);
    }
}

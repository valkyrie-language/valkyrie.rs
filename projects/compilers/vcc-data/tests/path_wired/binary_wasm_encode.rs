//! Migrated from `src/binary/wasm/encode.rs` (`tests`).

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

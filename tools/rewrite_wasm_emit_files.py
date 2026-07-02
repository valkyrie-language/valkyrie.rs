from pathlib import Path

ROOT = Path(r"E:\Goddess of Victory\valkyrie.rs\projects\nyar-emitter\src\lowering\backends\wasm")

# --- cabi.rs ---
(ROOT / "cabi.rs").write_text(
    r'''//! Canonical ABI linear heap: shared bump cursor for `cabi_realloc` and MIR allocations.
use crate::nyar_backend_wasi::WasmSection;
use std_data::binary::wasm::{
    VALTYPE_I32, WasmOpcode, encode_i32_and, encode_i32_const, encode_if_empty, encode_local_get, encode_local_set,
    encode_local_tee, encode_memory_copy, encode_memory_fill, encode_memory_grow, encode_memory_size, encode_return,
    encode_unreachable,
};

use super::sections::{encode_uleb128, global_section_with_i32_inits};

pub(crate) const LINEAR_HEAP_MIN_BASE: i32 = 4096;
pub(crate) const CABI_HEAP_DEFAULT_BASE: i32 = LINEAR_HEAP_MIN_BASE;
pub(crate) const CABI_HEAP_GLOBAL_INDEX: u32 = 0;

pub(crate) fn align_up_u32(value: u32, align: u32) -> u32 {
    let align = align.max(1);
    let mask = align - 1;
    (value + mask) & !mask
}

pub(crate) fn cabi_heap_base_after_data(data_len: usize) -> i32 {
    let aligned = align_up_u32(u32::try_from(data_len).unwrap_or(u32::MAX), 8);
    i32::try_from(aligned.max(LINEAR_HEAP_MIN_BASE as u32)).unwrap_or(LINEAR_HEAP_MIN_BASE)
}

pub(crate) fn cabi_heap_global_section(heap_base: i32) -> WasmSection {
    debug_assert!(heap_base >= LINEAR_HEAP_MIN_BASE, "heap base must be >= {LINEAR_HEAP_MIN_BASE}");
    global_section_with_i32_inits(&[heap_base.max(LINEAR_HEAP_MIN_BASE)])
}

pub(crate) fn wasm_cabi_realloc_bump_body() -> Vec<u8> {
    let mut body = Vec::new();
    encode_uleb128(1, &mut body);
    encode_uleb128(4, &mut body);
    body.push(VALTYPE_I32);

    encode_local_get(3, &mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    encode_if_empty(&mut body);
    encode_i32_const(0, &mut body);
    encode_return(&mut body);
    WasmOpcode::End.encode(&mut body);

    encode_local_get(0, &mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    encode_local_get(3, &mut body);
    encode_local_get(1, &mut body);
    WasmOpcode::I32LeU.encode(&mut body);
    encode_i32_and(&mut body);
    encode_if_empty(&mut body);
    encode_local_get(0, &mut body);
    encode_return(&mut body);
    WasmOpcode::End.encode(&mut body);

    encode_local_get(2, &mut body);
    encode_i32_const(1, &mut body);
    WasmOpcode::I32LtU.encode(&mut body);
    encode_if_empty(&mut body);
    encode_i32_const(1, &mut body);
    encode_local_set(2, &mut body);
    WasmOpcode::End.encode(&mut body);

    WasmOpcode::GlobalGet.encode(&mut body);
    encode_uleb128(CABI_HEAP_GLOBAL_INDEX, &mut body);
    encode_local_get(2, &mut body);
    WasmOpcode::I32Add.encode(&mut body);
    encode_i32_const(1, &mut body);
    WasmOpcode::I32Sub.encode(&mut body);
    encode_local_get(2, &mut body);
    encode_i32_const(1, &mut body);
    WasmOpcode::I32Sub.encode(&mut body);
    encode_i32_const(-1, &mut body);
    WasmOpcode::I32Xor.encode(&mut body);
    encode_i32_and(&mut body);
    encode_local_set(4, &mut body);

    encode_local_get(4, &mut body);
    encode_local_get(3, &mut body);
    WasmOpcode::I32Add.encode(&mut body);
    encode_local_tee(5, &mut body);
    encode_local_get(4, &mut body);
    WasmOpcode::I32LtU.encode(&mut body);
    encode_if_empty(&mut body);
    encode_unreachable(&mut body);
    WasmOpcode::End.encode(&mut body);

    encode_local_get(5, &mut body);
    encode_i32_const(65535, &mut body);
    WasmOpcode::I32Add.encode(&mut body);
    encode_i32_const(16, &mut body);
    WasmOpcode::I32ShrU.encode(&mut body);
    encode_memory_size(&mut body);
    WasmOpcode::I32Sub.encode(&mut body);
    encode_local_tee(6, &mut body);
    encode_i32_const(0, &mut body);
    WasmOpcode::I32GtS.encode(&mut body);
    encode_if_empty(&mut body);
    encode_local_get(6, &mut body);
    encode_memory_grow(&mut body);
    encode_i32_const(-1, &mut body);
    WasmOpcode::I32Eq.encode(&mut body);
    encode_if_empty(&mut body);
    encode_unreachable(&mut body);
    WasmOpcode::End.encode(&mut body);
    WasmOpcode::End.encode(&mut body);

    encode_local_get(4, &mut body);
    encode_i32_const(0, &mut body);
    encode_local_get(3, &mut body);
    encode_memory_fill(&mut body);

    encode_local_get(0, &mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    encode_local_get(1, &mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    WasmOpcode::I32Eqz.encode(&mut body);
    encode_i32_and(&mut body);
    encode_if_empty(&mut body);
    encode_local_get(1, &mut body);
    encode_local_get(3, &mut body);
    encode_local_get(1, &mut body);
    encode_local_get(3, &mut body);
    WasmOpcode::I32LtU.encode(&mut body);
    WasmOpcode::Select.encode(&mut body);
    encode_local_set(7, &mut body);
    encode_local_get(4, &mut body);
    encode_local_get(0, &mut body);
    encode_local_get(7, &mut body);
    encode_memory_copy(&mut body);
    WasmOpcode::End.encode(&mut body);

    encode_local_get(5, &mut body);
    WasmOpcode::GlobalSet.encode(&mut body);
    encode_uleb128(CABI_HEAP_GLOBAL_INDEX, &mut body);
    encode_local_get(4, &mut body);
    WasmOpcode::End.encode(&mut body);
    body
}

#[cfg(test)]
mod cabi_realloc_tests {
    use super::{
        CABI_HEAP_DEFAULT_BASE, LINEAR_HEAP_MIN_BASE, align_up_u32, cabi_heap_base_after_data, cabi_heap_global_section,
        wasm_cabi_realloc_bump_body,
    };
    use std_data::binary::wasm::{WasmMiscOpcode, WasmOpcode};

    #[test]
    fn bump_body_is_not_null_stub() {
        let body = wasm_cabi_realloc_bump_body();
        assert!(body.contains(&WasmOpcode::GlobalGet.as_u8()), "global.get");
        assert!(body.windows(2).any(|w| w == [WasmOpcode::MemorySize.as_u8(), 0]), "memory.size");
        assert!(
            body.windows(2).any(|w| w == [WasmOpcode::PrefixMisc.as_u8(), WasmMiscOpcode::MemoryCopy.as_u8()]),
            "memory.copy"
        );
    }

    #[test]
    fn heap_global_initializes_past_static_data() {
        assert_eq!(cabi_heap_base_after_data(0), LINEAR_HEAP_MIN_BASE);
        assert_eq!(cabi_heap_base_after_data(4097), 4104);
        assert_eq!(align_up_u32(5, 4), 8);
        let section = cabi_heap_global_section(CABI_HEAP_DEFAULT_BASE);
        assert_eq!(section.id, 6);
    }

    #[test]
    fn bump_body_checks_overflow_and_zero_fills() {
        let body = wasm_cabi_realloc_bump_body();
        assert!(body.windows(2).any(|w| w == [WasmOpcode::I32LtU.as_u8(), WasmOpcode::If.as_u8()]));
        assert!(body.windows(2).any(|w| w == [WasmOpcode::PrefixMisc.as_u8(), WasmMiscOpcode::MemoryFill.as_u8()]));
    }
}
''',
    encoding="utf-8",
)
print("cabi", (ROOT / "cabi.rs").stat().st_size)

# --- sections.rs patch ---
sec = (ROOT / "sections.rs").read_text(encoding="utf-8")
if "std_data::binary::wasm" not in sec:
    sec = sec.replace(
        "use crate::nyar_backend_wasi::{WasmBinaryModule, WasmSection};",
        """use crate::nyar_backend_wasi::{WasmBinaryModule, WasmSection};
use std_data::binary::wasm::{
    FIELD_MUTABLE, TYPE_FORM_FUNC, VALTYPE_FUNCREF, VALTYPE_I32, WasmExternalKind, WasmOpcode, encode_functype_raw,
    encode_i32_const, encode_uleb128 as std_encode_uleb128, write_sleb128_i32,
};""",
    )
    sec = sec.replace(
        """pub(crate) fn wasm_function_type(params: &[u8], results: &[u8]) -> Vec<u8> {
    let mut bytes = vec![0x60];
    encode_uleb128(u32::try_from(params.len()).unwrap(), &mut bytes);
    bytes.extend_from_slice(params);
    encode_uleb128(u32::try_from(results.len()).unwrap(), &mut bytes);
    bytes.extend_from_slice(results);
    bytes
}""",
        """pub(crate) fn wasm_function_type(params: &[u8], results: &[u8]) -> Vec<u8> {
    encode_functype_raw(params, results)
}""",
    )
    sec = sec.replace(
        "bytes.push(0x00);\n        encode_uleb128(*type_index",
        "bytes.push(WasmExternalKind::Func.as_u8());\n        encode_uleb128(*type_index",
    )
    old_uleb = """pub(crate) fn encode_uleb128(mut value: u32, bytes: &mut Vec<u8>) {
    loop {
        let mut byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

pub(crate) fn encode_sleb128_i32(mut value: i32, bytes: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        bytes.push(if done { byte } else { byte | 0x80 });
        if done {
            break;
        }
    }
}

/// 将 `i64` 编码为 `SLEB128` 字节序列并追加到 `bytes`。
pub(crate) fn encode_sleb128_i64(mut value: i64, bytes: &mut Vec<u8>) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        bytes.push(if done { byte } else { byte | 0x80 });
        if done {
            break;
        }
    }
}"""
    new_uleb = """pub(crate) fn encode_uleb128(value: u32, bytes: &mut Vec<u8>) {
    std_encode_uleb128(value, bytes);
}

pub(crate) fn encode_sleb128_i32(value: i32, bytes: &mut Vec<u8>) {
    write_sleb128_i32(value, bytes);
}

/// 将 `i64` 编码为 `SLEB128` 字节序列并追加到 `bytes`。
pub(crate) fn encode_sleb128_i64(value: i64, bytes: &mut Vec<u8>) {
    std_data::binary::wasm::encode_sleb128_i64(value, bytes);
}"""
    if old_uleb in sec:
        sec = sec.replace(old_uleb, new_uleb)
    sec = sec.replace(
        """    bytes.push(0x00);
    bytes.push(0x41);
    encode_sleb128_i32(i32::try_from(offset).unwrap(), &mut bytes);
    bytes.push(0x0B);""",
        """    bytes.push(0);
    encode_i32_const(i32::try_from(offset).unwrap(), &mut bytes);
    WasmOpcode::End.encode(&mut bytes);""",
    )
    for indent in ("        ", "            "):
        sec = sec.replace(
            f"""{indent}bytes.push(0x7F);
{indent}bytes.push(0x01);
{indent}bytes.push(0x41);
{indent}encode_sleb128_i32(init, &mut bytes);
{indent}bytes.push(0x0B);""",
            f"""{indent}bytes.push(VALTYPE_I32);
{indent}bytes.push(FIELD_MUTABLE);
{indent}encode_i32_const(init, &mut bytes);
{indent}WasmOpcode::End.encode(&mut bytes);""",
        )
    sec = sec.replace("bytes.push(0x70);", "bytes.push(VALTYPE_FUNCREF);")
    sec = sec.replace(
        """    bytes.push(0x41);
    encode_sleb128_i32(0, &mut bytes);
    bytes.push(0x0B);""",
        """    encode_i32_const(0, &mut bytes);
    WasmOpcode::End.encode(&mut bytes);""",
    )
    if "TYPE_FORM_FUNC" in sec and "_WASM_TYPE_FORM_FUNC" not in sec:
        sec += "\n#[allow(dead_code)]\npub(crate) const _WASM_TYPE_FORM_FUNC: u8 = TYPE_FORM_FUNC;\n"
    (ROOT / "sections.rs").write_text(sec, encoding="utf-8")
    print("sections patched")
else:
    print("sections already patched")

print("gc", "std_data" in (ROOT / "gc.rs").read_text(encoding="utf-8"))
print("cabi", "std_data" in (ROOT / "cabi.rs").read_text(encoding="utf-8"))
print("sections", "std_data" in (ROOT / "sections.rs").read_text(encoding="utf-8"))

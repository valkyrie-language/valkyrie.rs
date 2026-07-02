#!/usr/bin/env python3
"""Careful mir/mod.rs migration after emit helpers already use std-data."""
from __future__ import annotations

import re
from pathlib import Path

path = Path(r"E:\Goddess of Victory\valkyrie.rs\projects\nyar-emitter\src\lowering\backends\wasm\mir\mod.rs")
text = path.read_text(encoding="utf-8")

# Valtypes
for pat, repl in [
    (r"\b0x7F\b", "VALTYPE_I32"),
    (r"\b0x7E\b", "VALTYPE_I64"),
    (r"\b0x7C\b", "VALTYPE_F64"),
    (r"\b0x6E\b", "VALTYPE_ANYREF"),
    (r"\b0x6F\b", "VALTYPE_EXTERNREF"),
    (r"\b0x64\b", "VALTYPE_REF"),
]:
    text = re.sub(pat, repl, text)

# Safe leaf opcodes only
safe_opcodes = {
    0x03: "WasmOpcode::Loop",
    0x04: "WasmOpcode::If",
    0x05: "WasmOpcode::Else",
    0x0C: "WasmOpcode::Br",
    0x0D: "WasmOpcode::BrIf",
    0x10: "WasmOpcode::Call",
    0x11: "WasmOpcode::CallIndirect",
    0x1A: "WasmOpcode::Drop",
    0x20: "WasmOpcode::LocalGet",
    0x21: "WasmOpcode::LocalSet",
    0x22: "WasmOpcode::LocalTee",
    0x23: "WasmOpcode::GlobalGet",
    0x24: "WasmOpcode::GlobalSet",
    0x41: "WasmOpcode::I32Const",
    0x42: "WasmOpcode::I64Const",
    0x44: "WasmOpcode::F64Const",
    0x45: "WasmOpcode::I32Eqz",
    0x46: "WasmOpcode::I32Eq",
    0x47: "WasmOpcode::I32Ne",
    0x48: "WasmOpcode::I32LtS",
    0x49: "WasmOpcode::I32LtU",
    0x4A: "WasmOpcode::I32GtS",
    0x4C: "WasmOpcode::I32LeS",
    0x4D: "WasmOpcode::I32LeU",
    0x4E: "WasmOpcode::I32GeS",
    0x6A: "WasmOpcode::I32Add",
    0x6B: "WasmOpcode::I32Sub",
    0x71: "WasmOpcode::I32And",
    0x72: "WasmOpcode::I32Or",
    0x73: "WasmOpcode::I32Xor",
    0x74: "WasmOpcode::I32Shl",
    0x75: "WasmOpcode::I32ShrS",
    0x76: "WasmOpcode::I32ShrU",
    0xA0: "WasmOpcode::F64Add",
    0xA1: "WasmOpcode::F64Sub",
    0xA2: "WasmOpcode::F64Mul",
    0xA3: "WasmOpcode::F64Div",
    0xA4: "WasmOpcode::F64Min",
    0xA7: "WasmOpcode::I32WrapI64",
    0xAA: "WasmOpcode::I32TruncF64S",
    0xB7: "WasmOpcode::F64ConvertI32S",
    0xD0: "WasmOpcode::RefNull",
    0xD1: "WasmOpcode::RefIsNull",
}


def repl_push(m: re.Match[str]) -> str:
    byte = int(m.group(1), 16)
    if byte == 0x40:
        return "self.code.push(BLOCKTYPE_EMPTY)"
    if byte in safe_opcodes:
        return f"{safe_opcodes[byte]}.encode(&mut self.code)"
    return m.group(0)


text = re.sub(r"self\.code\.push\((0x[0-9A-Fa-f]{2})\)", repl_push, text)

# Instruction-context-only replacements (helpers already migrated)
for old, new in [
    ("self.code.push(0x00)", "encode_unreachable(&mut self.code)"),
    ("self.code.push(0x0B)", "WasmOpcode::End.encode(&mut self.code)"),
    ("self.code.push(0x0F)", "encode_return(&mut self.code)"),
    ("self.code.push(0x02)", "WasmOpcode::Block.encode(&mut self.code)"),
]:
    text = text.replace(old, new)

for byte, name in [
    (0x46, "I32Eq"),
    (0x47, "I32Ne"),
    (0x48, "I32LtS"),
    (0x4C, "I32LeS"),
    (0x4A, "I32GtS"),
    (0x4E, "I32GeS"),
    (0x71, "I32And"),
    (0x72, "I32Or"),
    (0x73, "I32Xor"),
    (0x74, "I32Shl"),
    (0x75, "I32ShrS"),
    (0xA0, "F64Add"),
    (0xA1, "F64Sub"),
    (0xA2, "F64Mul"),
    (0xA3, "F64Div"),
    (0xA4, "F64Min"),
]:
    text = text.replace(f"=> 0x{byte:02X}", f"=> WasmOpcode::{name}.as_u8()")
    text = text.replace(f"=> 0x{byte:02x}", f"=> WasmOpcode::{name}.as_u8()")

text = re.sub(
    r"exports\.push\(\(([^,]+),\s*0x00,",
    r"exports.push((\1, WasmExternalKind::Func.as_u8(),",
    text,
)
text = text.replace('exports.push(("memory", 0x02,', 'exports.push(("memory", WasmExternalKind::Memory.as_u8(),')
text = text.replace("Some(&0x60)", "Some(&std_data::binary::wasm::TYPE_FORM_FUNC)")
text = text.replace("body.push(0x00);", "body.push(0);")
text = text.replace("body.push(0x41);", "WasmOpcode::I32Const.encode(&mut body);")
text = text.replace("body.push(0x10);", "WasmOpcode::Call.encode(&mut body);")
text = text.replace("body.push(0x0B);", "WasmOpcode::End.encode(&mut body);")
text = text.replace("vec![0x00, 0x0B]", "vec![WasmOpcode::Unreachable.as_u8(), WasmOpcode::End.as_u8()]")

# extend_from_slice for ref.null anyref etc
text = text.replace(
    "self.code.extend_from_slice(&[0xD0, WASM_GC_ANYREF])",
    "encode_ref_null_anyref(&mut self.code)",
)
text = text.replace(
    "self.code.extend_from_slice(&[WasmOpcode::RefNull.as_u8(), WASM_GC_ANYREF])",
    "encode_ref_null_anyref(&mut self.code)",
)
# After valtype replace D0 path may use VALTYPE_ANYREF
text = text.replace(
    "self.code.extend_from_slice(&[0xD0, VALTYPE_ANYREF])",
    "encode_ref_null_anyref(&mut self.code)",
)

path.write_text(text, encoding="utf-8")
remaining = sorted(set(re.findall(r"0x[0-9A-Fa-f]+", text)))
print("remaining unique hex:", remaining)
print("count:", len(re.findall(r"0x[0-9A-Fa-f]+", text)))

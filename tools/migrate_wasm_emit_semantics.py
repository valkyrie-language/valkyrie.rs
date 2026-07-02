//! Migrate nyar-emitter wasm backend off raw opcode/valtype hex literals.
//! Run from repo: python tools/migrate_wasm_emit_semantics.py

from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(r"E:\Goddess of Victory\valkyrie.rs\projects\nyar-emitter\src\lowering\backends\wasm")

# Opcode byte -> WasmOpcode variant name (exact single-byte pushes of instructions)
OPCODE_MAP = {
    0x00: "Unreachable",
    0x01: "Nop",
    0x02: "Block",
    0x03: "Loop",
    0x04: "If",
    0x05: "Else",
    0x0B: "End",
    0x0C: "Br",
    0x0D: "BrIf",
    0x0E: "BrTable",
    0x0F: "Return",
    0x10: "Call",
    0x11: "CallIndirect",
    0x1A: "Drop",
    0x1B: "Select",
    0x20: "LocalGet",
    0x21: "LocalSet",
    0x22: "LocalTee",
    0x23: "GlobalGet",
    0x24: "GlobalSet",
    0x28: "I32Load",
    0x29: "I64Load",
    0x2A: "F32Load",
    0x36: "I32Store",
    0x37: "I64Store",
    0x39: "F64Store",
    0x3F: "MemorySize",
    0x40: "MemoryGrow",  # careful: also BLOCKTYPE_EMPTY / empty locals marker contexts
    0x41: "I32Const",
    0x42: "I64Const",
    0x44: "F64Const",
    0x45: "I32Eqz",
    0x46: "I32Eq",
    0x47: "I32Ne",
    0x48: "I32LtS",
    0x49: "I32LtU",
    0x4A: "I32GtS",
    0x4C: "I32LeS",
    0x4D: "I32LeU",
    0x4E: "I32GeS",
    0x6A: "I32Add",
    0x6B: "I32Sub",
    0x71: "I32And",
    0x72: "I32Or",
    0x73: "I32Xor",
    0x74: "I32Shl",
    0x75: "I32ShrS",
    0x76: "I32ShrU",
    0xA0: "F64Add",
    0xA1: "F64Sub",
    0xA2: "F64Mul",
    0xA3: "F64Div",
    0xA4: "F64Min",
    0xA7: "I32WrapI64",
    0xAA: "I32TruncF64S",
    0xB7: "F64ConvertI32S",
    0xD0: "RefNull",
    0xD1: "RefIsNull",
}

VALTYPE_MAP = {
    0x7F: "VALTYPE_I32",
    0x7E: "VALTYPE_I64",
    0x7C: "VALTYPE_F64",
    0x6E: "VALTYPE_ANYREF",
    0x6F: "VALTYPE_EXTERNREF",
    0x64: "VALTYPE_REF",
    0x70: "VALTYPE_FUNCREF",  # may need to add
}

# Files to migrate (production sources only; tests handled separately)
FILES = [
    ROOT / "gc.rs",
    ROOT / "sections.rs",
    ROOT / "host_imports.rs",
    ROOT / "host" / "js_glue.rs",
    ROOT / "host" / "wasi_cm.rs",
    ROOT / "mir" / "mod.rs",
    ROOT / "cabi.rs",
    ROOT / "suspend.rs",
]


def replace_code_push(content: str) -> str:
    def repl(m: re.Match[str]) -> str:
        byte = int(m.group(1), 16)
        if byte in OPCODE_MAP and byte != 0x40:
            # 0x40 is ambiguous (MemoryGrow vs blocktype empty)
            return f"WasmOpcode::{OPCODE_MAP[byte]}.encode(&mut self.code)"
        if byte == 0x40:
            return "self.code.push(BLOCKTYPE_EMPTY)"
        if byte == 0xFB:
            return "WasmOpcode::PrefixGc.encode(&mut self.code)"
        if byte == 0xFC:
            return "WasmOpcode::PrefixMisc.encode(&mut self.code)"
        if byte in VALTYPE_MAP:
            return f"self.code.push({VALTYPE_MAP[byte]})"
        if byte in (0x5E, 0x5F, 0x60):
            forms = {0x5E: "TYPE_FORM_ARRAY", 0x5F: "TYPE_FORM_STRUCT", 0x60: "TYPE_FORM_FUNC"}
            return f"self.code.push({forms[byte]})"
        if byte in (0x00, 0x01) and False:
            pass
        # Keep raw for unknown — mark for manual
        return m.group(0)

    # self.code.push(0xNN);
    content = re.sub(
        r"self\.code\.push\((0x[0-9A-Fa-f]{2})\)",
        lambda m: repl(m).rstrip(")") + ")" if False else (
            (lambda r: r if r.startswith("self.code.push") or r.startswith("WasmOpcode") else r)(
                (lambda byte: (
                    f"WasmOpcode::{OPCODE_MAP[byte]}.encode(&mut self.code)"
                    if byte in OPCODE_MAP and byte != 0x40
                    else (
                        "self.code.push(BLOCKTYPE_EMPTY)"
                        if byte == 0x40
                        else (
                            f"self.code.push({VALTYPE_MAP[byte]})"
                            if byte in VALTYPE_MAP
                            else (
                                f"WasmOpcode::PrefixGc.encode(&mut self.code)"
                                if byte == 0xFB
                                else (
                                    f"WasmOpcode::PrefixMisc.encode(&mut self.code)"
                                    if byte == 0xFC
                                    else m.group(0)
                                )
                            )
                        )
                    )
                ))(int(m.group(1), 16))
            )
        ),
        content,
    )
    return content


def main() -> None:
    print("This helper is informational; prefer hand-crafted migration for correctness.")
    for path in FILES:
        print(path, "exists" if path.exists() else "MISSING")


if __name__ == "__main__":
    main()

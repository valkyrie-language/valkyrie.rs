"""Scrub wasm test assertions from raw opcode hex to std-data semantics."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(r"E:\Goddess of Victory\valkyrie.rs\projects\nyar-emitter\src\lowering\backends\wasm")

IMPORT_MIR = (
    "use std_data::binary::wasm::{TYPE_FORM_ARRAY, TYPE_FORM_STRUCT, VALTYPE_I32, VALTYPE_REF, "
    "WasmGcOpcode, WasmMiscOpcode, WasmOpcode};"
)
IMPORT_TESTS = (
    "use std_data::binary::wasm::{WasmGcOpcode, WasmMiscOpcode, WasmOpcode, WasmExternalKind};"
)

# Common assertion replacements (order matters for longer patterns first)
REPLACEMENTS = [
    ("[0x20, 0x00]", "[WasmOpcode::LocalGet.as_u8(), 0]"),
    ("[0x20, 0x01, 0x28, 0x02]", "[WasmOpcode::LocalGet.as_u8(), 1, WasmOpcode::I32Load.as_u8(), 2]"),
    ("[0x21, 0x00, 0x0C, 0x01]", "[WasmOpcode::LocalSet.as_u8(), 0, WasmOpcode::Br.as_u8(), 1]"),
    ("[0x41, 0x04, 0x28, 0x02, 0x00, 0x11]", "[WasmOpcode::I32Const.as_u8(), 4, WasmOpcode::I32Load.as_u8(), 2, 0, WasmOpcode::CallIndirect.as_u8()]"),
    ("[0x00, 0x41, 0x00, 0x0B]", "[WasmOpcode::Unreachable.as_u8(), WasmOpcode::I32Const.as_u8(), 0, WasmOpcode::End.as_u8()]"),
    ("[0x3F, 0x00]", "[WasmOpcode::MemorySize.as_u8(), 0]"),
    ("[0xFC, 0x0A]", "[WasmOpcode::PrefixMisc.as_u8(), WasmMiscOpcode::MemoryCopy.as_u8()]"),
    ("*byte == 0x20", "*byte == WasmOpcode::LocalGet.as_u8()"),
    ("*byte == 0x11", "*byte == WasmOpcode::CallIndirect.as_u8()"),
    ("**byte == 0x11", "**byte == WasmOpcode::CallIndirect.as_u8()"),
    ("*byte == 0x0B", "*byte == WasmOpcode::End.as_u8()"),
    ("*byte == 0x21", "*byte == WasmOpcode::LocalSet.as_u8()"),
    ("contains(&0x0E)", "contains(&WasmOpcode::BrTable.as_u8())"),
    ("contains(&0x36)", "contains(&WasmOpcode::I32Store.as_u8())"),
    ("contains(&0x45)", "contains(&WasmOpcode::I32Eqz.as_u8())"),
    ("contains(&0x04)", "contains(&WasmOpcode::If.as_u8())"),
    ("contains(&0x11)", "contains(&WasmOpcode::CallIndirect.as_u8())"),
    ("contains(&0x1A)", "contains(&WasmOpcode::Drop.as_u8())"),
    ("contains(&0x05)", "contains(&WasmOpcode::Else.as_u8())"),
    ("contains(&0x0B)", "contains(&WasmOpcode::End.as_u8())"),
    ("contains(&0x23)", "contains(&WasmOpcode::GlobalGet.as_u8())"),
    ("assert_eq!(kind, 0x00)", "assert_eq!(kind, WasmExternalKind::Func.as_u8())"),
]


def scrub(path: Path, import_line: str | None) -> None:
    text = path.read_text(encoding="utf-8")
    if import_line and "WasmOpcode" not in text.split("fn ", 1)[0]:
        # insert after first use block roughly
        if "use super::" in text:
            text = text.replace("use super::", import_line + "\nuse super::", 1)
        elif "use crate::" in text:
            text = text.replace("use crate::", import_line + "\nuse crate::", 1)
    for old, new in REPLACEMENTS:
        text = text.replace(old, new)
    path.write_text(text, encoding="utf-8")
    left = sorted(set(re.findall(r"0x[0-9A-Fa-f]{2}", "\n".join(l for l in text.splitlines() if not l.strip().startswith("//") and '\"' not in l[:20]))))
    # count hex in non-comment, non-string-heavy lines in asserts
    code_hex = []
    for line in text.splitlines():
        s = line.strip()
        if s.startswith("//") or s.startswith("///"):
            continue
        if "0x" in s and ("assert" in s or "window" in s or "contains" in s or "position" in s or "filter" in s):
            code_hex.extend(re.findall(r"0x[0-9A-Fa-f]+", s))
    print(path.name, "assert-hex", sorted(set(code_hex)))


scrub(ROOT / "mir" / "tests.rs", None)  # already has import
scrub(ROOT / "tests.rs", IMPORT_TESTS)
print("done")

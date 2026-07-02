#!/usr/bin/env python3
"""Strip God fields from Call { ... } in Rust (ADR 0010 purge), including shorthand `field,`."""
from __future__ import annotations

import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / "projects"
KILL = {
    "dispatch",
    "witness",
    "evidence",
    "generic_function",
    "generic_arguments",
    "effect",
    "receiver_kind",
    "parameter_types",
    "intrinsic_opcode",
    "signature_complete",
    "has_this",
    "operands",
    "return_type",
    "param_types",
}

START = re.compile(
    r"(MirInstructionKind::Call|InstructionKind::Call|ExecutableInstructionKind::Call|crate::contracts::InstructionKind::Call|nyar_language::valkyrie::mir::MirInstructionKind::Call)\s*\{"
)


def find_matching_brace(text: str, open_idx: int) -> int:
    depth = 0
    i = open_idx
    while i < len(text):
        c = text[i]
        if c == "{":
            depth += 1
        elif c == "}":
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def slim_body(body: str) -> str:
    # Token-oriented: remove `name:` values and shorthand `name` / `name,` for KILL names.
    # Work line-by-line for readability; also scrub inline patterns.
    lines_out = []
    for line in body.splitlines(True):
        # Drop whole line if it only introduces a kill field (shorthand or keyed).
        if re.match(rf"^\s*({'|'.join(KILL)})\s*(:|,|\s*$)", line):
            continue
        # Inline: remove `kill,` or `kill: expr,` fragments carefully — for match patterns on one line.
        cleaned = line
        for f in KILL:
            cleaned = re.sub(rf"\b{f}\s*:\s*[^,}}]+,?\s*", "", cleaned)
            cleaned = re.sub(rf"\b{f}\s*,\s*", "", cleaned)
            cleaned = re.sub(rf",\s*{f}\b", "", cleaned)
            cleaned = re.sub(rf"\b{f}\b\s*", "", cleaned)
        # Fix double commas / trailing commas before }
        cleaned = re.sub(r",\s*,+", ", ", cleaned)
        cleaned = re.sub(r"\{\s*,", "{ ", cleaned)
        cleaned = re.sub(r",\s*\}", " }", cleaned)
        if cleaned.strip() in {"", ",", "}"}:
            continue
        lines_out.append(cleaned)
    return "".join(lines_out)


def process(text: str) -> tuple[str, int]:
    n = 0
    out = []
    i = 0
    while True:
        m = START.search(text, i)
        if not m:
            out.append(text[i:])
            break
        out.append(text[i : m.start()])
        brace = text.find("{", m.start())
        end = find_matching_brace(text, brace)
        if end < 0:
            out.append(text[m.start() :])
            break
        head = text[m.start() : brace + 1]
        body = text[brace + 1 : end]
        new_body = slim_body(body)
        out.append(head)
        out.append(new_body)
        out.append("}")
        n += 1
        i = end + 1
    return "".join(out), n


def main() -> None:
    total = 0
    for path in ROOT.rglob("*.rs"):
        try:
            text = path.read_bytes().decode("utf-8")
        except UnicodeDecodeError:
            continue
        if "::Call" not in text:
            continue
        new, n = process(text)
        if n and new != text:
            path.write_text(new, encoding="utf-8", newline="\n")
            print(f"{n:3} {path.relative_to(ROOT)}")
            total += n
    print("total", total)


if __name__ == "__main__":
    main()

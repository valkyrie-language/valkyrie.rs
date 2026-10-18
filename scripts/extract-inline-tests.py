#!/usr/bin/env python3
"""Move `#[cfg(test)] mod ...` blocks from crate src/ into tests/."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def crate_name(toml: Path) -> str:
    text = toml.read_text(encoding="utf-8")
    match = re.search(r'^name\s*=\s*"([^"]+)"', text, re.M)
    if not match:
        raise SystemExit(f"missing package.name in {toml}")
    return match.group(1).replace("-", "_")


def find_cfg_test_mods(content: str) -> list[tuple[int, int, str, str]]:
    """Return (start, end, mod_name, inner_body) for each #[cfg(test)] mod block."""
    pattern = re.compile(r"#\[cfg\(test\)\]\s*\n\s*mod\s+([A-Za-z0-9_]+)\s*\{", re.M)
    results: list[tuple[int, int, str, str]] = []
    for match in pattern.finditer(content):
        mod_name = match.group(1)
        open_brace = match.end() - 1
        depth = 0
        i = open_brace
        while i < len(content):
            ch = content[i]
            if ch == "{":
                depth += 1
            elif ch == "}":
                depth -= 1
                if depth == 0:
                    end = i + 1
                    inner = content[match.end() : i]
                    start = match.start()
                    results.append((start, end, mod_name, inner))
                    break
            i += 1
    return results


def rewrite_imports(inner: str, rust_crate: str) -> str:
    lines = inner.splitlines()
    out: list[str] = []
    for line in lines:
        stripped = line.strip()
        if stripped == "use super::*;" or stripped.startswith("use super::"):
            continue
        out.append(line)
    header = [
        f"use {rust_crate}::*;",
        "",
    ]
    body = "\n".join(out).strip("\n")
    return "\n".join(header) + ("\n\n" + body if body else "")


def process_file(src_path: Path, src_dir: Path, crate_dir: Path, tests_dir: Path, rust_crate: str, dry_run: bool) -> int:
    content = src_path.read_text(encoding="utf-8")
    blocks = find_cfg_test_mods(content)
    if not blocks:
        return 0

    rel = src_path.relative_to(src_dir)
    stem = rel.with_suffix("").name
    parent = rel.parent.as_posix().replace("/", "_")
    if parent == ".":
        parent = ""

    moved = 0
    new_content = content
    for start, end, mod_name, inner in reversed(blocks):
        suffix = mod_name if mod_name != "tests" else ""
        name_parts = [p for p in (parent, stem, suffix) if p]
        out_name = "_".join(name_parts) + ".rs"
        out_path = tests_dir / out_name
        transformed = rewrite_imports(inner, rust_crate)
        header = f"//! Migrated from `{src_path.relative_to(crate_dir).as_posix()}` (`{mod_name}`).\n\n"
        if not dry_run:
            out_path.parent.mkdir(parents=True, exist_ok=True)
            out_path.write_text(header + transformed + "\n", encoding="utf-8")
            new_content = new_content[:start] + new_content[end:]
        moved += 1
        print(f"  -> {out_path.name}")

    if moved and not dry_run:
        cleaned = re.sub(r"\n{3,}", "\n\n", new_content.rstrip()) + "\n"
        src_path.write_text(cleaned, encoding="utf-8")

    return moved


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate", required=True, help="crate directory name under projects/compilers")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    crate_dir = ROOT / "projects" / "compilers" / args.crate
    src_dir = crate_dir / "src"
    tests_dir = crate_dir / "tests"
    rust_crate = crate_name(crate_dir / "Cargo.toml")

    total = 0
    for src_path in sorted(src_dir.rglob("*.rs")):
        blocks = find_cfg_test_mods(src_path.read_text(encoding="utf-8"))
        if not blocks:
            continue
        print(src_path.relative_to(crate_dir))
        total += process_file(src_path, src_dir, crate_dir, tests_dir, rust_crate, args.dry_run)

    print(f"{'would move' if args.dry_run else 'moved'} {total} test module(s) in {args.crate}")


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""Wire migrated tests/ files back into src via #[path] (autotests=false)."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]

MARKER = re.compile(r"^//! Migrated from `(?P<src>[^`]+)` \(`(?P<mod>[^`]+)`\)\.\s*$", re.M)


def rel_path(from_file: Path, to_file: Path) -> str:
    return Path(
        *Path(
            Path(from_file).parent.relative_to(from_file.parent.anchor)
            if False
            else []
        )
    )
    # pathlib relative_to across same tree
    rel = Path(
        *[
            p
            for p in Path(to_file).relative_to(from_file.parent).parts
        ]
    )
    return rel.as_posix()


def compute_rel(from_src: Path, test_file: Path) -> str:
    return Path(os_relpath(from_src.parent, test_file)).as_posix()


def os_relpath(start: Path, end: Path) -> str:
    import os

    return os.path.relpath(end, start).replace("\\", "/")


def rewrite_test_body(text: str, rust_crate: str) -> str:
    lines = text.splitlines()
    # strip header through first blank after marker
    idx = 0
    if lines and lines[0].startswith("//! Migrated"):
        idx = 1
        while idx < len(lines) and lines[idx].strip() == "":
            idx += 1
    # drop crate-root imports injected by migration
    body: list[str] = []
    for line in lines[idx:]:
        stripped = line.strip()
        if stripped.startswith(f"use {rust_crate}::") or stripped == f"use {rust_crate}::*;":
            continue
        if stripped.startswith("use std::fs;") or stripped.startswith("use std::path::Path;"):
            continue
        if stripped.startswith("use miette::Result;"):
            continue
        line = line.replace(f"{rust_crate}::", "super::")
        body.append(line)
    while body and body[0].strip() == "":
        body.pop(0)
    prelude = ["use super::*;", ""]
    return "\n".join(prelude + body).rstrip() + "\n"


def wire_test(src_path: Path, test_path: Path, mod_name: str, dry_run: bool) -> None:
    rel = os_relpath(src_path.parent, test_path)
    hook = (
        f"\n#[cfg(test)]\n#[path = \"{rel}\"]\nmod {mod_name};\n"
    )
    src_text = src_path.read_text(encoding="utf-8")
    if hook.strip() in src_text:
        return
    if not dry_run:
        src_path.write_text(src_text.rstrip() + hook, encoding="utf-8")


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--crate", required=True)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    crate_dir = ROOT / "projects" / "compilers" / args.crate
    rust_crate = args.crate.replace("-", "_")
    tests_dir = crate_dir / "tests"
    src_dir = crate_dir / "src"

    for test_path in sorted(tests_dir.glob("*.rs")):
        text = test_path.read_text(encoding="utf-8")
        match = MARKER.search(text)
        if not match:
            continue
        rel_src = match.group("src")
        mod_tag = match.group("mod")
        src_path = crate_dir / rel_src
        if not src_path.is_file():
            raise SystemExit(f"missing source for {test_path}: {src_path}")

        mod_name = mod_tag if mod_tag != "tests" else f"{test_path.stem}_tests"
        new_body = (
            f"//! Migrated from `{rel_src}` (`{mod_tag}`).\n\n"
            + rewrite_test_body(text, rust_crate)
        )
        if not args.dry_run:
            test_path.write_text(new_body, encoding="utf-8")
            wire_test(src_path, test_path, mod_name, dry_run=False)
        print(f"{rel_src} <- {test_path.name} ({mod_name})")


if __name__ == "__main__":
    main()

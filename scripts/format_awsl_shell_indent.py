#!/usr/bin/env python3
"""Normalize indentation inside AWSL shell blocks (widget/template/style/script)."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
SHELL_TAGS = ('widget', 'template', 'style', 'script')
INDENT = '    '

OPEN_TAG = re.compile(r'^<([A-Za-z_][\w.-]*)(?:\s[^>]*)?>$')
SELF_CLOSE = re.compile(r'/>$')
CLOSE_TAG = re.compile(r'^</([A-Za-z_][\w.-]*)>$')
RAW_LINE = re.compile(r'^[^{<\s/]')


def format_html_lines(lines: list[str]) -> list[str]:
    depth = 0
    out: list[str] = []
    for raw in lines:
        stripped = raw.strip()
        if not stripped:
            out.append('')
            continue

        closing = CLOSE_TAG.match(stripped)
        if closing:
            depth = max(0, depth - 1)
            out.append(INDENT * depth + stripped)
            continue

        out.append(INDENT * depth + stripped)

        if SELF_CLOSE.search(stripped):
            continue
        if OPEN_TAG.match(stripped):
            depth += 1
    return out


def format_script_lines(lines: list[str]) -> list[str]:
    """Preserve blank lines; normalize 4-space dedent inside script/style."""
    if not any(line.strip() for line in lines):
        return ['']
    min_indent = None
    stripped_lines = [line.rstrip() for line in lines]
    for line in stripped_lines:
        if not line.strip():
            continue
        leading = len(line) - len(line.lstrip(' '))
        min_indent = leading if min_indent is None else min(min_indent, leading)
    base = min_indent or 0
    out: list[str] = []
    for line in stripped_lines:
        if not line.strip():
            out.append('')
            continue
        if line.startswith(' ' * base):
            out.append(line[base:])
        else:
            out.append(line.lstrip())
    return out


def format_shell_body(tag: str, body: str) -> str:
    lines = body.split('\n')
    if lines and not lines[0].strip():
        lines = lines[1:]
    if lines and not lines[-1].strip():
        lines = lines[:-1]
    if tag in ('script', 'style'):
        formatted = format_script_lines(lines)
    else:
        formatted = format_html_lines(lines)
    if not formatted:
        return '\n'
    return '\n' + '\n'.join(formatted) + '\n'


def format_awsl(text: str) -> str:
    for tag in SHELL_TAGS:
        pattern = re.compile(
            rf'(<{tag}(?:\s[^>]*)?>)(.*?)(</{tag}>)',
            re.DOTALL | re.IGNORECASE,
        )

        def repl(match: re.Match[str]) -> str:
            return match.group(1) + format_shell_body(tag.lower(), match.group(2)) + match.group(3)

        text = pattern.sub(repl, text)
    return text


def main() -> int:
    roots = [ROOT] if len(sys.argv) == 1 else [Path(p) for p in sys.argv[1:]]
    count = 0
    for root in roots:
        for path in root.rglob('*.awsl'):
            if any(part in path.parts for part in ('target', 'node_modules', 'build', '.intellijPlatform')):
                continue
            original = path.read_text(encoding='utf-8')
            migrated = format_awsl(original)
            if migrated != original:
                path.write_text(migrated, encoding='utf-8', newline='\n')
                print(f'updated: {path}')
                count += 1
    print(f'done: {count} files')
    return 0


if __name__ == '__main__':
    raise SystemExit(main())

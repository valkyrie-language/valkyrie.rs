#!/usr/bin/env python3
"""Migrate .awsl files to quoted DSL (:bind/@click) + snake_case widget + flat widget/script/style."""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]

EVENT_ATTRS = {
    "click", "submit", "input", "change", "keydown", "keyup", "focus", "blur",
}

DIRECTIVE_ATTRS = {"if", "loop", "style", "class", "bind", "ref"}


def stem_to_widget(stem: str) -> str:
    base = stem.strip().strip("[]")
    if not base:
        return "component"
    return base.replace("-", "_").lower()


def migrate_bindings(text: str) -> str:
    # bind:value={x} / bind:value="x"
    text = re.sub(
        r'\bbind:([a-zA-Z_][\w.-]*)=\{([^}]+)\}',
        r':bind="\2"',
        text,
    )
    text = re.sub(
        r'\bon:([a-zA-Z_][\w.-]*)=([^\s/>"]+)',
        lambda m: f'@click="{m.group(2)}"' if m.group(1) == "click" else f'@on_{m.group(1)}="{m.group(2)}"',
        text,
    )
    # bare @click=handler
    text = re.sub(
        r'@([a-zA-Z_][\w]*)\s*=\s*([^\s"\'<>/]+)(?=[\s/>])',
        r'@\1="\2"',
        text,
    )
    # KV braced props: title={title} on PascalCase-ish lines
    text = re.sub(
        r'(\s)([a-zA-Z_][\w.-]*)=\{([^}]+)\}',
        r'\1:\2="\3"',
        text,
    )
    # bare component props: title=title (identifier rhs, not quoted)
    text = re.sub(
        r'(\s)([a-z_][\w.-]*)=([a-zA-Z_][\w.]*)',
        lambda m: m.group(0)
        if m.group(2) in ("type", "class", "id", "href", "src", "alt", "placeholder", "label", "variant", "size", "name", "namespace")
        and '"' not in m.group(0)
        else f'{m.group(1)}:{m.group(2)}="{m.group(3)}"'
        if m.group(3)
        and not m.group(3).startswith('"')
        and m.group(2) not in EVENT_ATTRS
        else m.group(0),
        text,
    )
    # style="...{expr}..." -> @style="f\"...\""
    def style_interpolation(m: re.Match[str]) -> str:
        inner = m.group(1).replace('"', '\\"')
        return f'@style="f\\"{inner}\\""'

    text = re.sub(r'\bstyle="([^"]*\{[^"]+\}[^"]*)"', style_interpolation, text)
    # condition={x} on control tags
    text = re.sub(r'\bcondition=\{([^}]+)\}', r'condition="\1"', text)
    text = re.sub(r'\bcondition=([a-zA-Z_][\w.]*)', r'condition="\1"', text)
    return text


def fix_widget_tag(text: str, widget_name: str) -> str:
    text = re.sub(
        r'<widget(?:\s+[A-Za-z][\w.-]*)?>',
        f'<widget {widget_name}>',
        text,
        count=1,
    )
    text = re.sub(r'<widget\s*>', f'<widget {widget_name}>', text, count=1)
    return text


def dedent_block(text: str, open_tag: str, close_tag: str) -> str:
    pattern = re.compile(
        rf'({re.escape(open_tag)})(.*?)({re.escape(close_tag)})',
        re.DOTALL | re.IGNORECASE,
    )

    def _dedent(m: re.Match[str]) -> str:
        body = m.group(2)
        lines = body.splitlines()
        if not lines:
            return m.group(0)
        indents = [len(line) - len(line.lstrip(" ")) for line in lines if line.strip()]
        if not indents:
            return m.group(0)
        cut = min(indents)
        if cut <= 0:
            return m.group(0)
        new_lines = []
        for line in lines:
            if line.strip():
                new_lines.append(line[cut:] if len(line) >= cut else line)
            else:
                new_lines.append("")
        return m.group(1) + "\n".join(new_lines) + m.group(3)

    return pattern.sub(_dedent, text)


def migrate_file(path: Path) -> bool:
    original = path.read_text(encoding="utf-8")
    widget = stem_to_widget(path.stem)
    text = original
    text = fix_widget_tag(text, widget)
    text = migrate_bindings(text)
    text = dedent_block(text, f"<widget {widget}>", "</widget>")
    # script/style 保留块内缩进（CSS 规则体、micro 函数体）
    # collapse excessive blank lines inside widget shell
    text = re.sub(r"\n{3,}", "\n\n", text)
    if text != original:
        path.write_text(text, encoding="utf-8", newline="\n")
        return True
    return False


def main() -> int:
    roots = [ROOT / "valkyrie.v", ROOT / "valkyrie.rs", ROOT / "intellij-awsl", ROOT / "demo.android"]
    changed = 0
    for root in roots:
        if not root.exists():
            continue
        for path in root.rglob("*.awsl"):
            if migrate_file(path):
                print(path.relative_to(ROOT))
                changed += 1
    print(f"migrated {changed} files")
    return 0


if __name__ == "__main__":
    sys.exit(main())

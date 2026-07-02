from pathlib import Path
import re

root = Path(r"E:\Goddess of Victory\valkyrie.rs\projects\nyar-emitter\src\lowering\backends\wasm")
for p in sorted(root.rglob("*.rs")):
    if p.name.endswith("tests.rs"):
        continue
    text = p.read_text(encoding="utf-8")
    lines = [l for l in text.splitlines() if not l.strip().startswith("//")]
    code = "\n".join(lines)
    hexes = re.findall(r"0x[0-9A-Fa-f]+", code)
    if hexes:
        print(f"{p.relative_to(root)}: {len(hexes)} -> {sorted(set(hexes))}")

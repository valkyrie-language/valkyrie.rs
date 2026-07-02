# @valkyrie-language/legion

Legion compiler CLI for Node.js (WebAssembly).

**Requirements:** Node.js 20+

## Install

```bash
npm install @valkyrie-language/legion
```

```bash
pnpm add @valkyrie-language/legion
```

## Usage

```bash
legion --version
legion --help
legion build <project-dir> --target node -o dist/out
```

### Options (`build`)

| Flag | Description |
|------|-------------|
| `--target node` | Emit Node/Wasm artifacts (default: `node`) |
| `-o`, `--output` | Output directory (default: `./dist/build`) |
| `-v`, `--verbose` | Verbose logging |

## Contents

| Path | Description |
|------|-------------|
| `bin/legion.mjs` | CLI entry |
| `lib` | Host ABI and Wasm loader |
| `legion.wasm` | Compiler module |
| `provenance.json` | Build metadata (commit, toolchain, digests) |
| `SHA256SUMS` | File checksums |

## License

MIT — see `../../LICENSE.md`.

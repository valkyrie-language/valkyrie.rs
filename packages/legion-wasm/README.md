# @valkyrie-language/legion

Legion compiler CLI for Node.js (WebAssembly GC).

**Requirements:** Node.js 20+

## Install

```bash
npm install @valkyrie-language/legion@0.0.5
```

```bash
pnpm add @valkyrie-language/legion@0.0.5
```

## Usage

```bash
legion --version
legion --help
legion build <project-dir> --target node -o dist/out
```

Artifacts are produced by `legion build … --target node` and assembled into this package (`legion.mjs`, `legion.wasm`, `run-contracts.txt`, `provenance.json`).

## License

MPL-2.0 — see `LICENSE.md`.

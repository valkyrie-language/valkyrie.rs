# @valkyrie-language/vcc-unknown-wasm32

**Valkyrie Compiler Collect (VCC)** — WebAssembly GC host artifacts for **Legion** on Node.js.

This package ships the assembled Wasm runtime used by [`@valkyrie-language/legion`](../legion). End users typically
install `legion`, not this collect directly.

## Contents

When fully assembled, the tarball includes:

| File                | Description                       |
|---------------------|-----------------------------------|
| `legion.mjs`        | Node.js Wasm GC host bootstrap    |
| `legion.wasm`       | Compiled Legion CLI (Wasm module) |
| `run-contracts.txt` | Host/run contract metadata        |
| `provenance.json`   | Build provenance record           |
| `SHA256SUMS`        | Checksums for shipped artifacts   |
| `LICENSE.md`        | License text                      |

Subpath exports:

```js
import.meta.resolve("@valkyrie-language/vcc-unknown-wasm32/legion.mjs");
import.meta.resolve("@valkyrie-language/vcc-unknown-wasm32/legion.wasm");
```

## Install

```bash
npm install @valkyrie-language/vcc-unknown-wasm32@0.0.0
```

Prefer the assembled CLI:

```bash
npm install @valkyrie-language/legion@0.0.0
```

## Requirements

- **Node.js** 20 or newer (WebAssembly GC support)

## Usage

Do not invoke this package directly in normal workflows. Install [`@valkyrie-language/legion`](../legion) and run:

```bash
legion --version
legion build ./project --target node -o dist/out
```

## Assembly

Artifacts are produced by the Legion Wasm build pipeline and staged with `node scripts/build.mjs assemble` in the monorepo before publish.

## Related packages

| Package                     | Role                                    |
|-----------------------------|-----------------------------------------|
| `@valkyrie-language/legion` | Public CLI that depends on this collect |
| `@valkyrie-language/vcc`    | Shared host runner                      |

## License

[MPL-2.0](./LICENSE.md)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/vcc-unknown-wasm32)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)

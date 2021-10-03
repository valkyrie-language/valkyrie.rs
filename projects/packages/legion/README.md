# @valkyrie-language/legion

**Legion** is the Valkyrie compiler and workspace CLI, shipped as an assembled npm package.

It bundles a thin Node.js host (`bin/legion`) with VCC platform collects: optional native Node-API artifacts and a
WebAssembly GC fallback for cross-platform use.

## Install

```bash
npm install @valkyrie-language/legion
```

```bash
pnpm add @valkyrie-language/legion
```

Run without a global install:

```bash
npx @valkyrie-language/legion --version
```

## Quick start

```bash
legion --version
legion --help
legion build ./my-project --target node -o dist/out
```

## How it works

| Layer                       | Location in monorepo                                | Responsibility                          |
|-----------------------------|-----------------------------------------------------|-----------------------------------------|
| Rust library                | `projects/compilers/legion`                         | Compiler semantics, manifests, planning |
| VCC native                  | `projects/compilers/vcc-napi` + `projects/packages/vcc-*` | Node-API platform collects              |
| VCC Wasm                    | `projects/compilers/vcc-wasm` + `projects/packages/vcc-unknown-wasm32` | Wasm GC collect                         |
| **Assembly (this package)** | `projects/packages/legion`                          | Public `legion` command                 |

At runtime the host ([`@valkyrie-language/vcc`](../vcc)) tries an optional native `vcc_napi` cdylib for your platform,
then falls back to the Wasm collect (`legion.mjs` + `legion.wasm`).

## Requirements

- **Node.js** 20 or newer

## Programmatic API

```js
import {runCli, locateNativeCollect, resolveWasmMjs} from "@valkyrie-language/legion";

runCli(["--version"]);
```

## Related packages

| Package                                 | Role                                  |
|-----------------------------------------|---------------------------------------|
| `@valkyrie-language/vcc`                | Shared host runner                    |
| `@valkyrie-language/vcc-unknown-wasm32` | Wasm GC collect (required dependency) |
| `@valkyrie-language/vcc-win32-x64`      | Optional Windows x64 native collect   |
| `@valkyrie-language/vcc-linux-x64`      | Optional Linux x64 native collect     |
| `@valkyrie-language/vcc-darwin-x64`     | Optional macOS x64 native collect     |
| `@valkyrie-language/vcc-darwin-arm64`   | Optional macOS arm64 native collect   |

## Development

From the [`valkyrie.rs`](https://github.com/valkyrie-language/valkyrie.rs) monorepo:

```bash
pnpm install
pnpm test --filter @valkyrie-language/legion
```

## License

[MPL-2.0](https://www.mozilla.org/MPL/2.0/)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/legion)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)

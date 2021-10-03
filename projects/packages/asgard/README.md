# @valkyrie-language/asgard

**Asgard** is the cross-platform GUI application framework CLI for the Valkyrie ecosystem, shipped as an assembled npm
package.

It provides the `asgard` command for build, dev, and pack workflows, backed by VCC platform collects (optional native
Node-API artifacts and a Wasm fallback).

## Install

```bash
npm install @valkyrie-language/asgard
```

```bash
pnpm add @valkyrie-language/asgard
```

Run without a global install:

```bash
npx @valkyrie-language/asgard --help
```

## Quick start

```bash
asgard --help
asgard build
asgard dev
asgard pack
```

## How it works

| Layer                       | Location in monorepo                             | Responsibility             |
|-----------------------------|--------------------------------------------------|----------------------------|
| Rust library                | `projects/asgard`                                | AWSL, RenderIR, packaging  |
| VCC native                  | `projects/vcc-napi` + `packages/vcc-*`           | Node-API platform collects |
| VCC Wasm                    | `projects/vcc-wasm` + `packages/vcc-wasm32-wasi` | Wasm collect               |
| **Assembly (this package)** | `packages/asgard`                                | Public `asgard` command    |

At runtime the host ([`@valkyrie-language/vcc`](../vcc)) tries an optional native `vcc_napi` cdylib for your platform,
then falls back to the Wasm collect (`asgard.mjs` + `asgard.wasm`).

## Requirements

- **Node.js** 20 or newer

## Programmatic API

```js
import {runCli} from "@valkyrie-language/asgard";

runCli(["--help"]);
```

## Related packages

| Package                               | Role                                |
|---------------------------------------|-------------------------------------|
| `@valkyrie-language/vcc`              | Shared host runner                  |
| `@valkyrie-language/vcc-wasm32-wasi`  | Wasm collect (required dependency)  |
| `@valkyrie-language/vcc-win32-x64`    | Optional Windows x64 native collect |
| `@valkyrie-language/vcc-linux-x64`    | Optional Linux x64 native collect   |
| `@valkyrie-language/vcc-darwin-x64`   | Optional macOS x64 native collect   |
| `@valkyrie-language/vcc-darwin-arm64` | Optional macOS arm64 native collect |

## Development

From the [`valkyrie.rs`](https://github.com/valkyrie-language/valkyrie.rs) monorepo:

```bash
pnpm install
pnpm test --filter @valkyrie-language/asgard
```

## License

[MPL-2.0](https://www.mozilla.org/MPL/2.0/)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/asgard)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)

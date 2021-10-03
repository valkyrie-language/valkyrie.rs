# @valkyrie-language/vcc-wasm32-wasi

**Valkyrie Compiler Collect (VCC)** — `wasm32-wasip1` host artifacts for **Asgard**.

This package ships the Wasm runtime used by [`@valkyrie-language/asgard`](../asgard). End users typically install
`asgard`, not this collect directly.

## Contents

When fully assembled, the tarball includes:

| File          | Description                                |
|---------------|--------------------------------------------|
| `asgard.mjs`  | Node.js host bootstrap for the Wasm module |
| `asgard.wasm` | Compiled Asgard CLI (Wasm module)          |

Subpath exports:

```js
import.meta.resolve("@valkyrie-language/vcc-wasm32-wasi/asgard.mjs");
import.meta.resolve("@valkyrie-language/vcc-wasm32-wasi/asgard.wasm");
```

## Install

```bash
npm install @valkyrie-language/vcc-wasm32-wasi@0.0.0
```

Prefer the assembled CLI:

```bash
npm install @valkyrie-language/asgard@0.0.0
```

## Requirements

- **Node.js** 20 or newer

## Usage

Do not invoke this package directly in normal workflows. Install [`@valkyrie-language/asgard`](../asgard) and run:

```bash
asgard --help
asgard build
```

## Build

Native collect build copies `vcc_wasm.wasm` from `projects/vcc-wasm`; the Asgard assembly pipeline renames and wraps
artifacts into `asgard.mjs` / `asgard.wasm` before publish.

## Related packages

| Package                     | Role                                    |
|-----------------------------|-----------------------------------------|
| `@valkyrie-language/asgard` | Public CLI that depends on this collect |
| `@valkyrie-language/vcc`    | Shared host runner                      |

## License

[MIT](https://opensource.org/licenses/MIT)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/vcc-wasm32-wasi)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)

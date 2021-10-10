# @valkyrie-language/vcc

Shared **Valkyrie Compiler Collect (VCC) host runner** for assembled Node.js CLIs.

This package is a small library used by [`@valkyrie-language/legion`](../legion) and [
`@valkyrie-language/asgard`](../asgard). It resolves platform-specific native artifacts and WebAssembly collect entries,
then dispatches CLI execution (native first, Wasm fallback).

## Install

```bash
npm install @valkyrie-language/vcc
```

Most applications should install a CLI package instead:

```bash
npm install @valkyrie-language/legion
# or
npm install @valkyrie-language/asgard
```

## Requirements

- **Node.js** 20 or newer

## Usage

```js
import {createHostRunner, NATIVE_PACKAGES} from "@valkyrie-language/vcc";

const host = createHostRunner({
    wasmCollect: "@valkyrie-language/vcc-unknown-wasm32",
    wasmEntry: "legion.mjs",
});

// Programmatic entry (same routing as the `legion` bin)
host.runCli(process.argv.slice(2));

// Inspect routing
host.locateNativeCollect(); // path to vcc.<platform>.node or null
host.resolveWasmMjs();      // absolute path to legion.mjs in the wasm collect
```

### Exports

| Export                           | Description                                                    |
|----------------------------------|----------------------------------------------------------------|
| `NATIVE_PACKAGES`                | Optional native collect package names (Windows, Linux, macOS). |
| `locateNativeCollect(packages?)` | Find an installed platform `.node` addon, or `null`.                |
| `resolveWasmMjs(collect, entry)` | Resolve the Wasm host script path inside a collect package.    |
| `createHostRunner(config)`       | Bind wasm collect settings and return `runCli` helpers.        |

## Related packages

| Package                     | Role                                                |
|-----------------------------|-----------------------------------------------------|
| `@valkyrie-language/legion` | Valkyrie compiler & workspace CLI                   |
| `@valkyrie-language/asgard` | Cross-platform GUI app framework CLI                |
| `@valkyrie-language/vcc-*`  | Platform collect packages (native / Wasm artifacts) |

## License

[MPL-2.0](https://www.mozilla.org/MPL/2.0/)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/vcc)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)

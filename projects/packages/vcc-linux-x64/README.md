# @valkyrie-language/vcc-linux-x64

**Valkyrie Compiler Collect (VCC)** — Linux x64 native Node-API artifact for Legion and Asgard hosts.

## Overview

This optional platform package contains the `libvcc_napi.so` cdylib built from `projects/vcc-napi`. It is pulled in
automatically when you install [`@valkyrie-language/legion`](../legion) or [`@valkyrie-language/asgard`](../asgard) on
Linux x64.

You normally do **not** install this package directly.

## Platform

| Field | Value   |
|-------|---------|
| OS    | `linux` |
| CPU   | `x64`   |

npm skips this package on other platforms.

## Artifact

| File             | Description                   |
|------------------|-------------------------------|
| `libvcc_napi.so` | Node-API native host (cdylib) |

## Install

Installed transitively as an optional dependency:

```bash
npm install @valkyrie-language/legion
```

## Build (maintainers)

From the monorepo root:

```bash
pnpm build:napi
# or: node scripts/build.mjs napi --all
```

Cross-compilation uses `cargo zigbuild` for the `x86_64-unknown-linux-musl` target.

## Related packages

| Package                               | Role                |
|---------------------------------------|---------------------|
| `@valkyrie-language/vcc-win32-x64`    | Windows x64 collect |
| `@valkyrie-language/vcc-darwin-x64`   | macOS x64 collect   |
| `@valkyrie-language/vcc-darwin-arm64` | macOS arm64 collect |
| `@valkyrie-language/vcc`              | Shared host runner  |

## License

[MIT](https://opensource.org/licenses/MIT)

## Links

- [Repository](https://github.com/valkyrie-language/valkyrie.rs/tree/main/projects/packages/vcc-linux-x64)
- [Issues](https://github.com/valkyrie-language/valkyrie.rs/issues)

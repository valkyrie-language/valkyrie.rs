# Compilers

`projects/compilers/` 下的 Rust workspace 成员：

| Crate | 职责 |
| --- | --- |
| `legion` | CLI、工作区与构建编排 |
| `vcc-data` | 二进制/文本格式模型与编解码 |
| `vcc-napi` / `vcc-wasm` | 原生与 Wasm 宿主绑定 |
| `asgard` | 跨平台应用 CLI（可选） |

语言语义与 `nyar-emitter` 等 crate 来自 [nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs) git 依赖。

构建与 npm 装配见仓库根 `scripts/` 与 [`projects/packages/`](../packages/) 下各包 README。

# 🛡️ Valkyrie

Valkyrie 是一个以 Rust 装配层、V 语言源码和 `nyar-vm.rs` 编译器栈组成的编程语言工作区。仓库当前同时维护 Rust
编译器工具链、Node/Wasm 发布包，以及通过 submodule 引入的 `valkyrie.v` 语言与标准库源码。

> 这是仓库总览。安装、CLI 参数和具体 API 以各 npm 包自己的 README 为准；语言语义以 `valkyrie-2020` 规范为准。

## ✨ 当前能力

- **Legion CLI**：管理 `legion.von` 工程清单、源码收集、依赖规划和构建编排。
- **VCC host**：在 Node.js 中选择平台 native collect，并在需要时回退到 Wasm collect。
- **Node/Wasm 发布包**：提供 `@valkyrie-language/legion`、`@valkyrie-language/vcc`、`@valkyrie-language/asgard` 以及平台
  collect 包。
- **多种输出路径**：编译器装配层支持 native、Wasm 和数据格式相关的构建流程；各目标的实际支持范围以对应包和文档为准。
- **V 语言工作区**：`projects/valkyrie.v` 保存语言源码、标准库和工具工程，通过只读 submodule 接入。

## 🧩 工作区结构

| 路径                                                           | 作用                                       |
|----------------------------------------------------------------|--------------------------------------------|
| [`projects/compilers/`](projects/compilers/)                   | Rust 编译器与装配 crate                    |
| [`projects/compilers/legion/`](projects/compilers/legion/)     | CLI、manifest 解析、工作区规划与构建编排   |
| [`projects/compilers/vcc-data/`](projects/compilers/vcc-data/) | 二进制与文本格式模型、编解码和产物数据     |
| [`projects/compilers/vcc-napi/`](projects/compilers/vcc-napi/) | Node-API native host                       |
| [`projects/compilers/vcc-wasm/`](projects/compilers/vcc-wasm/) | Wasm host 与 Wasm 产物装配                 |
| [`projects/compilers/asgard/`](projects/compilers/asgard/)     | Asgard CLI 与跨平台应用入口                |
| [`projects/packages/`](projects/packages/)                     | npm 包源码、host wrapper 和平台 collect 包 |
| [`projects/valkyrie.v/`](projects/valkyrie.v/)                 | Valkyrie V 源码、标准库和工具工程          |
| [`documentation/`](documentation/)                             | 语言、架构、维护和示例文档                 |
| [`scripts/`](scripts/)                                         | 格式化、构建、测试、装配和发布脚本         |

语言语义、解析、优化和 emitter 由 [nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs) 提供。`valkyrie.rs` 负责
manifest、workspace、source closure、缓存和 npm/Wasm 产物装配；不要把这两层的职责混在一起。

## 📦 npm 包

| 包                                                       | 用途                                 |
|----------------------------------------------------------|--------------------------------------|
| [`@valkyrie-language/legion`](projects/packages/legion/) | Valkyrie 编译器与 workspace CLI      |
| [`@valkyrie-language/vcc`](projects/packages/vcc/)       | Node host runner 与 native/Wasm 路由 |
| [`@valkyrie-language/asgard`](projects/packages/asgard/) | Asgard 应用 CLI                      |
| `@valkyrie-language/vcc-unknown-wasm32`                  | Wasm GC collect                      |
| `@valkyrie-language/vcc-wasm32-wasi`                     | WASI collect                         |
| `@valkyrie-language/vcc-{win32,linux,darwin}-*`          | 平台 native collect                  |

CLI 与库的安装、Node.js 版本要求、API 示例和运行参数请查看对应包 README。仓库根目录的 `package.json` 只负责 monorepo
开发脚本，不是发布包的使用说明。

## 🛠️ 开发环境

要求：

- Node.js 20 或更高版本
- pnpm 10
- Rust toolchain，版本由 [`rust-toolchain.toml`](rust-toolchain.toml) 固定
- `nyar-vm.rs` checkout；Rust 依赖由根 [`Cargo.toml`](Cargo.toml) 中的 git 依赖解析

安装依赖：

```bash
pnpm install
```

常用命令：

```bash
pnpm check              # 检查主要 Rust crate
pnpm test               # 运行 npm 包测试
pnpm test:integration   # 运行 Legion 集成测试
pnpm build              # 执行默认构建流程
pnpm fmt:check          # 检查格式
```

Wasm capability、assemble 和发布流程使用专门脚本：

```bash
pnpm build:capability
pnpm build:assemble
pnpm publish:dry-run
```

这些命令的完整约束和失败条件见 [`documentation/maintenance/`](documentation/maintenance/)。

## 📚 文档入口

- [语言文档](documentation/)
- [编译器架构](documentation/maintenance/compiler-architecture.md)
- [项目架构](documentation/maintenance/project-architecture.md)
- [包管理](documentation/maintenance/package-management.md)
- [维护总览](documentation/maintenance/index.md)
- [发布说明](documentation/maintenance/releases/)

## 🤝 参与贡献

请先阅读 [`documentation/maintenance/`](documentation/maintenance/) 中与改动层级对应的说明。

提交变更前至少运行与改动相关的检查，并在提交信息中使用一个 gitmoji 和简洁的英文祈使句。涉及 compiler semantics、optimizer 或
emitter 的修改应提交到 `nyar-vm.rs` 对应仓库；涉及 workspace、manifest、装配和 npm 发布的修改提交到本仓库。

## 📄 许可证

本项目使用 [Mozilla Public License 2.0](LICENSE.md)。

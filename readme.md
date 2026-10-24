# 🛡️ Valkyrie

Valkyrie 语言语料与 fixtures 仓库。编译器、语言语义与标准库实现位于
[nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs) 与 [`projects/valkyrie.v`](projects/valkyrie.v/)；
本仓只保留 Oak 语法解析与构建的 fixture 语料及其黑盒验收。

> 语言语义以 `valkyrie-2020` 规范为准。本仓不重新实现编译器、CLI、host 或发布包。

## 🧩 工作区结构

| 路径                                                           | 作用                                      |
|----------------------------------------------------------------|-------------------------------------------|
| [`projects/compilers/valkyrie-fixtures/`](projects/compilers/valkyrie-fixtures/) | Valkyrie 源码的 lexer/parser/builder/printer fixture 与 golden |
| [`projects/compilers/von-fixtures/`](projects/compilers/von-fixtures/)           | VON 格式 fixture 与序列化 golden          |
| [`projects/compilers/voc-fixtures/`](projects/compilers/voc-fixtures/)           | VOC（AWSL 组件）fixture                   |
| [`projects/fixtures/`](projects/fixtures/)                     | Legion/工程级 `.v` 语料与 bootstrap 工程  |
| [`projects/valkyrie.v/`](projects/valkyrie.v/)                 | Valkyrie V 源码、标准库和工具工程（只读 submodule） |
| [`documentation/`](documentation/)                             | 语言与维护文档                            |

Oak 拥有语法树、恢复状态与 syntax diagnostics；Valkyrie frontend 拥有名称、类型、泛型替换、evidence 与调用语义。
fixture 只记录这些既成行为，不定义语言合法性，也不替代 `nyar-vm.rs` 的端到端验收。

## 🛠️ 开发环境

要求 Rust toolchain，版本由 [`rust-toolchain.toml`](rust-toolchain.toml) 固定。

```bash
cargo test --workspace     # 运行全部 fixture 与 golden 校验
cargo check --workspace    # 检查 fixtures crate
```

Golden 由 Oak 的 `REGENERATE_TESTS=1` 生成；更新 golden 必须与实际解析行为一致，不能用忽略或删除掩盖失败。

## 📚 文档入口

- [语言文档](documentation/)
- [编译器架构](documentation/maintenance/compiler-architecture.md)
- [项目架构](documentation/maintenance/project-architecture.md)
- [维护总览](documentation/maintenance/index.md)

## 🤝 参与贡献

提交变更前运行 `cargo test --workspace`，并在提交信息中使用一个 gitmoji 和简洁的英文祈使句。涉及编译器语义、
optimizer 或 emitter 的修改提交到 `nyar-vm.rs`；本仓只接受语料、golden 与 fixture 维护。

## 📄 许可证

本项目使用 [Mozilla Public License 2.0](LICENSE.md)。

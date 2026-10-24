# Valkyrie 项目架构与维护指南

本仓是 Valkyrie 的语料与 fixture 仓库：保存 `.v` 语料、Oak 解析/构建 golden 与最小黑盒验收。
编译器、语言语义、标准库与 CLI/host 实现位于 `nyar-vm.rs` 与 `projects/valkyrie.v`，本仓不重复实现。

## 目录与职责

- projects/compilers/valkyrie-fixtures：Valkyrie lexer/parser/builder/printer fixture 与 golden。
- projects/compilers/von-fixtures：VON 格式 fixture 与 serde golden。
- projects/compilers/voc-fixtures：VOC（AWSL 组件）fixture。
- projects/fixtures：Legion/工程级 `.v` 语料与 bootstrap 工程，供黑盒 harness 调用外部 CLI 验收。
- projects/valkyrie.v：语言源码与标准库（只读 submodule）。

成员与依赖以 Cargo.toml 为准。Oak 拥有语法树、恢复状态、syntax diagnostics 与 source edits；Valkyrie frontend
（`nyar-vm.rs` 的 `rusty-valkyrie`）拥有名称、类型、泛型替换、evidence、调用与 Semantic MIR；Acorn 拥有外部二进制
格式。fixture 只记录既成行为，不定义语言合法性。

## 深模块边界

外部接口为 WorkspaceResolver、Compiler、ArtifactStore，均在本仓之外。本仓不构造 AST/HIR/MIR、语义 planner 或
target lowering，也不以 wrapper 成功证明源码编译能力。黑盒验收只消费外部 CLI 的实际退出码、stdout 与产物。

## 维护流程

先定位缺失事实的唯一生产者，再选择仓和接口。每个工作包说明可观察行为、失败条件、正反验收与证据范围；不能用
题目名、operator 或当前报错作为特判依据。

Golden 由 Oak 的 `REGENERATE_TESTS=1` 生成。解析/构建行为变化时按实际输出更新 golden，并确认所有语料仍然成功；
禁止用 `#[ignore]`、删除用例或预期失败掩盖陈旧 golden。

详见 [编译协议](compiler-architecture.md)、[后端边界](backends/index.md)及[诊断](error-handling.md)。

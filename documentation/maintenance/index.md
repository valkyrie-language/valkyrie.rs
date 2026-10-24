# 架构与贡献指南

面向**公开仓库贡献者**：说明 `valkyrie.rs` 现行职责、语料边界与文档规则。编译器实现、发布流程与自举验收不在本文范围。

## Workspace 结构

根 `Cargo.toml` 只保留语料 crate 与黑盒 harness：

| Crate / 路径 | 职责 |
| --- | --- |
| `valkyrie-fixtures` | Valkyrie lexer/parser/builder/printer fixture 与 golden |
| `von-fixtures` | VON 文本格式 fixture 与 serde golden |
| `voc-fixtures` | VOC（AWSL 组件）fixture |
| `projects/fixtures/` | Legion 与 bootstrap 工程级 `.v` 语料 |
| `projects/harness/` | 调 `nyar-vm.rs` 生产 CLI 的语料黑盒验收 |
| `projects/valkyrie.v/` | 语言源码与标准库（只读 submodule） |

二进制与文本格式编解码归 Acorn / Oak 仓；本工作区不保留混合格式包、npm 包、CI 与发布脚本。

语言语义、`nyar-emitter` 与多数 `nyar-*` crate 来自 [nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs) git 依赖。

## 编译主链（摘要）

```text
WorkspaceResolver → immutable inventory / package graph
  → Compiler：binding / demand fixed point / HIR / Semantic MIR
  → linker / Canonical → RepresentationPlan → TargetExecutionPlan
  → Acorn 或 Nyar bytecode 编码 → ArtifactStore
```

该主链的实现位于 `nyar-vm.rs`；本仓只经 `projects/harness` 的 CLI 入口黑盒验收，不复制其中任何阶段。
细节见 [compiler-architecture.md](compiler-architecture.md)、[backends/index.md](backends/index.md)。

## 语料与 golden

- fixture 只记录既成解析/构建行为，不定义语言合法性。
- Golden 由 Oak 的 `REGENERATE_TESTS=1` 生成；更新后必须确认所有语料仍然成功，禁止用 `#[ignore]` 或删除用例掩盖失败。
- 新增语料工程放在 `projects/fixtures`，并让 `projects/harness` 能通过 `vcc plan` 解析它。

## 文档边界

- `documentation/language/`：用户向语言说明。
- `documentation/maintenance/`：与**当前代码一致**的架构说明；不得写入未公开计划、发布门槛或工作区外文档链接。
- 集成测试若需外部语言规范树，通过环境变量注入路径，不得在文档或源码中写死本机路径。

## 相关入口

- [project-architecture.md](project-architecture.md)
- [package-management.md](package-management.md)
- [error-handling.md](error-handling.md)
- 根 [readme.md](../../readme.md)

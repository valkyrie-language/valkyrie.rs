# 架构与贡献指南

面向**公开仓库贡献者**：说明 `valkyrie.rs` 现行 crate 分工、编译主链与文档边界。发布流程、自举验收与未公开里程碑不在本文范围。

## Workspace 结构

根 `Cargo.toml` 成员以 `projects/compilers/`、`projects/hosts/` 为准（节选）：

| Crate | 职责 |
| --- | --- |
| `legion` | CLI、清单解析、轻量编排 |
| `vcc-data` | 二进制/文本格式模型与编解码 |
| `vcc-napi` / `vcc-wasm` | 原生与 Wasm 宿主 cdylib |
| `asgard` | 跨平台 GUI 应用 CLI（可选） |

语言语义、`nyar-emitter` 与多数 `nyar-*` crate 来自 [nyar-vm.rs](https://github.com/nyar-vm/nyar-vm.rs) git 依赖。

## 编译主链（摘要）

```text
source → AST → HIR → MIR
  → FragmentSubmission / ArtifactPartitionPlan (nyar)
  → nyar-emitter 按目标降低
  → vcc-data 手写目标二进制
```

细节见 [compiler-architecture.md](compiler-architecture.md)、[backends/index.md](backends/index.md)。

## 文档边界

- `documentation/language/`：用户向语言说明。
- `documentation/maintenance/`：与**当前代码一致**的架构说明；不得写入未公开计划、发布门槛或工作区外文档链接。
- 集成测试若需外部语言规范树，通过环境变量注入路径，不得在文档或源码中写死本机路径。

## npm 发布说明

- 模板：[release-notes.template.md](release-notes.template.md)
- 起草对照：`pnpm change-logs --version X.Y.Z`（`--write` → `releases/vX.Y.Z.reference.md`，与 `vX.Y.Z.md` 同目录、gitignore 勿入库；`--tags` 列出 tag）
- 非 GitHub 邮箱 → 贡献者映射：[author-github.json](author-github.json)（`id` 稳定，`login` 可改）
- 已发布版本：`releases/v0.0.0.md` … `releases/v0.0.3.md`（对应 Git tag `v0.0.x`）

## 相关入口

- [project-architecture.md](project-architecture.md)
- [error-handling.md](error-handling.md)
- 根 [readme.md](../../readme.md)

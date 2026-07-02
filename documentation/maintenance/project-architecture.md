# Valkyrie 项目架构与维护指南

描述 `valkyrie.rs` 工作区的 crate 组织、职责边界与维护策略。

## 顶层原则

- `nyar-language` 维护语言主链：`HIR -> MIR -> …`（语义定界）；不把 emit 语义化。
- `nyar` 维护目标路线、backend input 与打包协议。
- `nyar-emitter` + `std-data` 负责各目标降低与**手写**二进制。
- `legion` 只做命令入口与清单编排（`legion.von` / `legions.von`）。
- 规范测试与架构文档同级重要。

## 工作区结构

根 `Cargo.toml` 成员以源码为准，核心包括：

```text
valkyrie.rs/
  Cargo.toml
  documentation/
  projects/
    legion/
    nyar/
    nyar-language/
    nyar-emitter/
    nyar-types/
    nyar-analyzer/
    nyar-optimizer/
    std-data/
    nvm/
    …
```

### `projects/nyar-language`

- 词法/语法之后的语言主链：名称解析、类型检查、`row / trait / class / sealed class / unite / effect`、HIR/MIR。
- 最先承接语言侧规范测试。

### `projects/nyar-types`

- 平台层类型（如 `NyarType`：含 `Bottom`/`Unit`/`Utf8`/`Utf16`），不回流前端独有构造。

### `projects/nyar`

- `ArtifactPartitionPlan`、后端注册/选择、打包协议。
- 不发明统一物理 IR，不补语言级 resolve。

### `projects/nyar-emitter`

- `FragmentSubmission` → clr / jvm / wasm / native / … lowering。

### `projects/std-data`

- `binary/pe`、`binary/class`、`binary/wasm`、`text/msil` 等。

### `projects/legion`

- CLI、`.von` 清单、轻量 planner；不持有 HIR/MIR 主结构。

## 语义边界如何映射到 crate

语言级事实须在 `nyar-language` 内定清：

- `row` / `trait` / `class` / `sealed class` / `unite` / `effect`
- 文本：`utf8`/`utf16`；`void`≠`unit`

进入 `nyar` / emitter 前：row 已闭合；开放 trait/effect 以 witness 表达或已静态化/失败。

## 维护流程

### 修改语义时

1. 更新 `documentation/zh-hans/maintenance/*.md`
2. 更新规范测试
3. 再改 HIR/MIR/`nyar` 实现

### 修改后端路线时

1. 先确认 backend input 契约
2. 不支持的开放调度须提前失败或静态化
3. 不把 row/未决 nominal 留给后端
4. 保持手写二进制与四目标同构约束

### 代码审查重点

- 是否破坏 `nyar-language` 与 `nyar` / emitter 分工
- 是否偷偷引入跨端统一 IR 或外部生成工具产盘
- 是否把语义补丁藏进 planner/emit
- 是否同步文档与测试

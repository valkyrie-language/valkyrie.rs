# Valkyrie 项目架构与维护指南

这份文档描述 `valkyrie.rs` Rust 工作区的 crate 组织、职责边界与维护策略。重点不是“把所有东西塞进一个编译器 crate”，而是让 parser、compiler、backend container、planner 和规范测试各守边界。

## 顶层原则

- `valkyrie-compiler` 维护语言主链：`HIR -> MIR (SSA) -> optimize -> artifact partition -> target LIR`。
- `nyar` 维护目标路线、backend input 与打包协议，不替代语言主链。
- `legion` 只做命令入口和轻量编排，不承载语言语义。
- 规范测试要与架构文档同级重要；语义不靠口头约定，而要靠 `tests/spec` 钉死。

## 工作区结构

当前工作区定义在根 `Cargo.toml`，核心成员包括：

```text
valkyrie.rs/
  Cargo.toml
  documentation/
  projects/
    legion/
    nyar/
    nyar-types/
    valkyrie-compiler/
    valkyrie-parser/
    valkyrie-types/
```

### `projects/valkyrie-parser`

- 负责词法、语法与 AST 前置阶段。
- 不负责最终语义闭合，也不持有后端输入模型。

### `projects/valkyrie-types`

- 放语言级共享类型定义与编译期公共数据结构。
- 不把这里扩写成“无所不包的架构垃圾场”。

### `projects/valkyrie-compiler`

- 维护语言主链源码。
- 负责名称解析、类型检查、`row / trait / class / sealed class / unite / effect` 语义定界、`HIR/MIR` 转换、优化前边界与分区计划。
- 这里是最先承接规范测试的地方。

### `projects/nyar-types`

- 放后端相关共享类型，不回流语言级语义。

### `projects/nyar`

- 承接 `ArtifactPartitionPlan` 之后的 target-specific 输入。
- 维护 `abstractions / lanes / backends / data_formats / packaging / selection`。
- 不重新发明统一物理 `IR`，不补做语言级 resolve。

### `projects/legion`

- 命令行入口、清单读取与轻量 planner。
- 不持有 `HIR / MIR / LIR` 主结构。
- 不把 planner 演化成新的伪 `IR` 或语义总线。

## 语义边界如何映射到 crate

### 语言级事实

这些事实必须在 `valkyrie-compiler` 内定清，而不是拖到 `nyar` 或 `legion`：

- `row`：匿名 `trait` 语法对应 `method row requirement`，只做方法行满足。
- `trait / imply`：具名协议与 witness。
- `class`：名义子类型。
- `sealed class`：显式封闭类层级。
- `unite`：抽象类与封闭 variant 集合的紧凑写法；默认表示通常是抽象类，也可以进一步优化为 tagged union 或利基布局。
- `effect`：handler evidence 与 effect 边界。

### 后端前边界

这些事实进入 `nyar` 之前必须已经定界：

- `row` 已闭合为已验证的方法调用事实。
- `trait / imply` 若仍开放，必须显式以 witness/evidence 表达。
- `class / sealed class` 的名义判定与 `unite` 的 variant 分析不得拖到 backend input 再做。
- `effect` 若目标路线不支持开放 lowering，必须更早失败或静态化。

## 目录分工

### `projects/valkyrie-compiler/src`

- 放 `hir`、`mir`、`pipeline` 与编译主链实现。
- 不放跨目标统一 `god ir`。
- 不为了某个单一后端临时改写语义事实。

### `projects/valkyrie-compiler/tests`

- 放编译器侧测试、语义规范测试、管线测试与目标边界护栏。
- 这里不仅测试“当前实现是否通过”，也测试“未来实现不得越界”。
- 应逐步形成：
  - `type_checker`：名义子类型、约束求解、trait/witness 事实
  - `pipeline`：`HIR/MIR/LIR` 边界
  - `optimizer`：静态化、去虚化与封闭类优化
  - `spec`：语义规范测试

### `projects/nyar/src`

- `abstractions`：最小公共协议
- `lanes`：从 `ArtifactPartitionPlan` 承接到目标路线
- `backends`：后端验证与编译
- `data_formats`：目标分区后的低层表示族
- `packaging`：`ArtifactSet`、`OutputSpec` 与最终交付协议
- `selection`：后端选择

## 规范测试策略

### 为什么要先立规范测试

- `row / trait / class / sealed class / unite / effect` 的边界很容易在实现中漂移。
- 如果没有单独的规范测试层，代码会不知不觉变成“为了让当前路线先跑通”的局部真相。
- 所以即便很多测试现在还无法通过，也要先把测试名称、场景和预期语义立起来。

### 建议的测试分层

- `tests/spec/row.rs`：匿名 `trait` 只做方法行判定，不支持 associated type。
- `tests/spec/trait_system.rs`：具名 trait 必须收敛到具名 witness。
- `tests/spec/nominal.rs`：`class / sealed class` 只走名义子类型，`unite` 只认已声明 variant 集合。
- `tests/spec/overload.rs`：`nominal exact > nominal subtype > trait > row`。
- `tests/spec/associated_types.rs`：关联类型只存在于具名 trait，且必须唯一可解。
- `tests/spec/diagnostics.rs`：报错必须区分 nominal、row、trait、effect。
- `tests/spec/backend_boundary.rs`：开放 `row` 不得以 evidence 形式下沉到 backend input。

### 已知缺口如何处理

- 可以使用 `#[ignore = "..."]` 标出当前尚未补齐的测试。
- 允许 `known gap` 或 `pending` 测试存在。
- 但不允许因为实现尚未完成，就完全不把语义写进测试。

## 维护流程

### 修改语义时

1. 先更新 `documentation/zh-hans/maintenance/*.md` 的边界说明。
2. 再更新 `projects/valkyrie-compiler/tests/spec` 的规范测试。
3. 最后再改 `HIR / MIR / nyar` 实现与现有回归测试。

### 修改后端路线时

1. 先确认它需要什么 backend input，而不是先改语言语义。
2. 若路线不支持开放 `trait/effect` 调度，必须在进入该路线前显式失败或静态化。
3. 不把 `row` 或未决 nominal 判定留给后端。

### 代码审查重点

- 是否破坏了 `valkyrie-compiler` 与 `nyar` 的分工。
- 是否偷偷引入了统一跨端 `IR`。
- 是否把语义补丁藏进 planner、backend 或 emit 层。
- 是否同时更新了对应的规范测试与维护文档。

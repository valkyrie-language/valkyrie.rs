# Valkyrie 编译器架构

这份文档描述 `valkyrie.rs` Rust 工作区里的编译主链、语义边界与测试策略。目标不是再发明一份“统一大 IR”，而是把语言语义、优化责任、后端输入和规范测试各自钉死。

## 核心判断

- 语言主语义必须停留在 `HIR / MIR`，不能在 emit 阶段临时补语义。
- `row`、`trait/imply`、`class`、`sealed class`、`unite`、`algebra effect` 不是同一种语义实体：
  - `row` 是方法行满足
  - `trait/imply` 是具名协议 witness
  - `class` 是名义子类型
  - `sealed class` 是显式封闭类层级
  - `unite` 是声明抽象类与封闭 variant 的紧凑语法
  - `effect` 是 handler evidence
- `LIR` 必须按目标路线分区；`CPU/VM`、`CLR/JVM/WASM`、`native/shader` 不共享一份伪统一物理 `IR`。
- 需要用大量规范测试把这些边界定死；即便当前还有实现缺口，也要先把语义测试作为显式护栏立起来。

## 编译流水线

Rust 工作区里的推荐主链是：

```text
source
  -> parser
  -> AST
  -> HIR
  -> MIR (SSA)
  -> optimize (EGraph-ready)
  -> ArtifactPartitionPlan
  -> target LIR / backend input
  -> ArtifactSet
```

当前 crate 对应关系如下：

- `projects/valkyrie-parser`：解析与 AST 前置阶段。
- `projects/valkyrie-compiler`：`HIR -> MIR -> optimize -> artifact partition` 主链。
- `projects/nyar`：目标路线、backend input、打包协议与后端共享容器。
- `projects/legion`：轻量编排与命令入口，不承载语言语义。

## 语义模型

### row

- 匿名 `trait` 语法在语义上视为 `method row requirement`。
- `field` 不作为独立结构成员参与判定，而是收敛为 `getter + setter` 方法面。
- `row` 只回答“这个值现在是否提供这组方法签名”，不属于具名 trait system。
- `row` 不允许 `associated type`、默认实现、trait inheritance 或独立 witness。

### trait / imply

- 具名 `trait/imply` 是协议实体，而不是匿名 row 的别名。
- 即便某个类型通过结构化方法面满足了具名 trait，主链里也必须收敛成具名 `witness/evidence`。
- 具名 trait 若带 `associated type`，满足结果必须能给出唯一的关联项绑定。

### class / unite

- `class` 走名义子类型，而不是结构匹配。
- `sealed class` 显式表达封闭类层级。
- `unite` 表达的是互斥分支组成的 union，但默认表示是抽象类。
- `unite variant` 对应这条抽象类之下的内部封闭变体类型。
- 如果显式写出 `[tag(XXXKind)]`，编译器可以选择 tagged union 形态。
- 少数特殊模式还会走利基优化，但这属于表示优化，不应反过来污染前端类型规则。

### effect

- `algebra effect` 走显式 handler/evidence 边界。
- effect handler 的选择与闭合必须在后端前完成，不能让 emit 层兜底。

## Dispatch 与边界

### HIR

- 负责名称解析、类型检查、约束绑定、语法糖收束。
- 负责闭合 `row` 方法约束，做 `class / sealed class` 名义子类型判定，并确认 `unite` 的 variant 归属与穷尽性检查。
- 负责把调用归类为：
  - `static dispatch`
  - `witness dispatch`
  - `effect-handler dispatch`
- `row` 不是独立 dispatch 种类；它应在 `HIR` 类型检查阶段闭合为已选定成员调用。

### MIR（SSA）

- 作为重分析与优化前的主表示。
- 保留显式控制流、显式调用种类、显式 `trait/effect` operand。
- 不应把开放 `trait/effect` 调度伪装成普通静态调用。
- 若某个调用最初来自 `row` requirement，进入 `MIR` 时应已是闭合后的成员调用，而不是开放 row witness。

### Optimize

- 负责单态化、去虚化、effect 摘要、可达实例展开与目标感知优化预算。
- 静态化是优化结果，不是语义前提。
- 若某目标路线不支持开放 `trait/effect` 调度，应在这一层或更早阶段显式失败。

### ArtifactPartitionPlan

- 负责目标分区、入口裁剪、模块拆分与体积预算。
- 决定哪些分区进入 `CPU/VM`、`CLR/JVM/WASM`、`native` 或其他路线。
- 这一层之后不应继续维持语言级“万用兼容壳”。

### Target LIR / Backend Input

- 不是一份跨目标共享的统一物理 `IR`。
- `projects/nyar` 只承接已经完成语义定界的输入。
- 后端只消费符合本路线约束的低层表示，不重新做 trait resolve、row match 或 effect handler 选择。

## 后端对齐原则

- `CPU/VM` 路线可保留更丰富的 `trait/effect` 低层操作。
- `CLR/JVM/WASM` 若暂不支持完整 witness/effect lowering，必须在进入路线前完成：
  - 闭世界单态化
  - 去虚化
  - 调用目标静态化
- `row`、`class` 判定与 `unite` 分支分析不得拖到后端；进入 backend input 前必须已经闭合。

## 规范测试优先

`projects/valkyrie-compiler/tests` 不只是回归测试目录，也要承担语义规范职责。

### 为什么先写规范测试

- 这套语义分层最大的风险不是“现在有测试会失败”，而是后续实现把边界再次搅混。
- 规范测试允许先把语义写成可执行护栏，再逐步补齐实现。
- 对 `CLR`、`JVM`、`WASM` 与未来路线来说，这比让某个单一路线先把语义写死在代码里更安全。

### 规范测试的优先分组

- `row`：匿名 `trait` 只做方法行判定，不支持 associated type。
- `trait`：具名协议满足必须收敛为具名 witness。
- `nominal`：`class / sealed class` 只走名义子类型，`unite` 只认已声明 variant 集合。
- `overload`：`nominal exact > nominal subtype > trait > row`。
- `associated-types`：只存在于具名 trait，且必须唯一可解。
- `diagnostics`：清楚区分 nominal failure、row failure、trait failure、effect failure。
- `backend-boundary`：验证开放 `row` 不得下沉为 backend-visible evidence，开放 `trait/effect` 在不支持的路线必须失败。

### 当前策略

- 允许一部分测试先以 `#[ignore = "..."]` 或 `known gap` 方式存在。
- 但测试文件、命名与断言目标必须先建立，防止语义漂移。
- 先把“规范”写进 `tests/spec`，再逐步把对应实现从 `HIR`、`MIR`、`nyar` 和各路线补绿。

## 一句话结论

`valkyrie.rs` 的 Rust 版主链不应再围绕“统一大 IR”组织，而应明确：`row` 在 `HIR` 内闭合，`trait/imply` 与 `effect` 以 `witness/evidence` 保真，`class / sealed class` 保持名义层级边界，`unite` 以抽象类与封闭 variant 集合的形式定界并在 lowering 时再选择具体表示，`Optimize` 负责静态化与去虚化，`ArtifactPartitionPlan` 负责目标分区，`nyar` 只接住已经定界的 backend input，而语义规范测试负责把这些边界长期钉死。

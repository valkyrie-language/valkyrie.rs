# Valkyrie 编译器架构

这份文档描述 `valkyrie.rs` 工作区里的编译主链、语义边界与测试策略。目标不是「统一大 IR」，而是把语言语义、优化责任、后端输入和规范测试各自钉死。

## 核心判断

- 语言主语义停留在 `HIR / MIR`，不能在 emit 阶段临时补语义。
- `row`、`trait/imply`、`class`、`sealed class`、`unite`、`algebra effect` 不是同一种语义实体（见下文）。
- 后端输入按目标分区；**不**共享一份伪统一物理 IR。
- 四目标 **clr / jvm / wasm / wasi** 的正式产物由 `nyar-emitter` + `std-data` **手写二进制**完成（与 `valkyrie.v` 同构）。
- 类型约束：无语言 `string`（用 `utf8`/`utf16`）；`void`（ADT=0，`NyarType::Bottom`）≠ `unit`（ADT=1）。

## 编译流水线

```text
source
  -> parser / AST
  -> HIR
  -> MIR (SSA)
  -> optimize（演进中；egraph 宿主尚未产品完成态）
  -> ArtifactPartitionPlan
  -> FragmentSubmission / backend input
  -> nyar-emitter 目标降低
  -> ArtifactSet / 手写二进制
```

### Crate 对应（现行）

- `projects/nyar-language`：语言主链（HIR/MIR、语义定界；历史名常称 valkyrie-compiler）。
- `projects/nyar`：目标路线、规划、`ArtifactPartitionPlan`、后端共享协议。
- `projects/nyar-emitter`：各目标 lowering。
- `projects/nyar-types` / `std-data`：平台类型与二进制/文本编解码。
- `projects/legion`：CLI 与清单编排，不承载语言语义。

> Legacy `LIR` 仍有过渡资产，但**不得**再当作跨目标主链；见源码中的 deprecated 标注。

## 语义模型

### row

- 匿名 `trait` 语法视为 method row requirement。
- `field` 收敛为 getter/setter 方法面。
- 无 associated type、默认实现、独立 witness。

### trait / imply

- 具名协议；满足结果收敛为具名 witness/evidence。

### class / unite

- `class` / `sealed class`：名义层级。
- `unite`：具名 union；默认抽象类表示；`[tag(...)]` 可选 tagged union。

### effect

- handler/evidence 边界；选择与闭合在后端前完成。

### 文本与单位类型

- `utf8` / `utf16`（及 `Utf8Text` / `Utf16Text`）；禁止语言层 `string`。
- `void` vs `unit` 见上文；concretize：`ValkyrieType::Void` → `NyarType::Bottom`，`Unit` → `Unit`。

## Dispatch 与边界

### HIR

名称解析、类型检查、row 闭合、名义判定、unite 穷尽性；调用归类为 static / witness / effect-handler。

### MIR

显式控制流与调用种类；开放 trait/effect 不得伪装成普通静态调用。

### Optimize / ArtifactPartitionPlan

静态化与见证消除是优化结果；分区决定进入 CLR/JVM/WASM/WASI/native/VM 等路线。

### Backend input

`nyar` 只承接已定界输入；后端不重做 trait/row/effect 解析。

## 后端对齐原则

- CLR/JVM/WASM/WASI 若暂不支持开放 witness/effect，须在进入路线前单态化或失败。
- FFI 与 text adaptor 按目标隔离。
- 生成路径禁止 ilasm/dnlib/javac/`wit-component` 等外部工具产盘。

## 规范测试优先

规范测试应落在语言主链测试树（现以 `nyar-language` 及相关 crate 为准；旧文档中的 `valkyrie-compiler/tests` 路径需随 crate 迁移理解）。

优先分组：`row`、`trait`、`nominal`、`overload`、`associated-types`、`diagnostics`、`backend-boundary`。

允许 `#[ignore]` / known gap，但须先立语义护栏。

## 一句话结论

语义在 `nyar-language` 的 HIR/MIR 钉死；`nyar` 做分区与协议；`nyar-emitter`+`std-data` 按目标手写发射；规范测试防止边界漂移。

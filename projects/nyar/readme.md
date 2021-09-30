# nyar

`nyar` 是所有编译器共享的后端基础设施层。

## 职责
- 提供目标文件容器模型，例如 `PE`、`COFF`。
- 提供文本汇编和低层镜像模型，例如 `MSIL`、`CLR image`。
- 提供目标 lane、产物描述与打包协议。
- 作为 `legion spy`、后端打包器和目标发射器共享的数据层。

## 核心判断
- `Valkyrie` 的语言主链是 `AST -> HIR -> MIR (SSA) -> Optimize (EGraph) -> ArtifactPartitionPlan -> target-specific LIR / Backend Input -> ArtifactSet`。
- `nyar` 统一的是编排协议，不是所有目标共用的一份物理 `IR`。
- `ArtifactPartitionPlan` 之后必须进入各自 target lane，再落到目标专用低层输入。
- `CLR / JVM / WASM / native / VM / GPU` 的约束彼此不同，不能重新糊成统一兼容壳。
- `validate()` 与 `compile()` 的分工必须清晰：前者负责边界验证，后者只负责目标相关编码与封装。

## 目标结构
```text
ArtifactPartitionPlan
  -> Target Lowering Lane
  -> Target-specific LIR / Backend Input
  -> validate()
  -> compile()
  -> OutputSpec
  -> ArtifactSet
```

这里真正统一的是“如何把分区送进正确路线并交付产物”，不是“所有路线共享一份 backend IR”。

## 上游边界
- `row`、`trait / imply`、`class`、`sealed class`、`unite`、`effect` 等语言语义必须在进入 `nyar` 前已经闭合到后端可消费的事实。
- `row` 在 `HIR` 阶段闭合为方法行满足与成员选择事实，不以下游 row witness 的形式进入 `nyar`。
- `trait / imply` 与 `effect` 若仍保持开放调度，必须显式保留 witness / evidence，而不是被伪装成普通静态调用。
- `class / sealed class` 的名义子类型判定与 `unite` 的 variant 分析属于上游事实，不在后端容器层重算。
- `nyar` 不负责 trait resolve，不负责 row match，不负责 nominal subtype 判定，也不负责 effect handler 选择。
- 若某条目标路线不支持开放 witness 或 effect 调度，必须在进入该路线前硬失败，而不是在 emit 阶段伪装成静态调用。

## HIR / MIR / LIR 详细边界

### HIR 做什么
- `HIR` 是语言真相第一次闭合的地方，负责 `resolve`、类型检查、约束绑定、名称消歧与结构化语法糖收束。
- `HIR` 必须显式区分调用种类：`static dispatch`、`witness dispatch`、`effect-handler dispatch`。
- `row` 不在这里以下游对象形式保留；`HIR` 只保留“方法 requirement 已满足、成员已选定”的闭合事实。
- `trait / imply` 必须在这里保留具名 witness / evidence 来源，以及关联类型绑定等协议事实。
- `class / sealed class` 在这里完成名义子类型判定，`unite` 在这里完成 variant 归属与穷尽性分析，不允许后端再通过结构匹配重猜。
- `effect` 在这里至少要保留 effect capability、operation 与 handler 绑定事实。
- `HIR` 不负责单态化、内联、分区决策、目标文件布局，也不允许把暂时不会表达的调用偷改成普通静态函数名。

### HIR 不该交给 nyar 什么
- 不能把“仍需 trait resolve 才知道调谁”的调用交给 `nyar`。
- 不能把“仍需 row method match 才知道成员”的调用交给 `nyar`。
- 不能把“仍需 nominal subtype 判定才知道是否合法”的值流交给 `nyar`。
- 不能把“仍需选择 effect handler” 的调用交给 `nyar`。

### MIR 做什么
- `MIR` 是主分析表示，必须是 `SSA`，具备 `CFG`、基本块、块参数、显式终结符和显式值依赖。
- `MIR` 负责单态化、去虚化前提分析、effect 摘要、逃逸分析、循环分析、数据流分析，以及闭世界展开。
- `MIR` 必须把 `HIR` 的结构化调用降成可分析形式：
- 当前调用属于哪种 dispatch。
- 若为 `witness dispatch`，对应 contract / slot / evidence operand 是什么。
- 若为 `effect dispatch`，对应 effect operation 与 handler binding 是什么。
- 若源自 `row`，只保留已选定成员与已验证签名，不保留 row witness。
- `MIR` 是语言语义与目标约束之间的主要分界线，也是“哪些调用必须在某目标前静态化”的裁决层。

### MIR 在自举阶段的现实策略
- `CLR` 自举优先时，`MIR` 必须负责把能静态化的 witness / effect 调用提前静态化。
- 若某个 `CLR` 调用仍然需要开放 witness/effect，而当前 `CLR` lane 还不会忠实 lowering，就必须在 `MIR -> Optimize -> ArtifactPartitionPlan` 之间硬失败。
- 不允许把开放 witness 伪装成 `call_static` 混过后端。

### LIR 做什么
- `LIR` 不是单一共享类型，而是目标分区后的低层表示族。
- `ArtifactPartitionPlan` 之后，每个分区进入自己的 target lane，再形成对应路线的 `LIR / Backend Input`。
- `CPU / VM` 线可以继续走 `NyarIR` 风格低层表示。
- `CLR` 线应进入更贴近 `ECMA-335` 的低层表示，例如 `ClrImage + Metadata + MSIL + PE`。
- `JVM` 线应进入 `ClassFile` 风格低层表示。
- `WASM` 线应进入结构化控制流和 section 模型。
- `GPU / Shader` 线必须直接进入 `DXIL / SPIR-V / MSL` 等专用表示。
- `LIR` 负责调用约定、布局、平台映射、目标元数据与编码前准备，但不再承担语言级重分析。

### LIR 与 Backend Input 的关系
- 对某些路线，`LIR` 与 backend input 可以几乎重合，例如 `CLR` 线上的 `ClrImage`。
- 对某些路线，`LIR` 仍可细分成“目标专用低层指令表示”和“最终容器表示”两段，例如 `native` 线的低层操作模型与最终 `COFF / PE`。
- `nyar` 的 `data_formats/*` 放的是这类目标专用低层表示和最终容器，不是上游语言级 `LIR` 的总超集。

## nyar 与 HIR / MIR / LIR 的接口
- `valkyrie-compiler` 负责 `HIR -> MIR (SSA) -> Optimize -> ArtifactPartitionPlan`。
- `nyar` 从 `ArtifactPartitionPlan` 之后开始接手：target lane、backend input、validate、compile、packaging。
- `nyar` 不拥有 `HIR`，不拥有语言级 `MIR`，也不拥有“所有目标共用的一份 LIR”。
- `nyar` 拥有的是目标相关 `LIR / Backend Input` 家族，以及最终容器与产物交付协议。

## 目录与详细设计
- `src/abstractions`：目标无关公共协议，例如 `TargetFamily`、`BackendInputKind`、最小 backend trait。
- `src/lanes`：从 `ArtifactPartitionPlan` 分区承接到 target-specific input 的路线层。
- `src/selection`：根据 lane、input kind、target 选择最合适的 backend。
- `src/backends`：每个后端显式声明 `validate()` 与 `compile()`，不再吃一份万能输入。
- `src/data_formats/*`：目标专用低层表示与容器模型，例如 `MSIL`、`PE`、`COFF`、`ClrImage`。
- `src/packaging`：`OutputSpec`、`ArtifactSet`、sidecar 和交付规则。

## 路线分工
- `CPU/VM` 线可以继续使用更语义化的 `NyarIR` 或后续等价低层输入。
- `CLR / JVM / WASM` 线必须允许各自拥有更贴近目标约束的 backend input，不要求共用 `NyarIR`。
- `GPU / Shader` 线必须直接进入目标专用模型，例如 `DXIL / SPIR-V / MSL`，不能再借道 CPU 导向兼容壳。
- `CLR` 自举是当前最高优先级，因此 `CLR` lane 必须优先保证“拒绝未闭合 witness 伪装成静态调用”。

## 禁止
- 不在这里放 `Valkyrie` 专属 `AST / HIR / MIR / LIR`。
- 不把这里做成跨语言统一 `god ir`。
- 不让语言前端语义直接耦合进 `PE / COFF / MSIL` 容器层。
- 不把 `Generate*` 风格的大统一 lowering 总线搬回 Rust。
- 不让后端容器模型承担语言级补语义职责。

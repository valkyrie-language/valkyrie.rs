# nyar src

这里是共享后端基础设施源码。

## 职责
- 维护目标容器、镜像、文本汇编和产物编排协议。
- 为 `CLR / JVM / WASM / native / VM` 等路线提供公共后端数据层。

## 分层原则
- `src/abstractions` 只放最小公共协议，不放统一物理 `IR`。
- `src/packaging` 只放 `ArtifactPartitionPlan` 之后的承接协议，例如 `ArtifactSet`、`OutputSpec`、lane 分发边界。
- `src/data_formats/*` 只放目标路线自己的容器和低层输入模型，例如 `MSIL`、`PE`、`COFF`。
- 任何目标专用约束都应在本路线自己的数据模型里表达，而不是回流到共享大对象。

## 与主链的关系
- `HIR / MIR` 才是语言语义的主表示；`nyar` 不替代这条主链。
- `Optimize (EGraph)` 与 `ArtifactPartitionPlan` 负责决定哪些调用已静态化、哪些分区走哪条路线。
- `nyar` 只接住已经完成语义定界的后端输入，并把它们送往目标相关编码、布局和打包。

## 详细分层

### abstractions
- 放最小公共协议，例如目标家族、backend input kind、产物格式与 backend 最小接口。
- 不放 `HIR`、不放 `MIR`、不放统一 `LIR`。

### lanes
- 负责承接 `ArtifactPartitionPlan` 输出。
- 每个 lane 只把某个分区降到本路线真正能消费的 backend input。
- 这里不做 trait resolve，不做 row 闭合，不做 effect handler 选择。

### backends
- 每个 backend 必须显式声明自己吃哪种 input。
- `validate()` 必须诚实拒绝错误路线、未闭合 witness/effect、未满足目标约束的输入。
- `compile()` 只能处理通过验证的输入，只做目标相关 lowering、编码、布局和产物生成。

### selection
- 根据 lane、input kind、target 和优先级选择后端。
- 这层只做选择，不做语义修复。

### data_formats
- 表示目标分区之后的低层表示族。
- 这些类型既不是 `HIR`，也不是语言级 `MIR`，更不是跨目标统一 `LIR`。
- `clr`、`msil`、`pe`、`coff` 只是各自路线的真实低层输入或容器。

### packaging
- 负责 `ArtifactSet`、`OutputSpec`、sidecar 和最终交付协议。
- 不负责重新解释调用语义。

## HIR / MIR / LIR 与源码目录的关系
- `HIR / MIR` 的定义与变换应继续留在 `valkyrie-compiler`。
- `nyar` 不定义语言级 `HIR / MIR`，只消费它们在 `ArtifactPartitionPlan` 之后形成的 target-specific 结果。
- `nyar` 内部的 `data_formats/*` 更接近目标相关 `LIR / Backend Input`，而不是上游语义主表示。
- 若未来 `CPU / VM` 线需要 `NyarIR`，它也应被视为 `CPU / VM` 路线的低层表示，而不是所有目标共享的最后一层。

## 禁止
- 不在这里闭合语言语义。
- 不在这里重建统一跨端大 `IR`。
- 不让某个语言的前端事实反向绑架公共后端层。
- 不让后端容器层替上游补语义。
- 不让 `MSIL / PE / COFF` 数据模型反过来决定前端和中层的设计。

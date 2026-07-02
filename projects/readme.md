# `nyar + valkyrie` 全架构迁移进度

最后更新：`2026-07-09`

## 目标

- 让 `valkyrie` 语言前端只负责语言语义、分析输入和中性规划。
- 让 `nyar-analyzer` 承接多数语言的 `HIR` 辅助分析。
- 让 `nyar-optimizer` 承接多数语言的优化与等式系统宿主职责。
- 让 `nyar` 负责中性规划、后端能力选择和发射调度。
- 让 `emitter` 只消费稳定的 driver 输入协议，不再反向侵入前端。
- 最终废弃前端 target-aware `LIR` 主链，改为 `fragment submission + backend interpreter`。

## 当前结论

- `nyar` 侧的 crate 拆分、规划层、选择层和 driver 家族分发已经基本成形。
- 但如果只看 `OA / egraph / PE` 三条主线，`nyar` 仍然只是“骨架已落地，语义还没完全长出来”：
  - `OA` 目前更接近组合边界和维度清单，还不是完整的 object algebra 解释体系。
  - `egraph` 目前更接近最小可用等价类宿主，还不是完整的 equality saturation 宿主。
  - `PE` 目前已经有 `projection policy + backend registry + interpreter selection` 骨架，但还没有真正进入解释器驱动特化。
- `valkyrie` 前端的中性提交接口已经落地，但仍处于过渡阶段。
- 最大未完成项集中在 `nyar-language`：
  - 仍保留 crate 内部 legacy `LIR` 过渡资产。
  - 仍残留 crate 内部显式 legacy `LIR lane` 入口。
  - 仍残留 `clr / wasm / wasi / jvm / com` 品牌级互操作表面标签识别与相关测试护栏。
- `validation::ControlFlowScheduler` 已收缩到 `HIR / MIR`，只服务 legacy `LIR` 的跨层 pipeline 校验与内部校验噪音已经移除。
- `legion` 已明显退化为装配层，但仍保留一套临时 `BackendRegistry` 生成逻辑，尚未完全切到 `nyar` 独立注册层。
- `nyar-language/tests` 当前不能作为“架构正确性”的直接依据，因为仍残留部分 legacy `LIR` 兼容断言、`CLR` 预期与后端品牌语义。
- 不过前端测试树里最直接的 `codegen` 绑定已经先移除，避免后端断言继续反向绑回 `nyar-language`。
- 当前测试树里更直接的过渡噪音，已经从旧的 `control_flow_scheduler` 跨层校验绑定，收缩为少量显式 legacy `LIR` 兼容 helper 与后端品牌护栏。
- 因此当前正确顺序必须是：先摆正架构边界，再分层迁移测试集，而不是反过来用旧测试约束新架构。

## 总体状态

| 领域 | 状态 | 说明 |
| --- | --- | --- |
| `nyar` 核心规划与选择 | 基本完成 | 已有 `PlanningInput`、`ArtifactPartitionPlan`、`BackendRegistry`、`BackendSelector` |
| `nyar` `OA` 边界 | 部分完成 | 已有 `ObjectAlgebraicProgram / Dimension`，但仍偏结构清单，不是完整解释器代数 |
| `nyar` `egraph` 宿主 | 部分完成 | 已能吸收等式并做代表元提取，但仍不是完整 equality saturation |
| `nyar` `PE` / 投影调度 | 部分完成 | 已有 `ProjectionPolicy`、`ProjectionPlan`、`BackendRegistry`，但仍带过渡性映射与模板式特化 |
| `emitter` family 分层 | 基本完成 | `clr / jvm / wasm / native` 已有统一 family 入口 |
| `legion` 装配层收缩 | 部分完成 | 已不直接持有后端实现，但仍有临时 registry 逻辑 |
| `valkyrie` 前端中性提交 | 部分完成 | 已有 `FrontendNeutralPlan` 和 fragment 输入接口 |
| `valkyrie` 前端纯化 | 未完成 | `LIR`、legacy lane API、后端品牌名仍留在前端 |
| `nyar-language` 测试集迁移 | 未完成 | 已移除 `compile_source_to_lir()` 直连与旧 pipeline 断言，仍残留部分 legacy `LIR` 兼容断言与后端品牌护栏 |
| 文档与测试同步 | 部分完成 | 新链路测试存在，但旧文档和旧断言仍未清理完 |

## 已完成

### `nyar`

- 新工作区主链已经切到 `nyar-analyzer`、`nyar-optimizer`、`emitter`、`nyar-language` 等 crate。
- `nyar` 已具备中性规划骨架：
  - `PlanningInput`
  - `ArtifactPartitionPlan`
  - `BackendRegistry`
  - `BackendSelector`
- `lane_for_projection()` / `input_kind_for_projection()` 这类核心硬编码映射已经从规划主路径中移除，规划开始转向 registry 驱动。
- `selection` / `backends` / `planning` 三层已经形成可持续演进的分工边界。
- `OA` 主线已经有最小公共边界：
  - `ObjectAlgebraicProgram`
  - `ObjectAlgebraicDimension`
  - `ObjectAlgebraicBuilder`
  - `ObjectAlgebraicInterpreter`
- `egraph` 主线已经有最小公共边界：
  - `RewriteTheory`
  - `RewriteEquation`
  - `TheoryBundle`
  - `EGraphSnapshot`
- `PE` 主线已经有最小公共边界：
  - `ProjectionPolicy`
  - `ProjectionPlan`
  - `BackendRegistry`
  - `BackendInterpreterSelection`

### `emitter`

- driver 的 bundled backend family 分发已经模块化。
- `clr`、`jvm`、`wasm`、`native` 已接入统一 family 入口。
- 对外公共接口已明显收缩，family 细节不再堆在单一大文件里。
- driver 侧已经开始消费更真实的 fragment 语义元数据，而不是只消费模块名。

### `valkyrie`

- `FrontendBuildOutput` 已不再公开旧式 `lir_module` 主输出。
- `FrontendNeutralPlan` 已成为语言前端的中性提交结果。
- 前端已经可以按 artifact partition 提供 backend input fragment。
- `semantic fragment` 已开始携带：
  - `fragment id`
  - `exported operations`
  - `required capabilities`
  - `entry operation`
  - `rewrite theory`

### `legion`

- `legion build` 已基本退回轻量编排层。
- 上层构建链已能沿着 `FrontendBuildOutput -> ArtifactPartitionPlan -> emitter` 装配。
- 集成测试已覆盖 `CLR / JVM / WASM / native` 的 bundled 构建路径。

## 部分完成

### 前后端解耦

- 前端已经开始走中性规划，但还没有彻底忘掉后端品牌。
- `frontend_contract/planning.rs` 目前已经只消费中性的 `ExternalImportLink`，不再持有
  `resolve_wasm_import()`、`resolve_clr_import_ref()`、`build_clr_method_signature()` 这类品牌 helper。
- 当前残留的品牌知识主要收缩在 `backend_contract/interop.rs` 对合法语法标签
  `clr / wasm / wasi / jvm / com` 的识别，以及语言表面测试对这些标签的护栏。
- 这说明规划主路径已经基本去品牌化，但互操作语义的最外层表面仍未完全与后端品牌脱钩。

### `LIR` 过渡层

- `LIR` 已经不再是唯一主链，但还没有真正退出舞台。
- `nyar-language` 根导出与 `valkyrie` 模块导出已经进一步收口，外部不再通过 crate root 直接拿到 legacy `LIR` 类型与 target lane 选择器。
- `LIR` 仍然只是过渡性的低层视图，而不是纯中性片段提交。
- legacy `LIR` 入口已收敛为无 target lane 参数的内部兼容视图，不再允许前端 API 携带目标选择语义。
- 只服务 legacy `LIR` 的 `validation` 辅助实现已经收缩，`ControlFlowScheduler` 现在只保留 `HIR / MIR` 校验入口。
- 测试树中的 `compile_source_to_lir()` facade 调用已经清空，legacy lowering 只在测试侧通过显式 helper 保留。
- `compile_source_to_lir()` / `lower_root_to_lir()` 已正式标记为 deprecated legacy facade，继续降级为兼容层。

### 优化宿主

- `nyar-optimizer` 已不再只是空壳规则筛选器，等式系统宿主已经开始成形。
- 但它距离完整的 `egraph / equality saturation` 宿主仍有距离。
- 当前状态更接近“最小可用雏形”，还不是最终形态。

### `nyar` 的 `OA`

- 现在的 `ObjectAlgebraicProgram` 明确拒绝统一节点池式 `god IR`，这一点方向是对的。
- 但当前 `OA` 仍然主要表现为：
  - 模块名
  - 导出操作
  - 语义维度清单
  - 能力标签
  - 引用管理提示
- 这说明当前 `OA` 更像“组合边界和解释入口目录”，还不是能承载语言语义构造与后端解释器代数的完整 object algebra。
- 换句话说，`nyar` 已经摆脱了“大一统节点枚举”的方向错误，但还没有完全进入“解释器按代数消费前端语义”的完成态。
- 这里必须额外写死一个约束：最终形态不能退化成单体 `ProgramAlgebra`。
- 对 `c++ / js / rust / sql / python` 这类异构语言，正确完成态不是“所有语言都实现同一个总代数”，而是：
  - 以前端 `semantic fragment` 为单位提交语义
  - 每个 fragment 只暴露自己需要的 object algebra 片段
  - `nyar` 以 `theory bundle + fragment capability` 组合这些片段
  - backend 按 capability 注册解释器，而不是按语言名或统一 IR 节点池消费输入
- 也就是说，`ObjectAlgebraicProgram / Dimension` 现在只能算“目录层边界”，后续必须继续长成 fragmented object algebra，而不是换皮的统一 IR。

### `nyar` 的 `egraph`

- 当前 `OptimizationSession` 已经会：
  - 按 capability 过滤规则和等式
  - 把操作吸收到最小 `EGraphHost`
  - 对等式做 `union`
  - 通过代表元提取 canonical program
- 但它目前仍缺少完整 `egraph` 宿主的关键部分：
  - 模式级匹配与重写触发
  - congruence closure
  - rebuild
  - 成本模型驱动 extraction
  - 真正的 equality saturation 迭代
- 所以当前最准确的描述不是“已经有 `egraph`”，而是“已经有最小等价类宿主，可以作为真正 `egraph` 的起点”。
- 最终目标也不是把所有语言先压成一个统一项语言再做 union-find，而是：
  - 让 fragment 自带 theory
  - 让共享 theory 进入 shared bundle
  - 让 `egraph` 在 fragment theory 与 shared theory 的交汇处做饱和和抽取
  - 让 cost model 按 target / capability / host boundary 选代表元

### `nyar` 的 `PE`

- 当前 `PE / Futamura Projection` 主线已经有三块骨架：
  - `ProjectionPolicy` 负责投影策略
  - `ProjectionPlan` 负责投影结果收口
  - `BackendRegistry` 负责 capability/interpreter 选择
- **`legacy-vm` 定位（已拍板）：** 语言无关的 PE / Futamura 解释器基板（GraalVM/Truffle-like）。它不是又一个语言运行时：怪语言靠 **guest 插拔**（`std-data` AST + `nyar-language` interpret + 薄适配），统一的是注册 / 调度 ABI；`run` = interpret，PE 第 1 投影产品路径是 **specialize → native residual → Windows PE**，**不是** `nyar-vm`（`.nyar` 至多探针）。也 **不等于** Valkyrie→nyar→CLR/JVM/WASM 手工全平台链。分层约定见 [`host-script-languages.md`](host-script-languages.md) / [`legacy-vm/readme.md`](legacy-vm/readme.md)。
- `ArtifactPartitionPlan` 也已经在优化后走 `backend_registry.resolve(...)`，说明核心规划层不再直接用旧式 `projection -> lane` 的硬编码主链。
- 但当前 `PE` 仍然不是完成态，因为：
  - `legion/src/cmds/build/mod.rs` 仍保留 `projection_family_for_backend()` 显式 `TargetBackendFamily -> FutamuraProjectionFamily` 装配映射
  - `legion/src/cmds/build/mod.rs` 仍保留 `backend_registration_shape()` 这类按 backend family 生成 interpreter/lane/input kind 的临时注册形状
  - driver 侧仍偏模板式 lowering，而不是 interpreter specialization
- 所以现在更准确的表述是：`PE` 的“选择与调度边界”已经有了，但“解释器特化执行”还没真正落地。
- 这里的完成态也必须写清楚：`PE` 不是“换个名字的 lowering”。
- 正确目标应该是：
  - backend 注册自己的 interpreter capability
  - `nyar` 先按 fragment / theory / target 做解释器选择
  - 再对选中的 interpreter 做 partial evaluation / Futamura projection
  - 最后把特化结果交给 emitter 物化成 artifact
- 也就是说，`ProjectedProgram` 如果存在，也只能是特化结果的承载，不是前端和后端共享的长期 god IR。

### 文档和规范

- 一部分维护文档已经切到新架构。
- 但仍有文档停留在旧的 `valkyrie-compiler / parser / types` 叙述上。
- 测试与说明文件之间也仍有不一致之处。

### `nyar-language` 测试集

- 当前测试入口 `tests/main.rs` 仍按旧聚合方式统一装载 `pipeline / spec / type_checker / typing / lsp`，只是最明显的前端 `codegen` 聚合入口已经移除。
- 测试集中不再直接调用 `compile_source_to_lir()`，legacy lowering 已下沉成测试侧显式 helper。
- `pipeline/control_flow_scheduler` 目录已不再直接调用 `validate_lir_module()` / `validate_pipeline()`，旧跨层校验接口不再被测试固化成主契约。
- 还有语言表面测试直接使用 `[clr(...)]` 这类后端品牌级注解。
- 当前更明显的品牌护栏并不只是 `clr`，`backend_contract/interop.rs` 还内建识别 `clr / wasm / wasi / jvm / com` 这些平台标签，并由对应测试保护其参数形状。
- 原 `tests/valkyrie/codegen` 已从 `nyar-language` 测试树删除；对应断言后续应在 `nyar` 或 driver 装配层按真实后端边界重建，而不是回灌到前端 crate。
- 这意味着当前测试集里混杂了三种不同性质的护栏：
  - 语言语义护栏
  - 旧 pipeline 过渡护栏
  - 具体后端品牌耦合护栏
- 在架构迁移完成前，不能把这三类测试继续视为同一层面的真相来源。

## 未完成

### `valkyrie` 前端纯化

- legacy `LIR` 已降为 crate 内部过渡资产，不再作为前端公开 API 暴露。
- legacy `LirTargetLane` 已删除，旧的显式 lane 选择步骤也已随之移除。
- `runtime_lane_prefix()` 已删除，runtime carrier 命名不再携带 `clr / jvm / wasm / native / vm` 品牌前缀。
- 多处测试仍把 legacy `LIR` 兼容形状和平台品牌标签当成基线护栏。

### 真正的互操作边界

- 前端还没有把互操作统一收敛为语言拥有的中性 contract。
- `frontend_contract/planning.rs` 已经不再持有 `clr / wasm` helper。
- 目前残留的后端品牌知识泄漏，主要是 `backend_contract/interop.rs` 对
  `clr / wasm / wasi / jvm / com` 表面标签的识别，以及测试里把这些表面标签当成架构契约的护栏。
- 这些逻辑应该迁到：
  - `backend_contract/interop`
  - 或独立 backend registry / interpreter 注册层

### `legion` 注册层收束

- `legion` 仍在本地拼装临时 `BackendRegistry`。
- 这层还没有完全切到 `nyar` 提供的独立 bundled 注册入口。
- 只要这一步没完成，装配层就还残留“知道太多”的问题。

### `nyar` 的 `OA / egraph / PE` 完整化

- `OA` 还没有把“前端按 fragment 提交什么语义代数片段、后端解释器怎样按 capability 消费这些片段”这条链闭合起来。
- `egraph` 还没有从“等价类合并器”升级为真正的 equality saturation 宿主。
- `PE` 还没有从“投影策略 + 注册选择”升级为真正的解释器特化执行模型。
- 因此 `nyar` 当前最核心的未完成项，不是 crate 数量或模块名，而是这三条语义主线尚未完全闭环。

### `OA` 完成态约束

- 不允许出现一个试图统一覆盖 `c++ / js / rust / sql / python` 的单体 `ProgramAlgebra`。
- 不允许把 `object algebra` 做成“接口化的统一 IR”。
- 允许存在共享的 algebra fragment，但必须按 fragment 组合，而不是按闭合节点全集统一。
- `nyar` 真正应该稳定的边界是：
  - fragment id
  - fragment-local algebra surface
  - theory bundle
  - required capabilities
  - backend interpreter selection
- 若未来新增语言特性需要扩展语义，优先新增 fragment 或扩展 fragment-local algebra，不允许回退到统一节点池。

### 解释器特化

- driver lowering 虽然已开始消费 fragment 元数据和 theory bundle，
  但本质仍偏模板式特化。
- 它还没有真正进入基于 backend capability / interpreter 的解释器驱动特化。
- `legion` 当前仍在 `build/mod.rs` 里本地生成 `BackendRegistry`，并通过 `backend_registration_shape()` 把 backend family 映射成具体 interpreter/lane/input kind 形状。

### 测试集迁移顺序

- 当前不能先整体修 `nyar-language/tests`，否则会把旧 `LIR`、平台品牌注解和已收缩接口重新固化为“正确行为”。
- 正确顺序应该是：
  1. 先清理前端公共边界中的 `LIR` 与后端品牌泄漏。
  2. 再把 `OA / egraph / PE` 的真实提交边界稳定下来。
  3. 最后按测试类型分批迁移测试集。
- 测试迁移时要明确分层：
  - `spec / type_checker / typing`
    - 这类测试优先保留，目标是保护语言语义。
  - `pipeline`
    - 需要先拆掉 `control_flow_scheduler` 对 `validate_lir_module()` / `validate_pipeline()` 的旧接口依赖，再改写成围绕 `HIR / MIR / neutral plan / fragment submission` 的中性断言。
  - `codegen / smoke / 部分 runtime pipeline`
    - `codegen` 已先从前端测试树剥离；后续需要把相关断言落到 `nyar` 或 driver 层，并从“默认 `CLR` lane”断言迁移到“分区计划 + backend input + driver 装配”断言。
  - `language_surface` 中的后端品牌注解场景
    - 需要先判断哪些属于合法 interop 语法，哪些属于应该迁出的后端泄漏。
- 在测试迁移完成前，允许保留一部分过渡性旧测试，但必须把它们视作“待拆旧护栏”，而不是新架构的最终契约。

### 控制流统一 TODO

- 这部分合并自 `nyar-language` 下已过期的 `control_flow.next.md` 分流文档，只保留仍未闭合的控制流待办。
- `HIR`：
  - 补齐 `.block` 的完整合法上下文，而不只停留在“函数体允许、`lambda` 拒绝”的最小闭环。
  - 补齐 `yield / yield from` 在函数体之外更细粒度的允许上下文与诊断。
  - 扩大 `break expr / return expr` 的静态兼容性检查范围，不再只覆盖函数参数、显式类型局部与 `Future<T> / Promise<T>` 的最小闭环。
  - 继续补齐更深的 pattern 变体、对象/数组 `rest` 绑定与跨 arm merge 规则。
- `MIR`：
  - 继续把 `catch / resume` 从“已显式 continuation 元数据化”推进到真正可执行的 continuation runtime 语义。
  - 完成 `yield / await / block` 的 runtime / frame 主链，补上状态机 emit、更强的 spill 收敛与多 lane 实际消费。
  - 扩大 `await / block` 的恢复类型闭环，不再只覆盖 `Future<T> / Promise<T>` 的最小类型形状。
  - 继续完善 `case statement` 的更深 pattern 变体与跨 arm merge 规则。
- `LIR`：
  - 只允许作为 crate 内部 lane-aware 承载层存在，不再继续发明新的公共控制流语义面。
  - 继续保证 `Jump / Branch / Return / PerformEffect`、`case_chains`、`frame_layouts`、`runtime_frames`、`runtime_continuations` 这些过渡资产只做保真透传，不引入新的后端品牌逻辑。
  - 若 `fragment submission + backend interpreter` 已能直接消费对应控制流元数据，则优先删除等价的 `LIR` 过渡承载。
- 统一原则：
  - 非法控制流源形状继续尽量前推到 `HIR` 出口拒绝。
  - `MIR` 负责显式 `CFG + block parameter SSA + suspend/resume`。
  - `LIR` 若仍存在，只负责内部承载，不再承担长期语义边界。

## 关键里程碑追踪

### 第一阶段：crate 拆分与主链收束

- [x] 建立 `nyar-*` 分层 crate
- [x] 让 `legion` 退回装配层
- [x] 建立 `emitter` family compiler 分发层

### 第二阶段：前端中性提交

- [x] 建立 `FrontendNeutralPlan`
- [x] 收缩 `FrontendBuildOutput`
- [x] 建立 fragment 级 backend input 接口
- [ ] 前端彻底忘掉具体后端品牌

### 第三阶段：规划与选择统一

- [x] 建立 `PlanningInput` / `ArtifactPartitionPlan`
- [x] 建立 `BackendRegistry` / `BackendSelector`
- [x] 开始用 registry 驱动规划
- [ ] 将 `legion` 的临时 registry 逻辑完全迁出

### 第四阶段：`OA` 边界固化

- [x] 建立 `ObjectAlgebraicProgram / Dimension` 组合边界
- [x] 拒绝统一闭合节点池式 `god IR`
- [ ] 建立按 fragment 组合、可被解释器消费的 object algebra 提交形态
- [ ] 让 backend interpreter 面向语义代数而不是模板输入特化
- [ ] 写死“禁止单体 `ProgramAlgebra` 回摆”的完成态约束

### 第五阶段：`egraph` 宿主成形

- [x] 建立 `RewriteTheory` / `TheoryBundle` 基础结构
- [x] 建立最小可用等价类宿主雏形
- [ ] 升级为真正的 `egraph` 宿主
- [ ] 引入完整 saturation / rebuild / extraction 机制

### 第六阶段：`PE` / 解释器特化

- [x] 建立 `ProjectionPolicy` / `ProjectionPlan`
- [x] 建立 registry 驱动的 interpreter 选择入口
- [ ] 迁出 `legion` 中的过渡性投影装配映射
- [ ] 让 backend interpreter 真正消费 `TheoryBundle + fragment entry`
- [ ] 让 driver 从模板式 lowering 升级为解释器驱动特化

### 第七阶段：废弃旧 `LIR` 主链

- [x] 主构建链已不再依赖 `LIR` 作为唯一出口
- [x] 移除默认 `CLR` legacy 入口
- [x] 移除 `valkyrie` 对 `LIR` 的公共导出
- [x] 收缩显式 legacy `LIR lane` 接口
- [x] 删除 `runtime_lane_prefix()` 一类 lane 品牌逻辑
- [x] 将剩余 `LIR` 降级为内部过渡资产或直接删除
- [x] 删除仅服务 legacy `LIR` 的跨层校验与内部 validation 噪音

### 第八阶段：测试集分层迁移

- [ ] 将 `nyar-language/tests` 从“旧 pipeline 聚合包”拆成语义护栏与过渡护栏
- [x] 先移除前端测试树中的 `codegen` 聚合入口，避免后端断言重新绑回 `nyar-language`
- [ ] 清理 legacy `LIR` 直连型测试
- [ ] 清理 `control_flow_scheduler` 对已收缩 `LIR` 校验接口的断言
- [ ] 将 pipeline 测试迁移到 `neutral plan / fragment submission` 断言
- [ ] 重新界定合法 interop 测试与后端品牌泄漏测试

## 当前优先级

1. 清理 `nyar-language` 中的后端品牌泄漏，尤其是 `LIR` 与 `backend_contract/interop.rs` 的表面品牌识别边界。
2. 把 `nyar` 的 `OA` 从“结构清单”推进到真正的语义代数提交边界。
3. 把 `nyar-optimizer` 从最小等价类宿主推进到真正的 `egraph` 宿主。
4. 把 `legion` 的临时 `BackendRegistry` 生成逻辑彻底迁移到 `nyar` 独立注册层。
5. 继续把 driver lowering 从模板式特化推进到 backend interpreter 驱动特化。
6. 继续拆掉 `nyar-language/tests` 里剩余的 legacy `LIR` 校验接口绑定与平台品牌护栏，再开始逐批修测试。
7. 清理文档、测试与代码现状之间的旧叙述和旧断言。

## 关键风险

- 如果 `LIR` 继续以公共前端接口存在，前端就会持续对后端知识泄漏。
- 如果 `OA` 只停留在维度和导出清单，最终会退化成“换皮元数据层”，而不是真正的 object algebra。
- 如果为了统一多语言语义而引入单体 `ProgramAlgebra`，最终只会把 god IR 从 `enum` 换皮成 `trait`。
- 如果 `egraph` 只停留在等价类合并，最终会退化成“union-find + 术语升级”。
- 如果 `legion` 继续自己组装 registry，装配层会重新变成架构硬编码聚集点。
- 如果 `TheoryBundle` 和 fragment entry 只进模板，不进 interpreter，`PE` 会退化成换皮 lowering。
- 如果在架构边界未收敛前就先大修 `nyar-language/tests`，测试集会反过来把旧 `LIR`、平台品牌护栏和过渡接口钉死。

## 测试集处理原则

- 先修架构，再修测试。
- 先保语言语义测试，再迁 pipeline 与 backend 过渡测试。
- 不允许为了让旧测试变绿而重新引入前端 target-aware 设计。
- 不允许把“当前实现恰好如此”误写成“新架构必须如此”。

## 建议维护方式

- 后续每次迁移落地时，同步更新本文件的：
  - `已完成`
  - `部分完成`
  - `未完成`
  - `关键里程碑追踪`
- 若某项已经完全落地，优先把它从“部分完成”移动到“已完成”，同时删除对应旧风险描述。
- 若新增过渡层或临时兼容逻辑，必须在本文件中明确标记其回收计划，避免临时方案长期固化。

# control flow next

`control_flow.next.md` 用来约束 `valkyrie-compiler` 下一阶段的统一控制流设计。

目标不是把所有控制流伪装成同一种后端指令，而是先在语言主链里把语义边界闭合，再分别交给各个 target lane 承载。

## 当前总览

- 已闭合：`label / continue / break / break expr` 的 `HIR -> MIR -> LIR` 主链、基础 region 校验、`loop_exit` block parameter merge 与 labeled loop 目标解析。
- 已闭合：`catch / resume / raise` 的 handler region 基础骨架、`catch_dispatch / catch_resume / catch_exit` CFG、未匹配 effect 外抛、`resume value` 到 `catch_resume` 参数位的最小类型回灌。
- 已闭合：源码层原始 object pattern 主线，`case Foo { bar }` 走名义子类型规则，`case { foo, bar }` 走结构字段规则，未知布局 fallback 已统一收紧到 `PatternMatch + unsupported_pattern`。
- 已接通未运行：`yield / yield from / .await / .awake / .block` 已进入 `HIR / MIR / LIR PerformEffect` 骨架，并具备 `resume_target` 参数位；其中 `await / block` 已能在 `Future<T> / Promise<T>` 这类最小类型闭环下把恢复值类型写回恢复点参数位，但 runtime lowering、frame、spill 与 lane 运行协议仍未完成。
- 已接通未收尾：顶层调度模块已开始做 `HIR / MIR / LIR` 一致性校验；`case fallthrough` 相关 parser/AST 目前已补上 `literal / variable / wildcard / tuple / or-pattern` 这组源级形状并接入回归，独立 `case` 源级入口也已接通；当前剩余重点是更深的 pattern 形状、跨 arm merge 规则与多 lane 真实运行验证。

## 实现路线

### 路线一：Region Control 闭环

- 收口 `label / continue / break / break expr / early return` 的前端语义、`HIR` 校验与 `MIR` merge 规则。
- 保持 `loop_header / loop_exit` 的 block parameter SSA 稳定，不为单 lane 改写共享语义。

### 路线二：Handler 与 Pattern 闭环

- 收口 `catch / resume / raise` 的 handler region。
- 完成 constructor / type / object pattern 在“类型已知”与“未知布局 fallback”两种场景下的统一规则。
- 继续把恢复值、exit merge 与外抛路径显式化，直到 continuation 不再依赖临时占位。

### 路线三：Effect Runtime 闭环

- 推进 `yield / yield from / .await / .awake / .block` 的 effect 类型、恢复协议、runtime lowering。
- 接上 suspend 活跃变量分析、spill、state 编号与 frame layout。

### 路线四：调度与一致性闭环

- 扩展顶层调度模块，让 `HIR / MIR / LIR` 的 region/effect 规则在进入 lane 前全部完成一致性校验。
- 补齐 `case fallthrough` 的 statement 主链，避免和 `match` 表达式语义混线。

### 路线五：多 Lane 集成闭环

- 在前四条主线稳定后，统一推进 `CLR / JVM / WASM / native` 的真实运行一致性验证。
- 不接受为某一个 lane 打补丁式提前收尾。

## 2026-06-27 日报

### 已完成

- `Region Control`：`label / continue / break / break expr` 主链、labeled loop 目标解析、`loop_exit` merge 与 `HIR` 基础语义校验已经闭合。
- `Handler Control`：`catch / resume / raise` 已形成 `catch_dispatch / catch_resume / catch_exit` 骨架，未匹配 effect 外抛、`resume value` 类型回灌、`catch` guard 分发均已进入主链；本轮继续把 handler arm 中的恢复目标前推成显式 continuation 句柄，`catch_resume` 参数位会在 continuation 创建时直接预绑定到 block parameter，并显式携带当前已知恢复类型，不再只靠 `resume` 时按块名临时回查。
- `Handler Control`：`catch / resume / raise` 本轮继续把 continuation 从 lowering 内部栈约定前推成 `MirFunction / LirFunction` 可见的显式元数据载体，当前会正式记录 `dispatch_block / resume_target / resume_parameter / handler_exit / resume_parameter_type`，不再只让恢复协议散落在 block 命名与 builder 局部状态里。
- `Pattern Control`：原始 object pattern 主线已经闭合，具名模式按子类型规则、匿名模式按结构字段规则运行，未知布局 fallback 统一收口到 `PatternMatch + unsupported_pattern`。
- `Effect Runtime`：`await / block` 当前已开始按 `Future<T> / Promise<T>` 这类已知类型形状把恢复值类型写回 `await_resume / block_resume` 参数位，`yield / yield from` 的恢复点也继续固定回灌 `unit`，effect 恢复点不再一律退回无类型占位。
- `Effect Runtime`：本轮继续把 `PerformEffect` 前推成显式 suspend 点元数据，`MirFunction / LirFunction` 当前会正式记录 `state_id / effect / suspend_block / resume_target / resume_parameter_count / payload_type / spill_candidates`，为后续 frame、spill 与状态机 lowering 提供第一版稳定入口。
- `Effect Runtime`：本轮继续把 suspend 点从“只记元数据”前推到“产出分析结果”；`MIR` 当前已补上基于显式 `CFG + block parameter SSA` 的第一版活跃变量分析，`spill_candidates` 会按恢复边后的真实 live-out 结果收敛，不再简单等于 lowering 时看到的 bindings 快照；对应 `LIR` 透传与 `pipeline` 漂移回归已接通。
- `Effect Runtime`：本轮继续把 suspend live-out 结果前推成显式 `frame_layouts`；`MIR / LIR` 当前都会稳定记录每个 suspend `state_id` 对应的 `slot_index / value / value_type` 槽位计划，顶层调度模块也已开始校验 `frame layout` 与 `suspend spill` 是否一致，为下一步状态机 frame 与 lane runtime 承载补上正式桥面。本轮继续把这条桥面再往前推一格：默认 `CLR` lane 当前不仅会从 `frame_layouts` 派生显式 `runtime_frames` carrier，还已开始把这些 carrier 降成显式 `MSIL` runtime type。
- `Pipeline`：顶层调度模块已开始统一检查 `HIR / MIR / LIR` 的 CFG、block argument、`PatternMatch` 与 `PerformEffect` 一致性；新增 effect 恢复点参数个数与 payload 形状校验后，`Yield / DelegateYield / Await / AsyncBlock / Raise` 与 `AsyncSpawn` 的恢复点形状，以及 `PerformEffect` 不允许缺失 payload 的约束，已经开始在 `MIR / LIR` 两侧和跨层同步收口；本轮继续补上 built-in effect 的静态 payload 类型护栏与恢复点参数静态类型护栏，当前会在类型可见时前置拒绝把 `await / awake / block` 的 payload 篡成非 `Future<T> / Promise<T>` 形状，或把 `yield / yield from / await / block` 的恢复点参数类型改坏；裸 `yield` 会先按 `yield ()` 补成 `unit` payload。
- `HIR Effect`：局部静态类型收敛当前已继续扩到函数参数、带显式类型的 `let` 变量、`await / block` 的最小 future 结果类型，以及 `yield / yield from / awake` 的 `unit` 结果类型；`break expr` 与 `return expr` 都不再只依赖字面量和块形状做前置冲突检查，`.await / .awake / .block` 也已开始在操作数类型静态可见时前置拒绝非 future 源形状，无值 `return` 也会在非 `unit / void` 返回函数里直接报错；新增的上下文护栏当前会在 `lambda` 体内直接拒绝 `block`、`yield` 与 `yield from`，同时裸 `yield` 会按 `yield ()` 语义继续保留。
- `Repo Hygiene`：`control_flow_scheduler.rs` 已继续拆出 `src/pipeline/control_flow_scheduler/hir_validation.rs`，把整块 `HIR` 控制流语义校验迁入子模块；当前入口文件实测已降到 `762` 行，新增子模块为 `419` 行。`cargo-cry.log` 中该文件的“测试位置不当”已是旧日志残留，`large-files.log` 中的 `1005` 行旧值也已不再反映仓库实况，但 effect / MIR-LIR 一致性辅助逻辑仍待继续外迁。
- `Repo Hygiene`：`src/mir/ssa/mod.rs` 的模式匹配与 effect 恢复回归已迁到 `tests/mir/mod.rs`，同时补出 `mir::ssa::test_support` 只读测试辅助出口；当前 `pattern lowering`、`effect lowering`、`suspend_analysis` 与 `frame_planning` 已分别外迁到子模块，`src/mir/ssa/mod.rs` 当前实测为 `1521` 行，仍低于 `large-files.log` 中的 `1655` 行旧值，并已回归通过 `cargo test mir:: -- --nocapture`。
- `Effect Runtime`：`awake` 当前已重新确认保持“有 `resume_target`、但无恢复点参数”的语义，不再和 `yield / await / block` 共用单参数恢复点模板；对应 `MIR` 与 `pipeline` 回归已重新通过，避免把 `AsyncSpawn` 的空恢复协议误写成单参数协议。
- `Repo Hygiene`：`src/module.rs` 中残留的内联测试已移除并切回 `tests/module/` 外部入口，源码文件当前已压到 `218` 行；同时删除了已与当前 `LIR` 结构脱节、仍断言 `module_imports` 字段的老旧 `tests/module/pipeline.rs`，并回归通过 `cargo test module:: -- --nocapture`。
- `Repo Hygiene`：本轮新增的 `LIR runtime carrier` 回归未继续留在 `src/lir/validation.rs`；相关用例已外迁到 `tests/lir_runtime.rs`，避免再次触发 `cargo-cry.log` 的“测试位置不当”告警，同时保持 `LIR` 源码文件只承载校验实现。
- `Repo Hygiene`：`CLR` runtime carrier lowering 已开始从 `src/nyar_backend_bridge/clr_lowering.rs` 外迁到独立 `src/nyar_backend_bridge/clr_runtime_lowering.rs` helper，先把 `runtime_frames / runtime_continuations -> MSIL runtime type` 这一段拆出，避免继续把 lane lowering 主入口越堆越大；`large-files.log` 中的 `clr_lowering.rs` 热点仍未彻底解决，但已开始朝模块化方向收口。
- `Region Control`：`fallthrough` 当前已从最小 bridge 继续推进到独立 statement 主链；语句路径的 `match` 会在 `HIR` 先显式落成 `Case`，`MIR / LIR` 也已开始用 `case_arm_* / case_exit` 这组专用 block 承载 arm 间跳转；值语义 `match expression` 中的 `fallthrough` 仍会显式报错，并新增独立 `tests/fallthrough.rs` 回归覆盖“语句路径接通 / 值语义拒绝 / 最后一 arm 拒绝 fallthrough”，以及“重新进入下一 arm 入口并继续走其 pattern / guard 链”这四侧约束。
- `Pipeline`：新增 `rejects_mir_await_perform_effect_with_non_future_payload_type`、`rejects_lir_awake_perform_effect_with_non_future_payload_type`、`rejects_pipeline_when_effect_payload_static_type_drifts` 三条回归，继续把 built-in effect 的 payload 类型闭环收口到调度器层，并回归通过 `cargo test pipeline::control_flow_scheduler -- --nocapture`。
- `Pipeline`：`MirFunction / LirFunction` 已正式透传 `value_types`，顶层调度模块现可继续校验 effect 恢复点参数静态类型；新增 `rejects_mir_yield_resume_block_with_non_unit_parameter_type`、`rejects_lir_block_resume_block_with_wrong_parameter_type`、`rejects_pipeline_when_effect_resume_static_type_drifts` 三条回归，并回归通过 `cargo test pipeline::control_flow_scheduler -- --nocapture`。
- `Pipeline`：顶层调度模块当前已继续补上 `catch_resume` 的跨层参数类型对齐；新增 `rejects_pipeline_when_catch_resume_parameter_type_drifts` 回归，开始把 `catch / resume` 的恢复值类型闭环从“仅 `MIR` 回灌”推进到 “`MIR / LIR / pipeline` 可见”。
- `Pipeline`：顶层调度模块本轮继续补上 `Jump -> block parameter` 的静态类型护栏，当前不仅会校验参数个数，也会在 `MIR`、`LIR` 与跨层比对时拒绝把 jump argument 的静态类型漂移成与目标 block parameter 不一致；新增 `rejects_mir_jump_argument_type_drift_to_catch_resume_parameter`、`rejects_lir_jump_argument_type_drift_to_catch_resume_parameter`、`rejects_pipeline_when_jump_argument_type_drifts` 三条回归。
- `Pipeline`：顶层调度模块本轮继续补上 continuation 元数据护栏，当前不仅会核对 `MIR / LIR` continuation 数量与结构是否一致，也会显式拒绝 continuation 的恢复参数脱离目标 resume block、或 continuation 恢复类型在跨层间漂移；新增 `records_catch_resume_continuation_metadata`、`rejects_mir_continuation_when_resume_parameter_leaves_target_block`、`rejects_pipeline_when_continuation_resume_type_drifts` 回归，并回归通过 `cargo test mir:: -- --nocapture` 与 `cargo test pipeline::control_flow_scheduler -- --nocapture`。
- `Pipeline`：顶层调度模块本轮继续补上 suspend 点元数据护栏，当前会核对 `MIR / LIR` suspend 点数量、`state_id`、effect、`resume_target`、恢复参数个数、payload 静态类型与 `spill_candidates` 是否一致，并显式拒绝 suspend 点恢复参数个数与目标恢复 block 脱节；本轮同时继续补上 `frame_layouts` 一致性校验，显式拒绝 `slot_index / value / value_type` 与 suspend spill 计划脱节；新增 `records_await_suspend_point_metadata`、`keeps_only_live_values_in_await_spill_candidates`、`builds_frame_layout_from_suspend_spill_candidates`、`rejects_mir_suspend_point_when_resume_parameter_count_drifts`、`rejects_pipeline_when_suspend_point_payload_type_drifts`、`rejects_pipeline_when_suspend_point_spill_candidates_drift`、`rejects_pipeline_when_frame_layout_slots_drift` 回归，并回归通过 `cargo test mir:: -- --nocapture` 与 `cargo test pipeline::control_flow_scheduler -- --nocapture`。
- `Handler Control`：`resume` 之后当前已补成显式 `Unreachable` 后继块，不再把 arm 末尾错误回流到 `catch_exit` merge；这一步同时修掉了调度器新护栏暴露出来的“`resume` 后仍把 `unit` 合并进 handler exit” 的真实主链 bug。
- `Pattern Control`：`case / match` 的 parser/AST 当前已补上 `literal`、`variable`、`wildcard`、`or-pattern` 这组源级形状；parser 不再吞掉 `A | B` 后半段，`HIR` lowering 也已把它们稳定落进既有 `HirPattern::Literal / Variable / Wildcard / Or / Type` 主链，guard 统一改走 `pattern.guard()` 读取。
- `Pattern Control`：本轮继续补上 `tuple pattern`；`case ((x, y), z)`、`case (_, 0)` 这类元组模式当前已能在 parser/AST 中稳定表达，并进一步 lower 到 `HirPattern::Tuple`，不再在 `match` 源级入口直接报“不支持 tuple pattern”。
- `Pattern Control`：新增独立 `cargo test -p valkyrie-parser --test match_patterns -- --nocapture` 与 `cargo test -p valkyrie-compiler --test match_patterns -- --nocapture` 回归，同时修正 `return match ...` 在当前 `HIR` 中实际落在 `Return` 语句而不是 `body.expr` 的断言偏差；`cargo test -p valkyrie-compiler --test main compiler_facade_lowers_literal_variable_and_or_match_patterns_into_hir -- --nocapture` 当前也已重新通过。
- `Pattern Control`：源码层 pattern 现按四类重新收口：`case X()` 与 `case []` 走 extractor 语义，必须依赖显式声明且可重载的 extractor；`case X {}` 与 `case {}` 走原始字段或 getter 模式；字面量模式与 `A | B` 这类模式表达式各自独立。`Some((x, y))`、`[]` 这类形状后续不得再被当作原始字段模式直接 lower。
- `Pattern Control`：`match / case arm` 的 pattern 绑定当前也已正式接进 `HIR` 校验作用域；guard 与 body 会在各自 arm 的局部作用域下读取 pattern 绑定，`match` 结果类型推断也会在 arm 作用域里看 body，不再把 arm body 统一当成“无绑定裸表达式”。
- `Region / Pattern Control`：`fallthrough` 当前也已补上“上一 arm pattern 绑定不继承到下一 arm”的显式 `HIR` 护栏；若前一 arm 通过 `fallthrough` 落到后一 arm，而后一 arm 的 guard/body 非法引用了前一 arm 独有的 pattern 绑定，`HIR` 会在进入 `MIR` 前直接拒绝这类绑定泄漏。
- `Region / Pattern Control`：`case / match` 当前也已开始产出显式 `case_chains` 元数据；每条 case-like 链路都会稳定记录 `dispatch / first_arm / no_match / exit` 与各 arm 的 `entry/check/guard/body/next/fallthrough` 目标，供 `MIR / LIR / pipeline` 直接校验，不再只靠块名约定反推跨 arm merge。
- `Pattern Control`：`Tuple / Or` pattern 当前也已从“只能保留成通用 `PatternMatch`”前推到“类型已知时可执行”；`MIR` 会显式 lower `tuple_get_N + compare + logical_and / logical_or`，并继续透传到 `LIR / CLR / JVM` 主链。
- `Repo Hygiene`：`cargo-cry.log` 当前为空，没有新增仓库卫生告警；但 `large-files.log` 里的热点仍包括 `src/mir/ssa/mod.rs`、`valkyrie-parser/src/ast/mod.rs` 与 `valkyrie-parser/src/parser/mod.rs`，本轮未继续拆分。

### 进行中

- `Effect Runtime`：`yield / yield from / .await / .awake / .block` 已有 `PerformEffect` 骨架，当前也已补出 suspend 点元数据入口，接上了第一版基于 `CFG` 的活跃变量分析，并补出显式 `frame_layouts` 计划；但仍只有 `await / block` 在 `Future<T> / Promise<T>` 这类最小类型闭环下补上了恢复点参数类型，状态机 emit、lane runtime 消费与更完整的 spill 收敛仍未闭合。
- `Handler Runtime`：`catch / resume / raise` 当前已把 continuation 句柄、恢复参数与 continuation 元数据载体前推到 `MIR / LIR / pipeline` 主链；默认 `CLR` lane 也已开始从中派生显式 `runtime_continuations` carrier，并继续降成显式 `MSIL` runtime type。但 effect 类型选择、更广义动态形态判定，以及真正可执行的 continuation runtime 逻辑仍待继续补齐。
- `HIR Effect`：`.block` 合法上下文、`yield / yield from` 允许上下文当前只完成到“函数体默认允许、`lambda` 默认拒绝”的最小闭环；更细粒度的上下文规则，以及超出函数参数 / 显式 `let` 类型 / `Future<T> | Promise<T>` 最小闭环之外的复杂动态 `break expr / return expr` 兼容性仍待继续补齐。
- `Pipeline`：当前已经补上 effect 恢复点参数个数校验、payload 静态类型护栏、恢复点参数静态类型护栏、`catch_resume` 的最小跨层参数类型对齐、`Jump -> block parameter` 的静态类型护栏、continuation 元数据一致性校验、suspend 点元数据一致性校验，以及 `frame_layouts` 一致性校验；本轮继续把 runtime carrier 侧的 `LIR` 护栏补齐，开始显式拒绝 `runtime_frames / runtime_continuations` 与 `frame_layouts / continuations` 脱节。但 `catch / resume` 的恢复值类型跨层闭合、更广义 effect payload 类型规则与面向 runtime 的执行语义还未彻底收口。
- `Region / Pattern Control`：`case / match` 当前已具备 `literal / variable / wildcard / tuple / or-pattern / range / array / typed bind` 的源级 parser/AST 形状，独立 `case statement` 源级入口、constructor/object 的嵌套子模式、对象/数组 rest 绑定、`fallthrough` 的绑定隔离护栏以及显式 `case_chains` 也已接入主链回归；但更深的 pattern 变体与更广义 merge 规则仍待继续补齐。

### 未开始

- `CLR / JVM / WASM / native` 的多 lane 真实运行一致性验证。

### 决策

- 不再用“一个节点拆十几个阶段”的写法追踪进度，后续统一按 `Region / Handler / Effect Runtime / Pipeline / Lane` 五条主线记录。
- 日报只记录四类信息：`已完成`、`进行中`、`未开始`、`决策`，不再铺开细碎阶段流水账。
- `large-files.log` 中 `src/mir/ssa/mod.rs` 的旧值仍可作为热点参考，但当前真正新的仓库热点也包括已长到 `1443` 行的 `src/pipeline/control_flow_scheduler.rs`；后续推进时要优先继续把 frame / continuation 校验辅助逻辑外迁，避免再次把入口文件堆回超大模块。
- `fallthrough` 不再继续停留在“语句路径的 match arm body bridge”；当前已经切到独立 `HIR::Case -> MIR/LIR case_arm_*` 主链，且独立 `case` 源级入口与 constructor/object 的嵌套子模式也已接通；后续要继续把跨 arm merge 规则与更深的 pattern 变体补齐，避免语义面和源级表示继续分裂。
- `large-files.log` 中 parser 侧 `src/ast/mod.rs` 与 `src/parser/mod.rs` 也已成为本轮主线相关热点；下一轮若继续推进 source-level `case/pattern`，必须同步拆出子模块，避免在源级入口补功能时把大文件进一步堆高。
- 未知布局或无法静态判定的 pattern 继续保留 `PatternMatch`，不为局部测试成功伪造字段投影或 extractor 语义。
- 非法控制流源形状继续尽量前推到 `HIR` 出口拒绝，`MIR / LIR` 只承载已经闭合的合法控制流。
- `iterator`、`generator`、`co` 继续按能力递进但相互区分；其中 `co` 只覆盖 `.await / .awake / .block`，`resume` 继续归到 effect / handler 的元恢复层，不并入协程能力面。

## 目标

- 统一 `label / continue / break / early return / fallthrough / yield / yield from / .await / .awake / .block / catch / resume` 的编译语义。
- 保持结构化局部控制流与可恢复控制流的边界，不为追求“统一”而错误同构。
- 让公共前端和 `MIR` 先闭合控制流规则，再把结果交给 `CLR / JVM / WASM / native` 等 lane。
- 避免为了某一个后端的临时实现，反向污染语言语义层。

## 非目标

- 不把 `break`、`continue`、`return` 直接提升成宿主 runtime 的 effect 对象。
- 不把 `.await`、`.awake`、`.block` 绑定成某个宿主 API 名称。
- 不在 emit 阶段才决定某个位置能不能挂起或阻塞。
- 不让 `match` 承担 `case fallthrough` 的 statement 语义。

## 两类控制流

统一控制流不代表语义同质。下一版主链明确分成两类：

### Region Control

- `label`
- `continue`
- `break`
- `early return`
- `case fallthrough`

这类控制流的核心问题是：

- 跳到哪里
- 哪些值需要 merge
- 哪些 region 允许被跳出或跳回

它们首先是 `CFG + region + merge` 问题。

### Effect Control

- `yield`
- `yield from`
- `.await`
- `.awake`
- `.block`
- `catch`
- `resume`

这类控制流的核心问题是：

- 触发什么 effect
- 在哪里挂起
- 恢复到哪里
- 恢复值的类型是什么

它们首先是 `effect + continuation + suspend/resume` 问题。

## 源语言语义

### label

- `label` 只附着在可跳转 region 上：`loop`、`while`、`case`
- 不支持任意 block label
- `label` 只参与 `break label` 与 `continue label` 的目标解析

### continue

- 支持 `continue`
- 支持 `continue label`
- `continue` 只能指向循环 header
- `continue` 不带值

### break

- 支持 `break`
- 支持 `break expr`
- 支持 `break label`
- 支持 `break label expr`
- `break expr` 用于给目标 region 提供退出值

### early return

- 继续使用显式 `return expr`
- `return` 直接终止当前函数
- `return` 不参与 label 目标解析

### case fallthrough

- `fallthrough` 只属于 `case` statement 体系
- `fallthrough` 不进入 `match` expression 体系
- `fallthrough` 不携带值
- `fallthrough` 不继承上一分支的局部 pattern 绑定

### yield

- `yield expr` 表示向外层生成器处理器发出一个可恢复 effect
- `yield` 本身不是普通函数调用
- `yield` 的恢复值默认为 `void`

### yield from

- `yield from expr` 保留为独立源语义
- 第一阶段不强制改写成 `loop x in expr { yield x }`
- typed lowering 再决定是保留委托语义，还是展开成显式循环

### .await

- `expr.await` 表示挂起当前协程，等待 future 完成并恢复
- 它是语言后缀控制流，不是宿主 API 名称

### .awake

- `expr.awake` 表示启动 future 并继续当前控制流
- 当前协程不等待结果
- 它仍然属于 effect control，只是恢复值为 `void`

### .block

- `expr.block` 表示在同步边界阻塞等待 future
- `.block` 的上下文合法性必须在语义阶段检查
- 不允许把 `.block` 是否可用推迟到 backend 再决定

### catch / resume

- `catch` 安装 effect handler
- `resume value` 从 handler 恢复 continuation
- `resume` 只允许出现在 `catch` arm 内部

## 统一 effect 模型

下一版控制流统一到下面这条抽象：

```valkyrie
trait Effectful {
    type Resume
}
```

建议保留内建 effect 族：

- `Yield<T>: Effectful<Resume = void>`
- `Await<T>: Effectful<Resume = T>`
- `AsyncSpawn<T>: Effectful<Resume = void>`
- `AsyncBlock<T>: Effectful<Resume = T>`

高层语法糖统一脱到 effect：

- `yield x` -> `raise Yield(x)`
- `f.await` -> `raise Await(f)`
- `f.awake` -> `raise AsyncSpawn(f)`
- `f.block` -> `raise AsyncBlock(f)`

这样可以在共享的 `effect + continuation` 元抽象上统一承载恢复协议，但源码能力面仍需严格区分：

- `iterator`
- `generator`
- `co`

其中 `co` 只覆盖 `.await / .awake / .block`，`resume` 继续属于 effect / handler 的元恢复层，而不是协程能力的一部分。

## HIR 约束

`HIR` 必须保留足够高层的控制流意图，不能过早抹平成后端细节。

建议稳定以下节点：

- `Loop { label, pattern, iterator, condition, body }`
- `Break { label, expr }`
- `Continue { label }`
- `Return(expr)`
- `Case { label, scrutinee, arms }`
- `Fallthrough`
- `Yield(expr)`
- `YieldFrom(expr)`
- `Await(expr)`
- `Awake(expr)`
- `Block(expr)`
- `Raise(expr)`
- `Catch { expr, arms }`
- `Resume(expr)`

`HIR` 需要负责的事情：

- 作用域与 label 解析
- `break expr` 的目标 region 校验
- `.block` 的上下文合法性
- `Effectful::Resume` 的类型规则
- `yield from` 的基础可用性校验

`HIR` 不负责的事情：

- coroutine frame 布局
- state id 编号
- 某个 lane 的调度器调用细节

## MIR 约束

`MIR` 统一承接两类控制流，但必须保持显式 `CFG` 与显式 merge。

下一版 `MIR` 采用：

- 基本块
- 块参数
- 显式 terminator
- 显式 effect 挂起点

不引入传统独立 `phi` 节点，而是固定走 block parameter SSA。

### MIR Terminator

至少需要以下 terminator：

- `Return(value)`
- `Jump(target, args)`
- `Branch(cond, then_target, else_target)`
- `PerformEffect { effect, payload, resume_target }`
- `Unreachable`

这里的重点是：

- `break`、`continue`、`fallthrough` 不是单独 terminator 种类，而是 `Jump(...)` 到不同 region 目标
- `yield`、`.await` 不应该再伪装成普通 `Call`
- effect 控制流要显式暴露 `resume_target`

### 为什么必须做 block parameter SSA

以下场景都天然要求 merge：

- `continue` 需要 loop-carried value
- `break expr` 需要 exit merge value
- `fallthrough` 需要 case arm 间显式传值
- `yield` 与 `.await` 的恢复点需要显式接收恢复值

所以这里不是“以后优化器再做 SSA”，而是语义正确性本身就要求 block parameter。

## Lowering 规则

### Region Control lowering

- `loop / while` lowering 为 `header / body / exit`
- `continue` lowering 为 `Jump(header, carried_args)`
- `break` lowering 为 `Jump(exit, break_args)`
- `break expr` 要求目标 exit block 有明确参数位
- `fallthrough` lowering 为 `Jump(next_case_arm, case_args)`
- `return expr` lowering 为函数级 `Return`

### Effect Control lowering

- `yield expr` lowering 为 `PerformEffect(Yield, payload, resume_target)`
- `expr.await` lowering 为 `PerformEffect(Await, future, resume_target)`
- `expr.awake` lowering 为 `PerformEffect(AsyncSpawn, future, next_block)`
- `expr.block` lowering 为 `PerformEffect(AsyncBlock, future, resume_target)`
- `catch` lowering 为 handler region
- `resume value` lowering 为 continuation 恢复

`yield from` 第一阶段允许保留独立 MIR 语义，typed lowering 再决定是否展开。

## 类型规则

### break / continue / return

- `continue` 只能进入循环 header
- `break expr` 的值类型必须与目标 region 结果类型兼容
- `return expr` 的值类型必须与函数返回类型兼容

### yield / yield from

- `yield expr` 只允许在生成器上下文或存在对应 effect handler 的上下文中使用
- `yield expr` 自身表达式结果视为 `void`
- `yield from expr` 的目标必须满足可委托产出协议

### await / awake / block

- `expr.await` 要求 `expr: future<T>`，结果类型为 `T`
- `expr.awake` 要求 `expr: future<T>`，结果类型为 `void`
- `expr.block` 要求 `expr: future<T>`，结果类型为 `T`
- `.block` 的允许上下文由公共语义层检查，不能交给 lane 私判

### catch / resume

- `resume value` 只允许出现在某个 `catch` arm 内
- `value` 的类型必须等于被恢复 effect 的 `Resume`

## backend 边界

公共控制流模型和具体 target lane 的边界必须保持清晰：

- 前端和 `MIR` 负责定义控制流语义
- `LIR` 和 backend 只负责承载
- lane 可以决定 frame 布局与调度器桥接方式
- lane 不能重新定义 `.await`、`.block`、`yield` 的语言含义

换句话说：

- 是否允许阻塞，属于公共语义
- 如何阻塞，属于 lane/runtime
- 是否挂起，属于公共语义
- 如何恢复，属于 lane/runtime

## 实现顺序

建议按下面的顺序推进：

1. 打通 `label / break label / continue label / break expr`
2. 在 `MIR` 固化 block parameter SSA
3. 引入 `case` statement 与 `fallthrough`
4. 在 `HIR` 补齐 `.await / .awake / .block`
5. 把 `yield / await` 正式 lower 到 `PerformEffect`
6. 再做 suspend 跨越活跃变量分析、frame layout 与 state 编号
7. 最后逐 lane 落到 `CLR / JVM / WASM / native`

这个顺序先解决 region merge，再解决 suspend/resume，风险最低。

## 测试矩阵

下一阶段至少需要覆盖：

- `continue` 的 loop-carried value
- `break expr` 的 exit merge
- `continue label`
- `break label expr`
- `case fallthrough`
- `yield`
- `yield from`
- `.await`
- `.awake`
- `.block`
- `catch / resume`
- 多 lane 真实运行一致性

## 一句话原则

`break / continue / return / fallthrough` 统一到 `region CFG + block parameter SSA`。

`yield / yield from / .await / .awake / .block / catch / resume` 统一到 `effect + continuation + explicit suspend/resume`。

二者在 `MIR` 汇合，但不在源语言层硬伪装成同一种东西。

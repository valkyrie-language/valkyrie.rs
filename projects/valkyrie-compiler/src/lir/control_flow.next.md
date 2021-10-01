# lir control flow next

这份文档描述 `LIR` 层如何承载统一控制流，而不反向篡改语言语义。

`LIR` 的角色不是第二次设计控制流，而是把已经在 `HIR / MIR` 闭合好的控制流模型，整理成 lane-aware 的低层输入。

## 定位

`LIR` 是 target lane 感知的低层控制流入口。

它要解决的是：

- 哪个 lane 来承载这段控制流
- lane 需要哪些低层操作形状
- 哪些 `MIR` 信息必须原样保留下去

它不应该解决的是：

- 语言上能不能 `yield`
- `.block` 在这里合不合法
- `await` 到底是不是挂起

这些问题必须在 `HIR / MIR` 之前就已经收口。

## 当前推进

- 已闭合：`LIR` 继续稳定承载 `Jump / Branch / Return / PerformEffect`，`continue`、`break expr` 与 `raise` 的上层控制流目标没有在跨层时丢失。
- 已闭合：`PatternMatch`、`resume target`、block argument 与 `catch` 主链的跨层一致性校验已经接通，进入 backend 前的 lane 输入不再是黑盒。
- 已闭合：`resume value` 类型回灌、`await / block` 恢复点参数类型回灌、原始 object pattern 子类型/结构匹配、未知布局 fallback 收紧等新增 `MIR` 语义，当前都能稳定透传到 `LIR`；顶层调度模块当前也会继续比对 `PerformEffect` 的恢复点参数个数、payload 形状、built-in effect 的静态 payload 类型与恢复点参数静态类型，避免 lane-aware lowering 把 `await / block / yield / raise` 与 `awake` 的恢复点形状混写、静默丢掉必需载荷，或把 `await / awake / block` 的 payload 篡成非 `Future<T> / Promise<T>` 源形状，或把 `yield / yield from / await / block` 的恢复点参数类型篡坏；其中裸 `yield` 会先在上层补成 `unit` payload，再进入 `LIR` 承载。本轮继续补上 `Jump -> block parameter` 的静态类型对齐校验，`LIR` 侧当前也会显式拒绝 jump argument 与目标 block parameter 类型漂移。
- 已推进：continuation 元数据当前也会从 `MIR` 稳定透传到 `LIR`，lane-aware lowering 已能看到 `dispatch_block / resume_target / resume_parameter / handler_exit / resume_parameter_type` 这组显式恢复协议载体，而不必只靠 block 名称约定猜测 handler continuation 形状。
- 已推进：suspend 点元数据当前也会从 `MIR` 稳定透传到 `LIR`，lane-aware lowering 已能看到 `state_id / effect / suspend_block / resume_target / resume_parameter_count / payload_type / spill_candidates`，为后续 frame / spill 桥接保留正式承载位置。
- 已推进：`spill_candidates` 当前也不再只是 lowering 时顺手记录的一份 bindings 快照；`LIR` 现在会稳定透传 `MIR` 基于显式 `CFG` 活跃变量分析得出的 suspend live-out 结果，lane-aware runtime/frame lowering 后续可直接消费这份更收敛的 spill 候选集。
- 已推进：`LIR` 当前也开始稳定承载显式 `frame_layouts`；每个 suspend `state_id` 对应的 `slot_index / value / value_type` 槽位布局都会从 `MIR` 原样透传，为后续各 lane 的状态机对象、frame 字段与 runtime 恢复协议提供统一输入。本轮继续把这层承载从“只有元数据”前推到显式 runtime carrier：默认 `CLR` lane 当前已会稳定生成 `runtime_frames / runtime_continuations`，把 frame 槽位与 `catch_resume` 恢复协议落实成可直接消费的低层载体，并由 `LIR` 校验显式拒绝 carrier 与 `frame_layouts / continuations` 漂移；对应 `CLR` lowering 当前也已开始把这些 carrier 进一步降成显式 `MSIL` runtime type。
- 已推进：`fallthrough` 当前也已能沿 `MIR -> LIR` 保真透传；语句路径的 `match` 已在上层先落成独立 `Case` 语义面，`LIR` 会继续稳定承载来自 `MIR case_arm_*` 的 `Jump(next_case_arm, [])`，不把 arm 间串落偷偷揉回隐式顺序执行；对应新增回归也已钉住 `fallthrough` 会先回到下一 arm 的 `case_arm_N` 入口，再继续走该 arm 的 check / guard 链，而不是直接绕进 body。
- 已推进：`HIR` 当前已补上 `fallthrough` 的源级绑定隔离护栏；若后一 arm 试图偷用前一 arm 独有的 pattern 绑定，会在进入 `MIR / LIR` 前被拒绝，因此 `LIR` 当前接收到的 `case_arm_*` 输入已不包含这种跨 arm 绑定泄漏的非法源形状。
- 已推进：`LIR` 当前也开始稳定承载 `case_chains` 元数据；每条 case-like 链路的 `dispatch / first_arm / no_match / exit` 以及各 arm 的 `entry/check/guard/body/next/fallthrough` 目标都会从 `MIR` 原样透传，并由 `LIR` 校验与 pipeline 调度校验继续核对，避免 lane-aware lowering 只能靠块名约定猜测 arm merge。
- 已推进：随着 parser/AST 已补上 `literal / variable / wildcard / or-pattern / tuple / range / array / typed bind`，`LIR` 当前接收到的 `case / match` 低层输入不再局限于 constructor/object 这两条来源；对应上游新增的 parser/compiler 回归已经验证这些源级形状可以稳定进入 `MIR / LIR` 主链，而不需要 lane 侧再去猜测源模式。
- 已推进：constructor/object pattern 的源级 parser AST 当前也已前推成真正的递归子模式树；其中 `constructor / array` 已在 `HIR` 层统一归一化成 `HirPattern::Extractor(...)`，`LIR` 继续透传 extractor 调用与 tuple payload 解包，而 `Object` 保持原始字段/getter 模式。像 `Some((x, y))`、`Wrapper { inner: Some(result) }` 这类嵌套子模式当前都能沿 `MIR -> LIR` 保真透传。
- 已验证：`runtime_frames / runtime_continuations` 的最小回归当前已固定在 `tests/lir_runtime.rs` 外部入口，继续覆盖 carrier 生成与漂移拒绝，不再把这类回归塞回 `src/lir/validation.rs` 内联测试。
- 已闭合：`mir::ssa` 相关回归已经迁到 `tests/mir/mod.rs`，`LIR` 当前继续只承载外部已验证过的 `PerformEffect` / `PatternMatch` 形状，不再依赖 `src/mir/ssa/mod.rs` 内联测试维持稳定性。
- 当前边界：`LIR` 仍不新增 label 专有节点，也不重新解释 `yield / await / block / resume` 的语言语义，继续只做 lane-aware 承载；`case statement` 当前虽已拥有独立 `case_arm_* / case_exit` 低层 block 形状，并接通 `case_chains`、constructor/object 的嵌套子模式、`tuple / or / range / array / typed bind` 的显式低层输入与 `fallthrough` 绑定隔离护栏，但更深的 pattern 变体与更完整 merge 规则仍待继续补齐。

## LIR 的控制流职责

### 1. 保留显式 block 结构

`LIR` 必须继续保留：

- 基本块
- block parameter
- 显式 terminator

也就是说，`LIR` 不能回退成“长指令列表 + 隐式跳转”。

### 2. 保留控制流分类

`LIR` 必须看得出以下几种东西：

- 普通 `Jump / Branch / Return`
- witness 分发调用
- effect handler 调用
- effect 挂起点与恢复点

如果这些在 `LIR` 被揉平，后端就只能靠模式猜测，语义会开始漂。

### 3. lane-aware 承载，而不是 lane-aware 语义

lane 可以改变：

- 具体 opcode
- frame 布局
- 调度器桥接方式
- helper 调用名

lane 不可以改变：

- `yield` 是产出 effect
- `.await` 是挂起等待恢复
- `.awake` 是触发但不等待
- `.block` 是同步阻塞边界

## LIR 与 Region Control

`LIR` 对 `break / continue / return / fallthrough` 的任务比较直接：

- 原样保留 `Jump / Branch / Return`
- 不丢 block arguments
- 不偷偷消解 merge

也就是说，`LIR` 不负责“理解 break”，它只负责把：

- 跳到哪个块
- 带哪些参数

继续清晰传给具体 backend。

## LIR 与 Effect Control

`LIR` 对 `yield / await / block / resume` 的任务是：

- 保留它们不是普通调用的事实
- 保留 effect payload
- 保留恢复点信息
- 为 lane-specific runtime 桥接预留稳定位置

这里也要保留来源分层，而不是只保留一个模糊的“effect”标签：

- `.await / .awake / .block` 来自 `co`
- `yield / raise / catch / resume` 来自 effect / handler 元控制流

如果下一阶段 `MIR` 引入：

- `PerformEffect { effect, payload, resume_target }`

那么 `LIR` 必须继续保留等价的低层概念，而不是立刻重写成：

- 某个 `CLR` helper 名称
- 某个 `JVM` runtime API
- 某个 `WASM` host import

## LIR 建议补齐的低层形状

当前 `LIR` 已经有：

- `Return`
- `Jump`
- `Branch`
- `Call { dispatch, witness, effect }`

下一阶段建议补齐：

- 显式 effect 挂起操作
- 显式 resume 参数位
- 对 coroutine frame / generator frame 的 lane 层占位表示

但这些新增形状依然应是“lane-aware 的承载结构”，不是宿主 API 直写。

## 后端边界

### backend 可以决定的事情

- `CLR` 是否生成状态机类
- `JVM` 是否生成 continuation 对象
- `WASM` 是否桥接 host async import
- `native` 是否用线程、poller 或事件循环

### backend 不可以决定的事情

- 某个位置是不是允许 `.block`
- `yield from` 是不是普通循环
- `await` 是否还能当成普通方法调用
- `resume` 是否允许出现在 handler 外

这些如果交给 backend 再决定，就等于让不同 target 重新定义语言。

## LIR 对多 lane 一致性的要求

下一阶段所有 lane 都必须共享：

- 同一套 `Jump / Branch / Return` 语义
- 同一套 effect 分类
- 同一套 block argument 语义
- 同一套恢复值入口语义

允许各 lane 不同的只有：

- 实际承载对象
- runtime helper 名称
- 低层布局与调度协议

## 最小实现方向

如果 `MIR` 先完成：

- block parameter SSA
- `PerformEffect`
- `resume_target`

那么 `LIR` 的最小任务就是：

- 继续按 block 粒度保留这些信息
- 给 lane-specific lowering 一个稳定输入
- 不在这里重新展开源语言规则

## 一句话原则

`LIR` 只承载控制流，不重新定义控制流。

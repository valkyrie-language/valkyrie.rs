# mir control flow next

这份文档描述 `MIR` 层如何统一承接结构化控制流与可恢复控制流。

`MIR` 的职责不是重新定义语言语义，而是把已经在 `HIR` 闭合的控制流规则展开成显式 `CFG + SSA`。

## 定位

`MIR` 是统一控制流的汇合层。

在这一层：

- `break / continue / return / fallthrough` 被展开成显式 region 边
- `yield / await / block / resume` 被展开成显式 suspend/resume 边
- 所有 merge 都必须显式
- 所有恢复值都必须显式

换句话说：

- `HIR` 负责“语言上允许什么”
- `MIR` 负责“控制流图具体怎么走”

## 当前推进

- 已闭合：`continue / break / break expr` 的 `MIR` 显式 CFG 已稳定，labeled loop、`loop_exit` merge 与 block parameter SSA 已接通。
- 已闭合：`catch / resume / raise` 的 handler region 骨架已成形，`catch_dispatch / catch_resume / catch_exit`、未匹配 effect 外抛与 `resume value` 参数位回灌均已进入主链；本轮继续把 handler arm 内部的恢复目标前推成显式 continuation 句柄，`catch_resume` 参数位会在 continuation 创建时直接绑定到对应 block parameter，并保留当前已知恢复类型，不再只依赖 `resume` 时按块名回查。
- 已推进：continuation 当前不再只存在于 lowering 过程中的局部栈状态；`MirFunction` 已开始显式记录 continuation 元数据，包含 `dispatch_block / resume_target / resume_parameter / handler_exit / resume_parameter_type`，为后续 runtime/frame lowering 提供稳定承载面；本轮对应的 `LIR` 默认 lane 不仅会把它继续落实成显式 `runtime_continuations` carrier，`CLR` 路线还已开始把这组 carrier 继续降成显式 `MSIL` runtime type，说明这组 `MIR` 元数据已经能被下层正式消费。
- 已推进：`await / block` 当前也会按 `Future<T> / Promise<T>` 这类已知类型形状把恢复值类型写回 `await_resume / block_resume` 参数位，effect 恢复点不再一律退回无类型占位。
- 已推进：`yield / yield from / await / awake / block / raise` 当前也会记录显式 suspend 点元数据；`MirFunction` 已开始稳定携带 `state_id / effect / suspend_block / resume_target / resume_parameter_count / payload_type / spill_candidates`，为后续活跃变量分析、spill 与 frame lowering 提供第一版挂点。
- 已推进：suspend 点当前不再只把“当前 bindings 快照”粗暴记成 spill 候选；`MIR` 已补上基于显式 `CFG + block parameter SSA` 的第一版活跃变量分析，`spill_candidates` 会按恢复边后的真实 live-out 结果收敛，只保留跨 suspend 后仍会在 `resume_target` 及其后继块使用的 SSA 值。
- 已推进：`MIR` 当前已把 suspend live-out 结果继续前推成显式 `frame_layouts` 计划；每个 suspend `state_id` 都会产出稳定的 `frame slot` 列表，显式记录 `slot_index / value / value_type`，为后续 lane/runtime lowering 承接状态机 frame 提供正式入口，不再只停留在 `spill_candidates` 候选集。
- 已推进：`fallthrough` 当前已开始走显式 region edge 主链；语句路径的 `match` 已在 `HIR` 先落成独立 `Case` 语义面，`MIR` 也会把它正式 lower 成 `case_arm_* / case_exit` 这组显式 block，并把 arm 内 `fallthrough` 落成 `Jump(next_case_arm, [])`，不再依赖隐式顺序串落；当前新增回归也已明确钉住：`fallthrough` 会重新进入下一 arm 的 `case_arm_N` 入口，再继续走该 arm 自己的 pattern check / guard 分发，而不是直接跳进 `case_arm_N_body` 绕过校验。值语义 `match expression` 中的 `fallthrough` 仍会在进入 `MIR` 前被 `HIR` 护栏拒绝。
- 已推进：`HIR` 当前还会在 `fallthrough` 进入下一 arm 前显式拒绝“后一 arm 的 guard/body 非法引用前一 arm 独有 pattern 绑定”这类源级绑定泄漏；因此 `MIR` 不需要再把这类非法源形状伪装成 block parameter、局部或额外 merge，只承接已经完成绑定隔离的合法 `case_arm_*` CFG。
- 已推进：`case / match` 当前也已开始显式记录 `case_chains` 元数据；每条 case-like CFG 都会稳定产出 `dispatch_block / first_arm / no_match_block / exit_block / produce_value`，以及各 arm 的 `entry/check/guard/body/next/fallthrough` 目标，供 `MIR` 自身校验、`LIR` 透传与 pipeline 一致性校验直接消费。
- 已闭合：pattern lowering 已从“可承载”推进到“部分类型已知时可执行”；当前稳定的直接 lowering 只保留给字面量、`Type`、`Tuple / Or / Range` 与原始 `Object` 模式。`Constructor / Array` 当前不再借 `PatternMatch` 或 `FieldGet / ArrayGet` 伪装，而是转发成显式 extractor 方法调用；`MIR` 约定 extractor 返回 tuple payload，其中第 0 槽是 match flag，后续槽位是绑定载荷。
- 已推进：`Tuple / Or` pattern 当前也已不再一律退回通用 `PatternMatch`；当 scrutinee 静态类型可见时，`MIR` 会显式 lower `tuple_get_N + compare + logical_and / logical_or`，并继续沿 `LIR / CLR / JVM` 主链保真透传。
- 已推进：parser/AST 新补上的 `literal / variable / wildcard / or-pattern / tuple / range / array / typed bind` 当前已接入 `HIR -> MIR` 主链；`A | B` 不再在 parser 阶段被吞掉后半段，`1..=10` 也不再被词法误吞成浮点前缀，`MIR` 会收到稳定的 `HirPattern::Or / Literal / Variable / Wildcard / Type / Tuple / Range / Array / TypedBind` 输入，而不是再靠 fallback 猜测源形状。
- 已推进：源码层 pattern 边界当前重新收口为四类：`case X()` 与 `case []` 属于 extractor 语义，`case X {}` / `case {}` 属于原始字段或 getter 模式，字面量与模式表达式各自独立。`MIR` 后续继续收窄 fallback 时，不得把 `Constructor / Array` 直接当成原始结构投影；若缺少 extractor 声明或布局不可知，应继续保留显式边界。
- 已修正：此前把 `array/rest` 直接描述成 `length/get` 检查与绑定，只适用于过时的伪 lowering；当前 `array` 与 `constructor` 都改走 extractor 调用，再用 `tuple_get_N` 解出 match flag 与绑定载荷。后续需要补 extractor 声明解析与重载分派，把当前按命名约定拼出的 callee 路径升级成真正的解析结果。
- 已闭合：原始 object pattern 的具名子类型规则、匿名结构字段规则与未知布局 fallback 已统一收口，不再在 `MIR` 内部分裂为多套绑定逻辑。
- 已保真：顶层调度模块与 `LIR` 校验已经接上，当前 `PatternMatch / PerformEffect / Jump / Branch` 的跨层形状不会再静默漂移；新增 effect 恢复点参数个数、payload 形状与恢复点参数静态类型校验后，`Yield / DelegateYield / Await / AsyncBlock / Raise` 需要单参数恢复点、`AsyncSpawn` 需要无参数恢复点，以及当前所有 effect 在 `MIR` 层都必须带 payload 的约束也已开始在 `MIR` 出口显式检查；裸 `yield` 会在 lowering 时先补成 `unit` payload。对 `Await / AsyncSpawn / AsyncBlock` 这三类 built-in effect，当前在 payload 静态类型可见时也会继续核对其是否满足 `Future<T> / Promise<T>` 形状；对 `Yield / DelegateYield / Await / AsyncBlock`，当前也会继续核对恢复点 block parameter 的静态类型是否和 lowering 回灌结果一致；其中 `AsyncSpawn` 已重新确认保持“仅有 `resume_target`、无恢复点参数”的空恢复协议。本轮同时补上了 `Jump -> block parameter` 的静态类型护栏，开始显式拒绝把 jump argument 的类型漂移成与目标 block parameter 不一致的形状。
- 已保真：顶层调度模块当前还会继续核对 continuation 元数据本身，显式拒绝 continuation 恢复参数脱离目标 resume block，或 continuation 恢复类型在 `MIR / LIR` 间静默漂移。
- 已保真：顶层调度模块当前也会继续核对 suspend 点元数据本身，显式拒绝 suspend 点恢复参数个数与目标恢复 block 脱节，或 `state_id / payload_type / spill_candidates` 在 `MIR / LIR` 间静默漂移。
- 已修正：`resume` 之后当前已显式落成 `Unreachable` 后继块，不再让 handler arm 在恢复后继续回流到 `catch_exit` merge；这一步同时修掉了新调度护栏暴露出的“`resume` 后仍把 `unit` 合并进 handler exit” 的真实主链 bug。
- 已闭合：`src/mir/ssa/mod.rs` 内联测试已外迁到 `tests/mir/mod.rs`，`MIR` 主实现文件不再继续承载 pattern / effect 恢复回归；当前 `effect / handler` lowering 已继续收口在 `src/mir/ssa/effect_lowering.rs`，`suspend` 活跃变量分析与 `frame` 计划已外迁到 `src/mir/ssa/suspend_analysis.rs` 与 `src/mir/ssa/frame_planning.rs`；`src/mir/ssa/mod.rs` 当前实测为 `1521` 行，仍低于 `large-files.log` 中的 `1655` 行旧值，相关回归已通过 `cargo test mir:: -- --nocapture` 与 `cargo test pipeline::control_flow_scheduler -- --nocapture`。
- 待继续：`catch / resume` 当前虽已把 continuation 句柄、恢复参数与显式 continuation 元数据前推到 lowering 主链，并已由 `LIR` 生成第一版 `runtime_continuations` carrier，`CLR` 路线也已开始把它们降成显式 `MSIL` runtime type，但 effect 类型选择、更广义动态形态判定，以及真正可执行的 continuation runtime 逻辑仍待继续补齐；`yield / await / block` 的完整 runtime / frame 主链也仍未完成。当前虽已接上第一版 suspend 活跃变量分析，并把 `spill_candidates` 从“候选记账”推进到真实 live-out 结果，再前推成显式 `frame_layouts` 计划，且 `LIR` 已能生成对应 `runtime_frames` carrier，`CLR` 也已开始消费成显式 carrier type，但状态机 emit、更强的 spill 收敛与多 lane 实际消费仍未接通；`await / block` 当前也仍只覆盖 `Future<T> / Promise<T>` 这类最小类型闭环。`case statement` 当前虽已正式脱离值语义 `match` 进入独立 lowering 主链，parser/AST 也已补上 `literal / variable / wildcard / or-pattern / tuple / range / array / typed bind` 这组源级形状，constructor/object 的嵌套子模式、对象/数组 rest 绑定与 `fallthrough` 绑定隔离护栏也已接通，但更深的 pattern 变体与跨 arm merge 规则仍待继续补齐。

## 统一原则

下一版 `MIR` 固定采用：

- 基本块
- 块参数
- 显式 terminator
- `SSA`

不引入传统独立 `phi` 节点，统一走 block parameter SSA。

原因不是审美，而是因为：

- `continue` 需要 loop-carried value
- `break expr` 需要 exit merge value
- `fallthrough` 需要 case arm 间显式传值
- `yield` / `.await` 的恢复点需要显式接收恢复值

## MIR 层的两类边

### Region Edge

- 进入循环 header
- 跳出 region exit
- case arm 串落到下一 arm
- 函数返回

它们都可以表达为：

- `Jump(target, args)`
- `Branch(cond, then_target, else_target)`
- `Return(value)`

### Effect Edge

- `yield`
- `.await`
- `.awake`
- `.block`
- `resume`

它们不能再伪装成普通 `Call`，因为这些操作会改变控制流连续性。

但这里仍要保持语义分层：

- `.await / .awake / .block` 属于 `co` 这一层
- `yield / raise / catch / resume` 属于更高层的 effect / handler 元控制流

`MIR` 可以用同一套显式 suspend / resume 图承载它们，但不能把这些来源能力重新混成一个语义面。

所以 `MIR` 需要显式的 effect terminator。

## 建议的 MIR Terminator

下一阶段至少需要下面这些 terminator：

- `Return { value }`
- `Jump { target, arguments }`
- `Branch { condition, then_target, else_target }`
- `PerformEffect { effect, payload, resume_target }`
- `Unreachable`

这里的关键点：

- `break` 和 `continue` 不需要单独 terminator 类型
- `fallthrough` 也不需要单独 terminator 类型
- 它们只是不同目标的 `Jump`
- `yield` 与 `.await` 则必须成为显式挂起点

## Block Parameter 规则

### Loop Header

循环 header 必须支持参数位，承接：

- 外层进入循环的初始值
- `continue` 回边传回的新值

也就是说，`continue` 不再只是“跳回 header”，而是“带着 loop-carried value 跳回 header”。

### Region Exit

带值 `break expr` 的目标 exit block 必须有参数位。

所有离开这个 region 的边都需要：

- 无值时传 `Unit` 或禁止
- 有值时传兼容目标类型的值

### Resume Block

effect 恢复点必须使用 block parameter 接收恢复值。

例如：

- `yield` 的 resume block 接收 `void`
- `.await` 的 resume block 接收 `T`
- `.block` 的 resume block 接收 `T`

这样恢复值不会依赖隐式寄存器或宿主私有约定。

## Region Control 的 lowering

### loop / while

统一 lowering 为：

- `loop_header`
- `loop_body`
- `loop_exit`

`while` 只是 `Loop { condition: Some(...) }` 的一个来源，不需要在 `MIR` 维持独立形状。

### continue

`continue` lowering 为：

- `Jump(loop_header, carried_args)`

### break

`break` lowering 为：

- `Jump(loop_exit, exit_args)`

### break expr

`break expr` lowering 时，`expr` 的值进入 `loop_exit` 的 block parameter。

### fallthrough

`fallthrough` lowering 为：

- `Jump(next_case_arm, case_args)`

不允许依赖“顺序排在下一条指令后面所以自然串落”的隐式控制流。

### return

`return expr` lowering 为：

- `Return { value }`

## Effect Control 的 lowering

### yield

`yield expr` lowering 为：

- `PerformEffect { effect: Yield, payload: expr, resume_target }`

`resume_target` 接收 `void` 参数，且 `payload` 不允许缺失；裸 `yield` 视为 `yield ()`，会先补成 `unit` payload。

### yield from

第一阶段建议保留独立 MIR 语义：

- `PerformEffect { effect: DelegateYield, payload: source, resume_target }`

typed lowering 再决定：

- 是继续保留委托语义
- 还是展开成显式遍历

### .await

`expr.await` lowering 为：

- `PerformEffect { effect: Await, payload: future, resume_target }`

`resume_target` 接收 `T` 参数。

### .awake

`expr.awake` lowering 为：

- `PerformEffect { effect: AsyncSpawn, payload: future, resume_target: next_block }`

它不要求当前协程等待，但仍然是 effect control 的一部分。

### .block

`expr.block` lowering 为：

- `PerformEffect { effect: AsyncBlock, payload: future, resume_target }`

`.block` 的上下文合法性必须已经在 `HIR` 判定完成。

### catch / resume

`catch` 要在 `MIR` 中表现为显式 handler region。

`resume value` 不是普通调用返回，而是 continuation 恢复：

- 恢复到挂起点指定的 `resume_target`
- 用 block parameter 显式传递恢复值

## Suspend 之后还需要什么

`PerformEffect` 进入主链后，`MIR` 下一步还需要承接：

- suspend 跨越活跃变量分析
- 需要 spill 的局部识别
- resume block 对 state id 的需求

但这些仍属于 `MIR` 或紧邻 `MIR` 的职责，不应拖到 backend 临时补洞。

## MIR 不应该做的事情

`MIR` 不应该决定：

- `CLR` 用状态机类还是闭包对象
- `JVM` 用哪种 scheduler 桥接
- `WASM` 用哪种 host async ABI
- `native` 用线程阻塞还是事件循环

这些是 lane/runtime 的事情。

`MIR` 只需要提供：

- 清晰的 block 参数
- 清晰的 suspend/resume 边
- 清晰的 effect 载荷

## 对现有结构的要求

当前 `MIR` 已经有：

- 基本块
- block parameter
- `Jump(arguments)`
- `Branch`
- `Return`

下一步不是推翻，而是继续补：

- region result merge
- label 目标解析后的 lowering
- `PerformEffect`
- effect 恢复点

## 一句话原则

`MIR` 不重新发明语言语义，只把控制流显式化到 block parameter SSA 与 suspend/resume 图。

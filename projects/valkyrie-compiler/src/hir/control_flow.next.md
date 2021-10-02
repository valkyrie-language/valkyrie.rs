# hir control flow next

这份文档描述 `HIR` 层对统一控制流的职责边界。

这里的重点不是如何发射目标指令，而是如何在进入 `MIR` 前把语言控制流语义收口、校验并保真。

## 定位

`HIR` 是语言控制流的最后一层高层表示。

在这一层：

- 控制流仍然保留源语言结构
- effect 仍然保留语言意图
- 上下文限制必须已经可判定
- 不允许把关键语义推迟到某个后端再决定

换句话说：

- `HIR` 负责“这段代码在语言上是什么意思”
- `MIR` 负责“把这个意思展开成显式控制流图”

## 当前推进

- 已闭合：`label / break / continue / resume` 的基础 `HIR` 入口与 region 校验已经接通，`resume` 当前已收紧为只能出现在 `catch arm body`。
- 已闭合：guard 中会打断控制流连续性的 `yield / yield from / await / awake / block / resume` 已统一在 `HIR` 出口前置拒绝。
- 已闭合：`break expr` 的最小值语义已经接通，当前会校验目标 loop 是否接受值，并在字面量、`block`、`if`、`match`、带显式类型的 `let` 变量、`await / block` 的最小 future 结果类型，以及 `yield / yield from / awake` 的 `unit` 结果类型场景下提前暴露类型冲突。
- 已闭合：`return expr` 与函数 `return_type` 的最小静态兼容性已经接通，当前会在字面量、带显式类型的局部变量、`await / block` 的最小 future 结果类型，以及无值 `return` 直接与非 `unit / void` 返回类型冲突时，于 `HIR` 出口提前拒绝明显错误返回。
- 已闭合：`.await / .awake / .block` 在操作数类型静态可见时已开始于 `HIR` 出口前置校验；当前会拒绝对非 `Future<T> / Promise<T>` 操作数执行这三类 effect control，避免明显错误继续漂到 `MIR`。
- 已推进：`.block` 与 `yield / yield from` 的允许上下文已开始进入独立 `HIR` 诊断；当前函数体默认允许，`lambda` 体内会前置拒绝 `block`、`yield` 与 `yield from`；裸 `yield` 当前重新按 `yield ()` 语义接受，并在 lowering 时补成 `unit` payload。
- 已推进：语句路径的 `match` 当前已不再继续借值语义 `HIR::Match` 承载；lowering 会显式落成独立 `HIR::Case`，`fallthrough` 也只对 `HIR::Case` arm body 放行，而值语义 `HIR::Match` 中的 `fallthrough` 仍会在控制流校验阶段被显式拒绝，避免把 statement 语义偷渡进值表达式。
- 已推进：`case / match` 的 parser/AST 源级形状当前已补上 `Variable / Wildcard / Literal / Type / Or / Tuple / Range / Array / TypedBind`，并接入既有 `HirPattern::Variable / Wildcard / Literal / Type / Or / Tuple / Range / Array / TypedBind`；guard 读取也已统一改走 `pattern.guard()`，不再只对单一 pattern 分类单独开洞。
- 已推进：源码层 pattern 边界当前重新收口为四类：`case X()` 与 `case []` 归 extractor 语义，需依赖显式声明且可重载的 extractor；`case X {}` 与 `case {}` 归原始字段或 getter 模式；字面量模式与 `A | B` 这类模式表达式各自独立成类。`HIR` 当前已把 `X()` / `[]` 统一归一化到 `HirPattern::Extractor(...)`，不再把它们和原始 `Object` 模式混为一谈。
- 已推进：`match / case arm` 的 pattern 绑定当前已正式接入 `HIR` 校验作用域；guard 与 body 会在各自 arm 的局部作用域下看到 pattern 绑定，`infer_static_match_type` 也会按 arm 作用域推断 `match` 结果类型，不再把 arm body 一律当成“无绑定裸表达式”处理。
- 已推进：`fallthrough` 当前也已补上“上一 arm pattern 绑定不继承到下一 arm”的显式 `HIR` 护栏；若前一 arm 通过 `fallthrough` 落到后一 arm，而后一 arm 的 guard/body 非法引用了前一 arm 独有的 pattern 绑定，`HIR` 会在进入 `MIR` 前直接拒绝，避免把源级绑定泄漏伪装成后续 block 参数或普通局部。
- 已推进：`case / match` 当前也已开始显式产出跨 arm merge 元数据；`HIR` 仍只负责语义闭合，但下游 `MIR / LIR` 已能稳定看到每个 arm 的入口、check、guard、body、next-arm 与 fallthrough 目标，不再只靠 `case_arm_*` 块名约定反推串落链路。
- 已闭合：源码层原始 object pattern 已进入 `HIR` 主链，具名 `Foo { bar }` 与匿名 `{ foo, bar }` 的语义边界已经固定，不再和 extractor 语义混用。
- 已保真：后续 `MIR` 已验证 object pattern 的子类型匹配、结构字段匹配、未知布局 fallback、`resume value` 类型回灌，以及 `await / block` 在 `Future<T> / Promise<T>` 这类最小闭环下的恢复值类型回灌；顶层调度模块当前也会继续核对 `yield / yield from / await / block` 的恢复点参数静态类型，说明 `HIR` 当前保留的高层语义没有在下层漂移。
- 已验证：新增 `rejects_await_on_non_future_operand_at_hir_validation`、`rejects_awake_on_non_future_operand_at_hir_validation`、`rejects_block_on_non_future_operand_at_hir_validation`、`infers_awake_as_unit_for_break_expr_validation`、`infers_yield_as_unit_for_break_expr_validation`、`infers_yield_from_as_unit_for_break_expr_validation`、`rejects_incompatible_return_expr_type_at_hir_validation`、`infers_typed_variable_for_return_expr_validation`、`infers_await_result_type_for_return_expr_validation`、`rejects_empty_return_for_non_unit_function`、`rejects_block_inside_lambda_body_without_blocking_context`、`rejects_yield_inside_lambda_body_without_generator_context`、`rejects_yield_from_inside_lambda_body_without_generator_context`、`accepts_bare_yield_as_unit_payload_sugar`、`lowers_bare_yield_into_unit_payload_effect`，并回归通过 `cargo test pipeline::control_flow_scheduler -- --nocapture`；`mir::ssa` 相关 pattern / effect 恢复回归也已外迁到 `tests/mir/mod.rs`，避免继续挤在主实现文件里。
- 已验证：新增独立 `compiler` 回归覆盖 `literal / variable / or-pattern` 在 `HIR` 中的实际落点，当前已显式按 `Return(Some(Match { ... }))` 这一真实结构断言，避免再把 `return match ...` 误判成 `body.expr`。
- 已验证：新增独立 parser/compiler 回归，当前已覆盖 `tuple pattern` 在源码层与 `HIR` 中的实际落点，说明 `((x, y), z)`、`(_, 0)` 这类基础元组模式已经可以稳定进入 `match` 主链。
- 待继续：`.block` 的完整合法上下文、`yield / yield from` 除函数体之外更细粒度的允许上下文，以及更广义动态表达式上的 `break expr / return expr` 类型兼容性仍需继续补齐；`case statement` 当前虽已在 `HIR` 正式拥有独立语义面，parser/AST 也已补上 `literal / variable / wildcard / or-pattern / tuple / range / array / typed bind` 这组源级形状，constructor/object 的嵌套子模式与对象/数组 rest 绑定也已接通，且 `fallthrough` 的绑定泄漏护栏已前推到 `HIR`，但更深的 pattern 变体与跨 arm merge 规则仍待继续补齐。当前新增的局部类型收敛与 future 操作数检查仍只覆盖函数参数、带显式类型的 `let` 变量与 `Future<T> / Promise<T>` 这类最小闭环。

## 两类控制流

`HIR` 需要同时承载两类控制流：

### Region Control

- `label`
- `continue`
- `break`
- `early return`
- `case fallthrough`

这些语义依赖：

- 作用域
- region 嵌套关系
- 目标 region 类型

### Effect Control

- `yield`
- `yield from`
- `.await`
- `.awake`
- `.block`
- `catch`
- `resume`

这些语义依赖：

- effect 类型
- `ResumeStatement` 类型
- 挂起上下文
- 阻塞上下文
- handler 安装范围

## HIR 必须保留的控制流节点

下一阶段 `HIR` 应稳定保留以下形状：

- `Loop { label, pattern, iterator, condition, body }`
- `Break { label, expr }`
- `Continue { label }`
- `Return(expr)`
- `Case { label, scrutinee, arms }`
- `FallthroughStatement`
- `Yield(expr)`
- `YieldFrom(expr)`
- `Await(expr)`
- `Awake(expr)`
- `Block(expr)`
- `Raise(expr)`
- `Catch { expr, arms }`
- `Resume(expr)`

这里的原则很简单：

- 只要某个语义在错误消息、上下文检查、类型推断上有独立规则，它就不应该在 `HIR` 太早被抹平
- `yield from`、`.await`、`.block` 都属于这种情况

## HIR 的职责

### 1. label 与 region 解析

`HIR` 必须完成：

- `break label` 的目标解析
- `continue label` 的目标解析
- `break expr` 的目标 region 结果类型关联
- `fallthrough` 的目标 case arm 合法性校验

`HIR` 不必产出具体 block id，但必须能表达“这条控制流面向哪个语义 region”。

### 2. 结构化控制流的值语义

`HIR` 必须定义：

- 哪些 `loop` / `case` 可以产生值
- `break expr` 产生的值如何约束目标 region 类型
- `return expr` 如何约束函数返回类型

这一步不做 block parameter，也不做 `SSA`，但要先把值语义钉死。

### 3. effect 语义的类型闭合

统一控制流建立在：

```valkyrie
trait Effectful {
    type Resume
}
```

`HIR` 层需要完成：

- `yield expr` 对应的 effect 与 `Resume = void`；裸 `yield` 按 `yield ()` 语义处理
- `.await` 对应的 effect 与 `Resume = T`
- `.awake` 对应的 effect 与 `Resume = void`
- `.block` 对应的 effect 与 `Resume = T`
- `resume value` 的值类型与当前 effect `ResumeStatement` 对齐

`HIR` 不需要决定 frame 布局，但必须先把 `ResumeStatement` 类型收口。

其中需要显式区分三层能力边界：

- `iterator`
- `generator`
- `co`

它们是能力递进，但不是同一个东西的别名；当前 `co` 只覆盖 `.await / .awake / .block`，`resume` 继续属于更高层的 effect / handler 元恢复语义，不并入协程能力面。

### 4. 上下文限制检查

必须在 `HIR` 或紧邻 `HIR` 的语义阶段判定：

- `.block` 是否位于允许阻塞的上下文
- `.await` 是否位于允许挂起的上下文
- `yield` 是否位于生成器上下文或等价 handler 上下文
- `resume` 是否只出现在 `catch` arm 内
- `fallthrough` 是否只出现在 `case` statement 体系中

这些检查如果拖到 backend，控制流语义就已经分裂了。

## HIR 不应该做的事情

以下内容不应在 `HIR` 决定：

- coroutine frame 的物理布局
- state id 编号
- 某个 lane 如何调用宿主调度器
- `yield from` 最终是委托对象还是循环展开
- `.block` 具体采用线程阻塞还是宿主轮询

这些属于 `MIR` 之后的承载问题，不是语言语义问题。

## Yield / Await / Block 的 HIR 边界

### yield

- `yield expr` 在 `HIR` 仍然是独立节点
- 它不是普通 `TermCallExpression`
- 它也不是立即展开成某个 runtime helper

### yield from

- `yield from expr` 在 `HIR` 继续保留独立节点
- 不在 parser 或 `HIR` 立刻展开成 `loop item in expr { yield item }`
- 原因是它可能承载真正的 delegation 语义，而不是普通遍历

### .await

- `.await` 在 `HIR` 应是后缀控制流节点，而不是普通成员访问
- 它的核心语义是“挂起当前协程并等待恢复”

### .awake

- `.awake` 在 `HIR` 应是后缀控制流节点
- 它不是“方法调用名为 awake 的普通调用”
- 它的语义是“触发并继续”

### .block

- `.block` 在 `HIR` 应是后缀控制流节点
- 它的合法性依赖上下文
- 所以不能等到 `LIR` 或 backend 才认出来

## HIR 到 MIR 的交接要求

`HIR -> MIR` 交接前必须已经明确：

- label 目标
- region 结果类型
- effect 的 `ResumeStatement` 类型
- 是否允许挂起
- 是否允许阻塞

`MIR` 接手时不应再重新发明这些规则，只负责显式化控制流图。

## 诊断要求

`HIR` 层应优先给出这类错误：

- `continue label` 指向非循环 region
- `break expr` 目标 region 不接受值
- guard 中出现会打断控制流连续性的 `yield / yield from / await / awake / block / resume`
- `fallthrough` 出现在 `match`
- `.block` 出现在禁止阻塞上下文
- `resume` 不在 `catch` 内
- `yield` 不在允许产出的上下文

这些诊断应尽量指向源级语法，而不是底层 lowered 形式。

## 一句话原则

`HIR` 要保留控制流意图并完成语义闭合，但不能越权替后端决定承载方式。

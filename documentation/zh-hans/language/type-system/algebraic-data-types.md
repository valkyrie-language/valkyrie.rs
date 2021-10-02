# 代数数据类型 (Algebraic Data Types)

Valkyrie 的类型系统建立在代数数据类型 (ADT) 的坚实基础上。通过**积类型**与**和类型**的组合，你可以构建出精确映射业务逻辑的领域模型。

## 形式化约定

本页把 ADT 视为两类具名类型构造：

- 积类型：把多个组成部分组合成一个值。
- 和类型：把多个互斥分支组合成一个具名声明家族。

在当前设计里：

- `structure`、`class`、`sealed class` 都属于具名声明。
- `unite` 也属于具名声明，并形成一个封闭的 variant family。
- ADT 的成员资格按声明身份解释，而不是按结构兼容解释。

## 积类型 (Product Types)

积类型之所以被称为“积”，是因为该类型可能的取值空间是其所有成员取值空间的**笛卡尔积**。在 Valkyrie 中，积类型体现为数据的组合。

### 1. 结构体与记录 (Structure & Class)
积类型由多个命名字段组合而成。

```valkyrie
structure Point {
    x: f64,
    y: f64,
}
```
`Point` 的状态空间 = `f64` 的空间 × `f64` 的空间。

### 2. 元组 (Tuples)
匿名、有序的积类型。

```valkyrie
let color: (u8, u8, u8) = (255, 0, 0)
```

### 3. 具名对象类型

除了 `structure` 之外，`class` 与 `sealed class` 也属于具名类型声明。

- `class` 表达对象类型与继承层级。
- `sealed class` 表达封闭的具名类族。
- 它们都依赖声明身份，而不是“长得像什么字段/方法”。

形式化地说，若 `A` 与 `B` 是具名对象类型，则只有当声明层级显式给出继承或成员关系时，文档才认为存在对应的名义包含关系。

这也是 Valkyrie 阅读顺序从具名类型开始的原因：先把“类型本体”和“声明家族”看清，再去看匿名约束、协议系统和更高阶的类型计算。

---

## 和类型 (Sum Types)

和类型之所以被称为“和”，是因为该类型可能的取值空间是其所有分支取值空间的**逻辑加和**。在 Valkyrie 中，和类型体现为状态的互斥选择。

在当前语义里，`unite` 应解释为 union 的具名定义方式。其语义约束如下：

- `unite` 表示一组互斥分支组成的和类型
- `unite` 的默认表示是抽象类
- `[tag(XXXKind)]` 是可选优化，用于要求显式的 tagged union
- 对特定形态，编译器还可以进一步做利基优化

这里最关键的形式化边界是：

- `unite` 的分支集合必须来自同一条 `unite` 声明。
- 一个值不得因为“结构上像某个 variant”就自动成为该 `unite` 的成员。
- 穷尽性检查必须只针对已声明 variant family 进行。

### 1. 联合类型 (`unite`)
Valkyrie 使用 `unite` 定义命名的和类型。它的默认表示是抽象类；`[tag(XXXKind)]` 是可选优化，用于声明 tagged union；语言不会自动生成 tag。

```valkyrie
unite LoadingState {
    Idle,
    Loading { progress: f32 },
    Success { result: utf8 },
    Failure { code: ErrorCode }
}
```
`LoadingState` 的状态空间 = `Idle` + `f32` + `utf8` + `ErrorCode`。

其语义可分解为：

- `LoadingState` 是一个具名 union
- `Idle / Loading / Success / Failure` 是它的分支
- 默认表示下，可以把它看成一个抽象类加若干具体分支
- 模式匹配的穷尽性来自这个已知分支集合

### 2. 枚举 (Enum)
当和类型的所有分支都不携带额外数据时，它退化为传统的枚举。

```valkyrie
unite Direction { North, South, East, West }
```

如果你明确要选择 tagged union 形态，可以显式写出 `tag`：

```valkyrie
[tag(DirectionKind)]
unite TaggedDirection { North, South, East, West }
```

---

## ADT 与模式匹配

和类型允许编译器对分支穷尽性进行检查。

```valkyrie
micro process(state: LoadingState) {
    match state {
        case Idle: print("等待中...")
        case Loading { progress }: print("加载中: {}%", progress * 100)
        case Success { result }: print("成功: {}", result)
        case Failure { code }: print("错误: {}", code)
    }
}
```

---

## 递归代数数据类型

ADT 可以是递归的，这使得它们非常适合描述树状或链式结构：

```valkyrie
unite List⟨T⟩ {
    Empty,
    Node { head: T, tail: List⟨T⟩ }
}

unite JSON {
    Null,
    Bool { value: bool },
    Number { value: f64 },
    String { value: utf8 },
    Array { items: [JSON] },
    Object { fields: { utf8: JSON } }
}
```

## 物理布局优化

Valkyrie 编译器对 ADT 进行极致优化：
- **空指针优化 (Non-zero Optimization)**：`Option⟨ref T⟩` 不占用额外空间。
- **标签压缩 (Tag Compression)**：对于只有少数分支的 `unite`，标签通常只占用 1 个字节甚至更少。
- **内存重叠 (Field Overlay)**：不同分支的数据在物理内存中共享同一块空间。

这些都属于实现层优化，不改变 `unite` 的语义本体。也就是说：

- 前端仍把 `unite` 当作 union 处理
- 后端再决定是保留抽象类表示，还是优化为 tagged union、利基布局或其他表示
- 不能反过来因为某种物理布局而改写 `unite` 的类型语义

因此，本页采用如下原则：

- 表示选择属于 lowering 问题。
- 成员资格、分支穷尽性和模式匹配语义属于前端类型问题。
- lowering 不得反向定义前端语义。

---

## 广义代数数据类型 (GADT)

广义代数数据类型（Generalized Algebraic Data Types）允许在定义 `unite` 分支时，显式指定该分支构造出的具体类型。这打破了传统 ADT 中“所有分支必须具有相同类型参数”的限制。

### 问题陈述：类型信息丢失
在普通 ADT 中，即使你构造了一个 `Literal(1.0)`，它的类型也只是宽泛的 `Expr⟨T⟩`。当你编写解释器时，你不得不再次通过模式匹配或类型转换来确定 `T` 到底是什么。

### Valkyrie 的解决方案：构造器签名
Valkyrie 允许为每个分支指定返回类型，从而在构造时就锁定类型参数。

```valkyrie
unite Expr⟨T⟩ {
    # 显式指定返回类型，锁定 T 为 f64
    Literal { value: f64 }: Expr⟨f64⟩
    
    # 锁定 T 为 bool
    IsZero { expr: Expr⟨f64⟩ }: Expr⟨bool⟩
    
    # 递归定义：结果类型 T 由子表达式决定
    If { condition: Expr⟨bool⟩, then_branch: Expr⟨T⟩, else_branch: Expr⟨T⟩ }: Expr⟨T⟩
}

# 静态类型检查：
let ok: Expr⟨f64⟩ = If { condition: IsZero { expr: Literal { value: 0.0 } }, then_branch: Literal { value: 1.0 }, else_branch: Literal { value: 2.0 } }

# 解释器逻辑：
# 因为类型已经在构造时锁定，我们不需要再做额外的类型转换
micro eval⟨T⟩(expr: Expr⟨T⟩) -> T {
    match expr {
        case Literal { value }: value
        case IsZero { expr }: eval(expr) == 0.0
        case If { condition, then_branch, else_branch }: if eval(condition) { eval(then_branch) } else { eval(else_branch) }
    }
}
```

---

## 应用：Final Tagless 范式

相比于传统的递归 ADT，Final Tagless 是一种更高级的抽象模式。它通过 Trait 来定义 DSL 的语义，从而在不需要中间数据结构的情况下实现高度可扩展的操作逻辑。

```valkyrie
# 定义 DSL 语义接口
trait Expr⟨F⟩ {
    micro literal(val: f64) -> F
    micro add(left: F, right: F) -> F
    micro mul(left: F, right: F) -> F
}

# 实现 1：直接求值解释器
imply Evaluator: Expr⟨f64⟩ {
    micro literal(val) -> f64 { val }
    micro add(l, r) -> f64 { l + r }
    micro mul(l, r) -> f64 { l * r }
}

# 实现 2：格式化打印解释器
imply Printer: Expr⟨utf8⟩ {
    micro literal(val) -> utf8 { val.to_utf8() }
    micro add(l, r) -> utf8 { "({} + {})".format(l, r) }
    micro mul(l, r) -> utf8 { "({} * {})".format(l, r) }
}

# 使用泛型函数编写业务逻辑
micro program⟨F, E: Expr⟨F⟩⟩(e: E) -> F {
    e.add(e.literal(1.0), e.mul(e.literal(2.0), e.literal(3.0)))
}
```

**优势**：
- **可扩展性**：可以随时添加新的解释器，而无需修改原有的 DSL 定义。
- **性能**：没有中间 ADT 的内存分配开销，逻辑直接展开为目标类型的操作。
- **类型安全**：在编译时确保所有操作都符合定义的语义。

---

## ADT、trait 与 row 的分工

在 Valkyrie 中，这几类抽象应严格区分：

- `unite`：表达和类型与互斥分支
- 具名 `trait`：表达协议与 witness
- 匿名 row：表达临时方法行 requirement

它们分别回答：

- `unite`：当前值属于哪一个分支
- `trait`：你是否满足这个具名协议
- `row`：你当前会不会这些方法

把这三层分开，模式匹配、重载解析、类型检查和后端 lowering 才会稳定。

---

## 应用：纽扣类型 (Newtype)

通过为现有类型创建包装，可以在不增加运行时开销的情况下提升代码安全性，防止混淆逻辑意义不同的同类数据。

```valkyrie
structure UserId(u64)
structure OrderId(u64)

micro fetch_order(user: UserId, order: OrderId) {
    # 业务逻辑...
}

micro main() {
    let u = UserId(1)
    let o = OrderId(100)
    
    # fetch_order(o, u) # 编译错误：类型不匹配
    fetch_order(u, o)   # 编译通过
}
```

---

**上一页**: [类型系统 (Index)](./index.md) | **下一页**: [联合类型 (Unite Types)](./union.md)

- 在 [交集与并集](intersection-union.md) 中了解如何使用匿名和类型与交集类型。
- [关联类型](associated-types.md) 说明 ADT 与具名 `trait` 之间的类型映射关系。

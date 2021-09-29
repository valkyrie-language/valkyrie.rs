# 类型级编程 (Type-Level Programming)

Valkyrie 的类型系统不仅是静态检查工具，还是一个编译时的计算引擎。类型级编程允许你在编译阶段进行逻辑推理、数据变换和协议验证。

## 告别“类型体操”

在 TypeScript 等语言中，类型级编程通常意味着复杂的递归条件类型，这被戏称为“类型体操”。Valkyrie 的设计哲学是：**类型级编程不应该是体操，而应该是正常的编程。**

### 痛点分析：为什么 TypeScript 需要“写两遍”？

在 TypeScript 中，类型系统和运行时的表达式系统是**完全隔离**的：
- **运行时系统**: 解释器/JIT 运行 JavaScript。
- **类型系统**: 编译器运行一套基于“结构化模式匹配”和“递归三元运算符”的特定领域语言（DSL）。

这就导致了逻辑的重复。如果你写了一个 `isEmpty(list)` 函数用于运行时判断，而你又希望在类型层面约束 `NonEmptyList<T>`，你必须用两套完全不同的语法再实现一遍。

### Valkyrie 的解决方案：统一的逻辑模型

Valkyrie 通过以下两种方式消除这种重复：

#### 1. 语法统一性 (Syntactic Unity)
无论是在 `micro` (运行时) 还是 `mezzo` (编译时) 中，你使用的都是相同的 `match`, `if`, `map`, `filter`。这意味着逻辑在心智模型上是**一套**，只是运行的时机不同。

#### 2. 跨层级复用 (Cross-level Reuse)
通过 `@const_fn` 和 `@evaluate`，同一段逻辑可以直接在两个世界穿梭。

```valkyrie
# 逻辑定义：只写一遍
@const_fn
micro validate_age(age: i32) -> bool {
    age >= 0 && age <= 150
}

# 运行时：直接调用
let ok = validate_age(25)

# 编译时：作为类型约束或常量
let VALID_DEFAULT: i32 = evaluate(if validate_age(20) { 20 } else { 0 })

# 类型级：在 mezzo 中复用
mezzo ValidAge⟨A: i32⟩ -> type {
    if evaluate(validate_age(A)) { A } else { never }
}
```

---

## 核心基石

### 1. 字面量类型 (Literal Types)
在 Valkyrie 中，字面量（如 `42`, `"hello"`, `true`）可以作为独立的类型存在。这被称为单例类型（Singleton Types）。

```valkyrie
# x 的类型不仅是 i32，还是字面量类型 42
let x: 42 = 42

# 错误：类型不匹配
# let y: 42 = 43
```

### 2. 类型函数 (Mezzo Functions)
使用 `mezzo` 定义的函数在编译时运行，接受并返回类型或常量。

```valkyrie
mezzo Add⟨N: int, M: int⟩ -> int {
    N + M
}

# 使用类型级计算
let buffer: array⟨u8, Add⟨10, 20⟩⟩ = uninitialized()
```

---

## 类型级列表与元组

你可以像操作运行时列表一样操作类型列表。

```valkyrie
mezzo Head⟨List⟩ {
    match List {
        case (H, ..): H
        case _: never
    }
}

# Head⟨(i32, utf8, bool)⟩ 将求值为 i32
type First = Head⟨(i32, utf8, bool)⟩
```

---

## 应用场景

### 1. 维度分析 (Dimensional Analysis)
确保物理单位（如米、秒）在计算中保持一致。

```valkyrie
type Quantity⟨Value, Unit⟩ = {
    value: Value
}

type Meter = { length: 1 }
type Second = { time: 1 }

# 定义单位乘法
mezzo MulUnit⟨U1, U2⟩ {
    { 
        length: U1::length + U2::length,
        time: U1::time + U2::time 
    }
}

micro multiply⟨V, U1, U2⟩(a: Quantity⟨V, U1⟩, b: Quantity⟨V, U2⟩) -> Quantity⟨V, MulUnit⟨U1, U2⟩⟩ {
    Quantity { value: a.value * b.value }
}
```

### 2. 静态断言与证明
利用类型系统证明代码的属性。

```valkyrie
trait IsTrue {}
imply true: IsTrue {}

# 如果 condition 不为 true，编译将报错
micro static_assert⟨condition: bool⟩() where condition: IsTrue {}

# 示例
static_assert⟨Add⟨2, 2⟩ == 4⟩() # 编译通过
# static_assert⟨2 + 2 == 5⟩()   # 编译失败
```

---

## 进阶应用：皮亚诺算术 (Peano Arithmetic)

在类型级别模拟自然数及其运算，是理解递归类型证明的基础。

```valkyrie
# 定义自然数结构
unite Nat {
    Zero,
    Succ { value: Nat },
}

# 类型级加法
mezzo Add⟨N: Nat, M: Nat⟩ -> Nat {
    match N {
        case Zero: M
        case Succ { value: N1 }: Succ { value: Add⟨N1, M⟩ }
    }
}

# 证明：1 + 1 = 2
type One = Succ { value: Zero }
type Two = Succ { value: Succ { value: Zero } }
static_assert⟨Add⟨One, One⟩ == Two⟩()
```

---

**上一页**: [高阶类型 (HKT)](./higher-kinded-types.md) | **下一页**: [依赖类型](./dependent-types.md)

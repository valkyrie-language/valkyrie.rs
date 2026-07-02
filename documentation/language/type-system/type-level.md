# 类型级编程 (Type-Level Programming)

Valkyrie 的类型系统不仅承担静态检查，还承担编译时计算。类型级编程用于在编译阶段进行逻辑推理、数据变换和协议验证。

## 基本目标

在 TypeScript 等语言中，类型级编程通常依赖复杂的递归条件类型。Valkyrie 的目标是让类型级逻辑与普通程序逻辑共享更统一的表达形式。

### 问题：类型层与值层分离

在 TypeScript 中，类型系统和运行时的表达式系统是**完全隔离**的：
- **运行时系统**: 解释器/JIT 运行 JavaScript。
- **类型系统**: 编译器运行一套基于“结构化模式匹配”和“递归三元运算符”的特定领域语言（DSL）。

这种分离会导致逻辑重复。如果某个判断同时需要出现在运行时与类型层，通常需要分别实现两次。

### 统一逻辑模型

Valkyrie 通过以下方式降低这种重复：

#### 1. 语法统一性 (Syntactic Unity)
无论是在 `micro` (运行时) 还是 `mezzo` (编译时) 中，都使用相同的 `match`、`if`、`map`、`filter` 等控制结构。

#### 2. 跨层级复用 (Cross-level Reuse)
通过 `@const_fn` 和 `@evaluate`，同一段逻辑可以在值层与类型层复用。

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

## 基本构件

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

## 适用范围

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

## 应用：皮亚诺算术 (Peano Arithmetic)

在类型级别模拟自然数及其运算，可用作递归类型证明的基本示例。

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

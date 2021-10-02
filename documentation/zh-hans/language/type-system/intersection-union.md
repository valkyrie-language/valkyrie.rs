# 交集类型与并集类型 (Intersection & Union Types)

Valkyrie 的类型系统支持代数化的类型组合。通过交集与并集操作，你可以构建出极具表达力的复合类型，精准描述协议约束、互斥分支集合与临时能力边界。

## 形式化约定

本页区分两类组合：

- `A | B` 表示并集，值在任一时刻属于 `A` 或 `B` 中的一个分支。
- `A & B` 表示交集，值必须同时满足 `A` 与 `B` 的要求。

在当前设计中：

- 若 `A | B` 通过 `unite` 声明，则其成员资格依赖具名 variant family。
- 若 `A & B` 由具名协议组成，则其语义是多个协议要求的合取。
- 匿名 row 可以出现为交集直觉上的近邻，但它不是具名协议交集的别名。

## 并集类型 (Union Types / Unite)

并集类型表示一个值可以是多种类型中的**其中之一**。Valkyrie 使用 `unite` 定义具名 union。`unite` 的默认表示是抽象类；`[tag(XXXKind)]` 是可选优化，用于要求 tagged union；此外还有少数利基优化形态。

### 1. 具名并集 (`unite`)
这是最常用的形式，通过显式的变体来区分不同的状态。

```valkyrie
unite Shape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Point
}
```

如果需要显式的 tagged union 布局，可以写成：

```valkyrie
[tag(ShapeKind)]
unite TaggedShape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Point
}
```

### 2. 匿名并集 (Anonymous Unions)
在某些临时场景下，可以使用 `|` 符号组合类型：

```valkyrie
# 变量可以是 i32 或 utf8
let data: i32 | utf8 = 42
```

> **语法约定**：Valkyrie 严格区分**状态叠加**与**属性扩展**。`|` 符号专用于并集类型（Union Types），表示析取关系；记录（Record）中的行扩展采用 `, ...R` 语法，并与模式匹配中的对象展开操作保持一致。详见 [行类型与多态](./row-types.md#与展开语法的一致性)。

### 3. 语义特征
- **排他性**：在任一时刻，并集类型的值只能属于其定义的某一个分支。
- **穷尽性检查**：编译器强制要求在 `match` 表达式中处理并集类型的所有可能分支。
- **声明边界**：`unite` 的分支来自同一条 `unite` 定义，而不是纯粹依赖结构兼容。

---

## 交集类型 (Intersection Types)

交集类型表示一个值必须**同时满足**多种类型的约束。Valkyrie 使用 `&` 符号来表达交集。

### 1. 协议交集
交集类型常用于要求一个值同时满足多个具名协议约束：

```valkyrie
# 变量必须同时满足 Display 和 Clone
micro process_data(item: Display & Clone) {
    print(item.fmt())
    let _ = item.clone()
}
```

### 2. 语义特征
- **能力叠加**：交集类型拥有所有组成约束共同要求的调用面。
- **多重约束**：它在逻辑上等价于泛型约束中的 `T: TraitA + TraitB`，但可以作为独立的类型直接使用。
- **不等于继承**：`A & B` 不是在制造一个新的父类，而是在叠加约束。

### 3. 与 row 的关系

交集类型最自然的宿主是具名协议与类型约束；而匿名 row 更适合表达局部方法 requirement。

例如：

```valkyrie
micro f(x: Display & Clone) { ... }
micro g(x: { fmt() -> utf8, clone() -> Self }) { ... }
```

两者虽然看起来都在要求多个能力，但语义不同：

- `Display & Clone` 是具名协议交集
- `{ ... }` 是匿名方法行约束
- 前者涉及具名 witness
- 后者不产生独立 witness

---

## 物理布局与优化

Valkyrie 编译器会对这些复合类型进行深度物理优化：

1. **并集类型表示选择**：
   - 默认情况下，`unite` 会按抽象类层级表示。
   - 如果显式写出 `[tag(XXXKind)]`，编译器可以将其 lowered 为 tagged union。
   - 对于 `Option⟨ref T⟩` 等特殊并集，编译器还会利用“空位优化”走利基优化路径。

2. **交集类型扁平化**：
   - 交集类型在底层通常被处理为指向多个特征虚表（Vtables）的“胖指针”集群，确保在多态调用时依然保持零成本抽象。

这些都属于 lowering 与表示问题，不改变前端语义分层：

- `unite` 仍然是 union 语义；默认表示是抽象类，也可以被优化成 tagged union 或利基布局
- 具名协议交集仍然是 trait 约束组合
- 匿名 row 仍然只是方法行 requirement

## 适用场景

- **并集类型**：状态机建模、错误处理（Result）、可选值（Option）、多态异构容器。
- **交集类型**：插件系统、依赖注入、多特征组合约束、精细化权限控制。

---

**上一页**: [行类型与多态](./row-types.md) | **下一页**: [型变与子类型](./polarity-type.md)

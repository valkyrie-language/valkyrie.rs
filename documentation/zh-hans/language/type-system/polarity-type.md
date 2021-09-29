# 型变与极性 (Variance & Polarity)

在代数子类型 (Algebraic Subtyping) 理论中，型变 (Variance) 并非孤立的规则，而是**类型极性 (Type Polarity)** 的直接体现。理解极性不仅能帮助我们掌握协变与逆变，还能揭示类型系统中“极点类型”的本质。

## 类型极性 (Type Polarity)

极性描述了一个类型参数在构造类型中所处的位置及其对子类型关系的影响方向。

1. **正极性 (Positive Polarity / `+`)**：对应**协变 (Covariance)**。子类型关系的方向与参数一致。
   - 若 `Sub <: Super`，则 `F⟨+Sub⟩ <: F⟨+Super⟩`。
   - **语义**：代表数据的“产出”或“源头” (Source)。

2. **负极性 (Negative Polarity / `-`)**：对应**逆变 (Contravariance)**。子类型关系的方向发生反转。
   - 若 `Sub <: Super`，则 `F⟨-Super⟩ <: F⟨-Sub⟩`。
   - **语义**：代表数据的“消耗”或“汇点” (Sink)。

3. **无极性 / 双极性 (Non-polar / Invariant)**：对应**不变 (Invariance)**。
   - 必须精确匹配，没有任何子类型关系。
   - **语义**：代表数据的“双向流动”（既读又写）。

---

## 极点类型 (Polar Types)

代数子类型系统拥有两个终极的“极点”，它们构成了类型格 (Type Lattice) 的顶端和底端。

### 1. 顶类型 (Top Type / ⊤)：`any`
- **极点位置**：所有类型的超类型。
- **语义**：表示“任何可能的值”。
- **型变表现**：在正极性位置（输出）时提供最少信息，在负极性位置（输入）时要求最严苛。

### 2. 底类型 (Bottom Type / ⊥)：`never`
- **极点位置**：所有类型的子类型。
- **语义**：表示“不可能发生”或“空集合”。
- **型变表现**：在正极性位置（输出）时可以赋值给任何类型（因为永远不会真的产生值），在负极性位置（输入）时表示该函数无法被调用。

---

## 结构化子类型 (Structural Subtyping)

对于记录（Record）类型，如果 `A` 包含 `B` 的所有字段，则 `A` 是 `B` 的子类型。这在底层由 [行类型与多态](./row-types.md) 机制支撑。
```valkyrie
type Point2D = { x: f64, y: f64 }
type Point3D = { x: f64, y: f64, z: f64 }

# Point3D <: Point2D
let p2: Point2D = Point3D { x: 1, y: 2, z: 3 }
```

---

## 类型转换 (Conversions)

Valkyrie 区分了基于子类型关系的隐式转换和显式转换。

### 1. 向上转型 (Upcasting)
从子类型到父类型的转换（如从 `Dog` 到 `Animal`，或从 `i32` 到 `any`）通常是隐式的，因为它是类型安全的。

### 2. 显式转换 (Casting)
使用 `@cast` 或特定方法进行显式转换，常用于存在信息丢失风险的场景（如浮点数转整数）。
```valkyrie
let a: f64 = 1.5
let b: i32 = @cast(a) # 显式截断转换
```

### 3. 原始指针转换
对于底层操作，可以使用 `@pointer_cast` 进行不安全的指针类型重新解释。

---

## 极性的代数应用：函数类型

函数是极性反转最经典的舞台。对于函数类型 `micro(P) -> R`：

- **返回类型 `R` 处于正极性位置**：它是函数的产出。
- **参数类型 `P` 处于负极性位置**：它是函数的消耗。

因此，一个函数 `f1` 是 `f2` 的子类型，当且仅当：
`f1.Input` 是 `f2.Input` 的超类型（逆变） **且** `f1.Output` 是 `f2.Output` 的子类型（协变）。

```valkyrie
# 极性标注示例
trait Function⟨-In, +Out⟩ {
    micro call(arg: In) -> Out
}
```

---

## 容器与可变性

### 1. 只读容器（正极性）
在 Valkyrie 中，只读容器（如 `[T]`）被视为 T 的生产者，因此 T 处于正极性位置，表现为协变。

### 2. 可变容器（不变性）
当一个泛型参数 `T` 同时出现在输入位置（`push(T)`）和输出位置（`get() -> T`）时，其极性相互抵消，最终表现为**不变性 (Invariance)**。

---

## 型变注解 (Variance Annotations)

在定义泛型类型时，你可以使用极性符号来显式声明：

- **`+T`**：显式声明为协变（正极性）。
- **`-T`**：显式声明为逆变（负极性）。

```valkyrie
# 生产者（正极性）
trait Producer⟨+T⟩ {
    micro produce() -> T
}

# 消费者（负极性）
trait Consumer⟨-T⟩ {
    micro consume(item: T)
}
```

---

## 总结

- **协变 (+)** 是向上的、生产性的极性。
- **逆变 (-)** 是向下的、消耗性的极性。
- **极点 (`any`/`never`)** 定义了子类型化关系的边界。

---

**上一页**: [交集与并集](./intersection-union.md) | **下一页**: [类型函数](./type-function.md)

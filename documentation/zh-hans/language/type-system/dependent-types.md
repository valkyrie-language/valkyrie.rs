# 依赖类型 (Dependent Types)

依赖类型是类型系统中最强大的特性之一，它允许**类型依赖于值**。在 Valkyrie 中，这意味着你可以在类型签名中直接引用运行时或编译时的值，从而实现极高精度的类型约束。

## 核心表现

### 1. 常量泛型 (Const Generics)
这是依赖类型最常见的形式，类型参数不仅可以是类型，还可以是具体的数值。

```valkyrie
# 数组长度是类型的一部分
structure Vector⟨T, N: usize⟩ {
    data: array⟨T, N⟩
}

# 两个不同长度的 Vector 是不同的类型
let v1: Vector⟨f32, 3⟩ = Vector::new([1.0, 2.0, 3.0])
let v2: Vector⟨f32, 4⟩ = Vector::new([1.0, 2.0, 3.0, 4.0])
```

### 2. 精化类型 (Refinement Types)
精化类型通过谓词（Predicate）来限制现有类型的取值范围。

```valkyrie
# 定义一个正整数类型
type PositiveInt = i32 where { % > 0 }

# 定义一个非空列表
type NonEmptyList⟨T⟩ = [T] where { %.length > 0 }

micro first⟨T⟩(list: NonEmptyList⟨T⟩) -> T {
    list[0] # 这里不需要返回 Option，因为类型保证了列表非空
}
```

---

## 依赖函数 (Dependent Functions)

函数的返回类型可以取决于其输入参数的值。

```valkyrie
# 根据输入的长度返回特定大小的数组
micro create_array(n: usize) -> array⟨i32, n⟩ {
    @uninitialized()
}

# 这里的 n 决定了返回值的具体类型
let arr = create_array(5) # 类型为 array⟨i32, 5⟩
```

---

## 为什么需要依赖类型？

1. **消除边界检查**: 通过类型保证索引永远不会越界，从而在运行时安全地移除边界检查。
2. **形式化验证**: 在编译时验证复杂的数学属性或业务逻辑（如：转账金额必须小于余额）。
3. **精准建模**: 描述高度结构化的数据协议，如网络数据包的大小字段必须与后续数据长度匹配。

---

## 局限性与挑战

依赖类型虽然强大，但也带来了以下挑战：
- **编译时负担**: 编译器需要执行更复杂的逻辑推理。
- **不可判定性**: 某些复杂的谓词可能导致编译器无法确定类型是否匹配。
- **语法复杂性**: 需要更精细的代码标注。

---

**上一页**: [类型级编程](./type-level.md) | **下一页**: [线性类型](./linear-types.md)

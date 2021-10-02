# 类型函数 (Type Function)

类型函数用于在类型层面进行计算。通过 `mezzo` 关键字定义，类型函数在编译时对类型进行操作和变换。

## 背景：类型层与值层分离

TypeScript 中常见的重复定义来自“类型与值的分离” (The Type-Value Gap)：

1.  **语言不统一**：TypeScript 实际上包含了两门完全不同的语言：运行时的 JavaScript 和编译时的 Type DSL。它们拥有不同的语法、不同的控制流（JS 用 `if/match`，类型用嵌套三元）和不同的执行引擎。
2.  **类型不是值**：在 TS 中，类型在运行时被擦除，它们不是“一等公民”。你不能像传递字符串一样传递一个类型，也不能用同一个函数同时处理 `1` 和 `number`。

### 统一求值模型

Valkyrie 在该问题上的基本设定是：**类型就是值 (Types are Values)**。

在 Valkyrie 中，编译器并不区分“类型级语言”和“值级语言”。它只有一套统一的语法和求值引擎。`mezzo` 函数和 `@const_fn` 只是在不同的阶段（编译时 vs 运行时）运行相同的逻辑。

#### 示例

下面的示例使用统一逻辑描述同一组变换规则。

```valkyrie
# 逻辑定义：描述一种“平方或长度”的变换规则
# 这里的逻辑既可以作用于具体的数值，也可以作用于类型本身
@const_fn
micro transform⟨T⟩(input: T) {
    match input {
        # 当 input 是具体的值时
        case i: i32: i * i
        case s: utf8: s.count()
        
        # 当 input 是类型本身时 (在 Valkyrie 中类型也是一种值)
        # 这消除了“写两遍”的必要：逻辑分支在同一处定义
        case i32: i32
        case utf8: i32
        
        case _: @error("Unsupported")
    }
}

# 1. 运行时：处理数据
let r1 = transform(10)      # 100

# 2. 编译时：直接在类型签名中复用 transform
# 注意：这里我们直接把函数调用写在了返回类型的位置
micro process_data⟨T: i32 | utf8⟩(val: T) -> transform(T) {
    transform(val)
}

# 3. 静态验证
type R = transform(i32) # R 就是 i32
```

**性质**
- **语法统一**：值级与类型级逻辑共用相同的控制结构。
- **逻辑唯一**：同一变换规则只定义一次。
- **阶段区分**：编译器根据使用位置决定逻辑在编译期还是运行期求值。

---

## 机制：`mezzo` 与 `@const_fn`

```valkyrie
mezzo FunctionName(param: Type) -> ReturnType {
    # 类型函数体
}
```

## 逻辑分支与匹配

类型函数可以使用 `if` 和 `match` 进行逻辑分支处理。

### 1. 条件选择 (If-Else)
```valkyrie
mezzo ConditionalType⟨T, U⟩(condition: bool) -> Type {
    if condition { T } else { U }
}
```

### 2. 类型模式匹配 (Match)
```valkyrie
mezzo MapType(input: Type) -> Type {
    match input {
        case i32: i64
        case f32: f64
        case _: input
    }
}
```

---

## 递归与性质

类型函数支持递归定义，这使得处理嵌套结构（如元组列表）成为可能。

### 1. 递归类型函数
```valkyrie
mezzo Flatten⟨T⟩(input: T) -> Type {
    match input {
        case (Head, Tail): Flatten⟨Tail⟩
        case _: input
    }
}
```

### 2. 核心特性
- **编译时执行**: 类型函数在编译阶段完全展开，不产生运行时开销。
- **纯粹性**: 类型函数必须是纯函数，不能产生任何副作用。
- **确定性**: 相同的输入必须产生相同的输出类型。
- **递归深度**: 编译器对递归深度有限制，以防止编译时死循环。

---

## 类型级映射

类型函数可以表达复杂的类型变换，例如异构列表处理或 API 绑定生成。

### 示例：自动 `Result` 包装
```valkyrie
mezzo ToResult(T: Type) -> Type {
    unite { Ok(T), Err(utf8) }
}

# 使用示例
type SafeInt = ToResult(i32) 
# 等价于 unite { Ok(i32), Err(utf8) }
```

---

## 适用范围

1. **类型验证**: 在编译时验证类型是否满足特定条件。
2. **类型转换**: 自动推导和转换相关类型。
3. **泛型约束**: 为泛型参数添加复杂的类型约束。
4. **元编程**: 实现高级的编译时代码生成。

---

**上一页**: [型变与子类型](./polarity-type.md) | **下一页**: [高阶类型 (HKT)](./higher-kinded-types.md)

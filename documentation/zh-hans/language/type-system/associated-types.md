# 关联类型 (Associated Types)

关联类型是 Valkyrie 具名 `trait` 系统中的高级抽象机制。它允许你在 `trait` 定义中声明一个占位类型，该类型的具体选择将由 `trait` 的实现者决定。

## 语义边界

关联类型只属于具名 `trait`，不属于匿名 row。

- 具名 `trait` 可以声明 `type Item`
- 匿名 `{ next() -> ... }` row 不可以声明 `type Item`
- 如果某个抽象需要 `associated type`，它就已经不是“轻量方法行约束”，而是完整的协议系统

## 定义

关联类型将“类型占位符”与具名 `trait` 绑定在一起。相比于泛型参数，关联类型更强调一种**函数式映射关系**：对于每一个实现该 `trait` 的具体类型，关联类型都有唯一确定的输出。

### 基本语法

在 `trait` 中使用 `type` 关键字声明关联类型：

```valkyrie
trait Iterator {
    type Item
    
    # next 方法返回该关联类型
    micro next(mut self) -> Option⟨Self::Item⟩
}
```

在实现 `trait` 时指定具体类型：

```valkyrie
imply [i32]: Iterator {
    type Item = i32
    
    micro next(mut self) -> Option⟨i32⟩ {
        # ... 实现细节
    }
}
```

## 关联类型 vs. 泛型参数

选择关联类型还是泛型参数是设计抽象接口时的关键决策。

### 1. 唯一性约束 (One Implementation per Type)
- **关联类型**：一个类型只能为给定的 `trait` 提供**一个**实现。例如，`[i32]` 只能有一个 `Iterator` 实现，其 `Item` 必须是 `i32`。
- **泛型参数**：一个类型可以为同一个 `trait` 提供**多个**实现。例如，`Data` 类型可以同时实现 `Convert⟨i32⟩` 和 `Convert⟨utf8⟩`。

匿名 row 不支持关联类型，因为 row 没有具名协议身份，也没有“每个 `(Type, Trait)` 唯一实现”的宿主。

### 2. 语法简洁性
使用关联类型可以显著减少函数签名中的泛型堆叠。

**使用泛型参数（冗长）**：
```valkyrie
micro process⟨I, T⟩(iter: I) where I: Iterator⟨T⟩ { ... }
```

**使用关联类型（简洁）**：
```valkyrie
micro process⟨I: Iterator⟩(iter: I) {
    # 可以通过双冒号访问关联类型
    let first: I::Item = iter.next()?
}
```

## 高级用法

### 1. 带有约束的关联类型
你可以为关联类型本身添加特征约束 (Trait Bounds)：

```valkyrie
trait Container {
    type Element: Display + Clone
    
    micro get(self, index: usize) -> Self::Element
}
```

### 2. 关联类型的默认值
在定义 `trait` 时可以提供默认类型：

```valkyrie
trait Logger {
    type Output = utf8
    micro log(self, msg: utf8) -> Self::Output
}
```

### 3. GATs (Generic Associated Types)
Valkyrie 支持泛型关联类型，允许关联类型本身携带泛型参数，用于表达更复杂的类型映射关系：

```valkyrie
trait Iterable {
    # 关联类型本身是泛型的
    type Collection⟨T⟩
    
    # 使用泛型关联类型定义变换操作
    micro map⟨T, U⟩(self: Self:::Collection⟨T⟩, f: micro(T) -> U) -> Self:::Collection⟨U⟩
}
```

## 与匿名 row 的对比

下面两种写法不应混为一谈：

```valkyrie
trait Iterator {
    type Item
    micro next(mut self) -> Option⟨Self::Item⟩
}
```

```valkyrie
{ next() -> Option⟨i32⟩ }
```

前者表示：

- 一个具名协议
- 有稳定的协议身份
- 可以携带 `Item`
- 需要具名 witness

后者只表示：

- 一个匿名方法行 requirement
- 只检查 `next()` 这个方法面
- 不引入 `Item`
- 不具备 trait 级别的关联事实

如果你需要表达 `next()` 返回的元素类型在系统中继续传播，应优先写具名 `trait`，而不是试图把匿名 row 扩展成半套 trait system。

---

**上一页**: [泛型编程](./generics.md) | **下一页**: [行类型与多态](./row-types.md)

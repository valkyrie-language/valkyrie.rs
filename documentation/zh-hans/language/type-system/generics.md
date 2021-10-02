# 泛型编程 (Generic Programming)

泛型用于编写可作用于多种类型的代码，而无需为每种类型重复定义逻辑。Valkyrie 的泛型系统结合静态参数化与约束机制。

## 形式化约定

本页把泛型抽象定义为参数化类型或参数化函数。

- `F⟨T⟩` 表示类型构造器 `F` 在参数 `T` 上的实例化。
- `micro f⟨T⟩(...)` 表示对类型参数 `T` 全称量化的函数。
- `T: TraitA + TraitB` 表示 `T` 必须同时满足多个具名协议约束。

除非文档另行说明，本页中的约束默认都是具名 `trait` 约束，而不是匿名 row。

## 泛型参数

泛型参数使用数学角括号 `⟨ ⟩` 声明：

```valkyrie
# 泛型结构体
structure Box⟨T⟩ {
    item: T
}

# 泛型函数
micro identity⟨T⟩(value: T) -> T {
    value
}

# 多个泛型参数
type Pair⟨K, V⟩ = {
    key: K,
    value: V,
}
```

## 默认泛型参数

你可以为泛型参数提供默认值。

```valkyrie
structure Map⟨K, V, S = DefaultHasher⟩ {
    # ...
}
```

---

## 泛型约束 (Constraints)

你可以通过 Trait 约束泛型参数必须具备的行为。

这里的约束默认指向**具名 `trait` 协议**，而不是匿名 row。

- `T: Display` 表示 `T` 满足具名协议 `Display`。
- 如果你只想表达“当前具备一组方法即可”，更适合直接使用匿名 row 参数。
- 一旦约束涉及 `associated type`、默认实现或协议 identity，就应放在具名 `trait` 这一层解释。

### 1. 内联约束
```valkyrie
micro print_item⟨T: Display⟩(item: T) {
    print(item.fmt())
}
```

### 2. Where 子句
对于复杂约束，可以使用 `where` 子句。
```valkyrie
micro process_data⟨T, U⟩(t: T, u: U) 
where
    T: Display + Clone,
    U: IntoIterator⟨Item = T⟩
{
    # ...
}
```

## 特化 (Specialization)

Valkyrie 支持为特定类型提供更优的泛型实现。

```valkyrie
imply⟨T⟩ Box⟨T⟩ {
    micro describe(self) -> utf8 { "A generic box" }
}

# 为 Box⟨i32⟩ 提供特化实现
imply Box⟨i32⟩ {
    micro describe(self) -> utf8 { "A box containing an integer" }
}
```

---

## 全称量化 vs 存在量化

Valkyrie 的类型系统区分了两种主要的量化形式：

- **全称量化 (Universal Quantification - `∀`)**: 
  - 形式：`micro func⟨T⟩(item: T)`
  - 含义：调用者决定 `T` 是什么，函数必须对**所有**满足约束的 `T` 有效。
- **存在量化 (Existential Quantification - `∃`)**:
  - 形式：`let item: Display = ...` (Trait 对象)
  - 含义：实现者决定 `T` 是什么，调用者只知道它满足 `Display` 特征，但不知道具体类型。
  - 详细参考：[Trait 系统中的见证表](../object-oriented/trait-system.md#底层原理见证表-witness-table)。

---

## 应用：虚幻类型 (Phantom Types)

虚幻类型是指在定义中使用了泛型参数，但该参数并未在结构体的字段中实际使用的模式。它常用于在编译时追踪对象的状态。

### 场景：类型安全的 Web 请求
```valkyrie
# 定义状态标记
structure Unvalidated {}
structure Validated {}

# Request 结构体包含一个虚幻类型参数 S
structure Request⟨S⟩ {
    url: utf8,
    body: utf8,
}

# 只有未验证的请求可以被验证
micro validate(req: Request⟨Unvalidated⟩) -> Request⟨Validated⟩ {
    # 执行验证逻辑...
    Request⟨Validated⟩ { url: req.url, body: req.body }
}

# 只有验证过的请求可以被发送
micro send(req: Request⟨Validated⟩) -> Unit / IO {
    # 发送请求...
}

micro main() {
    let req = Request⟨Unvalidated⟩ { url: "...", body: "..." }
    
    # send(req) # 编译错误：期待 Request⟨Validated⟩，得到 Request⟨Unvalidated⟩
    
    let valid_req = validate(req)
    send(valid_req) # 编译通过
}
```

---

**上一页**: [指针与引用](./pointer-type.md) | **下一页**: [关联类型](./associated-types.md)

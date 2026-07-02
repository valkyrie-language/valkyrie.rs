# 效应类型 (Effect Types)

在 Valkyrie 中，函数的类型不仅包含输入和输出的值类型，还包含了函数执行时产生的**效应 (Effects)**。效应系统允许编译器静态追踪代码的副作用，如 IO 操作、非局部跳转、状态变更等。

## 形式化定义

本页把函数类型记作：

```text
micro(P1, ..., Pn) -> R / E
```

其中：

- `P1 ... Pn` 是参数类型。
- `R` 是返回类型。
- `E` 是 effect 集；纯函数可视为 `E = ∅`。

据此，effect 检查至少包含两类问题：

- 传播：被调用表达式的 effect 是否包含于当前上下文声明中。
- 消除：某个 effect 是否被 handler 捕获并从外层 effect 集中移除。

## 语法

效应紧跟在返回类型之后，使用 `/` 符号分隔：

```valkyrie
# 纯函数：不产生任何效应（默认）
micro add(a: i32, b: i32) -> i32 {
    a + b
}

# 带有 IO 效应的函数
micro print_hello() -> Unit / IO {
    print("Hello")
}

# 带有多个效应的函数
micro process() -> i32 / IO + Error {
    # ...
}
```

## 常见的内置效应

| 效应 | 说明 |
| :--- | :--- |
| **`IO`** | 进行输入输出操作（文件、网络、控制台）。 |
| **`Error`** | 可能抛出异常或错误。 |
| **`Async`** | 异步执行。 |
| **`State`** | 访问或修改外部全局/闭包状态。 |
| **`NonDet`** | 非确定性计算。 |

---

## 效应的传播与消除

### 1. 自动传播
如果一个函数调用了另一个带有效应的函数，那么该效应会自动传播到当前函数。

```valkyrie
micro outer() -> Unit / IO {
    print_hello() # 产生 IO 效应，必须在签名中声明
}
```

### 2. 效应处理器 (Effect Handlers)
你可以通过效应处理器来捕获并消除效应，将其转化为具体的值或另一种效应。

```valkyrie
micro main() -> Unit {
    # try-catch 块可以消除效应
    try {
        run_app()
    }.catch {
        case IO: resume(())
    }
}
```

可以把这两条规则概括为：

- 若表达式 `e` 具有 effect 集 `E1`，则包裹它的函数签名必须声明至少包含 `E1` 的 effect 集。
- 若 handler 明确捕获 `E1` 中的某个 effect，并给出恢复语义，则该 effect 可以从外层暴露集里移除。

---

## 设计动机

1. **可见性**: 从函数签名就能一眼看出该函数是否安全，是否会修改全局状态或进行网络请求。
2. **解耦**: 逻辑代码只声明需要的效应，具体的实现（如存向文件还是数据库）由上层处理器决定。
3. **并发安全**: 编译器可以禁止在某些特定的并发上下文（如渲染循环）中执行带有 `IO` 或阻塞效应的函数。

## 与 trait、row、nominal type 的边界

效应类型和 `row`、具名 `trait`、`class / unite` 不是同一层抽象。

- `row` 回答“你当前会不会这些方法”。
- 具名 `trait` 回答“你是否满足这个协议，并能否形成 witness”。
- `class / unite` 回答“你是不是这个声明家族里的成员”。
- `effect` 回答“这个调用是否携带某种能力，并是否存在 handler 证据”。

把这些边界分开之后，类型检查、调用分发和后端 lowering 才不会互相冒充。

进一步说：

- `effect` 不得被解释为名义子类型。
- `effect` 不得被解释为具名 `trait` 满足。
- `effect` 不得被解释为匿名 row 方法覆盖。
- `effect` 只在调用、传播与 handler 消除这条语义链上起作用。

---

## 应用：依赖注入 (Dependency Injection)

效应系统可以用作依赖注入机制：逻辑层声明 effect，环境层提供 handler。

### 示例：可测试的数据库操作

定义效应载体结构体：

```valkyrie
# 数据库查询请求
structure DatabaseQuery {
    query: String
    params: Vec<Value>
}
```

业务逻辑代码：

```valkyrie
# 业务逻辑：使用 raise 发起效应请求
micro process_user(id: u64) -> User / IO {
    let result = raise DatabaseQuery {
        query = "SELECT * FROM users WHERE id = ?",
        params = [Value::from(id)]
    }
    result.as_user()
}
```

生产环境处理器：

```valkyrie
micro run_prod() -> Unit {
    catch process_user(1) {
        case DatabaseQuery { query, params }: 、
        resume(execute_sql(query, params))
    }
}
```

测试环境处理器 (Mock)：

```valkyrie
micro run_test() -> Unit {
    catch process_user(1) {
        case DatabaseQuery { query, params }: 
            resume(mock_query_result())
    }
}
```

---

**上一页**: [线性类型](./linear-types.md) | **下一页**: [类型系统 (Index)](./index.md)

- [效应系统](../effect-system/index.md) 说明自定义 effect 与 handler 的定义方式。
- [代数数据类型](./algebraic-data-types.md) 说明 ADT 与 effect 系统的配合方式。

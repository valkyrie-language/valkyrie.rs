# 效应类型 (Effect Types)

在 Valkyrie 中，函数的类型不仅包含输入和输出的值类型，还包含了函数执行时产生的**效应 (Effects)**。效应系统允许编译器静态追踪代码的副作用，如 IO 操作、非局部跳转、状态变更等。

## 核心语法

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

---

## 为什么需要效应类型？

1. **可见性**: 从函数签名就能一眼看出该函数是否安全，是否会修改全局状态或进行网络请求。
2. **解耦**: 逻辑代码只声明需要的效应，具体的实现（如存向文件还是数据库）由上层处理器决定。
3. **并发安全**: 编译器可以禁止在某些特定的并发上下文（如渲染循环）中执行带有 `IO` 或阻塞效应的函数。

---

## 进阶应用：依赖注入 (Dependency Injection)

效应系统提供了一种极其优雅的依赖注入方式：逻辑层声明效应，环境层提供处理器。

### 场景：可测试的数据库操作

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

- 探索 [效应系统](../effect-system/index.md) 了解如何定义自定义效应和处理器。
- 了解 [代数数据类型](./algebraic-data-types.md) 如何与效应系统协同工作。

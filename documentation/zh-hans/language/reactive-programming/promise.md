# Promise (承诺)

`Promise` 是 `Future` 的标准实现。它不仅代表一个未来的值，还提供了手动控制该值何时完成的能力。

## 基本用法

通常，你可以通过 `async` 块自动创建一个 `Promise`：

```valkyrie
let p: Promise⟨i32⟩ = async {
    42
}
```

### 显式创建与完成

在某些低级场景或与外部代码交互时，你可能需要手动控制 `Promise`：

```valkyrie
# 创建一个处于挂起状态的 Promise 和它的解析器
let (p, resolver) = Promise.pending⟨string⟩()

# 在稍后的某个时刻手动完成它
resolver.resolve("Success!")

# 或者让它失败
# resolver.reject(Error("Failed"))
```

## 运行控制

`Promise` 提供了三种主要的运行模式，通过 `.run` 控制器（通常可省略）访问：

### 1. 异步等待 (.await)
在异步函数中挂起，不阻塞线程。
```valkyrie
let data = fetch_data().await
```

### 2. 同步阻塞 (.block)
在同步环境中阻塞当前线程，直到结果返回。
```valkyrie
let data = fetch_data().block
```

### 3. 异步启动 (.awake)
启动任务但不等待其结果（Fire and Forget）。
```valkyrie
fetch_data().awake
```

## 静态方法

- `Promise.resolve(val)`: 创建一个已经成功的 Promise。
- `Promise.reject(err)`: 创建一个已经失败的 Promise。
- `Promise.all([p1, p2])`: 等待所有 Promise 完成，如果任一失败则整体失败。
- `Promise.any([p1, p2])`: 只要有一个 Promise 成功就返回其结果。
- `Promise.allSettled([p1, p2])`: 等待所有 Promise 完成（无论成功或失败）。

## 与 JavaScript Promise 的关系

Valkyrie 的 `Promise` 在编译到 JavaScript 后会直接映射为原生的 `Promise` 对象，确保了零开销的互操作性。

---
**相关章节**:
- [Future](./future.md) - 异步底层原语
- [运行控制](./index.md#运行控制runawait--runblock--runawake--awake) - 详细的运行模式说明

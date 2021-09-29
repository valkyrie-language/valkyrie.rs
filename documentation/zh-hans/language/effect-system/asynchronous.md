# 异步效应 (Async Effect)

在 Valkyrie 中，异步编程不仅仅是一套语法糖，它是 **代数效应 (Algebraic Effects)** 的一种具体应用。借鉴了 C# `async2` (Runtime-handled Tasks) 的理念，Valkyrie 将异步逻辑从编译器层面下沉到了运行时层面。

## 核心理念：Await 也是一种效应

在传统的异步模型（如 Rust 或 C# 5.0）中，`async` 函数会被编译器重写为一个复杂的**状态机**。

而在 Valkyrie 中：
- **`.await` 是一个效应**：当你调用 `.await` 时，它本质上是 `raise` 了一个携带 `Future` 或 `Task` 对象的效应。
- **调度器是处理器 (Handler)**：运行时环境（如 Nyar VM）提供了一个顶层的效应处理器。它捕获 `await` 效应，挂起当前的续体（Continuation），并将其交给异步调度器（Executor）管理。

## 为什么这样做？

### 1. 消除函数着色 (Function Coloring)
由于异步是通过效应系统实现的，异步函数和同步函数在底层结构上高度统一。编译器不需要为 `async` 生成完全不同的代码路径，这使得异步代码的性能和调用方式更接近同步代码。

### 2. 运行时托管 (Runtime-managed)
类似于 C# 的 `async2` 实验，Valkyrie 将任务的挂起和恢复交给运行时直接处理。
- **零成本重写**：字节码保持简洁，没有海量的状态机跳转。
- **动态优化**：运行时可以根据当前的 CPU 负载、I/O 状态，动态决定是立即恢复续体还是将其放入等待队列。

## 异步原语：解耦编译器与运行时

与许多语言不同，Valkyrie 的核心编译器（HIR）并不包含 `Future`、`Promise` 或 `async/await` 的特殊语法树结构。

- **库定义而非内建**：`Future` 和 `Promise` 是标准库中定义的普通 Trait 和 Class。
- **透明的异步**：对于编译器而言，`.await` 只是一个触发效应的操作，`.block` 只是一个普通的属性访问。
- **运行时调度**：这种设计借鉴了 C# `async2` 的核心理念。在 `async2` 中，运行时负责管理任务的暂停和恢复（通过轻量级续体），而不是让编译器为每个异步函数生成沉重的状态机代码。

### 带来的优势

1. **零成本抽象**：当异步代码同步执行时，没有状态机切换的开销。
2. **极简的字节码**：Nyar VM 只需要处理 `Perform`、`CaptureCont` 和 `ResumeWith` 等通用指令，即可支持复杂的异步逻辑。
3. **更强的互操作性**：由于异步只是效应的一种，你可以轻松地在异步代码中使用其他的代数效应（如依赖注入、异常处理等）。

## 运行机制

### 效应流转过程

1. **触发 (Raise)**: 执行到 `future.await` 时，虚拟机 `raise` 一个携带 Future 的效应。
2. **挂起 (Suspend)**: 虚拟机立即保存当前函数的执行状态（寄存器、栈帧、IP）。
3. **捕获 (Catch)**: 效应冒泡到最近的异步处理器（通常是 `AsyncRuntime`）。
4. **注册 (Register)**: 调度器将该 `future` 注册到 I/O 多路复用器（如 epoll/kqueue）或计时器中。
5. **恢复 (Resume)**: 当 `future` 完成时，调度器找到对应的续体，恢复虚拟机的执行状态。

## 示例：底层视角

## 示例：底层视角

当你写下：
```valkyrie
let data = socket.read().await
```

在底层，它等价于 `raise` 一个异步效应：
```valkyrie
let data = raise AsyncAwait { future: socket.read() }
```

如果是在一个没有异步处理器的同步环境中运行，这个效应会一直向上冒泡，直到被 `.block` 对应的处理器捕获，或者导致程序因"未处理效应"而崩溃。这确保了异步行为的可预测性和显式性。

## 异步语法

### 异步块：`async { }`

在 Valkyrie 中，你可以使用 `async { ... }` 创建一个异步任务。需要注意的是，这并不是一种特殊的关键字语法，而是 **函数调用配合尾随闭包** 的标准语法：
- `async` 是一个普通函数。
- `{ ... }` 是传递给该函数的尾随闭包。
- 该函数执行后返回一个 `Promise` 实例。

```valkyrie
let p = async {
    let data = fetch_data().await
    process(data)
}
```

### 自动执行与显式控制

为了简化代码，Valkyrie 对返回 `Future` 的函数调用应用了以下规则：

1. **自动等待**：在异步上下文中，`obj.call_fut()` 会被自动视为 `obj.call_fut().await`。
2. **后缀控制**：你可以显式使用后缀来改变行为：
   - `.await`：显式挂起并等待结果。
   - `.awake`：立即启动任务并继续执行（Fire and Forget）。
   - `.block`：在当前线程阻塞等待结果。

### 快捷函数：`go`

同样地，`go { }` 也是一个接收闭包的快捷函数，它立即以 `.awake` 模式运行任务：

```valkyrie
# 使用 go 函数启动后台任务
go {
    logger.info("Task started")
    do_some_work().await
    logger.info("Task finished")
}
```

其定义非常简单，本质上是调用 `async` 并紧接着调用 `.awake`：
```valkyrie
micro go(body: () -> T) -> Promise⟨T⟩ {
    async(body).awake
}
```

## 运行控制 (Execution Control)

为了统一控制异步任务的执行，Valkyrie 提供了三种核心的运行模式。从效应系统的视角来看，它们代表了不同的效应处理策略：

### 1. 异步等待 (`.await`)
**语义**：挂起当前协程，直到结果就绪。
- **底层机制**：`raise` 一个异步效应，由顶层异步处理器捕获并注册到调度器。
- **使用场景**：绝大多数异步编程场景。
```valkyrie
let data = fetch_api().await
```

### 2. 同步阻塞 (`.block`)
**语义**：阻塞当前物理线程，直到异步任务完成。
- **底层机制**：这是一个特殊的效应处理器。它捕获 `await` 效应后，并不将控制权交还给 OS 线程，而是原地启动一个简单的轮询循环（Spin/Poll），直到获取结果。
- **使用场景**：`main` 函数入口、单元测试、或者必须与同步遗留代码交互的边界。
```valkyrie
micro main() {
    let result = run_async_task().block
}
```

### 3. 异步启动 (`.awake`)
**语义**：触发并忽略 (Fire and Forget)。
- **底层机制**：它并不 `raise` 异步效应，而是直接向调度器发送一个"启动"信号。当前函数不需要挂起，立即继续执行。
- **使用场景**：日志记录、遥测统计、后台缓存刷新等非关键路径任务。
```valkyrie
# 使用后缀语法启动后台任务
refresh_cache().awake
```

## 异步原语与类型系统

### Future：底层契约
`Future` 是异步效应的载体。在底层，任何实现了 `poll` 效应的方法都可以被视为 `Future`。

### Promise：标准实现
`Promise` 是 `Future` 的具体实现，它与 JavaScript 的 Promise 具有零开销的互操作性。在 Valkyrie 中，你可以手动控制 Promise 的解析：
```valkyrie
let (p, resolver) = Promise.pending⟨string⟩()
resolver.resolve("Done")
```

## 与协程的关系

异步效应是协程的一种特化：
- **协程**：手动控制 `yield` 和 `resume`。
- **异步效应**：由运行时调度器自动控制 `yield` (await) 和 `resume` (ready)。

---
**相关章节**:
- [协程](./coroutine.md) - 异步效应的基础
- [Channel (通道)](../reactive-programming/channel.md) - 任务间的通信与协作
- [Future (未来量)](../reactive-programming/future.md) - 异步操作的承载体

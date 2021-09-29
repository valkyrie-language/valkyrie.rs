# Channel (通道)

`Channel` 是 Valkyrie 中用于并发任务间通信的核心原语。它通常 with `go { }` 块配合使用，实现 CSP (Communicating Sequential Processes) 编程模型。

## 从 go { } 到并发协作

当我们使用 `go { }` 启动一个后台任务时，该任务便脱离了当前的执行流。为了在不同任务之间安全地传递数据，我们引入了 `Channel`。

```valkyrie
let (tx, rx) = Channel::new⟨i32⟩()

# 启动生产者任务
go {
    loop i in 1..5 {
        tx.send(i).await
    }
    tx.close()
}

# 在主流程中消费数据
# rx 本身就是一个异步流 (Stream)
loop item in rx {
    print("Received: {item}")
}
```

## 核心特性

### 1. 生产者-消费者模型
`Channel::new()` 返回一对句柄：
- **Sender (tx)**: 用于发送数据。
- **Receiver (rx)**: 用于接收数据。`Receiver` 实现了 `Stream` 接口，因此可以像迭代器一样在 `for` 循环中使用。

### 2. 异步挂起
- **`tx.send(val).await`**: 如果通道缓冲区已满，发送操作会触发异步挂起，直到有空间可用。
- **`rx.receive().await`**: 如果通道为空，接收操作会挂起，直到有新数据进入。

### 3. 多对多通信
Valkyrie 的 `Channel` 支持：
- **MPMC (Multi-Producer, Multi-Consumer)**: 多个 `go` 任务可以共享同一个 `Sender` 或 `Receiver`。

## 通道拓扑模型

根据生产者和消费者的数量，Valkyrie 提供了多种通道模型以优化性能：

### 1. SPSC (Single-Producer, Single-Consumer)
最简单的模型，一个发送者对应一个接收者。适用于简单的流水线任务。
- **特点**：极高的性能，无锁或低锁实现。

### 2. MPSC (Multi-Producer, Single-Consumer)
最常见的模型，多个后台任务将结果汇总到一个中央处理器。
- **示例**：日志收集系统，多个 `go` 任务向同一个 `Logger` 发送消息。
```valkyrie
let (tx, rx) = Channel::mpsc()
go { tx.send("Task A done") }
go { tx.send("Task B done") }
```

### 3. MPMC (Multi-Producer, Multi-Consumer)
最通用的模型，多个任务发送，多个任务竞争处理。
- **场景**：工作池（Worker Pool）。
- **特点**：自动实现负载均衡，谁闲着谁处理。

## 通道类型

### 1. 无缓冲通道 (Rendezvous)
默认创建的通道通常是无缓冲的。发送者和接收者必须“同步”碰头，数据才能传递。
```valkyrie
let (tx, rx) = Channel::new()
```

### 2. 有缓冲通道
可以指定缓冲区大小，发送者在缓冲区未满时不会挂起。
```valkyrie
let (tx, rx) = Channel::buffered(10)
```

## 与 Stream 的关系

`Channel` 的接收端是 `Stream` 的一种动态实现。这意味着你可以对 `rx` 使用所有 `Stream` 的组合子：

```valkyrie
let doubled_stream = rx.map { % * 2 }
                       .filter { % > 10 }

doubled_stream.for_each { print(%) }.await
```

## 设计选择：Channel vs Async/Await

在编写并发程序时，你可能会纠结是直接使用 `async/await` 还是引入 `Channel`。以下是建议的选择标准：

### 什么时候使用 Async/Await (Future)？
- **请求-响应模型**：当你调用一个函数并期望在未来某个时间点获得**一个**明确的结果时。
- **简单的依赖链**：任务 A 必须在任务 B 之前完成，且 A 的输出是 B 的输入。
- **并发汇聚**：使用 `Future::join_all` 等工具同时等待多个任务的结果并汇总。
- **语义**：它更像是“会耗费时间的普通函数调用”。

### 什么时候使用 Channel？
- **数据流与管道**：当数据是**连续产生**的，且需要流经多个处理步骤（如解析 -> 过滤 -> 存储）。
- **生产者-消费者解耦**：当产生数据的速度与处理数据的速度不匹配，需要缓冲区来缓冲压力（Backpressure）。
- **多对多协作**：多个任务共同处理一个任务池，或者多个任务向同一个中心任务汇报状态。
- **语义**：它更像是“不同组件之间的通信线路”。

---
**相关章节**:
- [异步效应 (Asynchronous)](../effect-system/asynchronous.md) - 了解 `go { }` 的底层原理
- [Stream (流)](./stream.md) - 如何处理连续的数据序列

# Future (未来量)

`Future` 是 Valkyrie 异步编程模型中最基础的特征（Trait）。它代表一个在未来某个时间点才会变为可用的值。

## 核心概念

`Future` 描述了一个尚未完成的计算。它不代表计算本身，而是代表对计算结果的引用。

### Future 特征定义

在底层，`Future` 类似于如下定义：

```valkyrie
trait Future⟨T⟩ {
    # 尝试轮询 Future 的状态
    # 如果已完成，返回 Fine(T)
    # 如果未完成，返回 Pending
    micro poll(self, cx: Context) -> Poll⟨T⟩
}
```

## 自动等待语义

在 Valkyrie 中，绝大多数情况下你不需要手动调用 `poll`。语言提供了强大的自动等待语义：

1. **后缀等待**: `my_future.await` 是显式挂起当前协程并等待结果的标准方式。
2. **隐式等待**: 在异步上下文（如 `async { }`）中，直接调用返回 `Future` 的函数会自动应用 `.await` 语义。

## 组合子

`Future` 提供了丰富的组合子来处理复杂的异步逻辑：

- `fut.map(f)`: 当 Future 完成时，将其结果传递给函数 `f`。
- `fut.then(f)`: 当 Future 完成时，将其结果传递给返回另一个 Future 的函数 `f`（链式调用）。
- `Future.join(a, b)`: 等待两个 Future 同时完成，返回它们的元组结果。
- `Future.race(a, b)`: 等待两个 Future 中任意一个完成，返回最快完成的结果。

## 与协程的关系

Valkyrie 的 `Future` 与代数效应（Algebraic Effects）深度集成。当一个 `Future` 需要等待时，它会执行一个特殊的效应，由执行器（Executor）捕获并挂起当前任务，直到数据准备就绪。

---
**相关章节**:
- [Promise](./promise.md) - Future 的标准实现
- [异步块 (async)](./index.md#异步块async) - 创建 Future 的便捷语法

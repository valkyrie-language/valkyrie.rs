# Observable (可观察对象)

`Observable` 是响应式编程的核心，代表一个随时间推移而产生的一系列值。

## 核心理念

与只返回一个值的 `Future` 不同，`Observable` 可以：
- 产生 0 个或多个值。
- 在任何时候结束。
- 也可以产生错误并终止。

## 基本定义

```valkyrie
trait Observable⟨T⟩ {
    # 订阅该观察对象
    micro subscribe(self, observer: Observer⟨T⟩) -> Subscription
}
```

## 创建 Observable

你可以从多种来源创建可观察对象：

```valkyrie
# 从数组创建
let obs1 = Observable.from([1, 2, 3])

# 从定时器创建
let obs2 = Observable.interval(Duration.seconds(1))

# 从事件创建
let obs3 = Observable.from_event(button, "click")
```

## 响应式变换

Valkyrie 支持流式的操作符来处理这些值：

```valkyrie
let processed = obs1
    .filter { % % 2 == 0 }
    .map { "Value: {%" }
    .debounce(Duration.ms(300))
```

## 订阅与资源管理

当不再需要监听时，可以显式取消订阅：

```valkyrie
let sub = obs.subscribe {
    print("Received: {%}")
}

# 稍后取消
sub.unsubscribe()
```

## 与 Signal 的区别：事件 vs 状态

这是 Valkyrie 反应式架构中最核心的区分：

| 特性 | Observable (事件) | Signal (状态) |
| :--- | :--- | :--- |
| **代表含义** | “发生了什么” (动作序列) | “是什么” (当前数值) |
| **时效性** | 瞬时的、离散的 | 持续的、连续的 |
| **执行性质** | 惰性 (Lazy)：无人订阅不工作 | 热切 (Eager)：始终持有值 |
| **更新机制** | 异步推送 (Push) | 同步追踪 (Pull-Push 混合) |
| **适用场景** | 点击事件、Socket、定时器 | UI 绑定、配置、业务状态 |

---
**相关章节**:
- [Signal](./signal.md) - 代表当前状态的 Accessor / Settler 抽象
- [Stream](./stream.md) - 异步迭代器

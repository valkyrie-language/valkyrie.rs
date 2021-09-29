# 反应式编程

反应式编程是一种基于数据流和变化传播的编程范式。在 Valkyrie 中，反应式编程通过 Observable、Signal 和 Reactive 等抽象提供了强大的数据流处理能力。

## 核心原语：状态 (Signal) vs 事件 (Observable)

在 Valkyrie 中，反应式编程被划分为两个并行的核心概念，分别应对不同的场景：

### 1. 信号 (Signal) —— 状态管理
Signal 代表**当前状态**。它始终持有一个值，且任何对该值的读取都会自动建立依赖关系。

- **抽象**：通过 `Accessor` (读) 和 `Settler` (写) 进行角色化抽象。
- **特性**：同步更新、细粒度追踪、数据驱动。
- **适用**：UI 数据绑定、配置项、业务状态。

### 2. 观察对象 (Observable) —— 事件处理
Observable 代表**事件序列**。它描述了随时间推移发生的一系列动作。

- **抽象**：通过 `Observable` (源) 和 `Observer` (接收) 进行流式抽象。
- **特性**：异步传播、惰性执行（Lazy）、管道操作符丰富。
- **适用**：用户交互（点击/滚动）、网络请求流、定时器、WebSocket 数据流。

---

## 核心概念详解

### Signal (状态容器)

```valkyrie
# 基础用法：使用 raise 挂钩状态
let count = raise Signal(0)

# 衍生状态 (Accessor)
let doubled = Memo { count * 2 }

# 副作用
Effect {
    # 编译器自动追踪 count 和 doubled，无需 .value
    print("当前值: {count}, 二倍值: {doubled}")
}

count = 5 # 触发同步更新
```

### Observable (事件流)

```valkyrie
# 从事件源创建
let clicks = Observable.from_events("click")

# 链式操作符处理
let debounced_clicks = clicks
    .debounce(300ms)
    .map { "Clicked at: {now()}" }

# 订阅执行
debounced_clicks.subscribe {
    print(%)
}
```

## 基本操作符

### 转换操作符

```valkyrie
# map - 转换每个值
let numbers = Observable.from([1, 2, 3, 4, 5])
let squares = numbers.map { % * % }

# flat_map - 扁平化映射
let words = Observable.from(["hello", "world"])
let characters = words.flat_map {
    Observable.from(%.chars())
}

# scan - 累积操作
let numbers = Observable.from([1, 2, 3, 4, 5])
let running_sum = numbers.scan(0) { %1 + %2 }
# 输出: 1, 3, 6, 10, 15
```

### 过滤操作符

```valkyrie
# filter - 过滤值
let numbers = Observable.from([1, 2, 3, 4, 5, 6])
let evens = numbers.filter { % % 2 == 0 }

# take - 取前 N 个值
let first_three = numbers.take(3)

# skip - 跳过前 N 个值
let after_two = numbers.skip(2)

# distinct - 去重
let unique = Observable.from([1, 1, 2, 2, 3, 3]).distinct()
```

### 组合操作符

```valkyrie
# merge - 合并多个流
let stream1 = Observable.from([1, 3, 5])
let stream2 = Observable.from([2, 4, 6])
let merged = stream1.merge(stream2)

# zip - 配对组合
let names = Observable.from(["Alice", "Bob", "Charlie"])
let ages = Observable.from([25, 30, 35])
let people = names.zip(ages).map {
    Person { name: %1, age: %2 }
}

# combine_latest - 最新值组合
let temperature = Signal(20.0)
let humidity = Signal(60.0)
let comfort_index = temperature.combine_latest(humidity).map {
    calculate_comfort(%1, %2)
}
```

## 实际应用示例

### 用户界面反应式更新

```valkyrie
# 反应式 UI 组件
class CounterComponent {
    private count: Signal⟨i32⟩
    private increment_clicks: Observable⟨unit⟩
    private decrement_clicks: Observable⟨unit⟩
    
    micro CounterComponent() -> CounterComponent {
        let count = Signal(0)
        let increment_clicks = Observable.from_events("increment")
        let decrement_clicks = Observable.from_events("decrement")
        
        # 响应点击事件
        increment_clicks.subscribe {
            count.update { % + 1 }
        }
        
        decrement_clicks.subscribe {
            count.update { % - 1 }
        }
        
        CounterComponent {
            count,
            increment_clicks,
            decrement_clicks
        }
    }
    
    micro render(self) -> Widget {
        let count_text = self.count.map { "计数: {%" }
        
        VStack {
            Text(count_text)
            HStack {
                Button("增加").on_click(self.increment_clicks)
                Button("减少").on_click(self.decrement_clicks)
            }
        }
    }
}
```

### 数据流处理

```valkyrie
# 实时数据处理管道
class DataProcessor {
    micro process_sensor_data(sensor_stream: Observable⟨SensorReading⟩) -> Observable⟨ProcessedData⟩ {
        sensor_stream
            .filter { %.is_valid() }  # 过滤无效数据
            .map { %.normalize() }    # 标准化数据
            .buffer(5s)               # 5秒缓冲窗口
            .map { self.analyze_batch(%) } # 批量分析
            .filter { %.confidence > 0.8 } # 过滤低置信度结果
    }
    
    private micro analyze_batch(batch: [SensorReading]) -> ProcessedData {
        let average = batch.iter().map { %.value }.sum() / batch.length
        let variance = calculate_variance(batch)
        
        ProcessedData {
            timestamp: now(),
            average,
            variance,
            confidence: calculate_confidence(variance)
        }
    }
}

# 使用数据处理器
let processor = DataProcessor()
let sensor_stream = Observable.from_websocket("ws://sensor.example.com")
let processed_stream = processor.process_sensor_data(sensor_stream)

processed_stream.subscribe {
    print("处理结果: 平均值={%.average}, 置信度={%.confidence}")
    
    if %.confidence > 0.95 {
        alert_system.notify("高置信度数据: {%}")
    }
}
```

### 异步操作组合

```valkyrie
# 反应式 HTTP 客户端
class ReactiveHttpClient {
    micro get⟨T⟩(url: string) -> Observable⟨Result⟨T, HttpError⟩⟩ {
        Observable.create { observer ->
            async {
                try {
                    let response = http_get(url).await?
                    let data = response.json⟨T⟩().await?
                    observer.next(Fine { value: data })
                    observer.complete()
                }
                .catch {
                    case e:
                        observer.error(e)
                }
            }
        }
    }
    
    micro retry⟨T⟩(observable: Observable⟨Result⟨T, HttpError⟩⟩, max_retries: usize) -> Observable⟨Result⟨T, HttpError⟩⟩ {
        observable.catch_error {
            if max_retries > 0 {
                print("重试请求，剩余次数: {max_retries}")
                Observable.timer(1s)
                    .flat_map { self.retry(observable, max_retries - 1) }
            } else {
                Observable.error(%)
            }
        }
    }
}

# 使用示例
let client = ReactiveHttpClient()
let user_data = client.get⟨User⟩("https://api.example.com/user/123")
    .retry(3)  # 最多重试3次
    .timeout(10s)  # 10秒超时

user_data.subscribe {
    match % {
        case Fine { value: user }:
            print("用户信息: {user.name}")
        case Fail { error: err }:
            print("获取用户信息失败: {err}")
    }
}
```

## 错误处理

### 错误恢复策略

```valkyrie
# 错误处理操作符
trait ObservableErrorHandling⟨T⟩ {
    # 捕获错误并提供默认值
    micro catch_error⟨F⟩(self, handler: F) -> Observable⟨T⟩ where F: micro(any) -> Observable⟨T⟩
    
    # 重试操作
    micro retry(self, count: usize) -> Observable⟨T⟩
    
    # 超时处理
    micro timeout(self, duration: Duration) -> Observable⟨T⟩
}

# 实际使用
let unreliable_stream = fetch_data_stream()
    .catch_error {
        print("发生错误: {%}，使用缓存数据")
        Observable.from(cached_data)
    }
    .retry(3)
    .timeout(30s)

unreliable_stream.subscribe {
    process_data(%)
}
```

### 错误传播控制

```valkyrie
# 部分错误处理
let mixed_stream = Observable.from([1, 2, 3, 4, 5])
    .map {
        if % == 3 {
            raise ValueError { error: "无效值: 3" }
        }
        % * 2
    }
    .on_error_resume_next {
        print("跳过错误: {%}")
        Observable.empty()  # 跳过错误项
    }

mixed_stream.subscribe {
    print("处理值: {%}")
}  # 输出: 2, 4, 8, 10 (跳过了3)
```

## 资源管理

### 订阅生命周期

```valkyrie
# Subscription 管理
class Subscription {
    private is_disposed: bool
    private cleanup: micro() -> unit
    
    micro dispose(mut self) {
        if !self.is_disposed {
            self.cleanup()
            self.is_disposed = true
        }
    }
    
    micro is_disposed(self) -> bool {
        self.is_disposed
    }
}

# CompositeSubscription 用于管理多个订阅
class CompositeSubscription {
    private subscriptions: [Subscription]
    
    micro add(mut self, subscription: Subscription) {
        self.subscriptions.push(subscription)
    }
    
    micro dispose_all(mut self) {
        loop subscription in self.subscriptions {
            subscription.dispose()
        }
        self.subscriptions.clear()
    }
}

# 使用示例
let composite = CompositeSubscription()

let sub1 = timer_stream.subscribe { print("定时器触发") }
let sub2 = click_stream.subscribe { print("点击事件") }

composite.add(sub1)
composite.add(sub2)

# 在组件销毁时清理所有订阅
composite.dispose_all()
```

### 背压处理

```valkyrie
# 背压策略
unite BackpressureStrategy {
    Buffer { capacity: usize },
    Drop,
    Latest,
    Error
}

# 应用背压控制
let fast_producer = Observable.interval(1ms)  # 每毫秒产生数据
let slow_consumer = fast_producer
    .observe_on(Scheduler.computation())  # 在计算线程池处理
    .buffer(100)  # 缓冲100个元素
    .sample(1s)  # 每秒采样一次

slow_consumer.subscribe {
    print("处理批次，大小: {%.length}")
    # 慢速处理逻辑
    Thread.sleep(100ms)
}
```

## 调度器

### 线程调度

```valkyrie
# 调度器类型
unite Scheduler {
    CurrentThread,
    Computation,
    IO,
    NewThread,
    Trampoline
}

# 指定调度器
let data_stream = Observable.from_file("large_file.txt")
    .subscribe_on(Scheduler.IO())        # 在 I/O 线程读取文件
    .observe_on(Scheduler.Computation()) # 在计算线程处理数据
    .map { expensive_computation(%) }
    .observe_on(Scheduler.CurrentThread()) # 在主线程更新 UI

data_stream.subscribe {
    update_ui(%)  # UI 更新必须在主线程
}
```

## 测试支持

### 测试调度器

```valkyrie
# 测试用的虚拟时间调度器
class TestScheduler {
    private virtual_time: Duration
    private scheduled_actions: [(Duration, micro() -> unit)]
    
    micro advance_time_by(mut self, duration: Duration) {
        let target_time = self.virtual_time + duration
        
        while let (time, action)? = self.scheduled_actions.first() {
            if time <= target_time {
                self.virtual_time = time
                action()
                self.scheduled_actions.remove(0)
            } else {
                break
            }
        }
        
        self.virtual_time = target_time
    }
}

# 测试示例
@test
micro test_timer_observable() {
    let scheduler = TestScheduler()
    let timer = Observable.timer(5s, scheduler)
    let mut received_values = []
    
    timer.subscribe {
        received_values.push(%)
    }
    
    # 推进虚拟时间
    scheduler.advance_time_by(3s)
    assert_eq!(received_values.length, 0)  # 还没到时间
    
    scheduler.advance_time_by(3s)
    assert_eq!(received_values.length, 1)  # 定时器触发
}
```

## 最佳实践

### 1. 避免内存泄漏

```valkyrie
# 正确的订阅管理
class Component {
    private subscriptions: CompositeSubscription
    
    micro Component() -> Component {
        let subscriptions = CompositeSubscription()
        
        # 订阅数据流
        let sub = data_stream.subscribe {
            self.handle_data(%)
        }
        
        subscriptions.add(sub)
        
        Component { subscriptions }
    }
    
    micro destroy(mut self) {
        # 组件销毁时清理订阅
        self.subscriptions.dispose_all()
    }
}
```

### 2. 合理使用操作符

```valkyrie
# 优化操作符链
let optimized_stream = source_stream
    .filter { %is_valid() }     # 尽早过滤
    .take(1000)                   # 限制数量
    .map { %transform() }       # 转换数据
    .distinct()                   # 去重
    .buffer(1s)                  # 批处理

# 避免过长的操作符链
let intermediate = source_stream
    .filter { %is_valid() }
    .map { %normalize() }

let final_stream = intermediate
    .group_by { %category }
    .flat_map { %buffer(10) }
```

### 3. 错误边界

```valkyrie
# 设置错误边界防止整个流崩溃
let resilient_stream = risky_stream
    .map {
        try {
            process_item(%)
        }
        .catch {
            case ProcessingError { error: e }:
                log_error("处理失败: {e}")
                default_value()  # 提供默认值
            case _:
                raise  # 重新抛出严重错误
        }
    }
    .filter { %? }
```,old_str:

通过这些反应式编程模式，Valkyrie 提供了强大而灵活的数据流处理能力，使开发者能够构建响应式、可维护的应用程序。

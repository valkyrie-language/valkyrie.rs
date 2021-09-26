# 协程

Valkyrie 提供了强大的协程支持，通过 `yield` 关键字实现协作式多任务处理。协程允许函数在执行过程中暂停和恢复，非常适合处理异步操作和状态机。协程与生成器的主要区别在于协程更注重控制流的暂停和恢复，而不仅仅是产生值序列。

## 协程状态管理

### 协程生命周期

```valkyrie
# 协程状态枚举
union CoroutineState {
    Created,     # 已创建但未开始
    Running,     # 正在执行
    Suspended,   # 已暂停（yield）
    Completed,   # 已完成
    Error(Any)   # 发生错误
}

# 检查协程状态
micro example_coroutine() {
    print("开始执行")
    yield "第一个值"
    print("继续执行")
    yield "第二个值"
    print("执行完成")
}

let coro = example_coroutine()
print(coro.state())  # Created

let first = coro.next()
print(coro.state())  # Suspended
print(first)         # "第一个值"

let second = coro.next()
print(coro.state())  # Suspended
print(second)        # "第二个值"

coro.next()          # 完成执行
print(coro.state())  # Completed
```

### 协程控制

```valkyrie
# 手动控制协程执行
micro controlled_coroutine() {
    let mut state = "idle"
    loop {
        let command = yield state
        command.match {
            case "start": {
                state = "running"
            }
            case "pause": {
                state = "paused"
            }
            case "stop": {
                state = "stopped"
                break
            }
            case _: {
                state = "unknown_command"
            }
        }
    }
}

let coro = controlled_coroutine()
print(coro.next())           # "idle"
print(coro.send("start"))    # "running"
print(coro.send("pause"))    # "paused"
print(coro.send("stop"))     # "stopped"
```

## 异步协程

### 异步操作

```valkyrie
# 异步协程
async micro fetch_data(url: String) -> String {
    print("开始请求: ${ url }")
    let response = await http_get(url)
    yield "请求已发送"  # 可以在异步函数中使用 yield
    
    if response.status == 200 {
        yield "请求成功"
        response.body
    } else {
        raise "请求失败: ${ response.status }"
    }
}

# 使用异步协程
async micro main() {
    let fetcher = fetch_data("https://api.example.com/data")
    
    # 处理中间状态
    for status in fetcher {
        print("状态: ${ status }")
    }
    
    # 获取最终结果
    try {
        let data = await fetcher
        print("数据: ${ data }")
    } catch error {
        print("错误: ${ error }")
    }
}
```

### 并发协程

```valkyrie
# 并发执行多个协程
async micro concurrent_processing(items: [String]) {
    let tasks = items.map(async micro(item) {
        let result = await process_item(item)
        yield "处理完成: ${ item }"
        result
    })
    
    # 等待所有任务完成
    let results = await Promise.all(tasks)
    yield "所有任务完成"
    results
}

# 使用
async micro run_concurrent() {
    let processor = concurrent_processing(["item1", "item2", "item3"])
    
    for update in processor {
        print(update)
    }
    
    let final_results = await processor
    print("最终结果: ${ final_results }")
}
```

## 高级协程模式

### 状态机协程

```valkyrie
# 状态机实现
union State {
    Idle,
    Processing,
    Waiting,
    Complete
}

micro state_machine() {
    let mut state = State::Idle
    let mut data = null
    
    loop {
        state.match {
            case State::Idle: {
                yield "等待输入"
                data = yield_receive()  # 等待外部输入
                state = State::Processing
            }
            case State::Processing: {
                yield "处理中..."
                let result = process_data(data)
                if result.is_ok() {
                    state = State::Complete
                } else {
                    state = State::Waiting
                }
            }
            case State::Waiting: {
                yield "等待重试"
                sleep(1000)  # 等待1秒
                state = State::Processing
            }
            case State::Complete: {
                yield "处理完成"
                break
            }
        }
    }
}
```

### 协程池

```valkyrie
# 协程池管理
class CoroutinePool {
    coroutines: [Coroutine],
    max_size: i32,
    active_count: i32
    
    micro new(max_size: i32) -> Self {
        CoroutinePool {
            coroutines: [],
            max_size: max_size,
            active_count: 0
        }
    }
    
    micro spawn(task: micro() -> Any) -> bool {
        if self.active_count < self.max_size {
            let coro = Coroutine::new(task)
            self.coroutines.push(coro)
            self.active_count += 1
            true
        } else {
            false  # 池已满
        }
    }
    
    micro run_all() {
        while self.active_count > 0 {
            for coro in self.coroutines {
                if coro.state() == CoroutineState::Suspended {
                    let result = coro.resume()
                    yield "协程进度: ${ result }"
                    
                    if coro.state() == CoroutineState::Completed {
                        self.active_count -= 1
                    }
                }
            }
        }
        yield "所有协程完成"
    }
}
```

## 错误处理

### 协程异常处理

```valkyrie
# 协程中的异常处理
micro error_prone_generator() {
    try {
        yield "开始处理"
        
        let risky_operation = perform_risky_task()
        yield "风险操作完成"
        
        if risky_operation.is_error() {
            raise "操作失败"
        }
        
        yield "处理成功"
    } catch error {
        yield "发生错误: ${ error }"
        raise error  # 重新抛出异常
    }
}

# 使用带错误处理的协程
let gen = error_prone_generator()
try {
    for status in gen {
        print(status)
    }
} catch error {
    print("协程异常: ${ error }")
}
```

## 最佳实践

### 1. 协程设计原则

```valkyrie
# 保持协程简单和专注
micro good_generator(data: [String]) {
    for item in data {
        if item.is_valid() {
            yield item.process()  # 只做一件事
        }
    }
}

# 避免在协程中进行复杂的状态管理
# 不好的例子：
micro bad_generator() {
    let mut complex_state = ComplexState::new()
    # ... 复杂的状态逻辑
}
```

### 2. 资源管理

```valkyrie
# 确保资源正确释放
micro file_processor(filename: String) {
    let file = open_file(filename)
    try {
        while !file.eof() {
            let line = file.read_line()
            yield process_line(line)
        }
    }
    # 使用using确保文件关闭
    # using file = open_file(filename) { ... }
}
```

### 3. 性能考虑

```valkyrie
# 避免频繁的小yield
# 不好的例子：
micro inefficient_generator(data: [i32]) {
    for item in data {
        yield item  # 每个元素都yield
    }
}

# 好的例子：
micro efficient_generator(data: [i32]) {
    let mut batch = []
    for item in data {
        batch.push(item)
        if batch.len() >= 100 {
            yield batch  # 批量yield
            batch = []
        }
    }
    if !batch.is_empty() {
        yield batch  # 处理剩余项目
    }
}
```

### 4. 测试协程

```valkyrie
# 协程测试策略
micro test_generator() {
    let gen = count_up(3)
    
    # 测试生成的值
    @.assert_equal(gen.next(), Some(0))
@.assert_equal(gen.next(), Some(1))
@.assert_equal(gen.next(), Some(2))
@.assert_equal(gen.next(), None)
    
    # 测试状态
    @.assert_equal(gen.state(), CoroutineState::Completed)
}

# 异步协程测试
async micro test_async_generator() {
    let gen = async_data_processor()
    
    let first_result = await gen.next()
    assert!(first_result.is_some())
    
    let final_result = await gen.collect_all()
    @.assert_equal(final_result.len(), 5)
}
```

## 异步块：async { }

在异步函数之外或之内，都可以使用 `async { ... }` 创建一个可执行的异步任务对象（可视为一个延迟执行的协程/Promise）。该块内可以使用 `await` 等待其它异步结果。

```valkyrie
# 创建一个异步任务（不会立即阻塞当前线程）
let task = async {
    let user = await fetch_user(42)
    let posts = await fetch_posts(user.id)
    (user, posts)
}

# 任务可被组合
let composed = async {
    let (u, p) = await task
    render(u, p)
}
```

特点：
- `async { ... }` 是表达式，返回一个任务句柄，可被存入变量、作为参数传递或进一步组合。
- 任务不会自动阻塞当前线程，如何“运行”由下节的 run.* 与 `awake` 控制。

## 运行控制：run.await / run.block / run.awake / awake

为统一控制异步任务的执行与结果获取，约定任务句柄提供 `run` 控制器：

- `task.run.await`：在异步上下文中挂起当前协程，直至任务完成并返回结果。
- `task.run.block`：在同步上下文中阻塞当前线程直至任务完成，返回结果（适合 CLI、测试入口等）。
- `task.run.awake`：将任务调度到执行器上异步启动，但不等待结果，返回轻量句柄或 Unit。
- `awake <expr>`：语法糖，等价于对 `<expr>` 产生的任务执行“fire-and-forget”，即触发后忽略结果与错误（可用于日志、遥测等非关键路径）。

### 使用示例

```valkyrie
# 同步入口中（阻塞等待）
micro main() {
    let task = async {
        compute_heavy()  # 假设是计算密集操作
    }
    let result = task.run.block
    print("结果: ${ result }")
}
```

```valkyrie
# 异步上下文中（协作式等待）
async micro handle_request(id: i64) -> String {
    let task = async {
        let data = await fetch_by_id(id)
        transform(data)
    }
    let out = task.run.await
    out
}
```

```valkyrie
# 调度但不关心结果（fire-and-forget）
awake async {
    audit("user_login")
}

let bg = async { refresh_cache() }
_bg = bg.run.awake   # 触发后台刷新并忽略结果
```

### 与现有 await 语法的关系

- 在异步函数内，`await task` 可理解为 `task.run.await` 的简写，两者语义一致。
- 在同步函数内，若需要等待结果，使用 `task.run.block`；不等待则使用 `awake task` 或 `task.run.awake`。
- `awake` 的语义为 “fire then ignore”，适合非关键路径、可重试或可丢弃的任务。
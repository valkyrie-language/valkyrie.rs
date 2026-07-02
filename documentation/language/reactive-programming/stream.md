# Stream (流)

`Stream` 是异步版的迭代器（Iterator）。它允许你以异步的方式逐个处理一系列值。

## 核心概念

如果说 `Iterator` 是同步拉取数据，那么 `Stream` 就是异步拉取数据。

```valkyrie
trait Stream⟨T⟩ {
    # 异步获取下一个值
    # 返回 Fine(Some(T)) 表示有值
    # 返回 Fine(None) 表示流结束
    # 返回 Fail(E) 表示发生错误
    micro next(self) -> Result⟨T?, Error⟩
}
```

## 异步循环

处理 `Stream` 最自然的方式是使用 `for` 循环：

```valkyrie
let stream = get_user_stream()

loop user in stream {
    print("Processing user: {user.name}")
}
```

## 创建 Stream

### 使用生成器 (Generator)

你可以通过 `yield` 轻松创建流：

```valkyrie
micro count_up(n: i32) -> Stream⟨i32⟩ {
    loop i in 0..n {
        sleep(Duration.seconds(1)).await
        yield i
    }
}
```

### 组合子

与迭代器类似，`Stream` 也支持丰富的组合子：

```valkyrie
let processed = stream
    .filter { %is_active }
    .map_async { fetch_profile(%id) } # 异步映射
    .take(10)
```

## 与 Observable 的区别

- **Stream** 是**拉取型 (Pull-based)**: 消费者决定何时获取下一个值（通过 `next()`）。
- **Observable** 是**推送型 (Push-based)**: 生产者决定何时发送新值给订阅者。

`Stream` 非常适合处理分页数据、大文件读取或 WebSocket 消息。

---
**相关章节**:
- [生成器 (Generator)](../generator.md) - 创建流的工具
- [Observable](./observable.md) - 推送型异步原语

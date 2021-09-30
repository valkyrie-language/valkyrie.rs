# Stream (流)

`Stream` 是非同步版的迭代器（Iterator）。它允許你以非同步的方式逐個處理一系列值。

## 核心概念

如果說 `Iterator` 是同步拉取資料，那麼 `Stream` 就是非同步拉取資料。

```valkyrie
trait Stream⟨T⟩ {
    # 非同步獲取下一個值
    # 返回 Fine(Some(T)) 表示有值
    # 返回 Fine(None) 表示流結束
    # 返回 Fail(E) 表示發生錯誤
    micro next(self) -> Result⟨T?, Error⟩
}
```

## 非同步迴圈

處理 `Stream` 最自然的方式是使用 `for` 迴圈：

```valkyrie
let stream = get_user_stream()

loop user in stream {
    print("Processing user: {user.name}")
}
```

## 建立 Stream

### 使用產生器 (Generator)

你可以透過 `yield` 輕鬆建立流：

```valkyrie
micro count_up(n: i32) -> Stream⟨i32⟩ {
    loop i in 0..n {
        sleep(Duration.seconds(1)).await
        yield i
    }
}
```

### 組合子

與迭代器類似，`Stream` 也支援豐富的組合子：

```valkyrie
let processed = stream
    .filter { $is_active }
    .map_async { fetch_profile($id) } # 非同步映射
    .take(10)
```

## 與 Observable 的區別

- **Stream** 是**拉取型 (Pull-based)**: 消費者決定何時獲取下一個值（透過 `next()`）。
- **Observable** 是**推送型 (Push-based)**: 生產者決定何時發送新值給訂閱者。

`Stream` 非常適合處理分頁資料、大檔案讀取或 WebSocket 訊息。

---
**相關章節**:
- [產生器 (Generator)](../generator.md) - 建立流的工具
- [Observable](./observable.md) - 推送型非同步原語

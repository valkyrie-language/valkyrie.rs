# 反應式程式設計 (Reactive Programming)

## 非同步原語與型別系統

### Future：底層非同步原語

Valkyrie 的非同步系統基於 `Future` 作為底層原語。所有非同步操作最終都會產生 `Future` 實例：

- `Promise` - Future 的具體實現，用於非同步任務執行、值傳遞和組合
- **[Channel](./channel.md)** - 非同步任務間通訊的通道，其接收端是 Stream 的實現
- `async { ... }` 區塊 - 建立 Promise 實例的語法糖

```valkyrie
# 所有這些都是 Promise 實例（實作了 Future 介面）
let promise1: Promise⟨string⟩ = async { "hello" }
let promise2: Promise⟨i32⟩ = Promise.resolve(42)
let composed: Promise⟨string⟩ = async { promise1.await + promise2.await.to_string() }
```

## 非同步區塊：async { }

在非同步函式之外或之內，都可以使用 `async { ... }` 建立一個可執行的非同步 Promise 物件。該區塊內可以使用 `await` 等待其它非同步結果。

```valkyrie
# 建立一個非同步 Promise（不會立即阻塞當前執行緒）
let promise = async {
    let user = fetch_user(42).await?
    let posts = fetch_posts(user.id).await?
    (user, posts)
}

# Promise 可被組合
let composed = async {
    let (u, p) = promise.await?
    render(u, p)
}
```

特點：
- `async { ... }` 是運算式，返回一個 Promise 控制代碼，可被存入變數、作為參數傳遞或進一步組合。
- Promise 不會自動阻塞當前執行緒，如何「執行」由下節的 `.await`, `.block` 與 `.awake` 控制。

## 執行控制：.await / .block / .awake

為統一控制非同步 Promise 的執行與結果獲取，Promise 控制代碼提供以下控制操作：

- `promise.await`：在非同步上下文中掛起當前協程，直至 Promise 完成並返回結果。
- `promise.block`：在同步上下文中阻塞當前執行緒直至 Promise 完成，返回結果（適合 CLI、測試入口等）。
- `promise.awake`：將 Promise 排程到執行器上非同步啟動，但不等待結果，返回輕量控制代碼或 unit。

### 使用範例

```valkyrie
# 同步入口中（阻塞等待）
micro main() {
    let promise = async {
        compute_heavy()  # 假設是計算密集操作
    }
    let result = promise.block?
    print("結果: ${result}")
}
```

```valkyrie
# 非同步上下文中（協作式等待）
micro handle_request(id: i64) -> string {
    let promise = async {
        let data = fetch_by_id(id).await?
        transform(data)
    }
    let out = promise.await?
    out
}
```

```valkyrie
# 排程但不關心結果（fire-and-forget）
async {
    audit("user_login")
}.awake

let bg_promise = async { refresh_cache() }
bg_promise.awake   # 觸發後台重新整理並忽略結果
```

### 非同步方法呼叫規則

#### 執行控制語義

對於返回 Future 的方法呼叫（Promise 等 Future 實例）：

1. **自動執行規則**：
   - `obj.call_fut()` 本身就相當於 `obj.call_fut.await()`，會自動執行並等待結果
   - 括號可以省略：`obj.call_fut` 等價於 `obj.call_fut()`

2. **顯式控制語義**：
   - `obj.call_fut.await` - 顯式等待（與自動執行等價）
   - `obj.call_fut.awake` - fire-and-forget 語義，不等待結果
   - `obj.call_fut.block` - 阻塞等待（同步上下文中使用）

3. **函式繫結**：
   - `let f = obj.call_fut` - 不會自動執行，而是把返回 future 的函式繫結到 f
   - 靜態方法遵循同樣的規則

4. **錯誤處理**：
   - `?` 運算子用於 Result 型別的錯誤傳播，與 await 無效
   - `promise.await` 用於等待 Promise 完成
   - `promise.block` 用於阻塞等待 Promise 完成
   - 如果需要錯誤傳播，在整個運算式後使用：`promise.await?`

### Promise 進階用法

#### 1. 封裝回呼函式

Promise 可以用來封裝傳統的回呼式 API，將其轉換為非同步/await 模式：

```valkyrie
# 封裝回呼式 API
micro wrap_callback_api(url: string) -> Promise⟨string⟩ {
    Promise(micro(resolve, reject) {
        # 呼叫傳統的回呼式 API
        http_request_with_callback(url, micro(result) {
            if result.is_success() {
                resolve(result.data)
            } else {
                reject(result.error)
            }
        })
    })
}

# 使用封裝後的 Promise
micro fetch_data() {
    try {
        let data = wrap_callback_api("https://api.example.com").await?
        print("獲取資料: ${ data }")
    }
    .catch {
        case _:
            print("請求失敗: ${ error }")
    }
}
```

#### 2. Promise 取消功能

Promise 支援取消操作，這是 Future 基礎介面所不具備的功能：

```valkyrie
# 建立可取消的 Promise
let (promise, token) = Promise.cancellable {
    let mut count = 0
    loop {
        if $3() {
            $2("操作已取消")
            break
        }
        
        count += 1
        sleep(1000ms)  # 自動 await
        
        if count >= 10 {
            $1("操作完成")
            break
        }
    }
}

# 在另一個地方取消操作
sleep(5000ms) {
    token.cancel()
    print("已請求取消操作")
}

# 等待結果或取消
try {
    let result = promise.await?
    print("結果: ${ result }")
}
.catch {
    case _:
        print("操作被取消或失敗: ${ error }")
}
```

**注意**：Future 作為底層原語不提供 cancel 功能，只有 Promise 等具體實作才支援取消操作。

### Future 系統的統一性

由於 Promise 是 Future 的具體實作，所有非同步操作都透過 Promise 提供統一的執行控制介面：

```valkyrie
# 所有非同步操作都返回 Promise
let promise1 = async { compute() }
let promise2 = Promise.resolve(42)

# 統一的執行控制
promise1.await    # 等待 Promise 完成
promise2.await    # 等待 Promise 完成
promise1.awake    # fire-and-forget Promise
promise2.awake    # fire-and-forget Promise
```

Promise 作為 Future 的唯一實作，提供了完整的非同步功能，包括取消操作等進階特性。

### 與現有 await 語法的關係

- 在非同步函式內，Promise 方法呼叫通常會自動 await，不需要手動寫 .await
- 在同步函式內，若需要等待 Promise 結果，使用 `.block`；不等待則使用 `.awake`
- `awake` 的語義為 "fire then ignore"，適合非關鍵路徑、可重試或可丟棄的任務
- 所有 Promise 實例都遵循相同的執行語義

## 非同步流：Stream

### Stream 概念

當協程和產生器結合非同步操作時，需要一種特殊的 `Stream` 型別來處理非同步迭代。Stream 是非同步版本的迭代器，能夠處理非同步產生的值序列。

```valkyrie
# Stream 特徵定義
trait Stream⟨T⟩ {
    micro next(mut self) -> T?
    micro collect(self) -> [T]
    micro for_each⟨F⟩(self, f: F) where F: micro(T) -> unit
}
```

### 協程 Stream 化

協程可以轉換為 Stream，提供非同步迭代能力：

```valkyrie
# 協程轉 Stream
micro fetch_pages(base_url: string) -> Stream⟨string⟩ {
    let mut page = 1
    loop {
        let url = "${ base_url }?page=${ page }"
        let response = http_get(url).await?
        
        if response.is_empty() {
            break
        }
        
        yield response  # 非同步產生值
        page += 1
    }
}

# 使用 Stream
micro process_all_pages() {
    let page_stream = fetch_pages("https://api.example.com/data")
    
    # 非同步迭代
    loop page in page_stream {
        try {
            process_page(page).await?
        }
        .catch {
            case NetworkError(e):
                print("網路錯誤，跳過: ${ e }")
                continue
            case _:
                break  # 其他錯誤則停止處理
        }
    }
}
```

### Future Iterator vs Iterator Future

#### Future Iterator（推薦模式）

每次迭代返回一個 Future，適合處理獨立的非同步操作：

```valkyrie
# Future Iterator: Iterator⟨Promise⟨T⟩⟩
class FutureIterator⟨T⟩ {
    micro next(mut self) -> Promise⟨T⟩?
}

# 使用範例
micro process_urls(urls: [string]) -> FutureIterator⟨string⟩ {
    urls.into_iter().map {
        http_get($).await?
    }
}

# 並行處理
micro handle_concurrent() {
    let futures = process_urls(["url1", "url2", "url3"])
    let results = Promise.all(futures.collect()).await?
    
    loop result in results {
        print("結果: ${ result }")
    }
}
```

#### Iterator Future（特殊場景）
整個迭代過程是非同步的，適合有序相依的場景：

```valkyrie
# Iterator Future: Promise⟨Iterator⟨T⟩⟩
class IteratorFuture⟨T⟩ {
    micro resolve(self) -> Iterator⟨T⟩
}

# 使用範例：需要認證後才能獲取迭代器
micro authenticated_data() -> IteratorFuture⟨UserData⟩ {
    let token = authenticate().await?
    let data_iter = fetch_user_data(token).await?
    IteratorFuture(data_iter)
}
```

### Stream 錯誤處理策略

#### 1. 錯誤傳播（Fail Fast）

```valkyrie
# 遇到錯誤立即停止
micro strict_processing() {
    let stream = fetch_pages("https://api.example.com")
    
    loop page in stream {
        let processed = process_page(page).await?  # 錯誤會立即傳播
        save_result(processed).await?
    }
}
```

#### 2. 錯誤跳過（Continue on Error）

```valkyrie
# 跳過錯誤項，繼續處理
micro resilient_processing() {
    let stream = fetch_pages("https://api.example.com")
    
    loop page_result in stream {
        try {
            let page = page_result?  # 解包 Result
            let processed = process_page(page).await?
            save_result(processed).await?
        }
        .catch {
            case ProcessingError(e):
                log_error("處理失敗，跳過: ${ e }")
                continue
            case _:
                break  # 嚴重錯誤則停止
        }
    }
}
```

#### 3. 錯誤收集（Collect Errors）

```valkyrie
# 收集所有錯誤和成功結果
micro collect_all_results() {
    let stream = fetch_pages("https://api.example.com")
    let mut results = []
    let mut errors = []
    
    loop page_result in stream {
        match page_result {
            case Fine { value: page }:
                try {
                    let processed = process_page(page).await?
                    results.push(processed)
                }
                .catch {
                    case e:
                        errors.push(e)
                }
            case Fail { error: e }:
                errors.push(e)
        }
    }
    
    (results, errors)
}
```

### Stream 組合操作

```valkyrie
# Stream 的函式式操作
micro stream_operations() {
    let stream = fetch_pages("https://api.example.com")
    
    let processed_stream = stream
        .filter { !$is_empty() }  # 過濾空頁面
        .map { parse_json($).await? }  # 解析 JSON
        .take(10)  # 只取前10個
        .buffer(3)  # 緩衝3個並行請求
    
    let results = processed_stream.collect().await?
    print("處理完成: ${ results.length } 個結果")
}
```

### 背壓控制（Backpressure）

```valkyrie
# 控制 Stream 的生產速度
class BackpressureStream⟨T⟩ {
    private buffer_size: usize
    private current_buffer: [T]
    
    micro next_batch(mut self, batch_size: usize) -> [T] {
        # 實作背壓控制邏輯
        while self.current_buffer.length < batch_size {
            if let item? = self.source.next().await {
                self.current_buffer.push(item)
            } else {
                break
            }
        }
        
        self.current_buffer.drain(..batch_size.min(self.current_buffer.length))
    }
}
```

透過 Stream 抽象，協程和產生器能夠優雅地處理非同步迭代場景，提供靈活的錯誤處理策略和高效的資源管理。

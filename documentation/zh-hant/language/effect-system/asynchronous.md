# 非同步效應 (Async Effect)

在 Valkyrie 中，非同步程式設計不僅僅是一套語法糖，它是 **代數效應 (Algebraic Effects)** 的一種具體應用。借鑑了 C# `async2` (Runtime-handled Tasks) 的理念，Valkyrie 將非同步邏輯從編譯器層面下沉到了執行時層面。

## 核心理念：Await 也是一種效應

在傳統的非同步模型（如 Rust 或 C# 5.0）中，`async` 函數會被編譯器重寫為一個複雜的**狀態機**。

而在 Valkyrie 中：
- **`.await` 是一個效應 (Effect)**：當你呼叫 `.await` 時，它本質上是觸發（perform）了一個名為 `await` 的效應，並攜帶了一個 `Future` 或 `Task` 物件。
- **調度器是處理器 (Handler)**：執行時環境（如 Nyar VM）提供了一個頂層的效應處理器。它捕獲 `await` 效應，掛起當前的續體（Continuation），並將其交給非同步調度器（Executor）管理。

## 為什麼這樣做？

### 1. 消除函數著色 (Function Coloring)
由於非同步是通過效應系統實現的，非同步函數和同步函數在底層結構上高度統一。編譯器不需要為 `async` 生成完全不同的程式碼路徑，這使得非同步程式碼的效能和呼叫方式更接近同步程式碼。

### 2. 執行時託管 (Runtime-managed)
類似於 C# 的 `async2` 實驗，Valkyrie 將任務的掛起和恢復交給執行時直接處理。
- **零成本重寫**：位元組碼保持簡潔，沒有海量的狀態機跳轉。
- **動態優化**：執行時可以根據當前的 CPU 負載、I/O 狀態，動態決定是立即恢復續體還是將其放入等待佇列。

## 非同步原語：解耦編譯器與執行時

與許多語言不同，Valkyrie 的核心編譯器（HIR）並不包含 `Future`、`Promise` 或 `async/await` 的特殊語法樹結構。

- **函式庫定義而非內建**：`Future` 和 `Promise` 是標準函式庫中定義的普通 Trait 和 Class。
- **透明的非同步**：對於編譯器而言，`.await` 只是一個觸發效應的操作，`.block` 只是一個普通的屬性存取。
- **執行時調度**：這種設計借鑑了 C# `async2` 的核心理念。在 `async2` 中，執行時負責管理任務的暫停和恢復（通過輕量級續體），而不是讓編譯器為每個非同步函數生成沉重的狀態機程式碼。

### 帶來的優勢

1. **零成本抽象**：當非同步程式碼同步執行時，沒有狀態機切換的開銷。
2. **極簡的位元組碼**：Nyar VM 只需要處理 `Perform`、`CaptureCont` 和 `ResumeWith` 等通用指令，即可支援複雜的非同步邏輯。
3. **更強的互操作性**：由於非同步只是效應的一種，你可以輕鬆地在非同步程式碼中使用其他的代數效應（如依賴注入、異常處理等）。

## 執行機制

### 效應流轉過程

1. **觸發 (Perform)**: 執行到 `future.await` 時，虛擬機執行 `perform await(future)`。
2. **掛起 (Suspend)**: 虛擬機立即儲存當前函數的執行狀態（暫存器、棧幀、IP）。
3. **捕獲 (Catch)**: 效應冒泡到最近的非同步處理器（通常是 `AsyncRuntime`）。
4. **註冊 (Register)**: 調度器將該 `future` 註冊到 I/O 多路復用器（如 epoll/kqueue）或計時器中。
5. **恢復 (Resume)**: 當 `future` 完成時，調度器找到對應的續體，恢復虛擬機的執行狀態。

## 範例：底層視角

當你寫下：
```valkyrie
let data = socket.read().await
```

在底層，它等價於觸發了一個 `await` 效應：
```valkyrie
let data = effect.perform("await", socket.read())
```

如果是在一個沒有非同步處理器的同步環境中執行，這個效應會一直向上冒泡，直到被 `.block` 對應的處理器捕獲，或者導致程式因「未處理效應」而崩潰。這確保了非同步行为的可預測性和顯式性。

## 非同步語法

### 非同步塊：`async { }`

在 Valkyrie 中，你可以使用 `async { ... }` 建立一個非同步任務。需要注意的是，這並不是一種特殊的關鍵字語法，而是 **函數呼叫配合尾隨閉包** 的標準語法：
- `async` 是一個普通函數。
- `{ ... }` 是傳遞給該函數的尾隨閉包。
- 該函數執行後返回一個 `Promise` 實例。

```valkyrie
let p = async {
    let data = fetch_data().await
    process(data)
}
```

### 自動執行與顯式控制

為了簡化程式碼，Valkyrie 對返回 `Future` 的函數呼叫應用了以下規則：

1. **自動等待**：在非同步上下文中，`obj.call_fut()` 會被自動視為 `obj.call_fut().await`。
2. **後綴控制**：你可以顯式使用後綴來改變行為：
   - `.await`：顯式掛起並等待結果。
   - `.awake`：立即啟動任務並繼續執行（Fire and Forget）。
   - `.block`：在當前執行緒阻塞等待結果。

### 快捷函數：`go`

同樣地，`go { }` 也是一個接收閉包的快捷函數，它立即以 `.awake` 模式執行任務：

```valkyrie
# 使用 go 函數啟動背景任務
go {
    logger.info("Task started")
    do_some_work().await
    logger.info("Task finished")
}
```

其定義非常簡單，本質上是呼叫 `async` 並緊接著呼叫 `.awake`：
```valkyrie
micro go(body: () -> T) -> Promise⟨T⟩ {
    async(body).awake
}
```

## 執行控制 (Execution Control)

為了統一控制非同步任務的執行，Valkyrie 提供了三種核心的執行模式。從效應系統的視角來看，它們代表了不同的效應處理策略：

### 1. 非同步等待 (`.await`)
**語義**：掛起當前協程，直到結果就緒。
- **底層機制**：觸發 `await` 效應，由頂層非同步處理器捕獲並註冊到調度器。
- **使用場景**：絕大多數非同步程式設計場景。
```valkyrie
let data = fetch_api().await
```

### 2. 同步阻塞 (`.block`)
**語義**：阻塞當前物理執行緒，直到非同步任務完成。
- **底層機制**：這是一個特殊的效應處理器。它捕獲 `await` 效應後，並不將控制權交還給 OS 執行緒，而是原地啟動一個簡單的輪詢循環（Spin/Poll），直到獲取結果。
- **使用場景**：`main` 函數入口、單元測試、或者必須與同步遺留程式碼互動的邊界。
```valkyrie
micro main() {
    let result = run_async_task().block
}
```

### 3. 非同步啟動 (`.awake`)
**語義**：觸發並忽略 (Fire and Forget)。
- **底層機制**：它並不觸發 `await` 效應，而是直接向調度器發送一個「啟動」訊號。當前函數不需要掛起，立即繼續執行。
- **使用場景**：日誌記錄、遙測統計、背景快取刷新等非關鍵路徑任務。
```valkyrie
# 使用後綴語法啟動背景任務
refresh_cache().awake
```

## 非同步原語與型別系統

### Future：底層契約
`Future` 是非同步效應的載體。在底層，任何實現了 `poll` 效應的方法都可以被視為 `Future`。

### Promise：標準實現
`Promise` 是 `Future` 的具體實現，它與 JavaScript 的 Promise 具有零開銷的互操作性。在 Valkyrie 中，你可以手動控制 Promise 的解析：
```valkyrie
let (p, resolver) = Promise.pending⟨string⟩()
resolver.resolve("Done")
```

## 與協程的關係

非同步效應是協程的一種特化：
- **協程**：手動控制 `yield` 和 `resume`。
- **非同步效應**：由執行時調度器自動控制 `yield` (await) 和 `resume` (ready)。

---
**相關章節**:
- [協程](./coroutine.md) - 非同步效應的基礎
- [Channel (通道)](../reactive-programming/channel.md) - 任務間的通信與協作
- [Future (未來量)](../reactive-programming/future.md) - 非同步操作的承載體

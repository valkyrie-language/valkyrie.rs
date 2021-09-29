# Channel (通道)

`Channel` 是 Valkyrie 中用於併發任務間通訊的核心原語。它通常與 `go { }` 塊配合使用，實現 CSP (Communicating Sequential Processes) 程式設計模型。

## 從 go { } 到併發協作

當我們使用 `go { }` 啟動一個後台任務時，該任務便脫離了當前的執行流。為了在不同任務之間安全地傳遞資料，我們引入了 `Channel`。

```valkyrie
let (tx, rx) = Channel::new⟨i32⟩()

# 啟動生產者任務
go {
    loop i in 1..5 {
        tx.send(i).await
    }
    tx.close()
}

# 在主流程中消費資料
# rx 本身就是一個非同步流 (Stream)
loop item in rx {
    print("Received: ${item}")
}
```

## 核心特性

### 1. 生產者-消費者模型
`Channel::new()` 返回一對控制代碼：
- **Sender (tx)**: 用於發送資料。
- **Receiver (rx)**: 用於接收資料。`Receiver` 實作了 `Stream` 介面，因此可以像迭代器一樣在 `for` 迴圈中使用。

### 2. 非同步掛起
- **`tx.send(val).await`**: 如果通道緩衝區已滿，發送操作會觸發非同步掛起，直到有空間可用。
- **`rx.receive().await`**: 如果通道為空，接收操作會掛起，直到有新資料進入。

### 3. 多對多通訊
Valkyrie 的 `Channel` 支援：
- **MPMC (Multi-Producer, Multi-Consumer)**: 多個 `go` 任務可以共享同一個 `Sender` 或 `Receiver`。

## 通道拓撲模型

根據生產者和消費者的數量，Valkyrie 提供了多種通道模型以優化效能：

### 1. SPSC (Single-Producer, Single-Consumer)
最簡單的模型，一個發送者對應一個接收者。適用於簡單的流水線任務。
- **特點**：極高的效能，無鎖或低鎖實作。

### 2. MPSC (Multi-Producer, Single-Consumer)
最常見的模型，多個後台任務將結果彙總到一個中央處理器。
- **示例**：日誌收集系統，多個 `go` 任務向同一個 `Logger` 發送訊息。
```valkyrie
let (tx, rx) = Channel::mpsc()
go { tx.send("Task A done") }
go { tx.send("Task B done") }
```

### 3. MPMC (Multi-Producer, Multi-Consumer)
最通用的模型，多個任務發送，多個任務競爭處理。
- **場景**：工作池（Worker Pool）。
- **特點**：自動實現負載平衡，誰閒著誰處理。

## 通道類型

### 1. 無緩衝通道 (Rendezvous)
預設建立的通道通常是無緩衝的。發送者和接收者必須「同步」碰頭，資料才能傳遞。
```valkyrie
let (tx, rx) = Channel::new()
```

### 2. 有緩衝通道
可以指定緩衝區大小，發送者在緩衝區未滿時不會掛起。
```valkyrie
let (tx, rx) = Channel::buffered(10)
```

## 與 Stream 的關係

`Channel` 的接收端是 `Stream` 的一種動態實作。這意味著你可以對 `rx` 使用所有 `Stream` 的組合子：

```valkyrie
let doubled_stream = rx.map { $ * 2 }
                       .filter { $ > 10 }

doubled_stream.for_each { print($) }.await
```

## 設計選擇：Channel vs Async/Await

在編寫併發程式時，你可能會糾結是直接使用 `async/await` 還是引入 `Channel`。以下是建議的選擇標準：

### 什麼時候使用 Async/Await (Future)？
- **請求-回應模型**：當你呼叫一個函式並期望在未來某個時間點獲得**一個**明確的結果時。
- **簡單的相依鏈**：任務 A 必須在任務 B 之前完成，且 A 的輸出是 B 的輸入。
- **併發匯聚**：使用 `Future::join_all` 等工具同時等待多個任務的結果並彙總。
- **語義**：它更像是「會耗費時間的普通函式呼叫」。

### 什麼時候使用 Channel？
- **資料流與管道**：當資料是**連續產生**的，且需要流經多個處理步驟（如解析 -> 過濾 -> 儲存）。
- **生產者-消費者解耦**：當產生資料的速度與處理資料的速度不匹配，需要緩衝區來緩衝壓力（Backpressure）。
- **多對多協作**：多個任務共同處理一個任務池，或者多個任務向同一個中心任務回報狀態。
- **語義**：它更像是「不同元件之間的通訊線路」。

---
**相關章節**:
- [非同步效應 (Asynchronous)](../effect-system/asynchronous.md) - 了解 `go { }` 的底層原理
- [Stream (流)](./stream.md) - 如何處理連續的資料序列

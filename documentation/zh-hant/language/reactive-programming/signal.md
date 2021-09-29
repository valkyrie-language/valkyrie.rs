# 信號 (Signal)

`Signal` 是 Valkyrie 中用於狀態同步與響應式傳播的核心原語。它代表一個隨時間變化的「當前值」，並能自動追蹤其在響應式上下文（如 UI 渲染、副作用區塊）中的相依關係。

## 核心原語：狀態 vs 事件

在 Valkyrie 的響應式版圖中，`Signal` 與 `Observable` 承擔著不同的職責：

| 特性 | Signal (信號) | Observable (觀察物件) |
| :--- | :--- | :--- |
| **語義** | **狀態** (State)：它是什麼 | **事件** (Events)：發生了什麼 |
| **時效性** | 持續的數值，始終有當前值 | 離散的序列，代表瞬時行為 |
| **驅動方式** | 細粒度同步更新 | 非同步流式傳播 |
| **典型應用** | UI 資料繫結、業務狀態、配置項 | 點擊流、WebSocket、計時器 |

---

## 初始化與代數效應

Valkyrie 使用 `raise` 關鍵字初始化響應式容器。這表明該操作是一個**掛鉤 (Hooking)** 行為，它會向當前的執行上下文請求一個受代管的響應式狀態。

```valkyrie
# 初始化一個基礎信號 (State)
let count = raise Signal(0)

# 初始化一個非同步資源 (Resource)
let user = raise Resource { fetch_user(id) }
```

透過 `raise` 宣告，編譯器可以將該變數繫結到最近的響應式作用域（如 Widget），並確保在作用域銷毀時自動排程清理邏輯。

---

## 抽象介面：存取器 (Accessor) 與 建立器 (Settler)

為了實現嚴格的讀寫分離與多態性，Signal 體系基於兩個核心 Trait 建構：

### Accessor⟨T⟩ (存取器)
代表對狀態的**觀察權限**。任何接受 `Accessor` 的函式或元件都可以讀取該值，並自動將其註冊為響應式相依。
- `property value: T { get }`

### Settler⟨T⟩ (建立器)
繼承自 `Accessor`，代表對狀態的**修改權限**。
- `property value: T { get, set }`

---

## 狀態容器矩陣

| 容器類型 | 實作介面 | 生命周期 | 說明 |
| :--- | :--- | :--- | :--- |
| **Signal** | `Settler` | 作用域持久 | 基礎的可變狀態源。 |
| **Memo** | `Accessor` | 隨相依自動銷毀 | 基於其他信號生成的衍生計算值。 |
| **Resource** | `Accessor` | 受代管非同步 | 封裝了非同步操作及其載入/錯誤狀態。 |
| **Bridge** | `Settler` | 跨端同步 | 由框架代管，實現前後端狀態的自動一致性。 |

---

## 基本用法

### 1. 自動相依追蹤
得益於編譯器的深度整合，使用者可以直接操作信號變數，而無需顯式呼叫解包方法。

```valkyrie
let count = raise Signal(0)
let doubled = Memo { count * 2 }

# 在 Effect 區塊中讀取，自動建立訂閱
Effect {
    print("Value: ${count}, Double: ${doubled}")
}

# 像普通變數一樣賦值，觸發細粒度更新
count = count + 1
```

### 2. 衍生狀態 (Memo)
`Memo` 用於建立唯讀的計算屬性，它僅在相依項發生變化時才會重新計算。

```valkyrie
let list = raise Signal([1, 2, 3])
let sum = Memo { list.iter().sum() }
```

---

## 進階特性

### 批處理 (Batching)
在高效能場景下，可以使用批處理合併多次修改，使訂閱者僅在最後觸發一次更新。

```valkyrie
std::reactive::batch {
    # 無論迴圈多少次，相依該信號的 Effect 只執行一次
    loop i in 1..100 {
        count = i
    }
}
```

### 跨端同步 (Bridge)

`Bridge` 是 Valkyrie 專為分散式環境（如前後端分離、多程序協作）設計的增強型信號。它打破了記憶體邊界，使得狀態可以在不同的執行環境之間自動保持同步。

#### 核心原理
`Bridge` 本質上是一個具備**透明傳輸能力**的 `Settler`。當狀態在一端發生變更時，框架會執行以下流程：
1. **變更捕獲**：利用編譯器生成的追蹤資訊，識別受影響的資料片段。
2. **差分序列化**：僅將變更的部分（Delta）序列化為緊湊的二進位格式（如 ProtoBuf 或內部格式）。
3. **傳輸協定**：透過底層抽象的 `Transport` 介面（支援 WebSocket, gRPC, 或 SharedMemory）發送到對端。
4. **狀態對齊**：對端接收到變更後，原子化地更新本地副本，並觸發本地的響應式相依。

#### 使用場景
- **全端即時同步**：在後端修改 `user_score`，前端 UI 即時跳動。
- **協同編輯**：多使用者共享同一個 `Bridge` 容器，實現類似 Google Docs 的即時回饋。
- **分散式配置**：在配置中心修改信號，所有微服務實例自動感知。

#### 程式碼範例
```valkyrie
# 在後端定義並匯出
export let server_status = raise Bridge("status", "Initializing")

# 在前端透過識別碼掛鉤
let status = raise Bridge("status")

Effect {
    print("伺服器狀態即時感知: ${status}")
}
```

#### 衝突處理與一致性
`Bridge` 預設遵循**最終一致性 (Eventual Consistency)**。在極高效能要求的場景下，可以配置不同的同步策略：
- `SyncStrategy::Eager`：立即發送，適用於即時 UI。
- `SyncStrategy::Debounced(ms)`：防抖發送，合併頻繁修改，節省頻寬。
- `SyncStrategy::Reliable`：確保送達，適用於關鍵業務邏輯。

---

## 效能與生命週期

- **細粒度更新**：Signal 系統建構了精確的拓撲相依圖，更新時僅觸及真正受影響的節點，避免了昂貴的全域比對（Diff）。
- **確定性終結**：透過 AIFD 模型，當響應式作用域結束時，所有相關的 Signal 節點及其訂閱關係都會被編譯器插入的程式碼自動清理，確保無記憶體洩漏。

# Observable (可觀察物件)

`Observable` 是響應式程式設計的核心，代表一個隨時間推移而產生的一系列值。

## 核心理念

與只返回一個值的 `Future` 不同，`Observable` 可以：
- 產生 0 個或多個值。
- 在任何時候結束。
- 也可以產生錯誤並終止。

## 基本定義

```valkyrie
trait Observable⟨T⟩ {
    # 訂閱該觀察物件
    micro subscribe(self, observer: Observer⟨T⟩) -> Subscription
}
```

## 建立 Observable

你可以從多種來源建立可觀察物件：

```valkyrie
# 從陣列建立
let obs1 = Observable.from([1, 2, 3])

# 從計時器建立
let obs2 = Observable.interval(Duration.seconds(1))

# 從事件建立
let obs3 = Observable.from_event(button, "click")
```

## 響應式變換

Valkyrie 支援流式的運算子來處理這些值：

```valkyrie
let processed = obs1
    .filter { $ % 2 == 0 }
    .map { value -> "Value: {value}" }
    .debounce(Duration.ms(300))
```

## 訂閱與資源管理

當不再需要監聽時，可以顯式取消訂閱：

```valkyrie
let sub = obs.subscribe { value ->
    print("Received: {value}")
}

# 稍後取消
sub.unsubscribe()
```

## 與 Signal 的區別：事件 vs 狀態

這是 Valkyrie 響應式架構中最核心的區分：

| 特性 | Observable (事件) | Signal (狀態) |
| :--- | :--- | :--- |
| **代表含義** | 「發生了什麼」 (動作序列) | 「是什麼」 (當前數值) |
| **時效性** | 瞬時的、離散的 | 持續的、連續的 |
| **執行性質** | 惰性 (Lazy)：無人訂閱不工作 | 熱切 (Eager)：始終持有值 |
| **更新機制** | 非同步推送 (Push) | 同步追蹤 (Pull-Push 混合) |
| **適用場景** | 點擊事件、Socket、計時器 | UI 繫結、配置、業務狀態 |

---
**相關章節**:
- [Signal](./signal.md) - 代表當前狀態的 Accessor / Settler 抽象
- [Stream](./stream.md) - 非同步迭代器

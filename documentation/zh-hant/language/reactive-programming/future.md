# Future (未來量)

`Future` 是 Valkyrie 非同步程式設計模型中最基礎的特徵（Trait）。它代表一個在未來某個時間點才會變為可用的值。

## 核心概念

`Future` 描述了一個尚未完成的運算。它不代表運算本身，而是代表對運算結果的引用。

### Future 特徵定義

在底層，`Future` 類似於如下定義：

```valkyrie
trait Future⟨T⟩ {
    # 嘗試輪詢 Future 的狀態
    # 如果已完成，返回 Fine(T)
    # 如果未完成，返回 Pending
    micro poll(self, cx: Context) -> Poll⟨T⟩
}
```

## 自動等待語義

在 Valkyrie 中，絕大多數情況下你不需要手動呼叫 `poll`。語言提供了強大的自動等待語義：

1. **後綴等待**: `my_future.await` 是顯式掛起當前協程並等待結果的標準方式。
2. **隱式等待**: 在非同步上下文（如 `async { }`）中，直接呼叫返回 `Future` 的函式會自動應用 `.await` 語義。

## 組合子

`Future` 提供了豐富的組合子來處理複雜的非同步邏輯：

- `fut.map(f)`: 當 Future 完成時，將其結果傳遞給函式 `f`。
- `fut.then(f)`: 當 Future 完成時，將其結果傳遞給返回另一個 Future 的函式 `f`（鏈式呼叫）。
- `Future.join(a, b)`: 等待兩個 Future 同時完成，返回它們的元組結果。
- `Future.race(a, b)`: 等待兩個 Future 中任意一個完成，返回最快完成的結果。

## 與協程的關係

Valkyrie 的 `Future` 與代數效應（Algebraic Effects）深度整合。當一個 `Future` 需要等待時，它會執行一個特殊的效應，由執行器（Executor）捕獲並掛起當前任務，直到資料準備就緒。

---
**相關章節**:
- [Promise](./promise.md) - Future 的標準實作
- [非同步區塊 (async)](./index.md#非同步區塊async) - 建立 Future 的便捷語法

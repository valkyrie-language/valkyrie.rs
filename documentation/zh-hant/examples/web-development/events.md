# 事件 (Events)

在 Valkyrie 中，事件是實現組件間解耦和響應式編程的關鍵機制。Valkyrie 提供了兩種類型的事件聲明，以區分其預期行為和消費者數量：單點事件 (`event`) 和廣播事件 (`events`)。

## 1. 單點事件 (Single-Point Event)

單點事件是指那些**預期只有一個或少數特定訂閱者**的事件。这類事件通常用於回調、特定狀態的唯一響應或生命週期事件。

### 聲明語法

使用 `event` 關鍵字聲明單點事件：

```valkyrie
event event_name(parameter1: Type1, parameter2: Type2, ...)
```

### 範例

```valkyrie
class AsyncOperation {
    // 定義一個單點事件，用於通知操作完成
    event on_completed(result: Result<String, Error>)

    micro start(mut self) {
        print("異步操作開始...")
        // 模擬異步操作完成並觸發事件
        // 假設這裡有一個內部機制來調用 on_completed
        // 例如：self.on_completed(Fine { value: "操作成功" })
    }
}

class MyProcessor {
    micro process(self) {
        let op = AsyncOperation::new()
        // 訂閱單點事件
        op.on_completed = micro(result) {
            match result {
                case Fine { value }: print("操作完成: ${value}")
                case Fail { error }: print("操作失敗: ${error}")
            }
        }
        op.start()
    }
}
```

### 改善

*   **明確的意圖**: 開發者一眼就能看出 `on_completed` 是一個單點事件，不應該有多個消費者。
*   **編譯時/運行時檢查**: 語言可以強制執行“單點”的約束，防止意外地附加多個處理器。
*   **簡化 API**: 訂閱和取消訂閱的 API 可能更簡單，例如直接賦值即可替換舊的處理器。

## 2. 廣播事件 (Broadcast Event)

廣播事件是指那些**預期可以有零個、一個或多個訂閱者**的事件。这類事件通常用於 UI 事件、系統級通知或領域事件。

### 聲明語法

使用 `events` 關鍵字聲明廣播事件：

```valkyrie
events event_name(parameter1: Type1, parameter2: Type2, ...)
```

### 範例

```valkyrie
class Button {
    // 定義一個廣播事件，用於通知點击
    events clicked(sender: &Button, args: &ClickEventArgs)

    micro simulate_click(mut self) {
        print("按鈕被點击了！")
        // 觸發所有訂閱者
        // 假設語言提供內置的觸發機制，例如：
        self.clicked.trigger(self, ClickEventArgs::new())
    }
}

class Logger {
    micro log_click(self, sender: &Button, args: &ClickEventArgs) {
        print("日誌：按鈕 '${sender.id}' 被點击。")
    }
}

class Analytics {
    micro track_click(self, sender: &Button, args: &ClickEventArgs) {
        print("分析：記錄按鈕 '${sender.id}' 的點击事件。")
    }
}

micro main() {
    let mut my_button = Button { id: "submit_btn" }
    let logger = Logger::new()
    let analytics = Analytics::new()

    // 訂閱廣播事件
    my_button.clicked.add(micro(s, a) { logger.log_click(s, a) })
    my_button.clicked.add(micro(s, a) { analytics.track_click(s, a) })

    my_button.simulate_click()
}
```

### 改善

*   **明確的意圖**: 清楚地表明這是一個可以被多個消費者訂閱的事件。
*   **內置的訂閱管理**: 語言自動處理訂閱者列表的添加 (`add`)、移除 (`remove`) 和遍歷調用。
*   **標準化的 API**: 提供統一的訂閱/取消訂閱接口，提高代碼的一致性和可讀性。

## 總結

在 Valkyrie 中引入 `event` 和 `events` 關鍵詞來區分單點事件和廣播事件，將帶來以下顯著改善：

1.  **提高代碼清晰度**: 開發者通過關鍵詞就能立即理解事件的預期行為和消費者數量。
2.  **增強類型安全和約束**: 語言可以在編譯時或運行時強制執行事件的“單點”或“多點”約束。
3.  **簡化開發**: 語言內置的事件管理機制將大大減少樣板代碼。
4.  **優化性能**: 针對單點事件，語言可以進行更激進的優化。
5.  **促進良好設計**: 鼓勵開發者在設計事件時就考慮其傳播範圍和消費者數量。
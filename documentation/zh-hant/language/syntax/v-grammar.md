# 原生 UI 語法 (V-Grammar)

**V-Grammar** 是 Valkyrie 中用於構建聲明式界面（UI）的核心語法。它允許開發者以極其自然、流暢的方式描述嵌套的組件結構，而無需繁瑣的方法鏈或外部模板語言。

V-Grammar 針對不同的應用場景提供了兩個特化版本：**跨平台通用開發 (Cross-platform)** 與 **Web HTML 特化**。

---

## 1. 跨平台通用開發 (Cross-platform)

這一風格主要用於跨平台 UI 開發（如原生應用、桌面端）。它強調組件的高度抽象和布局容器的概念。

### 核心特性
- **布局容器**：使用 `Column`, `Row`, `ZStack` 等語義化容器。
- **屬性配置**：通過 ApplyBlock 進行直觀的欄位賦值或方法調用。
- **類型安全**：每個組件都是一個具體的類或結構體。

```valkyrie
# 通用 UI 示例
Column {
    spacing = 10
    alignment = .center
    
    Image("logo.png") {
        width = 100
        height = 100
    }
    
    Text("歡迎回來") {
        font = .headline
        color = .blue
    }
    
    Button("進入控制台") {
        on_click = micro() { navigate_to("/dash") }
    }
}
```

---

## 2. HTML 特化風格

當 Valkyrie 用於 Web 開發時，V-Grammar 提供了一套直接映射到標準 HTML 標籤的特化語法。這一版本旨在消除 Web 開發者的遷移成本，同時保留 ApplyBlock 的邏輯能力。

### 核心特性
- **標籤映射**：直接使用 `div`, `span`, `section`, `a` 等小寫 HTML 標籤。
- **屬性簡化**：支援標準的 HTML 屬性名。
- **混合渲染**：可以直接在標籤塊內混寫文本字面量和子標籤。

```valkyrie
# HTML 特化示例
div {
    class = "container mx-auto"
    
    h1 { "儀表板" }
    
    section {
        id = "stats-grid"
        class = "grid grid-cols-3 gap-4"
        
        loop stat in dashboard_stats {
            div {
                class = "card p-4 shadow"
                span { class = "label"; stat.title }
                span { class = "value"; stat.value }
            }
        }
    }
    
    footer {
        p { "© 2024 Valkyrie Project" }
    }
}
```

## 3. 交互處理：極致靈活的事件綁定

V-Grammar 繼承了 ApplyBlock 的靈活性，允許開發者根據語義需求選擇最合適的事件綁定方式。

### 核心特性：多範式綁定
- **賦值/覆蓋 (`=`)**：直接替換原有的處理邏輯。
- **追加 (`+=` / `.append`)**：在原有邏輯後添加新的處理函數。
- **顯式設置 (`set`)**：語義化地設置處理邏輯。
- **函數式簡寫**：像調用方法一樣直接定義處理塊。

```valkyrie
Button("交互演示") {
    # 1. 函數式簡寫 (最常用)
    on_click {
        println("直接觸發")
    }

    # 2. 賦值語法
    on_hover = micro() { is_hovered = true }

    # 3. 運算符重載 (追加邏輯)
    on_click += micro() {
        log_event("button_clicked")
    }

    # 4. 顯式方法調用
    on_close.set(micro() { cleanup() })
    on_scroll.append(micro(e) { update_position(e) })
}
```

---

## 4. 語法基礎：應用塊 (ApplyBlock)

無論是哪種風格，V-Grammar 的底層統一基於 **[應用塊 (ApplyBlock)](./braces.md)**。

ApplyBlock 在 V-Grammar 中統一了四種核心操作：
1. **欄位賦值**：`class = "..."` 或 `spacing = 10`。
2. **事件綁定**：如上所述的多種靈活語法（`=`、`+=`、`{}` 等）。
3. **方法調用**：`.modifier()` 風格的鏈式調用。
4. **子節點注入**：直接在塊內編寫另一個組件/標籤。

具體的語義解釋由後續的類型系統決定。例如，如果 `div` 被標記為 `HtmlElement`，塊內的嵌套調用將被自動解釋為 `appendChild`。

---

## 4. 動態 UI：原生邏輯控制

V-Grammar 不需要 `v-for` 或 `ng-if` 等特殊指令，它直接使用 Valkyrie 的原生控制流：

- **條件渲染**：使用標準的 `if-else`。
- **列表循環**：使用標準的 `loop-in`。
- **複雜狀態**：使用標準的 `match` 模式匹配。

這些控制流在兩種風格中完全一致，確保了邏輯層的高度可復用。

---

## 5. 語法特性總結

| 特性 | 跨平台通用開發 | Web HTML 特化 |
| :--- | :--- | :--- |
| **主要目標** | 原生應用 / 桌面端 | Web 頁面 / SSR |
| **標籤風格** | 大寫字母 (Component) | 小寫字母 (Tag) |
| **嵌套方式** | `Child { ... }` | `tag { ... }` |
| **適用環境** | 原生渲染引擎 | 瀏覽器 / DOM |

---

## 6. 魔法的真相：結構優先

V-Grammar 的強大之處在於它遵循了 **[應用塊 (ApplyBlock)](./braces.md)** 的核心設計原則：**先解析結構，後校驗語義**。

1. **結構化解析**：編譯器首先將塊解析為一個通用的「語句流」。
2. **延遲綁定**：直到類型檢查階段，編譯器才會根據調用者（是 `Column` 還是 `div`）來決定塊內的語句是屬性設置還是 DOM 操作。
3. **零開銷抽象**：這種設計使得 UI 描述在運行時可以被編譯為極其高效的直接操作，避免了虛擬 DOM 對比或模板解析的開銷。

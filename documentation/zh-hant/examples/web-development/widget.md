# 界面組件 (Widget)：一切的基石

在 Valkyrie 的世界裡，UI 並非虛幻的標記，而是由真實的**對象**構建而成的。`widget` 是 Valkyrie 中專門為 UI 設計的類類型，它是整個 UI 系統的邏輯原點。

## 1. 組件的對象本質

每一個界面元素在底層都是一個 `widget` 實例。它通過構造函數初始化，並通過方法鏈配置屬性。

```valkyrie
widget Button {
    text: utf8,
    enabled: bool,
    style: ButtonStyle,
    on_click: Option<micro() -> ()>,
    
    new(text: utf8) {
        self.text = text
        self.enabled = true
        self.style = .default
        self.on_click = None
    }
    
    render(self) -> Element {
        Element::button()
            .text(self.text)
            .enabled(self.enabled)
            .style(self.style)
            .on_click(self.on_click)
    }
    
    # 鏈式配置方法
    on_click(mut self, handler: micro() -> ()) -> Self {
        self.on_click = Some(handler)
        self
    }
}
```

## 2. 佈局與組合模式

Widget 系統使用**組合模式**來管理嵌套結構。

### 彈性佈局 (FlexBox)
```valkyrie
widget FlexBox {
    children: [FlexChild],
    direction: FlexDirection,
    
    new(direction: FlexDirection = .Row) {
        self.children = []
        self.direction = direction
    }
    
    render(self) -> Element {
        let mut element = Element::div().display(.Flex).flex_direction(self.direction)
        loop child in self.children {
            element = element.child(child.widget.render())
        }
        element
    }
    
    add_child(mut self, widget: Box<Widget>) -> Self {
        self.children.push(FlexChild { widget })
        self
    }
}
```

## 3. 狀態管理與交互

組件的屬性即是它的狀態。通過修改字段並請求更新，可以實現響應式交互。

```valkyrie
widget Counter {
    count: i32,
    
    new(initial: i32 = 0) { self.count = initial }
    
    render(self) -> Element {
        FlexBox::new(.Row)
            .add_child(Box::new(Button::new("-").on_click { self.decrement() }))
            .add_child(Box::new(Text::new(self.count.to_string())))
            .add_child(Box::new(Button::new("+").on_click { self.increment() }))
            .render()
    }
    
    increment(mut self) { self.count += 1; self.request_update() }
    decrement(mut self) { self.count -= 1; self.request_update() }
}
```

## 4. 聲明式構建 (V-Grammar)

雖然可以通過方法鏈來配置 Widget，但 Valkyrie 提供了更自然的 [應用塊 (ApplyBlock)](../../language/syntax/v-grammar.md) 語法。這種方式通常被称為 **V-Grammar**，它允許以聲明式的方式構建複雜的組件樹。

```valkyrie
FlexBox {
    direction = .Column
    
    Button("提交") {
        on_click = micro() { print("Submitted") }
    }
    
    Text { "請輸入您的姓名" }
}
```

## 5. 高級特性

### 響應式設計 (Responsive)
利用組件的生命週期和方法，可以輕鬆實現媒體查詢：
```valkyrie
widget Responsive {
    current_breakpoint: utf8,
    children: {utf8: Box<Widget>},
    
    # 根據窗口大小更新 current_breakpoint 並觸發 render()
    update_width(mut self, width: f32) { ... }
}
```

### 虛擬滾動與模態框
這些高級組件同樣基於 Widget 類實現，將複雜的 DOM 操作封裝在對象方法中。

---

## 魔法的真相：對象層級即 UI 結構

為什么 Widget 系統讓人感到自然？因為它回歸了軟件工程最經典的**組合模式 (Composite Pattern)**。

當你在屏幕上看到一個複雜的界面時，你在內存中擁有的是一顆完整、透明的對象樹。這種設計保證了 UI 系統具有極高的可預測性：
- **沒有黑盒**：每一個組件都是一個可以被調試、擴展的普通對象。
- **配置即方法**：流式接口 (Fluent Interface) 讓對象配置讀起來像自然語言。
- **單一真理來源**：UI 的視覺嵌套直接映射為對象的組合嵌套。

這種“對象化”的基石，為上層的語法簡化（如 V-Grammar 和 X-Grammar）提供了最堅實的邏輯保障。

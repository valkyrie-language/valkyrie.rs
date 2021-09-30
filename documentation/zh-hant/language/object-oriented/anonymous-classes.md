# 匿名類別

匿名類別是一種臨時定義的類別，通常用於一次性使用的場景，如事件處理、回調函數或臨時資料結構。

## 基本語法

### 定義匿名類別

```valkyrie
# 基本匿名類別
let handler = class {
    name: string = "Handler"
    
    micro handle(self, event: Event) {
        print("Handling event: {event.type}")
    }
}

# 使用匿名類別
let h = handler {}
h.handle(Event { type: "click" })
```

### 實現介面的匿名類別

```valkyrie
# 定義介面
trait ClickHandler {
    micro on_click(self, x: i32, y: i32)
    micro on_double_click(self, x: i32, y: i32)
}

# 建立實現介面的匿名類別
let button_handler = impl ClickHandler {
    micro on_click(self, x, y) {
        print("Clicked at ({x}, {y})")
    }
    
    micro on_double_click(self, x, y) {
        print("Double-clicked at ({x}, {y})")
    }
}
```

### 捕獲外部變數

匿名類別可以捕獲其定義環境中的變數：

```valkyrie
micro create_counter(start: i32) {
    let mut count = start
    
    impl Counter {
        micro increment(mut self) -> i32 {
            count += 1
            count
        }
        
        micro get_value(self) -> i32 {
            count
        }
    }
}
```

## 使用場景

### 事件處理

```valkyrie
# 事件處理器
button.add_click_handler(impl ClickHandler {
    micro on_click(self, x, y) {
        print("Button clicked!")
    }
})

# 多事件處理
form.set_handlers(impl FormHandlers {
    micro on_submit(self, data) {
        submit_to_server(data)
    }
    
    micro on_cancel(self) {
        form.reset()
    }
    
    micro on_error(self, error) {
        show_error_dialog(error)
    }
})
```

### 回調函數

```valkyrie
# 非同步回調
fetch_data(url, impl AsyncCallback⟨Data⟩ {
    micro on_success(self, data) {
        update_ui(data)
    }
    
    micro on_error(self, error) {
        show_error(error)
    }
})
```

### 比較器

```valkyrie
# 自定義排序
let sorted_users = users.sort_by(impl Comparator⟨User⟩ {
    micro compare(self, a, b) -> i32 {
        a.name.compare(b.name)
    }
})
```

### 迭代器實現

```valkyrie
# 自定義迭代器
let custom_iter = impl Iterator⟨i32⟩ {
    mut current: i32 = 0
    max: i32 = 10
    
    micro has_next(self) -> bool {
        self.current < self.max
    }
    
    micro next(mut self) -> i32 {
        let value = self.current
        self.current += 1
        value
    }
}
```

## 最佳實踐

### 1. 保持簡潔

匿名類別應該保持簡潔，複雜邏輯應該提取到命名類別中：

```valkyrie
# 好的實踐：簡單的事件處理
button.on_click(impl ClickHandler {
    micro on_click(self, x, y) {
        submit_form()
    }
})

# 避免：複雜的業務邏輯
# 這種情況應該使用命名類別
```

### 2. 明確型別

當匿名類別需要明確型別時，使用型別註解：

```valkyrie
let handler: ClickHandler = impl ClickHandler {
    micro on_click(self, x, y) {
        # ...
    }
}
```

### 3. 重用考慮

如果匿名類別需要多次使用，考慮將其提取為命名類別：

```valkyrie
# 如果多個地方使用相同邏輯
class DefaultClickHandler {
    micro on_click(self, x, y) {
        # 共用邏輯
    }
}

let handler1 = DefaultClickHandler {}
let handler2 = DefaultClickHandler {}
```

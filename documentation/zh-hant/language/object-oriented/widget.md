# Widget 元件

Widget 是 Valkyrie 中用於構建宣告式使用者介面的基礎元件。Widget 採用不可變、組合式的設計理念，類似於 Flutter 或 React 的元件模型。

## 基本 Widget

### 定義 Widget

```valkyrie
# 定義一個簡單的 Widget
widget Greeting {
    name: string
    
    view {
        Text("Hello, {self.name}!")
    }
}
```

### 使用 Widget

```valkyrie
# 在其他 Widget 中使用
widget WelcomeScreen {
    user: User
    
    view {
        Column {
            Greeting { name: self.user.name }
            Text("Welcome to Valkyrie!")
        }
    }
}
```

## Widget 生命週期

### 狀態管理

```valkyrie
# 有狀態的 Widget
widget Counter {
    state mut count: i32 = 0
    
    view {
        Column {
            Text("Count: {self.count}")
            Button {
                text: "Increment"
                on_click: { self.count += 1 }
            }
        }
    }
}
```

### 生命週期方法

```valkyrie
widget DataWidget {
    data: Data?
    
    # 初始化時呼叫
    micro on_init(mut self) {
        self.load_data()
    }
    
    # 銷毀時呼叫
    micro on_dispose(self) {
        self.cleanup()
    }
    
    # 依賴更新時呼叫
    micro on_update(self, old_props: Self::Props) {
        if old_props.url != self.props.url {
            self.load_data()
        }
    }
    
    view {
        match self.data {
            case Some(d): DataView { data: d }
            case None: LoadingIndicator {}
        }
    }
}
```

## 組合與佈局

### 佈局 Widget

```valkyrie
widget ProfileCard {
    user: User
    
    view {
        Card {
            padding: 16
            child: Column {
                spacing: 8
                children: [
                    Avatar { 
                        url: self.user.avatar_url 
                        size: 64 
                    },
                    Text { 
                        text: self.user.name
                        style: TextStyle.bold 
                    },
                    Text { 
                        text: self.user.email
                        style: TextStyle.caption 
                    }
                ]
            }
        }
    }
}
```

### 條件渲染

```valkyrie
widget UserStatus {
    user: User
    
    view {
        Row {
            if self.user.is_online {
                Circle { color: Colors.green, size: 8 }
            } else {
                Circle { color: Colors.gray, size: 8 }
            }
            
            Text(self.user.status_text)
        }
    }
}
```

### 列表渲染

```valkyrie
widget UserList {
    users: [User]
    on_user_click: (User) -> Unit
    
    view {
        ListView {
            items: self.users
            item_builder: { user =>
                UserTile {
                    user: user
                    on_click: { self.on_user_click(user) }
                }
            }
        }
    }
}
```

## 樣式系統

### 內聯樣式

```valkyrie
widget StyledButton {
    text: string
    on_click: () -> Unit
    
    view {
        Container {
            padding: EdgeInsets.all(8)
            decoration: BoxDecoration {
                color: Colors.blue
                border_radius: BorderRadius.circular(4)
            }
            child: Text {
                text: self.text
                color: Colors.white
                font_size: 16
            }
            on_click: self.on_click
        }
    }
}
```

### 主題系統

```valkyrie
widget ThemedWidget {
    view {
        ThemeBuilder { theme =>
            Container {
                background: theme.colors.background
                child: Text {
                    text: "Themed Text"
                    color: theme.colors.text_primary
                    font: theme.typography.body
                }
            }
        }
    }
}
```

## 事件處理

### 事件回調

```valkyrie
widget InteractiveWidget {
    on_tap: () -> Unit
    on_long_press: () -> Unit
    on_hover: (bool) -> Unit
    
    view {
        GestureDetector {
            on_tap: self.on_tap
            on_long_press: self.on_long_press
            on_hover: self.on_hover
            child: Container {
                # ...
            }
        }
    }
}
```

### 表單處理

```valkyrie
widget LoginForm {
    state mut email: string = ""
    state mut password: string = ""
    on_submit: (string, string) -> Unit
    
    view {
        Form {
            Column {
                TextField {
                    placeholder: "Email"
                    value: self.email
                    on_change: { self.email = $ }
                }
                TextField {
                    placeholder: "Password"
                    value: self.password
                    obscure_text: true
                    on_change: { self.password = $ }
                }
                Button {
                    text: "Login"
                    on_click: { self.on_submit(self.email, self.password) }
                }
            }
        }
    }
}
```

## 效能優化

### 懶載入

```valkyrie
widget LazyList {
    items: [Item]
    
    view {
        LazyListView {
            items: self.items
            item_builder: { item =>
                ItemWidget { item: item }
            }
            estimated_item_height: 80
        }
    }
}
```

### 記憶化

```valkyrie
widget ExpensiveWidget {
    data: Data
    
    # 快取計算結果
    @memoize
    micro processed_data(self) -> ProcessedData {
        # 耗時計算
        self.data.heavy_processing()
    }
    
    view {
        DataView { data: self.processed_data() }
    }
}
```

## 最佳實踐

### 1. 單一職責

```valkyrie
# 好的實踐：每個 Widget 只負責一件事
widget UserAvatar {
    user: User
    
    view {
        Image { url: self.user.avatar_url }
    }
}

widget UserName {
    user: User
    
    view {
        Text { text: self.user.name }
    }
}

# 組合使用
widget UserHeader {
    user: User
    
    view {
        Row {
            UserAvatar { user: self.user }
            UserName { user: self.user }
        }
    }
}
```

### 2. 狀態提升

```valkyrie
# 將共享狀態提升到共同父元件
widget TodoApp {
    state mut todos: [Todo] = []
    
    view {
        Column {
            TodoHeader { 
                count: self.todos.length 
            }
            TodoList { 
                todos: self.todos
                on_toggle: { self.toggle_todo($id) }
            }
            TodoInput {
                on_add: { self.add_todo($text) }
            }
        }
    }
    
    micro add_todo(mut self, text: string) {
        self.todos.push(Todo { id: generate_id(), text, done: false })
    }
    
    micro toggle_todo(mut self, id: string) {
        loop todo in self.todos {
            if todo.id == id {
                todo.done = !todo.done
            }
        }
    }
}
```

### 3. 不可變 Props

```valkyrie
# Props 應該是不可變的
widget GoodWidget {
    # 所有 props 都是只讀的
    user: User
    on_update: (User) -> Unit
    
    view {
        Column {
            Text(self.user.name)
            Button {
                text: "Update"
                on_click: { 
                    # 通過回調更新，而不是直接修改
                    self.on_update(self.user.with_name("New Name"))
                }
            }
        }
    }
}
```

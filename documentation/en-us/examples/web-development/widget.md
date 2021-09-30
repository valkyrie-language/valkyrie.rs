# Widget

Widget is the core abstraction for UI components in Valkyrie. It provides a declarative way to build interactive user interfaces.

## Basic Widget

```valkyrie
widget Greeting {
    name: string
    
    div {
        class = "greeting"
        h1 { "Hello, {name}!" }
        p { "Welcome to Valkyrie." }
    }
}
```

## Widget with State

```valkyrie
widget Counter {
    count: Signal⟨i32⟩ = Signal.new(0)
    
    div {
        class = "counter"
        
        button {
            on_click { count.update { $ - 1 } }
            "-"
        }
        
        span { " {count.get()} " }
        
        button {
            on_click { count.update { $ + 1 } }
            "+"
        }
    }
}
```

## Props and Children

### Props

```valkyrie
widget Button {
    text: string
    variant: ButtonVariant = ButtonVariant.primary
    disabled: bool = false
    on_click: Option⟨micro()⟩ = None
    
    button {
        class = "btn btn-{variant}"
        disabled = disabled
        on_click { on_click?() }
        text
    }
}
```

### Children

```valkyrie
widget Card {
    title: string
    children: Widget
    
    div {
        class = "card"
        
        div {
            class = "card-header"
            h3 { title }
        }
        
        div {
            class = "card-body"
            children
        }
    }
}

# Usage
Card {
    title = "My Card"
    p { "This is the card content." }
}
```

## Lifecycle Hooks

```valkyrie
widget DataFetcher {
    data: Signal⟨Option⟨Data⟩⟩ = Signal.new(None)
    
    # Called when widget is mounted
    micro on_mount() {
        self.fetch_data()
    }
    
    # Called when widget is updated
    micro on_update(prev_props: Props) {
        if prev_props.url != self.url {
            self.fetch_data()
        }
    }
    
    # Called when widget is unmounted
    micro on_unmount() {
        # Cleanup resources
        self.cancel_pending_requests()
    }
    
    async micro fetch_data() {
        let result = fetch(self.url).await?
        data.set(Some(result))
    }
    
    div {
        if let Some(d) = data.get() {
            DataView { data: d }
        } else {
            Loading { }
        }
    }
}
```

## Computed Properties

```valkyrie
widget ShoppingCart {
    items: Signal⟨[CartItem]⟩ = Signal.new([])
    
    # Computed signal
    micro total(self) -> f64 {
        self.items.get()
            .iter()
            .map { $.price * $.quantity }
            .sum()
    }
    
    # Computed signal with dependencies
    micro item_count(self) -> i32 {
        self.items.get()
            .iter()
            .map { $.quantity }
            .sum()
    }
    
    div {
        h2 { "Shopping Cart" }
        
        for item in items.get() {
            CartItemRow { item: item }
        }
        
        div {
            class = "cart-summary"
            span { "Items: {self.item_count()}" }
            span { "Total: ${self.total()}" }
        }
    }
}
```

## Event Handling

```valkyrie
widget Form {
    email: Signal⟨string⟩ = Signal.new("")
    password: Signal⟨string⟩ = Signal.new("")
    
    micro submit(mut self) {
        # Validate and submit
        if self.validate() {
            self.send_form()
        }
    }
    
    micro validate(self) -> bool {
        let e = email.get()
        let p = password.get()
        
        e.contains("@") && p.length >= 8
    }
    
    form {
        on_submit { event |
            event.prevent_default()
            self.submit()
        }
        
        input {
            type = "email"
            placeholder = "Email"
            value = email.get()
            on_input { email.set($.value) }
        }
        
        input {
            type = "password"
            placeholder = "Password"
            value = password.get()
            on_input { password.set($.value) }
        }
        
        button {
            type = "submit"
            "Sign In"
        }
    }
}
```

## Conditional Rendering

```valkyrie
widget UserDashboard {
    user: Signal⟨Option⟨User⟩⟩ = Signal.new(None)
    loading: Signal⟨bool⟩ = Signal.new(true)
    
    div {
        if loading.get() {
            LoadingSpinner { }
        } else if let Some(u) = user.get() {
            UserProfile { user: u }
        } else {
            LoginPrompt { }
        }
    }
}
```

## List Rendering

```valkyrie
widget TodoList {
    todos: Signal⟨[Todo]⟩ = Signal.new([])
    
    div {
        class = "todo-list"
        
        for todo in todos.get() {
            TodoItem {
                key = todo.id
                todo = todo
                on_toggle = { self.toggle_todo(todo.id) }
                on_delete = { self.delete_todo(todo.id) }
            }
        }
        
        if todos.get().is_empty() {
            p { "No todos yet. Add one above!" }
        }
    }
}
```

## Slots

```valkyrie
widget Layout {
    header: Option⟨Widget⟩ = None
    footer: Option⟨Widget⟩ = None
    children: Widget
    
    div {
        class = "layout"
        
        if let Some(h) = header {
            header { h }
        }
        
        main {
            children
        }
        
        if let Some(f) = footer {
            footer { f }
        }
    }
}

# Usage
Layout {
    header = { nav { "My App" } }
    footer = { p { "© 2024" } }
    
    p { "Main content" }
}
```

## Context

Share data across the component tree:

```valkyrie
widget ThemeProvider {
    theme: Theme
    children: Widget
    
    provide(ThemeContext, theme) {
        children
    }
}

widget ThemedButton {
    # Access theme from context
    let theme = use_context(ThemeContext)
    
    button {
        style = {
            "background-color": theme.primary_color,
            "color": theme.text_color
        }
        "Themed Button"
    }
}
```

## Best Practices

1. **Keep widgets small**: Each widget should have a single responsibility
2. **Use signals for state**: Signals provide fine-grained reactivity
3. **Extract reusable logic**: Create custom hooks for shared behavior
4. **Avoid deep nesting**: Flatten component hierarchies when possible
5. **Use keys for lists**: Help the renderer identify items

---
**Related Sections**:
- [V-Grammar](../../language/syntax/v-grammar.md) - UI syntax
- [Signal](../../language/reactive-programming/signal.md) - State management
- [Events](./events.md) - Event handling

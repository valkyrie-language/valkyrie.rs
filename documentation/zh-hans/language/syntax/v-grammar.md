# 原生 UI 语法 (V-Grammar)

**V-Grammar** 是 Valkyrie 中用于构建声明式界面（UI）的核心语法。它允许开发者以极其自然、流畅的方式描述嵌套的组件结构，而无需繁琐的方法链或外部模板语言。

V-Grammar 针对不同的应用场景提供了两个特化版本：**跨平台通用开发 (Cross-platform)** 与 **Web HTML 特化**。

---

## 1. 跨平台通用开发 (Cross-platform)

这一风格主要用于跨平台 UI 开发（如原生应用、桌面端）。它强调组件的高度抽象和布局容器的概念。

### 核心特性
- **布局容器**：使用 `Column`, `Row`, `ZStack` 等语义化容器。
- **属性配置**：通过 ApplyBlock 进行直观的字段赋值或方法调用。
- **类型安全**：每个组件都是一个具体的类或结构体。

```valkyrie
# 通用 UI 示例
Column {
    spacing = 10
    alignment = .center
    
    Image("logo.png") {
        width = 100
        height = 100
    }
    
    Text("欢迎回来") {
        font = .headline
        color = .blue
    }
    
    Button("进入控制台") {
        on_click = micro() { navigate_to("/dash") }
    }
}
```

---

## 2. HTML 特化风格

当 Valkyrie 用于 Web 开发时，V-Grammar 提供了一套直接映射到标准 HTML 标签的特化语法。这一版本旨在消除 Web 开发者的迁移成本，同时保留 ApplyBlock 的逻辑能力。

### 核心特性
- **标签映射**：直接使用 `div`, `span`, `section`, `a` 等小写 HTML 标签。
- **属性简化**：支持标准的 HTML 属性名。
- **混合渲染**：可以直接在标签块内混写文本字面量和子标签。

```valkyrie
# HTML 特化示例
div {
    class = "container mx-auto"
    
    h1 { "仪表盘" }
    
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

## 3. 交互处理：极致灵活的事件绑定

V-Grammar 继承了 ApplyBlock 的灵活性，允许开发者根据语义需求选择最合适的事件绑定方式。

### 核心特性：多范式绑定
- **赋值/覆盖 (`=`)**：直接替换原有的处理逻辑。
- **追加 (`+=` / `.append`)**：在原有逻辑后添加新的处理函数。
- **显式设置 (`set`)**：语义化地设置处理逻辑。
- **函数式简写**：像调用方法一样直接定义处理块。

```valkyrie
Button("交互演示") {
    # 1. 函数式简写 (最常用)
    on_click {
        println("直接触发")
    }

    # 2. 赋值语法
    on_hover = micro() { is_hovered = true }

    # 3. 运算符重载 (追加逻辑)
    on_click += micro() {
        log_event("button_clicked")
    }

    # 4. 显式方法调用
    on_close.set(micro() { cleanup() })
    on_scroll.append(micro(e) { update_position(e) })
}
```

---

## 4. 语法基础：应用块 (ApplyBlock)

无论是哪种风格，V-Grammar 的底层统一基于 **[应用块 (ApplyBlock)](./braces.md)**。

ApplyBlock 在 V-Grammar 中统一了四种核心操作：
1. **字段赋值**：`class = "..."` 或 `spacing = 10`。
2. **事件绑定**：如上所述的多种灵活语法（`=`、`+=`、`{}` 等）。
3. **方法调用**：`.modifier()` 风格的链式调用。
4. **子节点注入**：直接在块内编写另一个组件/标签。

具体的语义解释由后续的类型系统决定。例如，如果 `div` 被标记为 `HtmlElement`，块内的嵌套调用将被自动解释为 `appendChild`。

---

## 4. 动态 UI：原生逻辑控制

V-Grammar 不需要 `v-for` 或 `ng-if` 等特殊指令，它直接使用 Valkyrie 的原生控制流：

- **条件渲染**：使用标准的 `if-else`。
- **列表循环**：使用标准的 `loop-in`。
- **复杂状态**：使用标准的 `match` 模式匹配。

这些控制流在两种风格中完全一致，确保了逻辑层的高度可复用。

---

## 5. 语法特性总结

| 特性 | 跨平台通用开发 | Web HTML 特化 |
| :--- | :--- | :--- |
| **主要目标** | 原生应用 / 桌面端 | Web 页面 / SSR |
| **标签风格** | 大写字母 (Component) | 小写字母 (Tag) |
| **嵌套方式** | `Child { ... }` | `tag { ... }` |
| **适用环境** | 原生渲染引擎 | 浏览器 / DOM |

---

## 6. 魔法的真相：结构优先

V-Grammar 的强大之处在于它遵循了 **[应用块 (ApplyBlock)](./braces.md)** 的核心设计原则：**先解析结构，后校验语义**。

1. **结构化解析**：编译器首先将块解析为一个通用的“语句流”。
2. **延迟绑定**：直到类型检查阶段，编译器才会根据调用者（是 `Column` 还是 `div`）来决定块内的语句是属性设置还是 DOM 操作。
3. **零开销抽象**：这种设计使得 UI 描述在运行时可以被编译为极其高效的直接操作，避免了虚拟 DOM 对比或模板解析的开销。

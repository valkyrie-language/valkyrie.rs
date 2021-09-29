# Native UI Syntax (V-Grammar)

**V-Grammar** is the core syntax used in Valkyrie for building declarative interfaces (UI). It allows developers to describe nested component structures in an extremely natural and fluid way, without the need for cumbersome method chains or external template languages.

V-Grammar provides two specialized versions for different application scenarios: **Cross-platform Universal Development** and **Web HTML Specialization**.

---

## 1. Cross-platform Universal Development

This style is primarily used for cross-platform UI development (such as native apps, desktop applications). It emphasizes high-level abstraction of components and the concept of layout containers.

### Core Features
- **Layout Containers**: Uses semantic containers like `Column`, `Row`, `ZStack`.
- **Attribute Configuration**: Intuitive field assignment or method calls via ApplyBlocks.
- **Type Safety**: Each component is a specific class or struct.

```valkyrie
# Universal UI Example
Column {
    spacing = 10
    alignment = .center
    
    Image("logo.png") {
        width = 100
        height = 100
    }
    
    Text("Welcome back") {
        font = .headline
        color = .blue
    }
    
    Button("Enter Console") {
        on_click = micro() { navigate_to("/dash") }
    }
}
```

---

## 2. Web HTML Specialization

When Valkyrie is used for Web development, V-Grammar provides a set of specialized syntax that maps directly to standard HTML tags. This version aims to eliminate migration costs for Web developers while retaining the logical power of ApplyBlocks.

### Core Features
- **Tag Mapping**: Directly uses lowercase HTML tags like `div`, `span`, `section`, `a`.
- **Attribute Simplification**: Supports standard HTML attribute names.
- **Mixed Rendering**: Text literals and sub-tags can be mixed directly within tag blocks.

```valkyrie
# HTML Specialization Example
div {
    class = "container mx-auto"
    
    h1 { "Dashboard" }
    
    section {
        id = "stats-grid"
        class = "grid grid-cols-3 gap-4"
        
        for stat in dashboard_stats {
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

---

## 3. Interaction Handling: Extremely Flexible Event Binding

V-Grammar inherits the flexibility of ApplyBlocks, allowing developers to choose the most suitable event binding method based on semantic needs.

### Core Features: Multi-paradigm Binding
- **Assignment/Override (`=`)**: Directly replaces the original processing logic.
- **Append (`+=` / `.append`)**: Adds a new processing function after the original logic.
- **Explicit Set (`set`)**: Semantically sets the processing logic.
- **Functional Shorthand**: Directly defines a processing block like calling a method.

```valkyrie
Button("Interaction Demo") {
    # 1. Functional Shorthand (Most common)
    on_click {
        println("Triggered directly")
    }

    # 2. Assignment Syntax
    on_hover = micro() { is_hovered = true }

    # 3. Operator Overloading (Append logic)
    on_click += micro() {
        log_event("button_clicked")
    }

    # 4. Explicit Method Call
    on_close.set(micro() { cleanup() })
    on_scroll.append(micro(e) { update_position(e) })
}
```

---

## 4. Syntactic Foundation: ApplyBlock

Regardless of the style, the underlying layer of V-Grammar is uniformly based on **[ApplyBlock](./braces.md)**.

ApplyBlock unifies four core operations in V-Grammar:
1. **Field Assignment**: `class = "..."` or `spacing = 10`.
2. **Event Binding**: Various flexible syntaxes as described above (`=`, `+=`, `{}`).
3. **Method Calling**: `.modifier()` style chained calls.
4. **Sub-node Injection**: Writing another component/tag directly within the block.

Specific semantic interpretation is determined by the subsequent type system. For example, if `div` is marked as `HtmlElement`, nested calls within the block will be automatically interpreted as `appendChild`.

---

## 5. Dynamic UI: Native Logic Control

V-Grammar does not require special directives like `v-for` or `ng-if`; it directly uses Valkyrie's native control flow:

- **Conditional Rendering**: Use standard `if-else`.
- **List Iteration**: Use standard `for-in`.
- **Complex State**: Use standard `match` pattern matching.

These control flows are completely consistent across both styles, ensuring high reusability of the logic layer.

---

## 6. Summary of Features

| Feature | Cross-platform Universal | Web HTML Specialization |
| :--- | :--- | :--- |
| **Primary Goal** | Native Apps / Desktop | Web Pages / SSR |
| **Tag Style** | Uppercase (Component) | Lowercase (Tag) |
| **Nesting Method** | `Child { ... }` | `tag { ... }` |
| **Target Environment** | Native Rendering Engine | Browser / DOM |

---

## 7. The Truth Behind the Magic: Structure First

The power of V-Grammar lies in its adherence to the core design principle of **[ApplyBlock](./braces.md)**: **Parse structure first, validate semantics later**.

1. **Structured Parsing**: The compiler first parses the block into a generic "statement stream".
2. **Late Binding**: Until the type checking phase, the compiler decides whether a statement within the block is an attribute setting or a DOM operation based on the caller (whether it's `Column` or `div`).
3. **Zero-Cost Abstraction**: This design allows UI descriptions to be compiled into extremely efficient direct operations at runtime, avoiding the overhead of Virtual DOM diffing or template parsing.


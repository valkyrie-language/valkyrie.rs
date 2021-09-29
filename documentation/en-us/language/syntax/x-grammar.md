# X-Grammar

After mastering the [Widget](../../examples/web-development/widget.md) object model and the [V-Grammar](./v-grammar.md) closure syntax, X-Grammar provides us with a **visual projection** of UI logic.

Consistent with V-Grammar, X-Grammar also provides two application versions: **Cross-platform Component Style** and **Web HTML Style**.

---

## 1. Cross-platform Component Style

This style maps X-Grammar tags to cross-platform UI components (such as `Column`, `Button`). It is suitable for non-Web environments that require the visual structure of X-Grammar.

```xml
<Column spacing=10 alignment=".center">
    <Image src="logo.png" width=100 height=100 />
    
    <Text font=".headline" color=".blue">
        Welcome back
    </Text>
    
    <Button on_click={ navigate_to("/dash") }>
        Enter Console
    </Button>
</Column>
```

---

## 2. Web HTML Style

This style maps directly to standard HTML tags, suitable for Web development and Server-Side Rendering (SSR).

```xml
<div class="container">
    <h1>Welcome to Valkyrie</h1>
    
    <!-- disabled accepts a boolean, on_click accepts a closure -->
    <button disabled=(count >= 10) on_click={ count += 1 }>
        <if (count == 0)> Start <else/> Continue </if>
    </button>
    
    <p>Current progress: $progress%</p>
</div>
```

---

## 3. Basic Syntax and Attribute Binding

X-Grammar uses tags to describe UI structures. All interaction and data flow are achieved through a unified attribute binding system:

- **Immediate Attributes `( )`**: Used for scenarios where immediate calculation and assignment are required.
    - **Literal Shorthand**: `name="value"` or `name=10`.
    - **Identifier Shorthand**: `name=variable`.
    - **Expression Evaluation**: `name=(expression)`.
- **Closure Attributes `{ }`**: Used to pass logic blocks (closures). Under the hood, this usually corresponds to a Widget's event registration method (e.g., `on_click`).
- **Content Interpolation `${ }` / `$ident`**: In tag text content, use `$` for dynamic interpolation.

```xml
<div class="container">
    <h1>Welcome to Valkyrie</h1>
    
    <!-- disabled accepts a boolean, on_click accepts a closure -->
    <button disabled=(count >= 10) on_click={ count += 1 }>
        <if (count == 0)> Start <else/> Continue </if>
    </button>
    
    <!-- Event forwarding: essentially passing a closure prop from the parent to the child -->
    <CustomWidget on_click=on_click />
    
    <p>Current progress: ${progress}%</p>
</div>
```

## 4. Logic Keywords

In X-Grammar mode, logic tags (`if`, `else`, `match`, `for`, `slot`) are no longer ordinary UI components but are promoted to **Native Keywords**. This means they have special Parser syntax support and can map directly to Valkyrie's core control flow.

### Conditional Rendering (`<if>`)
Supports standard `if-else` structures, with boolean expressions inside parentheses. As a keyword, it supports more flexible nesting and shorthands.
```xml
<if (count > 5)>
    <p>Halfway there</p>
<else/>
    <p>Keep going</p>
</if>
```

### Pattern Matching (`<match>`)
Maps directly to Valkyrie's `match` statement, supporting type matching and destructuring.
```xml
<match (user.role)>
    <case "admin">  <badge>Admin</badge> </case>
    <case "user">   <badge>User</badge> </case>
    <else>          <badge>Guest</badge>    </else>
</match>
```

### Iteration (`<for>`)
Supports `for ... in ...` syntax. Since it's handled as a keyword, the Parser can more accurately parse iterators and destructuring assignments.
```xml
<for (item, index) in (list)>
    <li key=index>${item.name}</li>
<else/>
    <p>List is empty</p>
</for>
```

### Content Projection (`<slot>`)
`<slot>` is a keyword for content projection. It is not a real DOM node but a **compiler placeholder**, indicating where component fields should be rendered.

#### 1. Declaration and Basic Usage
In a `widget` definition, use `$` followed by the field name to mark a slot:
```valkyrie
widget Card {
    header: Widget
    content: [Widget]
    
    <div class="card">
        <slot $header />
        <div class="card-content">
            <slot $content />
        </div>
    </div>
}
```

#### 2. Scoped Slots
If a field is a factory function (like `micro`), you can pass parameters via attribute syntax to achieve reverse data passing:
```valkyrie
widget List {
    items: [T]
    render_item: micro(T) -> Widget

    <div class="list">
        <for item in (items)>
            <slot $render_item=(item) />
        </for>
    </div>
}
```

#### 3. Default Content
When the parent component doesn't provide content, the children inside the tag are rendered:
```xml
<slot $footer>
    <p>This is the default footer</p>
</slot>
```

---

## 5. Extension: Single File Components (SFC)

Single File Components (SFC) are an advanced application pattern for X-Grammar. It organizes different concerns through top-level tags, where the `<template>` block contains the X-Grammar view.

```xml
<template>
    <div class="container">
        <h1>Hello, $name</h1>
        <!-- handleClick is a function, passed as a closure -->
        <button on_click=handleClick>
            Click count: $count
        </button>
    </div>
</template>

<script>
let name = "Valkyrie";
let count = 0;

micro handleClick() {
    count += 1;
}
</script>
```

Top-level tags usually include:
- `<template>`: View template (X-Grammar).
- `<script>`: Logic code (Valkyrie code).
- `<style>`: Style definitions.
- `<router>`: Routing configuration.
- `<meta>`: Metadata definitions.

## 6. Syntax Comparison and Principles

X-Grammar has no "magic directives"; all its tags and attributes are converted 1:1 to corresponding attribute assignments or closure passes in [V-Grammar](./v-grammar.md).

| X-Grammar | Semantics | V-Grammar Equivalent |
| :--- | :--- | :--- |
| `name=(val)` | Attribute assignment (immediate) | `.name(val)` or `name = val` |
| `name={...}` | Closure passing (deferred) | `name { ... }` or `on_name { ... }` |
| `$ident` / `${expr}` | Text interpolation | Converted to string and rendered |
| `<if (cond)>` | Conditional branch | `if cond { ... }` |
| `<match (val)>` | Pattern matching | `match val { ... }` |
| `<for (i) in (L)>` | Iteration | `for i in L { ... }` |

---

## 7. The Truth Behind the Magic: Visual Projection of Logic


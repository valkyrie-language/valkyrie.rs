# X-Grammar

在掌握了 [Widget](../../examples/web-development/widget.md) 的对象模型和 [V-Grammar](./v-grammar.md) 的闭包语法后，X-Grammar 为我们提供了 UI 逻辑的**视觉投影**。

与 V-Grammar 一致，X-Grammar 同样提供两个应用版本：**跨平台组件风格**与 **Web HTML 风格**。

---

## 1. 跨平台组件风格

这一风格将 X-Grammar 标签映射到跨平台的 UI 组件（如 `Column`, `Button`）。它适用于需要 X-Grammar 视觉结构的非 Web 环境。

```xml
<Column spacing="10" alignment=".center">
    <Image src="logo.png" width="100" height="100" />
    
    <Text font=".headline" color=".blue">
        欢迎回来
    </Text>
    
    <Button :on_click="navigate_to(\"/dash\")">
        进入控制台
    </Button>
</Column>
```

---

## 2. Web HTML 风格

这一风格直接映射到标准 HTML 标签，适用于 Web 开发和服务器端渲染（SSR）。

```xml
<div class="container">
    <h1>欢迎来到 Valkyrie</h1>
    
    <button :disabled="count >= 10" :on_click="count += 1">
        <if (count == 0)> 开始 <else/> 继续 </if>
    </button>
    
    <p>当前进度：{progress}%</p>
</div>
```

---

## 3. 基础语法与属性绑定

X-Grammar 使用标签来描述 UI 结构。属性 KV 与 AWSL 一样**不在属性上使用花括号**——避免 JSX 式 `={expr}` 的视觉污染。

| 场景 | 写法 | 含义 |
|:---|:---|:---|
| 静态属性 | `class="container"` | 字符串字面量 |
| 动态绑定 | `:on_click="increment"` | 表达式（引号内为 Valkyrie 源码） |
| 正文插值 | `{count}` | 元素文本内的动态片段 |

**规则**：
- 小写 HTML / 组件标签上的静态属性 **必须加引号**：`href="/about"`
- 动态绑定 **加 `:` 前缀 + 引号表达式**：`:disabled="count >= 10"`、`:on_click="increment"`
- **禁止** `on_click={increment}`、`class={name}` 等 KV 花括号写法
- 花括号 `{expr}` **仅用于元素正文插值**

```xml
<div class="container">
    <h1>欢迎来到 Valkyrie</h1>
    
    <button :disabled="count >= 10" :on_click="count += 1">
        <if (count == 0)> 开始 <else/> 继续 </if>
    </button>
    
    <!-- 事件转发：将父组件闭包 prop 传给子组件 -->
    <CustomWidget :on_click="on_click" />
    
    <p>当前进度：{progress}%</p>
</div>
```

### 与 AWSL 的对照

| | AWSL | `.vx` X-Grammar |
|:---|:---|:---|
| 静态 KV | `class="counter"` | `class="counter"` |
| 动态 KV | `:prop="expr"` | `:on_click="expr"` |
| DSL / 事件 | `@click="fn"` `@loop="..."` | — |
| 正文插值 | `{count}` | `{count}` |
| KV 花括号 | 禁止 | 禁止 |

二者都拒绝属性上的 `{}`；AWSL 用 `@` DSL + `:` 动态绑定，`.vx` 用 `:` + 引号。

## 4. 逻辑关键字 (Logic Keywords)

在 X-Grammar 模式下，逻辑标签（`if`, `else`, `match`, `loop`, `slot`）不再是普通的 UI 组件，而是被晋升为 **原生关键字 (Native Keywords)**。这意味着它们拥有特殊的 Parser 语法支持，能够直接映射到 Valkyrie 的核心控制流。

### 条件渲染 (`<if>`)
支持标准的 `if-else` 结构，括号内为布尔表达式。由于是关键字，它支持更灵活的嵌套和简写。
```xml
<if (count > 5)>
    <p>计数已过半</p>
<else/>
    <p>继续努力</p>
</if>
```

### 模式匹配 (`<match>`)
直接映射到 Valkyrie 的 `match` 语句，支持类型匹配和解构。
```xml
<match (user.role)>
    <case "admin">  <badge>管理员</badge> </case>
    <case "user">   <badge>普通用户</badge> </case>
    <else>          <badge>访客</badge>    </else>
</match>
```

### 循环迭代 (`<loop>`)
支持 `loop ... in ...` 语法。由于作为关键字处理，Parser 可以更精确地解析迭代器和解构赋值。
```xml
<loop (item, index) in (list)>
    <li :key="index">{item.name}</li>
<else/>
    <p>列表为空</p>
</loop>
```

### 内容投影 (`<slot>`)
`<slot>` 是用于内容投影的关键字。它不是一个真实的 DOM 节点，而是一个**编译器占位符**，指示组件字段的渲染位置。

#### 1. 声明与基本用法
在 `widget` 定义中，使用 `$` 引导字段名来标记槽位：
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

#### 2. 作用域插槽 (Scoped Slots)
如果字段是一个工厂函数（如 `micro`），可以通过属性语法传递参数，实现数据的反向传递：
```valkyrie
widget List {
    items: [T]
    render_item: micro(T) -> Widget

    <div class="list">
        <loop item in (items)>
            <slot $render_item=(item) />
        </loop>
    </div>
}
```

#### 3. 默认内容
当父组件未提供对应内容时，渲染标签内部的子节点：
```xml
<slot $footer>
    <p>这是默认页脚</p>
</slot>
```

---

## 5. 扩展：`.vx` 文件（Valkyrie + X-Grammar）

`.vx` 是约定使用 **X-Grammar 内联标记** 的 Valkyrie 源文件扩展名，定位类似 **TSX**。XML 是 **Valkyrie 语言本身的扩展**，不是独立方言；**没有** AWSL 式的 `<template>` / `<script>` / `<style>` 块结构（那是 AWSL）。

```valkyrie
widget Counter {
    count: i32 = 0

    micro increment() {
        count += 1
    }

    micro view() {
        <div class="container">
            <button :on_click="increment">
                点击次数: {count}
            </button>
        </div>
    }
}
```

- 视图：显式 `micro view() { ... }` 或脚本末尾的 X-Grammar 尾表达式（见上文「显式 / 隐式视图」）。
- 逻辑与状态：与普通 `.v` 文件相同，写在 `widget` 体内。
- 编译：`compile_vx_source` 解析 Valkyrie AST，将 `view` 规范为 `render() -> Element`，X-Grammar 降级为 `Element` 树（实现进行中）。

### 5.1 `.vx` 中的 T-Grammar 混写

在 `view` / `render` 方法体中，可将 [T-Grammar](./t-grammar.md) 的 `<% %>` meta 与 X-Grammar 标签混写：

```valkyrie
micro view() {
    <% if show %>
        <div>{x}</div>
    <% end %>
}
```

词法分析器会跳过 `<% ... %>`，因此 `AstParser::parse_vx_root` 在检测到方法体源码含 `<%` 时，对 `view` / `render` 体做**源码重解析**（`parse_xgrammar_with_meta`），生成 `XgNode::Meta { nodes: TgRoot }`。编译期 **TgExpandPass** 将 Meta 展开为 `if` / `loop` / `match` 等 Valkyrie 控制流后再降级 HIR；展开后剩余的 `XmlMarkup` 仍通过 `Element::from_markup` 占位接入 widget 管线（完整 Element 树降级进行中）。

### 三种 UI 入口如何共存

| 入口 | 文件 | 类比 | 说明 |
|:---|:---|:---|:---|
| `widget` | `.v` | React | `render(self) -> Element`，命令式拼树 |
| Valkyrie + XML | `.vx` | **TSX** | 内联 X-Grammar，`view` / 尾表达式；**不支持 HTML void 元素**（须显式闭合或自闭合） |
| AWSL | `.awsl` | Vue | 单文件 `<widget>` + 指令，经 `voa` 降级；**兼容 HTML5 void 标签**（`br` / `img` / `input` 等，支持但不推荐） |

AWSL 可直接粘贴 HTML 片段（含 `<br>`、`<img src="…">` 等无闭合 void 标签）；`.vx` 的 X-Grammar 要求显式标记结构，不能照搬 HTML void 规则——这是 AWSL 作为 Vue 式 DSL 的实用优势之一。

三者最终应汇入同一 **widget HIR → MIR** 主线；AWSL 当前经 `voa` 前置降级，`.vx` 经 X-Grammar 内联降级（实现进行中）。

## 6. 语法对比与原理

X-Grammar 没有任何“魔法指令”，它的所有标签 and 属性都会 1:1 地转换为 [V-Grammar](./v-grammar.md) 中对应的属性赋值或闭包传递。

| X-Grammar | 语义 | V-Grammar 等效代码 |
| :--- | :--- | :--- |
| `name="val"` | 静态字符串 | `.name("val")` |
| `:name="expr"` | 动态绑定 | `.name(expr)` 或 `on_name { ... }` |
| `{expr}`（正文） | 文本插值 | 转换为字符串并渲染 |
| `<if (cond)>` | 条件分支 | `if cond { ... }` |
| `<match (val)>` | 模式匹配 | `match val { ... }` |
| `<loop (i) in (L)>` | 循环迭代 | `loop i in L { ... }` |

---

## 7. 魔法的真相：逻辑的视觉投影

**虽然 X-Grammar 看上去很魔法，但本质上没有那么多魔法。** 它不是一个庞大的运行时框架，而是一层轻薄的、直观的**语法投影**。

### 1. 零指令设计 (Zero Directive)

Valkyrie 不需要 `v-bind`, `on:` 或 `v-if` 这种“属性指令”，因为 X-Grammar 深度信任底层的对象模型。

- **回归编程本质**：如果一个 Widget 有 `on_click` 方法或 `disabled` 字段，动态绑定写 `:on_click="handler"`、`:disabled="flag"`。
- **Valkyrie 的统一方案**：
    - **逻辑归关键字**：`<if>`, `<loop>`, `<slot>` 等逻辑容器直接处理结构控制。
    - **内容归插值**：使用 `{expr}` 进行动态文本注入（仅正文，不进 KV）。
    - **片段归属性**：所有的“插槽传递”在 Valkyrie 中都被统一为**带块的属性赋值**。

### 2. 定界符分工

- **`"..."`（KV 静态）**：字符串字面量，`class="container"`。
- **`:name="expr"`（KV 动态）**：表达式绑定，与 AWSL 一样不在 KV 上使用 `{}`。
- **`{expr}`（正文）**：文本插值。

这种区分消除了歧义，并允许编译器生成最优化的底层代码。

### 3. 静态转换：消失的开销

所有的 X-Grammar 语法在编译阶段都会被“拍扁”成最高效的原生方法链。这种“非魔法”的设计，让 Valkyrie 既拥有了 X-Grammar 的直观，又彻底消除了传统前端框架带来的学习成本和运行负担。一切你看到的“魔法”，最终都只是标准的编程概念在视觉上的延伸。

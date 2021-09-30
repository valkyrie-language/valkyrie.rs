# X-Grammar

在掌握了 [Widget](../../examples/web-development/widget.md) 的对象模型和 [V-Grammar](./v-grammar.md) 的闭包语法后，X-Grammar 为我们提供了 UI 逻辑的**视觉投影**。

与 V-Grammar 一致，X-Grammar 同样提供两个应用版本：**跨平台组件风格**与 **Web HTML 风格**。

---

## 1. 跨平台组件风格

这一风格将 X-Grammar 标签映射到跨平台的 UI 组件（如 `Column`, `Button`）。它适用于需要 X-Grammar 视觉结构的非 Web 环境。

```xml
<Column spacing=10 alignment=".center">
    <Image src="logo.png" width=100 height=100 />
    
    <Text font=".headline" color=".blue">
        欢迎回来
    </Text>
    
    <Button on_click={ navigate_to("/dash") }>
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
    
    <!-- disabled 接受布尔值，on_click 接受闭包 -->
    <button disabled=(count >= 10) on_click={ count += 1 }>
        <if (count == 0)> 开始 <else/> 继续 </if>
    </button>
    
    <p>当前进度：$progress%</p>
</div>
```

---

## 3. 基础语法与属性绑定

X-Grammar 使用标签来描述 UI 结构。所有的交互和数据流动都通过统一的属性绑定实现：

- **立即属性 `( )`**：用于需要立即计算并赋值的场景。
    - **字面量简写**：`name="value"` 或 `name=10`。
    - **标识符简写**：`name=variable`。
    - **表达式求值**：`name=(expression)`。
- **闭包属性 `{ }`**：用于传递逻辑块（闭包）。在底层，这通常对应于 Widget 的事件注册方法（如 `on_click`）。
- **内容插值 `{ }`**：在标签文本内容中，使用 `{expr}` 进行动态插值。

```xml
<div class="container">
    <h1>欢迎来到 Valkyrie</h1>
    
    <!-- disabled 接受布尔值，on_click 接受闭包 -->
    <button disabled=(count >= 10) on_click={ count += 1 }>
        <if (count == 0)> 开始 <else/> 继续 </if>
    </button>
    
    <!-- 事件转发：本质上就是将父组件传入的闭包 prop 传递给子组件 -->
    <CustomWidget on_click=on_click />
    
    <p>当前进度：{progress}%</p>
</div>
```

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
    <li key=index>{item.name}</li>
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

## 5. 扩展：单文件组件 (SFC)

单文件组件 (Single File Component) 是 X-Grammar 的一种高级应用模式。它通过顶级标签组织不同的关注点，其中 `<template>` 块包含 X-Grammar 视图。

```xml
<template>
    <div class="container">
        <h1>Hello, $name</h1>
        <!-- handleClick 是一个函数，作为闭包传递 -->
        <button on_click=handleClick>
            点击次数: $count
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

顶级标签通常包括：
- `<template>`：视图模板（X-Grammar）。
- `<script>`：逻辑代码（Valkyrie 代码）。
- `<style>`：样式定义。
- `<router>`：路由配置。
- `<meta>`：元数据定义。

## 6. 语法对比与原理

X-Grammar 没有任何“魔法指令”，它的所有标签 and 属性都会 1:1 地转换为 [V-Grammar](./v-grammar.md) 中对应的属性赋值或闭包传递。

| X-Grammar | 语义 | V-Grammar 等效代码 |
| :--- | :--- | :--- |
| `name=(val)` | 属性赋值 (立即) | `.name(val)` 或 `name = val` |
| `name={...}` | 闭包传递 (延迟) | `name { ... }` 或 `on_name { ... }` |
| `{expr}` | 文本插值 | 转换为字符串并渲染 |
| `<if (cond)>` | 条件分支 | `if cond { ... }` |
| `<match (val)>` | 模式匹配 | `match val { ... }` |
| `<loop (i) in (L)>` | 循环迭代 | `loop i in L { ... }` |

---

## 7. 魔法的真相：逻辑的视觉投影

**虽然 X-Grammar 看上去很魔法，但本质上没有那么多魔法。** 它不是一个庞大的运行时框架，而是一层轻薄的、直观的**语法投影**。

### 1. 零指令设计 (Zero Directive)

Valkyrie 不需要 `v-bind`, `on:` 或 `v-if` 这种“属性指令”，因为 X-Grammar 深度信任底层的对象模型。

- **回归编程本质**：如果一个 Widget 有 `on_click` 方法或 `disabled` 字段，你就在 X-Grammar 里直接写 `on_click` 或 `disabled`。
- **Valkyrie 的统一方案**：
    - **逻辑归关键字**：`<if>`, `<loop>`, `<slot>` 等逻辑容器直接处理结构控制。
    - **内容归插值**：使用 `{expr}` 进行动态文本注入。
    - **片段归属性**：所有的“插槽传递”在 Valkyrie 中都被统一为**带块的属性赋值**。

#### 场景 1：直接在标签内定义 (Inline Slot)
```xml
<Card>
    <!-- 1. 具名传递：header 是 Card 的一个属性/方法 -->
    <header>
        <Icon name="user" />
        <Text>用户信息</Text>
    </header>
    
    <!-- 2. 匿名传递：直接写在标签内的内容通常映射到 default 属性 or appendChild -->
    <p>这是卡片的正文内容</p>
</Card>

#### 场景 2：显式视图函数 (Explicit View Function)

在某些复杂的逻辑脚本中，你可能不希望通过“脚本末尾的表达式”来隐式定义视图。此时可以显式定义一个 `view` 函数。这个函数不需要放在脚本末尾，它会被自动识别为组件的渲染入口。

```valkyrie
# 显式定义视图，无需放在文件末尾
micro view() {
    let x = xxx;
    <div class="layout">
        <header> $title </header>
        <main> $content </main>
    </div>
}

# 逻辑代码可以放在 view 之后
let title = "系统概览";
let content = "这里是主要的业务逻辑内容...";
```

| 特性 | 隐式视图 (Trailing Expression) | 显式视图 (View Function) |
| :--- | :--- | :--- |
| **定义方式** | 脚本最后一行是一个 X-Grammar 标签 | 定义一个名为 `view` 的函数 |
| **位置要求** | 必须在末尾 | 可以在脚本的任何位置 |
| **适用场景** | 简单组件、快速原型 | 复杂逻辑、需要明确入口的组件 |

---

### 2. 括号的力量

通过 `()`, `{}` 和 `${}`，X-Grammar 在编译阶段就明确了“值”、“逻辑”与“插值”的区别。

- **`()` (Value)**: 立即求值的静态或动态数据。
- **`{}` (Block)**: 延迟执行的代码块或对象配置。
- **`${}` (Hole)**: 注入到文本环境中的动态表达式。

这种区分消除了歧义，并允许编译器生成最优化的底层代码。

### 3. 静态转换：消失的开销

所有的 X-Grammar 语法在编译阶段都会被“拍扁”成最高效的原生方法链。这种“非魔法”的设计，让 Valkyrie 既拥有了 X-Grammar 的直观，又彻底消除了传统前端框架带来的学习成本和运行负担。一切你看到的“魔法”，最终都只是标准的编程概念在视觉上的延伸。

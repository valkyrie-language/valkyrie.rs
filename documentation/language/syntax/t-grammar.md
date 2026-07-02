# 模板语法 (T-Grammar)

在理解了 [V-Grammar](./v-grammar.md) 的对象构建和 [X-Grammar](./x-grammar.md) 的视觉投影后，**T-Grammar** 提供了对**代码生成**和**文本处理**的支持。

T-Grammar 面向 [宏](../meta-programming/macro.md) 与编译期文本展开：模板在解析阶段构造成 `TgRoot` AST，再由编译器的 **TgExpandPass** 展开为普通 Valkyrie 表达式与控制流。

## 1. 基础语法

T-Grammar 使用 **`t"`** 前缀的模板字符串字面量（也支持 `t"""` 多行形式）。`quote"` 前缀保留在语言规划中，当前实现仅识别 `t"`。

```valkyrie
let template = t"<% let name = \"Valkyrie\" %>
Hello, {name}!
<# 这是一个注释，不会出现在生成的代码中 #>"
```

T-Grammar 使用 `<%` 和 `%>` 作为界定符：

- **表达式插值 `{ expr }`**：将表达式结果转换为字符串并拼接。
- **块级关键词 `<% if %>` / `<% loop %>` / `<% match %>`**：以 `<%` 后紧跟的关键词识别控制流块。
- **语句 `<% stmt %>`**：非块级关键词时，整段视为 Valkyrie 语句（`TgNode::Stmt`）。
- **注释 `<# comment #>`**：模板内部注释，展开时丢弃。
- **闭合**：所有控制流块统一以 `<% end %>` 闭合（不使用 `<% end if %>` 等变体）。

## 2. 逻辑控制

T-Grammar 继承 Valkyrie 的控制流语义，但包裹在 `<% %>` 中。逻辑标签可以独立存在于模板内；在普通 Valkyrie 代码中直接嵌入标签（如 `class <% name %>`）是非法的。

### 条件分支 (`if`)

```valkyrie
t"<% if condition %>
    # 当 condition 为真时生成的代码
<% else if other_condition %>
    # 当 other_condition 为真时生成的代码
<% else %>
    # 默认生成的代码
<% end %>"
```

### 循环迭代 (`loop`)

```valkyrie
t"<% loop i in items %>
    {expr(i)}
<% end %>"
```

### 模式匹配 (`match`)

```valkyrie
t"<% match status %>
    <% case Loading %>
        print(\"Loading...\")
    <% case Success { data } %>
        print(\"Data: {data}\")
    <% case Error { err } %>
        print(\"Error: {err}\")
<% end %>"
```

## 3. 编译期展开

`t"..."` 在词法分析后由 `parse_tgrammar_template` 解析为 `TermExpression::Template { nodes: TgRoot }`。在 HIR 降级前，`TgExpandPass`（`expand_tgrammar_in_root`）会：

| AST 节点 | 展开结果 |
| :--- | :--- |
| `TgNode::Text` | 字符串字面量与 `{expr}` 的 `+` 拼接 |
| `TgNode::If` / `Loop` / `Match` | 对应 Valkyrie `if` / `loop` / `match` |
| `TgNode::Stmt` | 解析为语句并嵌入块表达式 |
| `TgNode::Comment` | 丢弃 |

宏与代码生成应显式使用 `t"` 字面量，而不是依赖隐式 `TokenStream` 字符串解读。

```valkyrie
macro generate_struct(name: string, fields: [Field]) -> TokenStream {
    let source = t"
class {name} {
    <% loop f in fields %>
        {f.name}: {f.type},
    <% end %>
}";
    compile_template_to_tokens(source)
}
```

## 4. 与 X-Grammar 混写

在 `.vx` 视图体中，可将 `<% %>` 与 X-Grammar 标签混写。词法层会跳过 `<% ... %>`，因此编译器在 `parse_vx_root` 阶段对 `view` / `render` 方法体做源码重解析，生成带 `XgNode::Meta` 的 `XmlMarkup`，再由 TgExpandPass 在编译期展开。详见 [X-Grammar §5](./x-grammar.md#5-vx-中的-t-grammar-混写)。

## 5. 语法对比

| 特性 | T-Grammar | X-Grammar | V-Grammar |
| :--- | :--- | :--- | :--- |
| **定位** | 文本/代码生成 | UI 视觉投影 | 对象 DSL 构建 |
| **字面量** | `t"..."` | （嵌入 `.vx`） | 普通表达式 |
| **界定符** | `<% ... %>` | `<tag> ... </tag>` | `{ ... }` |
| **插值** | `{ expr }` | `{expr}` | `{expr}` |
| **逻辑** | `<% loop ... %>` | `<if>` / `<loop>` 关键字 | `loop ... { ... }` |
| **闭合** | `<% end %>` | 标签闭合 | `}` |

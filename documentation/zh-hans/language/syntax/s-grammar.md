# S-Grammar (字符串语法)

Valkyrie 的字符串语法设计遵循“词法与语义分离”原则。由底层的 **S-Grammar** 负责物理边界扫描，再由应用层的 **DSL** 负责内容解析。

---

## 1. 词法层：S-Grammar

S-Grammar 是一种极其简单的字符串扫描规则，其核心是 **等量开闭原则 (Symmetric Delimiter)**。

### 1.1 核心原则
- **空字符串 ($N=2$)**：两个连续的引号（`""` 或 `''`）直接识别为空字符串。
- **等量开闭原则**：除了 $N=2$ 以外，使用 $N$ 个引号开始，就必须使用 $N$ 个引号结束。
- **天然多行**：任何 $N$ 的字符串（包括 $N=1$）都可以包含换行符。
- **无转义与无插值**：S-Grammar 层面不解析任何字符，所有的 `\` 和 `{}` 都会被原样扫描。

### 1.2 词法示例
```valkyrie
# 1. 空字符串 (N=2)
let empty = "" 

# 2. 标准字符串 (N=1)
let s1 = "Hello, Valkyrie ⚔️"
let s2 = "可以
跨行"

# 3. 对称匹配 (N=3)
let s3 = """
这里可以包含 "引号" 而不需要转义
"""

# 4. 高阶匹配 (N=4)
let s4 = """" 这里可以包含 """ 符号 """"
```

---

## 2. 应用层：DSL (Domain Specific Language)

当 S-Grammar 完成边界扫描后，其内部内容将交给应用层 DSL 进行解释。Valkyrie 编译器仅内置了少数核心前缀的处理逻辑，其它的前缀被视为 **Tagged Strings**，由用户定义的库或宏在后续阶段（如语义分析或过程宏）进行解析。

### 2.1 Slot String (s / 默认) 🦄
Valkyrie 的插值字符串设计旨在提供高性能的内联插值。它是默认的字符串行为。

- **变量插值**：使用 `{name}` 嵌入变量。
- **表达式插值**：支持在 `{}` 中编写任意 Valkyrie 表达式。
- **符号转义**：支持 `\{` 和 `\}` 来表示字面量的花括号。
- **字符转义**：支持 `\n`, `\r`, `\t`, `\\` 等标准转义，以及 `\u{...}` Unicode 转义。

```valkyrie
let name = "Alice"
# 默认即为 Slot String
let s1 = "Hello, {name}" 
# 显式使用 s 前缀效果相同
let s2 = s"Hello, {name}"
```

### 2.2 Localized String (i18n) 🌍
Valkyrie 集成了 **Project Fluent** (Mozilla) 作为其国际化引擎。
通过在插值变量前添加 `߷` (Gwot) 符号，该插值将被标记为“Fluent 变量”。

- **Fluent 集成**：Valkyrie 编译器会将包含 `߷` 的字符串自动映射到 `.ftl` 资源文件中的消息。
- **高级语法支持**：支持 Fluent 的复数形式（Plurals）、性别（Gender）以及选择器（Selectors）。
- **类型安全变量**：插值中的变量名直接对应 Fluent 消息中的参数。

```valkyrie
let name = "Valkyrie"
let count = 3

# 1. 基础翻译
# 映射到 Fluent: hello-user = Hello, { $name }!
let s1 = "Hello, {߷name}!"

# 2. 复数形式与属性
# 映射到 Fluent: 
# shared-photos = { $userName } added { $photoCount ->
#     [one] a new photo
#    *[other] { $photoCount } new photos
# } to the album.
let s2 = "{߷userName} added {߷photoCount} new photos to the album."
```

- **自动键生成规则**：如果没有显式指定 Key，编译器会根据原始文本生成唯一的 Slug 作为 Fluent 标识符。

### 2.3 Format String (f) 🏗️
**Format String** 通过 `f` 前缀启用。它不捕捉当前作用域，而是声明一个**函数模板**。它更像 C++ 的 `std::format` 或 `std::vformat`，适用于延迟绑定或日志库。

```valkyrie
# 声明一个模板
let log_template = f"ID:{} - Event: {}"
# 之后再进行绑定
let message = log_template.format(1024, "Login Success")
```

### 2.4 Template String (t) 🎭
通过 `t` 前缀启用。它支持类似 Handlebars 或 Jinja2 的模板语法，通常用于多行文本生成。

```valkyrie
let tpl = t"
<% loop user in users %>
  - {user.name}
<% end loop %>
"
```

### 2.5 Raw String (r) 🧱
通过 `r` 前缀启用。它会完全禁用插值和转义处理，实现真正的“所见即所得”。

```valkyrie
# 路径无需担心转义
let path = r"C:\Windows\System32"
```

### 2.6 其它 DSL (Tagged Strings) 📦
除了上述核心前缀（`s`, `f`, `t`），Valkyrie 支持任意标识符作为前缀。编译器会将这些字符串识别为“带有标签的原始文本”，其内容不进行默认的插值解析，由对应的库在语义阶段或宏中进行处理。

- **Regex (`re`)**：由正则库处理。
- **Byte String (`b`)**：解析为字节数组 `[u8]`。
- **JSON5 (`json`)**：解析为 JSON5 对象。

```valkyrie
# 编译器不硬编码 re 的逻辑，而是将其作为 Tagged String 传给正则库
let pattern = re"^\d+$"

# 用户也可以定义自己的前缀
let sql = sql"SELECT * FROM users WHERE id = {id}"
```

---

## 3. 最佳实践总结 💡

1.  **优先使用 Slot String**：内联插值（`"{name}"`）最直观，适用于 90% 的字符串拼接场景。
2.  **国际化必用 ߷**：在需要翻译的变量前加上 `߷`，让编译器和翻译引擎自动完成 Key 提取和 Locale 替换，无需手动维护 ResourceBundle。
3.  **日志与库开发使用 Format String**：`f"{}"` 带来的延迟绑定和编译时检查非常适合高性能或复用场景。
4.  **善用 N 阶差异**：遇到需要包含引号的文本时，不要寻找转义，而是增加外部引号的数量（如 `"""..."""`）。
5.  **路径必用 r-string**：避免在 Windows 路径中陷入 `\\` 的泥潭。
6.  **Emoji 友好**：Valkyrie 源码和字符串均采用 UTF-8 编码，请放心使用 🚀。

# 大括号 `{}` 的用法枚举

在 Valkyrie 语言中，大括号 `{}` 承担了多种语法功能。为了保持解析器的简洁，我们将大括号的用法分为以下几类。

## 1. 命名空间与定义块 (Namespace & Definition Blocks)

用于组织代码结构，内部包含定义语句（如 `let`, `class`, `fn` 等）。

```valkyrie
namespace Core {
    class Point {
        x: f32,
        y: f32,
    }
}
```

## 2. 控制流块 (Control Flow Blocks)

用于 `if`, `match`, `loop`, `for`, `while` 等控制流语句。

```valkyrie
if condition {
    # 语句块
} match x {
    case 1 => { # 分支块 }
}
```

## 3. 应用块 (ApplyBlock)

这是 Valkyrie 最具特色的语法构造。当 `{}` 紧跟在表达式（如类型名、函数调用等）后时，解析器将其解析为 **ApplyBlock**。

### 核心设计：结构优先，语义后置

为了保持解析器的极简与高性能，Valkyrie 的解析器对 ApplyBlock 采取了“只管挂树，不问对错”的策略。

#### 语法同构性
在解析阶段，以下形式在语法树上是完全同构的：
- **对象初始化**：`Point { x: 1.0, y: 2.0 }`
- **尾随闭包**：`list.map { % * 2 }`
- **声明式 DSL**：`Node { child { "Hello" } }`

#### 极致的自由度
ApplyBlock 内部被解析为一个**通用的语句流 (Statement Stream)**。这意味着你可以在块内混杂各种节点：
- **赋值语句**：`width = 100`
- **方法/函数定义**：`modifier on_click(self) { ... }`
- **嵌套块**：`header { title: "Welcome" }`
- **控制流**：`if condition { ... }`

具体的语义解析由后续的类型系统决定。

---

### 应用场景一：对象构造 (Object Construction)

当 ApplyBlock 的左侧是一个类或结构体时，它被解释为对象构造模式。

#### 直接初始化 `C { ... }`
块内语句直接映射为字段赋值。
```valkyrie
let config = Config {
    debug: true
    port: 8080
}
```

#### 后初始化 `C() { ... }`
先执行构造函数，再在大括号内进行补充配置。
```valkyrie
let user = User("Alice") {
    verified: true
    bio: "Valkyrie developer"
}
```

---

### 应用场景二：尾随闭包 (Trailing Closures)

当 ApplyBlock 的左侧是一个函数或微函数时，块被视为该函数的最后一个参数。

```valkyrie
# 函数调用示例
request.send {
    header("Content-Type", "application/json")
    on_success { print("Done!") }
}
```

#### 隐式接收者 (Implicit Receivers)
在某些 DSL 上下文中，ApplyBlock 内部的 `self` 会自动指向当前构建的对象，使得你可以直接调用方法而无需显式引用。

---

### 应用场景三：声明式 DSL (如 V-Grammar)

ApplyBlock 语法广泛应用于构建嵌套的声明式结构。通过利用 ApplyBlock 的自由度，开发者可以在描述结构的同时无缝嵌入逻辑。

有关在 UI 开发中的具体应用，请参阅：
- **[原生 UI 语法 (V-Grammar)](./v-grammar.md)**

---

### 底层逻辑：解析与校验

#### 解析算法
解析器将 `{` 视为一个**高优先级中缀操作符**。当遇到 `{` 且左侧已有表达式时，解析器会递归地将后续内容封装进 ApplyBlock 节点。

#### 分层校验机制
1.  **Type Checker**：在语义分析阶段，根据左侧调用者的类型签名判定块内每个节点的含义。
2.  **Linter**：在类型检查后介入。如果 Linter 发现你在一个纯数据结构的初始化块中定义了方法，或者在不支持的上下文中使用了类型声明，它会抛出编译错误。

## 4. 字符串插值 (String Interpolation)

在字符串中使用 `{}` 嵌入表达式。由 Lexer 处理，不进入 Pratt 解析逻辑。

```valkyrie
print("Hello, {name}!")
```

## 5. 集合字面量 (Collection Literals) - 预留

未来可能用于 Set 或 Map 的字面量表示。

```valkyrie
let set = {1, 2, 3}
let map = {"key": "value"}
```

---

## 设计原理：构造-解构对称性

Valkyrie 采用 `:` 作为对象构造语法，与模式匹配保持一致：

```valkyrie
构造:  User { name: "Alice" }
解构:  case { name: "Alice" }: ...
```

这种对称性让代码具有视觉可逆性，降低认知负担。

# 模式匹配 (Match)

Valkyrie 提供了一套统一的模式匹配机制，覆盖 `match`、`if let`、`while let`、`until not`、`case` / `case if` 与 `let` 解构。所有形式共用同一套模式表达式文法。

> **状态总览**
>
> - **已落地**：`match` 表达式（含 literal / range / tuple / object / array / or / type / typed-bind / extractor 模式 + guard）、`if let`、`case` / `case if`（语句语义 + fallthrough）、`while let`、`until not`、`let` 仅允许 irrefutable pattern、sealed/enum 名义穷尽性 + 重复 arm 检测、值类型 pattern binding 走 `AggregateCopy`、extractor 显式合约（nullable payload，`null` 表示不匹配）。
> - **部分落地**：MIR probe/bind 已显式化，但仍保留 `PatternMatch` fallback 兜底（extractor 未 resolved 时）；后端（CLR / JVM / WASM）能跑通主路径，但遇 fallback 时行为待收口。
> - **规划中**：guard 不计为无条件覆盖的穷尽性分析、range / object / tuple / extractor pattern-space 穷尽性、unreachable arm reachability 分析、多后端 parity 回归矩阵。

## 基本 Match 语法

### 标准 Match 语句

```valkyrie
# 基本模式匹配
match value {
    case 1: "one"
    case 2: "two"
    case 3: "three"
    else: "other"
}

# 范围匹配
match score {
    case 90..=100: "A"
    case 80..=89: "B"
    case 70..=79: "C"
    case 60..=69: "D"
    else: "F"
}

# 多值匹配
match day {
    case "Saturday" | "Sunday": "Weekend"
    case "Monday"..="Friday": "Weekday"
    else: "Invalid day"
}
```

`match` 按顺序对 scrutinee 与每个 arm 的模式进行匹配，命中第一个匹配的 arm 后执行其右侧表达式并返回。`else` 作为无条件兜底。本节展示了字面量模式（literal）、范围模式（range）与或模式（or，用 `|` 连接）。

> **状态**: 已落地。literal / range / or 模式与 guard 在编译器与各后端主路径均可用。

### 表达式 Match 语法

```valkyrie
# 表达式形式的 match
let result = match value {
    case 1: "one"
    case 2: "two"
    case 3: "three"
    else: "other"
}

# 链式调用
let processed = match input.transform() {
    case Fine(value): value * 2
    case Fail(error): 0
}
```

`match` 是表达式，可直接用于 `let`、函数参数与返回位置。scrutinee 可以是任意右值表达式。

> **状态**: 已落地。

## 解构匹配

### 元组解构

```valkyrie
match point {
    case (0, 0): "Origin"
    case (x, 0): "On X-axis at {x}"
    case (0, y): "On Y-axis at {y}"
    case (x, y): "Point at ({x}, {y})"
}

# 嵌套元组
match nested {
    case ((a, b), c): "Nested: {a}, {b}, {c}"
    case (x, (y, z)): "Other nested: {x}, {y}, {z}"
    else: "No match"
}
```

> **状态**: 已落地。tuple pattern 的穷尽性分析为规划中，需用 `else` 或通配 `_` 兜底。

### 数组解构

```valkyrie
match array {
    case []: "Empty array"
    case [x]: "Single element: {x}"
    case [first, second]: "Two elements: {first}, {second}"
    case [head, ..tail]: "Head: {head}, Tail length: {tail.length}"
    case [.., last]: "Last element: {last}"
    case [first, .., last]: "First: {first}, Last: {last}"
}

# 固定长度匹配
match coordinates {
    case [x, y]: "2D point: ({x}, {y})"
    case [x, y, z]: "3D point: ({x}, {y}, {z})"
    else: "Unsupported dimension"
}
```

数组模式支持定长匹配与切片（`..tail`、`.., last`、`first, .., last`）。

> **状态**: 已落地。切片语义在 WASM 后端已验证，CLR / JVM 走相同 MIR 路径。

### 对象解构

```valkyrie
match person {
    case { name: "Alice", age }: "Alice is {age} years old"
    case { name, age: 18..=65 }: "{name} is working age"
    case { name, age, ...rest }: "{name} from {rest.city}, age {age}"
}
```

对象模式支持字段字面量匹配、字段绑定、嵌套范围模式与 `...rest` 行扩展。`...rest` 与记录类型的行扩展语义保持一致。

> **状态**: 已落地。object pattern-space 的穷尽性分析为规划中。

## Guard 条件

可以使用 `if` 子句为模式匹配添加额外的过滤条件：

```valkyrie
match point {
    case (x, y) if x == y: "On diagonal"
    case (x, y) if x > y: "Below diagonal"
    case (x, y): "Above diagonal"
}
```

guard 在模式成功匹配后求值；guard 为假时该 arm 不命中，继续向下尝试。guard 内可引用当前 arm 绑定的变量。

> **状态**: 已落地（运行时语义）。guard 不计为无条件覆盖的穷尽性分析为规划中——带 guard 的 arm 不会被穷尽性检查视为"覆盖"，因此含 guard 的 `match` 仍需 `else` 或无 guard 的通配 arm 兜底。

## 类型匹配

模式匹配也可以用于检查和转换类型：

```valkyrie
match shape {
    case s: Circle: "Circle with radius {s.radius}"
    case s: Rectangle: "Rectangle {s.width}x{s.height}"
    case _: "Unknown shape"
}
```

`s: Circle` 是 typed-bind 模式：先做类型检查，命中后将 scrutinee 以 `s` 绑定为 `Circle` 类型。

> **状态**: 已落地。类型模式的穷尽性仅对 `sealed` / `enum` 名义类型生效（见下文「穷尽性分析」）。

## if let 模式匹配

`if let` 将单 arm 模式匹配用作条件分支：

```valkyrie
# 基本 if let
if let Some { value } = maybe {
    print("Got: {value}")
}
else {
    print("Nothing")
}

# 搭配元组 / 对象
if let (x, 0) = point {
    print("On x-axis at {x}")
}
```

模式匹配成功时执行 `then` 块，否则执行 `else` 块（可省略）。`if let` 与 `if` 一样是表达式。

> **状态**: 已落地。

## while let 模式匹配

`while let` 在每次迭代前尝试模式匹配，匹配成功则执行循环体，匹配失败则退出循环：

```valkyrie
while let Some { value: item } = iterator.next() {
    process(item)
}
```

`while let` 与 `while` 共享相同的循环控制流（`break` / `continue` / 标签）。

> **状态**: 已落地。

## until not 模式匹配

`until not` 是 `while let` 的对偶形式：当模式不匹配时执行循环体，当模式匹配成功时退出循环。常用于"持续处理直到读到结束符"的流式场景。

```valkyrie
# 持续消费，直到 peek 到结束符才退出
until not Some(end) = stream.peek() {
    consume(stream.next())
}
```

`until not` 与 `until` 一样支持标签与 `break` / `continue`。语义上，`while let Pat = expr` 在 Pat 匹配时继续循环，`until not Pat = expr` 在 Pat 匹配时退出循环。

> **状态**: 已落地。

## case / case if 语句

`case` / `case if` 是语句形式（statement）的模式匹配，与作为表达式的 `match` 不同。`case` 系列支持 fallthrough——命中后除非显式 `break`，否则继续执行下一个 arm。

```valkyrie
# case 语句
case value {
    case 1: print("一")
    case 2: print("二")
    case 3: print("三")
    else: print("其他")
}
```

```valkyrie
# case if 带 guard 与 fallthrough
case status {
    case 200 if is_success:
        log("ok")
        # fallthrough 到下一个 case
    case 3xx:
        log("redirect")
        break
    case 4xx:
        log("client error")
        break
    else:
        log("unknown")
}
```

`case if` 在 `case` 之上叠加 guard 条件。由于存在 fallthrough，`case` 是语句而非表达式，不返回值。

> **状态**: 已落地。语句语义与 fallthrough 在 MIR 中已显式化；后端在主路径下行为正确。

## let 与 irrefutable pattern

`let` 解构仅允许 irrefutable（不可反驳）模式：

```valkyrie
# 数组解构（长度已知匹配）
let [first, second, ..rest] = array
let (x, y, z) = (1, 2, 3)
let { name, age } = person
let { x: new_x, y: new_y } = point  # 重命名
let { name, ..rest } = user
```

可反驳模式（如 `let 1 = x`、`let Some(v) = opt`）在 `let` 上下文中会被编译器拒绝，必须改用 `match` / `if let` / `while let`。

> **状态**: 已落地。irrefutable 检查在语义分析阶段完成。

## Pattern Binding 语义

Valkyrie 在 `match` / `case` / `catch` / `let` / `if let` / `while let` 中使用统一的模式表达式文法。

### Bind (`<-`)

`name <- pattern` 在 `pattern` 成功匹配后，将整个 scrutinee 绑定到 `name`：

```valkyrie
match pair {
    case both <- (true, true): both
    else: (false, false)
}
```

最外层绑定必须使用 `<-`（不能用 `:`）；extractor 内部字段可用 `name: subpattern` 进行重命名绑定：

```valkyrie
match value {
    case Wrapper(inner: Some(result), fallback):
        result
    else: 0
}
```

### `mut` in pattern

模式中的 `mut` 表示可变引用绑定语义（等价于 Rust 的 `ref mut`）：

```valkyrie
let mut x = value
match value {
    case mut item: update(item)
    else: ()
}
```

### `pin` in pattern

`pin mut` 表示 pinned 可变引用语义（等价于 Rust 的 `Pin<&mut T>`）：

```valkyrie
match frame {
    case pin mut state: poll(state)
    else: ()
}
```

> **状态**: 已落地。值类型（`structure`）的 pattern binding 通过 `AggregateCopy` 完成拷贝语义。

## Extractor 合约

Extractor pattern 是显式、合约制的：

- `extractor` 必须返回 nullable payload（`T?` / `Union(T, null)`）。
- 返回 `null` 表示该 arm 不匹配，继续尝试下一个 arm。
- `extractor(mut self)` 在 pattern extraction 上下文中不被允许。
- 不提供 Scala 风格的隐式 `unapply`；pattern extraction 必须显式声明。

```valkyrie
class Point {
    micro extractor(self) -> (bool, bool)? {
        return (true, true)
    }
}

match p {
    case Point(a, b): print("({a}, {b})")
    else: print("not a point")
}
```

> **状态**: 部分落地。extractor 合约与 nullable payload 语义已实现；当 extractor 未能 resolved 时，MIR 仍会回退到 `PatternMatch` 兜底路径，此时 CLR / JVM / WASM 后端行为待收口。

## 隐式 Class Deref / Structure 按值

`class` scrutinee 在模式匹配与绑定时会被透明解引用；`structure` scrutinee 保持按值语义。这是隐式的运行时 lowering 行为，模式语法中不需要 `box` 关键字。

> **状态**: 已落地。

## 穷尽性分析

Valkyrie 的穷尽性检查当前覆盖以下范围：

- **sealed / enum 名义穷尽**：对 `sealed` class 或 `enum` / `unite` 的所有变体列出 arm 时，视为穷尽，无需 `else`。
- **重复 arm 检测**：同一 `match` 中出现完全相同的 arm 模式时，编译器报错。
- **typed-bind 全覆盖**：对 `sealed` 类型的所有子类型列出 typed-bind arm 时，视为穷尽。

```valkyrie
unite Option⟨T⟩ {
    Some { value: T }
    None
}

# 穷尽：覆盖了 Some 与 None，无需 else
match opt {
    case Some { value }: value
    case None: default_value
}
```

下列分析能力尚未实现：

- **guard 不计为无条件覆盖**：带 guard 的 arm 不被视为"覆盖"该模式，含 guard 的 `match` 仍需 `else` 或无 guard 通配兜底。
- **range / object / tuple / extractor pattern-space 穷尽性**：对这些模式的组合空间覆盖分析未实现。
- **unreachable arm reachability 分析**：被前面 arm 完全覆盖的后续 arm 不会被标记为 unreachable。

> **状态**: 部分落地。名义穷尽性与重复 arm 检测已落地；guard 语义、pattern-space 穷尽性与 reachability 分析为规划中。

## 后端支持

| 后端 | 主路径 | fallback 收口 |
| --- | --- | --- |
| CLR | 可用 | 待收口 |
| JVM | 可用 | 待收口 |
| WASM | 可用 | 待收口 |

> **状态**: 部分落地。各后端在 extractor 已 resolved 的主路径下行为一致；extractor 未 resolved 触发 `PatternMatch` fallback 时，行为待收口。多后端 parity 回归矩阵为规划中。

# Variant 三层语义标准说明

本文档是 Rust / JetBrains / VSCode 三端共享的 **variant 语义真源摘要**。

## 三层分离

| 层 | 语法示例 | 说明 |
|:---|:---|:---|
| **Declaration（声明层）** | `Some { value: T }` | 在 `unite` / sealed class 内定义 variant 结构，必须使用 record-style 字段体 |
| **Constructor（构造层）** | `Some(0)` / `Fine(err)` | variant 构造函数调用，由 overload / imply 提供 |
| **Extractor（匹配层）** | `case Some(x)` / `case Fine(v)` | variant extractor 模式入口，用于 `match` / `while let` 等 |

## 禁止混淆

- `(T)` **不是** variant 声明语法。`unite Option<T> { Some(T) None }` 必须报错。
- 诊断文案：`Expected '{` for variant body; '(T)' is not valid variant declaration syntax...`
- IDE / LSP / 文档不得再使用 **tuple variant declaration** 概念。

## 合法示例

```valkyrie
unite Option<T> {
    Some { value: T }
    None
}

let x = Some(42)              # constructor
let y = Some { value: 42 }    # record construction

match opt {
    case Some(v): v           # extractor
    case None: 0
}
```

## 标准库口径

- `Option.v`：`Some { value: T }`
- `Result.v`：`Fine { value: T }` / `Fail { error: E }`
- 使用层仍可用 `Some(x)` / `Fine(x)` / `case Some(x)` / `case Fine(x)`

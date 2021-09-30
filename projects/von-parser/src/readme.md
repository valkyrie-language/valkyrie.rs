# von-parser src

这里是 `von-parser` 的源码目录。

## 职责
- `lib.rs` 定义 `VonValue`、`VonParseError`、`VonParser`。
- 保持 `VON` 文本模型和解析行为稳定。
- 维护和 `serde` 的值层互通。

## 禁止
- 不在这里解析 `legion` 项目清单语义。
- 不引入与语言 `AST / HIR / MIR / LIR` 相关的结构。

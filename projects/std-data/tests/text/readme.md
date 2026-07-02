# von-parser

`von-parser` 提供 `VON` 文档的值模型、解析器与 `serde` 支持。

## 职责
- 提供 `VonValue`、`VonParseError`、`VonParser`。
- 负责把 `VON` 文本解析成稳定的通用值树。
- 提供 `serde` 的 `Serialize / Deserialize` 支持，便于和外部工具或缓存格式互通。

## 禁止
- 不在这里实现 `legion` 的 manifest 语义。
- 不在这里放工作区编排、构建计划或目标后端逻辑。
- 不把它扩张成新的万能配置系统。

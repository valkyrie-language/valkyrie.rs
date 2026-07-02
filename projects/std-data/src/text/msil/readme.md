# msil-parser src

这里是 `msil-parser` 的源码目录。

## 职责
- `src/model.rs` 定义 `MSIL` 文本结构。
- `src/writer.rs` 负责把结构化模型写回文本。
- `src/parser.rs` 负责从 `MSIL` 文本提取方法块与签名信息。

## 禁止
- 不在这里实现 `legion` 的命令行输出格式。
- 不在这里放 `CLR` 构建计划或后端选择逻辑。

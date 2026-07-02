# `std-data`

`std-data` 只负责标准二进制 / 文本格式的模型与编解码。

当前目录按格式树组织：

- `src/binary/*`：`class`、`jar`、`coff`、`pe`、`wasm`
- `src/text/*`：`msil`、`valkyrie`、`von`、`wat`、`wit`，以及 host-script 语法树（`bash` / `lua` / `tcl` / `powershell` / `c` 等）
- `src/sql/*`：Atlas **SQL 物化**（方言 AST / printer；非 migrations）
- `src/hermes/*`：Hermes schema / query（走向 Query IR）

它不负责编译流程编排、语义分析、interpret、后端调度或运行时逻辑。Host-script 分层（guest 在 `nyar-language`，PE 基板在 `legacy-vm`）见 [`../host-script-languages.md`](../host-script-languages.md)。

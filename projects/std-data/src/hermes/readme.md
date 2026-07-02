# `std-data::hermes`

Atlas 默认 **schema / query** 真源（Hermes）。扩展名：`.hermes` / `.her`。  
走向 **Query IR**。**非** migrations 工作流；与 yyds 无关。

## 职责（本竖切）

- Lexer / parser：最小子集（`namespace`、`storage`/`model`、`@@`/`@` 字段、`select`）
- AST + `Display`（Query IR 方向的起点）
- 可选：投影到 SQL 物化输入（见 `sql`）——**不是**查询执行的唯一定义

## 不负责

- SQL 方言打印细节（见 `std_data::sql`）
- Migration / ORM 运行时

今日 `to_query_ir` → `sql::QueryIr` 是 SQL 物化 / 测方言的辅助桥。

## 快速试跑

```text
cargo test -p std-data hermes -- --nocapture
```

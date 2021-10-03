# `std-data::sql`

**SQL 物化**：方言感知的 SQL AST + 打印机（升级计划、显式 SQL、测试）。  
在 Atlas 需要 SQL 字符串时，从 Query IR / 手填结构投影而来。  
**非** migrations 工作流，与 yyds 无关。

## 职责

- `SqlDialect`：`Sqlite` / `PostgreSql` / `MySql`
- `ast`：最小 SQL AST
- `query_ir`：面向打印机的输入（历史名；语义是 SQL 物化输入）
- `render` / `render_ir`：按方言打印 SQL 字符串（此处 render = 打印 SQL）

查询默认 Hermes → Query IR → 执行；不必须先经本模块。需要 SQL 文本时才物化。

## 示例（SQL 物化）

```rust
use std_data::sql::{
    ColumnIr, ColumnType, CreateTableIr, QueryIr, SqlDialect, render_ir,
};

let ddl = QueryIr::CreateTable(CreateTableIr {
    table: "user".into(),
    if_not_exists: true,
    columns: vec![
        ColumnIr {
            name: "id".into(),
            ty: ColumnType::Integer,
            primary_key: true,
            not_null: true,
            unique: false,
            autoincrement: true,
            default: None,
        },
    ],
});
let sql = render_ir(&ddl, SqlDialect::Sqlite);
```

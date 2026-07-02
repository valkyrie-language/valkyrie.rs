//! Integration tests for `std_data::sql` (CREATE TABLE / SELECT vertical slice).

use std_data::sql::{
    ColumnDef, ColumnIr, ColumnType, CreateTable, CreateTableIr, Expr, QueryIr, Select, SelectIr, SelectItem, SqlDialect, SqlStatement, render,
    render_ir,
};

#[test]
fn sqlite_create_table_from_ast() {
    let stmt = SqlStatement::CreateTable(CreateTable {
        table: "user".into(),
        if_not_exists: true,
        columns: vec![
            ColumnDef {
                name: "id".into(),
                sql_type: "INTEGER".into(),
                primary_key: true,
                not_null: true,
                unique: false,
                autoincrement: true,
                default: None,
            },
            ColumnDef::new("email", "TEXT"),
        ],
    });

    let sql = render(&stmt, SqlDialect::Sqlite);
    assert!(sql.starts_with("CREATE TABLE IF NOT EXISTS \"user\""));
    assert!(sql.contains("\"id\" INTEGER PRIMARY KEY AUTOINCREMENT"));
    assert!(sql.contains("\"email\" TEXT"));
}

#[test]
fn postgres_create_table_serial_from_ir() {
    let ir = QueryIr::CreateTable(CreateTableIr {
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
            ColumnIr {
                name: "name".into(),
                ty: ColumnType::Text,
                primary_key: false,
                not_null: true,
                unique: false,
                autoincrement: false,
                default: None,
            },
        ],
    });

    let sql = render_ir(&ir, SqlDialect::PostgreSql);
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"user\""));
    assert!(sql.contains("\"id\" SERIAL"));
    assert!(sql.contains("PRIMARY KEY (\"id\")"));
    assert!(sql.contains("\"name\" TEXT NOT NULL"));
}

#[test]
fn mysql_select_where_param() {
    let ir = QueryIr::Select(SelectIr {
        columns: vec![],
        from: "orders".into(),
        where_eq: Some(("id".into(), Expr::Param("@id".into()))),
        limit: Some(1),
    });
    let sql = render_ir(&ir, SqlDialect::MySql);
    assert_eq!(sql, "SELECT * FROM `orders` WHERE `id` = @id LIMIT 1;");
}

#[test]
fn dialect_enum_stubs_quote() {
    assert_eq!(SqlDialect::Sqlite.quote_ident("t"), "\"t\"");
    assert_eq!(SqlDialect::PostgreSql.quote_ident("t"), "\"t\"");
    assert_eq!(SqlDialect::MySql.quote_ident("t"), "`t`");
    assert!(SqlDialect::PostgreSql.supports_returning());
    assert!(!SqlDialect::Sqlite.supports_returning());
}

#[test]
fn select_ast_with_binary_where() {
    let stmt = SqlStatement::Select(Select {
        columns: vec![SelectItem::Expr { expr: Expr::Ident("name".into()), alias: None }],
        from: "people".into(),
        where_clause: Some(Expr::Binary { left: Box::new(Expr::Ident("age".into())), op: ">".into(), right: Box::new(Expr::Integer(18)) }),
        limit: None,
    });
    let sql = render(&stmt, SqlDialect::Sqlite);
    assert_eq!(sql, "SELECT \"name\" FROM \"people\" WHERE \"age\" > 18;");
}

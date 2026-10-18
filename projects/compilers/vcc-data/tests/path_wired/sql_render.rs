//! Migrated from `src/sql/render.rs` (`tests`).

use super::*;

use crate::sql::{
    ast::{ColumnDef, CreateTable, Expr, Select, SelectItem, SqlStatement},
    query_ir::{ColumnIr, ColumnType, CreateTableIr, QueryIr, SelectIr},
};

#[test]
fn render_sqlite_create_table() {
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
            ColumnDef {
                name: "name".into(),
                sql_type: "TEXT".into(),
                primary_key: false,
                not_null: true,
                unique: false,
                autoincrement: false,
                default: None,
            },
        ],
    });
    let sql = render(&stmt, SqlDialect::Sqlite);
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS \"user\""));
    assert!(sql.contains("INTEGER PRIMARY KEY AUTOINCREMENT"));
    assert!(sql.contains("\"name\" TEXT NOT NULL"));
    assert!(!sql.contains("PRIMARY KEY (\"id\")"));
}

#[test]
fn render_ir_select_postgres() {
    let ir = QueryIr::Select(SelectIr {
        columns: vec!["id".into(), "name".into()],
        from: "users".into(),
        where_eq: Some(("id".into(), Expr::Param("@id".into()))),
        limit: None,
    });
    let sql = render_ir(&ir, SqlDialect::PostgreSql);
    assert_eq!(sql, "SELECT \"id\", \"name\" FROM \"users\" WHERE \"id\" = @id;");
}

#[test]
fn render_ir_create_mysql_bool() {
    let ir = QueryIr::CreateTable(CreateTableIr {
        table: "flags".into(),
        if_not_exists: false,
        columns: vec![ColumnIr {
            name: "active".into(),
            ty: ColumnType::Boolean,
            primary_key: false,
            not_null: true,
            unique: false,
            autoincrement: false,
            default: Some(Expr::Bool(true)),
        }],
    });
    let sql = render_ir(&ir, SqlDialect::MySql);
    assert!(sql.contains("`active` BOOLEAN NOT NULL DEFAULT 1"));
}

#[test]
fn select_star_limit() {
    let stmt = SqlStatement::Select(Select { columns: vec![SelectItem::Star], from: "t".into(), where_clause: None, limit: Some(5) });
    assert_eq!(render(&stmt, SqlDialect::Sqlite), "SELECT * FROM \"t\" LIMIT 5;");
}

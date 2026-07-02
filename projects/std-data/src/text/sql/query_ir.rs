//! SQL materialization structs for Query IR → dialect SQL text.
//!
//! "IR" is informal; no god IR.
//! Feeds [`crate::sql::render_ir`] to print SQL strings (upgrade plans, tests).

use super::ast::{ColumnDef, CreateTable, Expr, Select, SelectItem, SqlStatement};

/// Input to SQL dialect printing (query-stack helper for materialization).
#[derive(Debug, Clone, PartialEq)]
pub enum QueryIr {
    /// Create a table.
    CreateTable(CreateTableIr),
    /// Select rows.
    Select(SelectIr),
}

/// Neutral `CREATE TABLE` request.
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTableIr {
    /// Table name.
    pub table: String,
    /// Prefer `IF NOT EXISTS`.
    pub if_not_exists: bool,
    /// Columns.
    pub columns: Vec<ColumnIr>,
}

/// Neutral column description.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnIr {
    /// Column name.
    pub name: String,
    /// Logical column type (mapped per dialect when lowered, or kept as custom SQL).
    pub ty: ColumnType,
    /// Primary key.
    pub primary_key: bool,
    /// Not null.
    pub not_null: bool,
    /// Unique.
    pub unique: bool,
    /// Autoincrement / serial.
    pub autoincrement: bool,
    /// Optional default as a simple expr.
    pub default: Option<Expr>,
}

/// Logical column types Hermes can emit without knowing dialect type names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    /// 32-bit integer affinity.
    Integer,
    /// 64-bit integer.
    BigInt,
    /// Text / varchar.
    Text,
    /// Floating point.
    Real,
    /// Boolean.
    Boolean,
    /// Binary blob.
    Blob,
    /// Pass-through SQL type string (already dialect-specific).
    Custom(String),
}

/// Neutral `SELECT` request.
#[derive(Debug, Clone, PartialEq)]
pub struct SelectIr {
    /// Projection; empty means `*`.
    pub columns: Vec<String>,
    /// From table.
    pub from: String,
    /// Optional equality filter `col = value`.
    pub where_eq: Option<(String, Expr)>,
    /// Optional limit.
    pub limit: Option<u64>,
}

/// Lower neutral IR into the SQL AST (dialect-agnostic structure).
///
/// Type name mapping for primitives uses a portable baseline; the renderer then
/// adjusts autoincrement / quoting per [`super::SqlDialect`].
pub fn lower(ir: &QueryIr) -> SqlStatement {
    match ir {
        QueryIr::CreateTable(t) => SqlStatement::CreateTable(CreateTable {
            table: t.table.clone(),
            if_not_exists: t.if_not_exists,
            columns: t
                .columns
                .iter()
                .map(|c| ColumnDef {
                    name: c.name.clone(),
                    sql_type: baseline_sql_type(&c.ty),
                    primary_key: c.primary_key,
                    not_null: c.not_null,
                    unique: c.unique,
                    autoincrement: c.autoincrement,
                    default: c.default.clone(),
                })
                .collect(),
        }),
        QueryIr::Select(s) => {
            let columns = if s.columns.is_empty() {
                vec![SelectItem::Star]
            }
            else {
                s.columns.iter().map(|name| SelectItem::Expr { expr: Expr::Ident(name.clone()), alias: None }).collect()
            };
            let where_clause = s.where_eq.as_ref().map(|(col, val)| Expr::Binary {
                left: Box::new(Expr::Ident(col.clone())),
                op: "=".into(),
                right: Box::new(val.clone()),
            });
            SqlStatement::Select(Select { columns, from: s.from.clone(), where_clause, limit: s.limit })
        }
    }
}

fn baseline_sql_type(ty: &ColumnType) -> String {
    match ty {
        ColumnType::Integer => "INTEGER".into(),
        ColumnType::BigInt => "BIGINT".into(),
        ColumnType::Text => "TEXT".into(),
        ColumnType::Real => "REAL".into(),
        ColumnType::Boolean => "BOOLEAN".into(),
        ColumnType::Blob => "BLOB".into(),
        ColumnType::Custom(s) => s.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lower_select_star() {
        let ir = QueryIr::Select(SelectIr { columns: vec![], from: "users".into(), where_eq: None, limit: Some(10) });
        match lower(&ir) {
            SqlStatement::Select(s) => {
                assert_eq!(s.columns, vec![SelectItem::Star]);
                assert_eq!(s.limit, Some(10));
            }
            other => panic!("expected select, got {other:?}"),
        }
    }
}

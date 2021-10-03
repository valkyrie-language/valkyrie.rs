#![doc = include_str!("readme.md")]

/// SQL statement AST (`CREATE TABLE` / `SELECT` subset).
pub mod ast;
/// Dialect enum and quoting helpers.
pub mod dialect;
/// Neutral IR Hermes / Atlas can fill without touching this printer.
pub mod query_ir;
/// Dialect-aware SQL rendering.
pub mod render;

pub use ast::{ColumnDef, CreateTable, Expr, Select, SelectItem, SqlStatement};
pub use dialect::SqlDialect;
pub use query_ir::{ColumnIr, ColumnType, CreateTableIr, QueryIr, SelectIr, lower};
pub use render::{render, render_ir};

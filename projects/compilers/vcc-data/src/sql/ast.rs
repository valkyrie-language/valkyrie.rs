//! Minimal SQL AST for CREATE TABLE / SELECT (Atlas query path).

/// Top-level SQL statement.
#[derive(Debug, Clone, PartialEq)]
pub enum SqlStatement {
    /// `CREATE TABLE …`
    CreateTable(CreateTable),
    /// `SELECT …`
    Select(Select),
}

/// `CREATE TABLE [IF NOT EXISTS] name (columns…)`
#[derive(Debug, Clone, PartialEq)]
pub struct CreateTable {
    /// Table name (unquoted).
    pub table: String,
    /// Emit `IF NOT EXISTS`.
    pub if_not_exists: bool,
    /// Column definitions.
    pub columns: Vec<ColumnDef>,
}

/// Column definition inside `CREATE TABLE`.
#[derive(Debug, Clone, PartialEq)]
pub struct ColumnDef {
    /// Column name (unquoted).
    pub name: String,
    /// Declared SQL type text (dialect-neutral or pre-mapped), e.g. `INTEGER`, `TEXT`.
    pub sql_type: String,
    /// Primary key.
    pub primary_key: bool,
    /// `NOT NULL`.
    pub not_null: bool,
    /// `UNIQUE`.
    pub unique: bool,
    /// Autoincrement / serial (dialect printer chooses syntax).
    pub autoincrement: bool,
    /// Optional default expression (already SQL-ish, or a simple literal via [`Expr`]).
    pub default: Option<Expr>,
}

impl ColumnDef {
    /// Build a simple typed column.
    pub fn new(name: impl Into<String>, sql_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            sql_type: sql_type.into(),
            primary_key: false,
            not_null: false,
            unique: false,
            autoincrement: false,
            default: None,
        }
    }
}

/// `SELECT columns FROM table [WHERE …] [LIMIT n]`
#[derive(Debug, Clone, PartialEq)]
pub struct Select {
    /// Projection list (`*` or named columns).
    pub columns: Vec<SelectItem>,
    /// `FROM` table (unquoted).
    pub from: String,
    /// Optional `WHERE` predicate.
    pub where_clause: Option<Expr>,
    /// Optional `LIMIT`.
    pub limit: Option<u64>,
}

/// Select list item.
#[derive(Debug, Clone, PartialEq)]
pub enum SelectItem {
    /// `*`
    Star,
    /// Column or expression with optional alias.
    Expr {
        /// Expression.
        expr: Expr,
        /// Optional `AS alias`.
        alias: Option<String>,
    },
}

/// Minimal expression subset for WHERE / DEFAULT / select items.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// Identifier / column reference.
    Ident(String),
    /// Integer literal.
    Integer(i64),
    /// String literal (unescaped source text).
    String(String),
    /// Boolean literal.
    Bool(bool),
    /// `NULL`.
    Null,
    /// `left op right`
    Binary {
        /// Left operand.
        left: Box<Expr>,
        /// Operator (`=`, `<>`, `<`, `>`, `AND`, `OR`, …).
        op: String,
        /// Right operand.
        right: Box<Expr>,
    },
    /// Named parameter (`@id`, `$1`, …) — emitted as-is.
    Param(String),
}

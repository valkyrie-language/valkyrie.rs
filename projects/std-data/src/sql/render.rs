//! Dialect-aware SQL printer.

use super::{
    ast::{ColumnDef, CreateTable, Expr, Select, SelectItem, SqlStatement},
    dialect::SqlDialect,
    query_ir::{ColumnType, QueryIr, lower},
};

/// Render a SQL AST statement for `dialect`.
pub fn render(stmt: &SqlStatement, dialect: SqlDialect) -> String {
    match stmt {
        SqlStatement::CreateTable(t) => render_create_table(t, dialect),
        SqlStatement::Select(s) => render_select(s, dialect),
    }
}

/// Print [`QueryIr`] as dialect SQL (SQL materialization).
pub fn render_ir(ir: &QueryIr, dialect: SqlDialect) -> String {
    let mut stmt = lower(ir);
    refine_types_for_dialect(&mut stmt, ir, dialect);
    render(&stmt, dialect)
}

fn refine_types_for_dialect(stmt: &mut SqlStatement, ir: &QueryIr, dialect: SqlDialect) {
    let QueryIr::CreateTable(t) = ir
    else {
        return;
    };
    let SqlStatement::CreateTable(ct) = stmt
    else {
        return;
    };
    for (col, ir_col) in ct.columns.iter_mut().zip(t.columns.iter()) {
        col.sql_type = map_column_type(&ir_col.ty, dialect);
    }
}

fn map_column_type(ty: &ColumnType, dialect: SqlDialect) -> String {
    match (ty, dialect) {
        (ColumnType::Custom(s), _) => s.clone(),
        (ColumnType::Integer, _) => "INTEGER".into(),
        (ColumnType::BigInt, SqlDialect::Sqlite) => "INTEGER".into(),
        (ColumnType::BigInt, _) => "BIGINT".into(),
        (ColumnType::Text, _) => "TEXT".into(),
        (ColumnType::Real, SqlDialect::Sqlite) => "REAL".into(),
        (ColumnType::Real, _) => "DOUBLE PRECISION".into(),
        (ColumnType::Boolean, SqlDialect::Sqlite) => "INTEGER".into(),
        (ColumnType::Boolean, _) => "BOOLEAN".into(),
        (ColumnType::Blob, SqlDialect::PostgreSql) => "BYTEA".into(),
        (ColumnType::Blob, _) => "BLOB".into(),
    }
}

fn render_create_table(t: &CreateTable, dialect: SqlDialect) -> String {
    let mut out = String::from("CREATE TABLE ");
    if t.if_not_exists {
        out.push_str("IF NOT EXISTS ");
    }
    out.push_str(&dialect.quote_ident(&t.table));
    out.push_str(" (\n");

    let mut table_pk: Option<&str> = None;
    let mut parts: Vec<String> = Vec::with_capacity(t.columns.len() + 1);

    for col in &t.columns {
        parts.push(render_column(col, dialect, &mut table_pk));
    }

    if let Some(pk) = table_pk {
        parts.push(format!("    PRIMARY KEY ({})", dialect.quote_ident(pk)));
    }

    out.push_str(&parts.join(",\n"));
    out.push_str("\n);");
    out
}

fn render_column<'a>(col: &'a ColumnDef, dialect: SqlDialect, table_pk: &mut Option<&'a str>) -> String {
    let mut line = format!("    {} ", dialect.quote_ident(&col.name));

    // Autoincrement + PK: dialect-specific type/keyword packing.
    if col.autoincrement && col.primary_key {
        match dialect {
            SqlDialect::Sqlite => {
                line.push_str("INTEGER PRIMARY KEY AUTOINCREMENT");
                // PK already inline — do not emit table-level PRIMARY KEY.
                return finish_column_flags(line, col, dialect, /*skip_pk*/ true, /*skip_nn*/ true);
            }
            SqlDialect::PostgreSql => {
                line.push_str("SERIAL");
                *table_pk = Some(col.name.as_str());
                return finish_column_flags(line, col, dialect, /*skip_pk*/ true, /*skip_nn*/ false);
            }
            SqlDialect::MySql => {
                line.push_str(&format!("{} AUTO_INCREMENT", col.sql_type));
                *table_pk = Some(col.name.as_str());
                return finish_column_flags(line, col, dialect, /*skip_pk*/ true, /*skip_nn*/ false);
            }
        }
    }

    line.push_str(&col.sql_type);

    if col.primary_key {
        *table_pk = Some(col.name.as_str());
    }

    finish_column_flags(line, col, dialect, false, false)
}

fn finish_column_flags(mut line: String, col: &ColumnDef, dialect: SqlDialect, skip_pk: bool, skip_not_null: bool) -> String {
    if col.primary_key && !skip_pk && !col.autoincrement {
        // Table-level PRIMARY KEY preferred when not autoincrement.
    }
    if col.not_null && !skip_not_null {
        line.push_str(" NOT NULL");
    }
    if col.unique {
        line.push_str(" UNIQUE");
    }
    if let Some(default) = &col.default {
        line.push_str(" DEFAULT ");
        line.push_str(&render_expr(default, dialect));
    }
    let _ = skip_pk;
    line
}

fn render_select(s: &Select, dialect: SqlDialect) -> String {
    let mut out = String::from("SELECT ");
    let cols: Vec<String> = s
        .columns
        .iter()
        .map(|c| match c {
            SelectItem::Star => "*".into(),
            SelectItem::Expr { expr, alias } => {
                let mut piece = render_expr(expr, dialect);
                if let Some(a) = alias {
                    piece.push_str(" AS ");
                    piece.push_str(&dialect.quote_ident(a));
                }
                piece
            }
        })
        .collect();
    out.push_str(&cols.join(", "));
    out.push_str(" FROM ");
    out.push_str(&dialect.quote_ident(&s.from));
    if let Some(w) = &s.where_clause {
        out.push_str(" WHERE ");
        out.push_str(&render_expr(w, dialect));
    }
    if let Some(limit) = s.limit {
        out.push_str(&format!(" LIMIT {limit}"));
    }
    out.push(';');
    out
}

fn render_expr(expr: &Expr, dialect: SqlDialect) -> String {
    match expr {
        Expr::Ident(name) => dialect.quote_ident(name),
        Expr::Integer(n) => n.to_string(),
        Expr::String(s) => dialect.quote_string(s),
        Expr::Bool(b) => dialect.format_bool(*b).to_string(),
        Expr::Null => "NULL".into(),
        Expr::Param(p) => p.clone(),
        Expr::Binary { left, op, right } => {
            format!("{} {} {}", render_expr(left, dialect), op, render_expr(right, dialect))
        }
    }
}

#[cfg(test)]
mod tests {
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
}

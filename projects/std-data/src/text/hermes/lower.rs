//! Project Hermes AST into SQL materialization input (`crate::sql::QueryIr`).
//!
//! Hermes → Query IR direction; SQL text is materialization when needed.
//! Function name `lower_*` is historical.

use crate::sql::{ColumnIr, ColumnType, CreateTableIr, Expr, QueryIr, SelectIr};

use super::ast::{FieldDecl, FieldKeyKind, HermesDocument, Literal, ModelDecl, SelectQuery};

/// Build SQL-print inputs from a Hermes document.
///
/// - Each model → [`QueryIr::CreateTable`] (`IF NOT EXISTS`)
/// - Each select → [`QueryIr::Select`]
pub fn lower_document(doc: &HermesDocument) -> Vec<QueryIr> {
    let mut out = Vec::new();
    for model in doc.models() {
        out.push(QueryIr::CreateTable(lower_model(model)));
    }
    for query in doc.queries() {
        out.push(QueryIr::Select(lower_select(query)));
    }
    out
}

/// Lower a single model to `CREATE TABLE` IR.
pub fn lower_model(model: &ModelDecl) -> CreateTableIr {
    CreateTableIr { table: model.name.clone(), if_not_exists: true, columns: model.fields.iter().map(lower_field).collect() }
}

/// Lower a Hermes select to select IR.
pub fn lower_select(query: &SelectQuery) -> SelectIr {
    SelectIr {
        columns: query.columns.clone(),
        from: query.from.clone(),
        where_eq: query.where_eq.as_ref().map(|(col, lit)| (col.clone(), lower_literal(lit))),
        limit: query.limit,
    }
}

fn lower_field(field: &FieldDecl) -> ColumnIr {
    let (primary_key, unique) = match field.key {
        FieldKeyKind::Primary => (true, false),
        FieldKeyKind::Unique => (false, true),
        FieldKeyKind::Plain => (false, false),
    };
    let optional = field.ty.starts_with("option<");
    ColumnIr {
        name: field.name.clone(),
        ty: map_type(&field.ty),
        primary_key,
        not_null: !optional && (primary_key || unique || field.key == FieldKeyKind::Plain),
        unique,
        autoincrement: primary_key && matches!(map_type(&field.ty), ColumnType::Integer | ColumnType::BigInt),
        default: None,
    }
}

fn map_type(ty: &str) -> ColumnType {
    let base = ty.strip_prefix("option<").and_then(|s| s.strip_suffix('>')).unwrap_or(ty);
    match base {
        "i8" | "i16" | "i32" | "u8" | "u16" | "u32" => ColumnType::Integer,
        "i64" | "u64" => ColumnType::BigInt,
        "f32" | "f64" => ColumnType::Real,
        "utf8" | "utf16" | "string" => ColumnType::Text,
        "bool" => ColumnType::Boolean,
        "blob" | "bytes" => ColumnType::Blob,
        "uuid" => ColumnType::Text,
        other => ColumnType::Custom(other.to_string()),
    }
}

fn lower_literal(lit: &Literal) -> Expr {
    match lit {
        Literal::Integer(n) => Expr::Integer(*n),
        Literal::String(s) => Expr::String(s.clone()),
        Literal::Bool(b) => Expr::Bool(*b),
        Literal::Param(p) => Expr::Param(p.clone()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hermes::parse;

    #[test]
    fn lower_model_and_select() {
        let doc = parse(
            r#"
model User {
    @@id: i64,
    @email: utf8,
    name: utf8,
}
select name from User where id = 1 limit 5
"#,
        )
        .expect("parse");
        let irs = lower_document(&doc);
        assert_eq!(irs.len(), 2);
        match &irs[0] {
            QueryIr::CreateTable(t) => {
                assert_eq!(t.table, "User");
                assert_eq!(t.columns.len(), 3);
                assert!(t.columns[0].primary_key);
                assert!(t.columns[0].autoincrement);
                assert!(t.columns[1].unique);
            }
            other => panic!("expected create table, got {other:?}"),
        }
        match &irs[1] {
            QueryIr::Select(s) => {
                assert_eq!(s.from, "User");
                assert_eq!(s.columns, vec!["name".to_string()]);
                assert_eq!(s.limit, Some(5));
            }
            other => panic!("expected select, got {other:?}"),
        }
    }
}

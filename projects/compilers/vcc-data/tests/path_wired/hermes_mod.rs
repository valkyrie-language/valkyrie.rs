//! Migrated from `src/hermes/mod.rs` (`tests`).

use super::*;

use crate::sql::{QueryIr, SqlDialect, render_ir};

#[test]
fn parse_storage_model_display() {
    let source = r#"
namespace app;

storage Main {
model User {
    @@id: i64,
    @email: utf8,
    name: utf8,
    note: option<utf8>,
}
}
"#;
    let doc = HermesDocument::parse(source).expect("parse");
    assert_eq!(doc.namespace.as_deref(), Some("app"));
    assert_eq!(doc.models().len(), 1);
    let text = doc.to_string();
    assert!(text.contains("namespace app;"));
    assert!(text.contains("storage Main"));
    assert!(text.contains("@@id: i64"));
    assert!(text.contains("@email: utf8"));
    assert!(text.contains("option<utf8>"));
}

#[test]
fn parse_select_and_render_via_sql() {
    let doc = HermesDocument::parse(
        r#"
model Post {
@@id: i64,
title: utf8,
}
select title from Post where id = 42 limit 1
"#,
    )
    .expect("parse");
    let irs = doc.to_query_ir();
    assert_eq!(irs.len(), 2);
    let ddl = render_ir(&irs[0], SqlDialect::Sqlite);
    assert!(ddl.contains("CREATE TABLE"));
    assert!(ddl.contains("\"Post\"") || ddl.contains("Post"));
    let dml = render_ir(&irs[1], SqlDialect::Sqlite);
    assert!(dml.contains("SELECT"));
    assert!(dml.contains("42"));
    match &irs[0] {
        QueryIr::CreateTable(_) => {}
        other => panic!("expected CreateTable, got {other:?}"),
    }
}

#[test]
fn hermes_path_extensions() {
    assert!(is_hermes_path("schema.her"));
    assert!(is_hermes_path("Schema.HERMES"));
    assert!(!is_hermes_path("schema.sql"));
}

#[test]
fn rejects_empty() {
    assert_eq!(HermesDocument::parse("   ").unwrap_err(), HermesError::EmptyDocument);
}

//! Integration tests: Hermes AST → SQL **render/emit** (not query-exec pipeline).

use std_data::{
    hermes::{FieldKeyKind, HermesDocument, HermesItem, is_hermes_path},
    sql::{QueryIr, SqlDialect, render_ir},
};

#[test]
fn end_to_end_schema_to_sqlite_ddl() {
    let doc = HermesDocument::parse(
        r#"
namespace demo;

storage Game {
    model Player {
        @@id: i64,
        @name: utf8,
        score: i32,
    }
}
"#,
    )
    .expect("parse hermes");

    assert!(matches!(
        &doc.items[0],
        HermesItem::Storage(s) if s.name == "Game"
    ));
    let player = doc.models()[0];
    assert_eq!(player.fields[0].key, FieldKeyKind::Primary);
    assert_eq!(player.fields[1].key, FieldKeyKind::Unique);

    let irs = doc.to_query_ir();
    assert_eq!(irs.len(), 1);
    let sql = render_ir(&irs[0], SqlDialect::Sqlite);
    assert!(sql.contains("CREATE TABLE IF NOT EXISTS"));
    assert!(sql.contains("PRIMARY KEY"));
    assert!(sql.contains("AUTOINCREMENT") || sql.contains("INTEGER"));
}

#[test]
fn end_to_end_select_to_mysql() {
    let doc = HermesDocument::parse("select * from orders where id = $id limit 10").expect("parse query");
    let irs = doc.to_query_ir();
    assert!(matches!(irs[0], QueryIr::Select(_)));
    let sql = render_ir(&irs[0], SqlDialect::MySql);
    assert_eq!(sql, "SELECT * FROM `orders` WHERE `id` = $id LIMIT 10;");
}

#[test]
fn file_extension_helpers() {
    assert!(is_hermes_path("a/b/schema.her"));
    assert!(is_hermes_path("models.hermes"));
}

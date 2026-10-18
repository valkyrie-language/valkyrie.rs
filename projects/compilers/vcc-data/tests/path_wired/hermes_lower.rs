//! Migrated from `src/hermes/lower.rs` (`tests`).

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

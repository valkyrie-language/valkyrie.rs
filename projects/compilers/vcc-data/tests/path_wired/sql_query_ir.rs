//! Migrated from `src/sql/query_ir.rs` (`tests`).

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

//! Migrated from `src/sql/dialect.rs` (`tests`).

use super::*;

#[test]
fn from_name_aliases() {
    assert_eq!(SqlDialect::from_name("pgsql"), Some(SqlDialect::PostgreSql));
    assert_eq!(SqlDialect::from_name("SQLITE"), Some(SqlDialect::Sqlite));
    assert_eq!(SqlDialect::from_name("nope"), None);
}

#[test]
fn quote_ident_mysql_vs_sqlite() {
    assert_eq!(SqlDialect::MySql.quote_ident("user"), "`user`");
    assert_eq!(SqlDialect::Sqlite.quote_ident("user"), "\"user\"");
}

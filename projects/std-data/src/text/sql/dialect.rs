//! SQL dialect flags (MySQL / PostgreSQL / SQLite).

use std::fmt::{Display, Formatter};

/// Target SQL dialect for Atlas optional query paths.
///
/// Full printers may start with one dialect; the enum keeps Hermes dialect-aware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SqlDialect {
    /// SQLite (`"` quoting, `AUTOINCREMENT`, bool as `0`/`1`).
    Sqlite,
    /// PostgreSQL (`"` quoting, `SERIAL`, bool as `TRUE`/`FALSE`).
    PostgreSql,
    /// MySQL (`` ` `` quoting, `AUTO_INCREMENT`, bool as `0`/`1`).
    MySql,
}

impl SqlDialect {
    /// Parse a dialect name (`sqlite` / `postgresql`|`pgsql` / `mysql`).
    pub fn from_name(name: &str) -> Option<Self> {
        match name.trim().to_ascii_lowercase().as_str() {
            "sqlite" => Some(Self::Sqlite),
            "postgresql" | "pgsql" | "postgres" => Some(Self::PostgreSql),
            "mysql" => Some(Self::MySql),
            _ => None,
        }
    }

    /// Canonical lowercase name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Sqlite => "sqlite",
            Self::PostgreSql => "postgresql",
            Self::MySql => "mysql",
        }
    }

    /// Left identifier quote character(s).
    pub fn quote_left(self) -> &'static str {
        match self {
            Self::Sqlite | Self::PostgreSql => "\"",
            Self::MySql => "`",
        }
    }

    /// Right identifier quote character(s).
    pub fn quote_right(self) -> &'static str {
        self.quote_left()
    }

    /// Quote a bare identifier for this dialect.
    pub fn quote_ident(self, ident: &str) -> String {
        format!(
            "{}{}{}",
            self.quote_left(),
            ident.replace(self.quote_left(), &format!("{}{}", self.quote_left(), self.quote_left())),
            self.quote_right()
        )
    }

    /// Escape and quote a string literal (`'...'`, `'` → `''`).
    pub fn quote_string(self, value: &str) -> String {
        let _ = self;
        format!("'{}'", value.replace('\'', "''"))
    }

    /// Format a boolean literal for this dialect.
    pub fn format_bool(self, value: bool) -> &'static str {
        match (self, value) {
            (Self::PostgreSql, true) => "TRUE",
            (Self::PostgreSql, false) => "FALSE",
            (_, true) => "1",
            (_, false) => "0",
        }
    }

    /// Autoincrement / serial keyword fragment (column-type dependent; see render).
    pub fn autoincrement_keyword(self) -> &'static str {
        match self {
            Self::Sqlite => "AUTOINCREMENT",
            Self::PostgreSql => "SERIAL",
            Self::MySql => "AUTO_INCREMENT",
        }
    }

    /// Whether `INSERT ... RETURNING` is supported.
    pub fn supports_returning(self) -> bool {
        matches!(self, Self::PostgreSql)
    }
}

impl Display for SqlDialect {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

#[cfg(test)]
mod tests {
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
}

#![doc = include_str!("readme.md")]

use std::fmt::{Display, Formatter};

pub mod ast;
mod lexer;
mod lower;
mod parser;

pub use ast::{FieldDecl, FieldKeyKind, HermesDocument, HermesItem, Literal, ModelDecl, SelectQuery, StorageDecl};
pub use lexer::{Token, TokenKind, tokenize};
pub use lower::{lower_document, lower_model, lower_select};
pub use parser::parse;

/// Canonical Hermes schema / query file extensions (CS HermesCompiler).
pub const HERMES_EXTENSIONS: &[&str] = &[".hermes", ".her"];

/// True if `path` ends with a Hermes source extension.
pub fn is_hermes_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    HERMES_EXTENSIONS.iter().any(|ext| lower.ends_with(ext))
}

/// Hermes parse / lower error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HermesError {
    /// Empty input.
    EmptyDocument,
    /// More than one `namespace`.
    DuplicateNamespace,
    /// Expected an identifier.
    ExpectedIdent,
    /// Expected a keyword.
    ExpectedKeyword(String),
    /// Expected a token kind.
    ExpectedToken(TokenKind),
    /// Expected a literal in a query filter.
    ExpectedLiteral,
    /// Invalid integer literal.
    InvalidInteger,
    /// Unexpected token text.
    UnexpectedToken {
        /// Found lexeme.
        found: String,
    },
}

impl Display for HermesError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyDocument => write!(f, "Hermes document is empty"),
            Self::DuplicateNamespace => write!(f, "duplicate namespace declaration"),
            Self::ExpectedIdent => write!(f, "expected identifier"),
            Self::ExpectedKeyword(kw) => write!(f, "expected keyword `{kw}`"),
            Self::ExpectedToken(kind) => write!(f, "expected token {kind:?}"),
            Self::ExpectedLiteral => write!(f, "expected literal"),
            Self::InvalidInteger => write!(f, "invalid integer literal"),
            Self::UnexpectedToken { found } => write!(f, "unexpected token `{found}`"),
        }
    }
}

impl std::error::Error for HermesError {}

impl HermesDocument {
    /// Parse Hermes source (`.hermes` / `.her` subset).
    pub fn parse(source: &str) -> Result<Self, HermesError> {
        parse(source)
    }

    /// Project into SQL materialization structs.
    pub fn to_query_ir(&self) -> Vec<crate::sql::QueryIr> {
        lower_document(self)
    }
}

#[cfg(test)]
mod tests {
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
}

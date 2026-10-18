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
#[path = "../../tests/path_wired/hermes_mod.rs"]
mod hermes_mod_tests;

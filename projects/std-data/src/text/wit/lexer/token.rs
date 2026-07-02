//! WIT 词法记号。

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Package,
    Interface,
    Semicolon,
    LBrace,
    RBrace,
    Identifier,
    Statement,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

impl Token {
    pub(crate) fn eof(offset: usize) -> Self {
        Self { kind: TokenKind::Eof, span: offset..offset }
    }
}

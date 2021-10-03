//! MSIL 词法记号。

use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Line,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
    pub line_number: usize,
}

impl Token {
    pub(crate) fn eof(offset: usize, line_number: usize) -> Self {
        Self { kind: TokenKind::Eof, span: offset..offset, line_number }
    }
}

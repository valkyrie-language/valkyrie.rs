//! WIT 词法分析。

mod token;

pub use token::{Token, TokenKind};

use std::ops::Range;

use super::WitError;

pub struct Lexer {
    source: String,
    offset: usize,
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn tokenize(source: &str) -> Result<Vec<Token>, WitError> {
        let mut lexer = Self { source: source.replace("\r\n", "\n"), offset: 0, tokens: Vec::new() };
        while lexer.offset < lexer.source.len() {
            lexer.skip_trivia();
            if lexer.offset >= lexer.source.len() {
                break;
            }
            lexer.lex_token()?;
        }
        lexer.tokens.push(Token::eof(lexer.source.len()));
        Ok(lexer.tokens)
    }

    fn skip_trivia(&mut self) {
        loop {
            self.skip_whitespace();
            if self.peek_starts_with("//") {
                while let Some(ch) = self.peek_char() {
                    self.offset += ch.len_utf8();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            return;
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.offset += ch.len_utf8();
            }
            else {
                break;
            }
        }
    }

    fn lex_token(&mut self) -> Result<(), WitError> {
        let start = self.offset;
        if self.peek_starts_with("package ") {
            self.offset += "package ".len();
            self.read_identifier()?;
            self.skip_whitespace();
            self.expect_char(';')?;
            self.tokens.push(Token { kind: TokenKind::Package, span: span(start, self.offset) });
            return Ok(());
        }
        if self.peek_starts_with("interface ") {
            self.offset += "interface ".len();
            self.read_identifier()?;
            self.skip_whitespace();
            self.expect_char('{')?;
            self.tokens.push(Token { kind: TokenKind::Interface, span: span(start, self.offset) });
            return Ok(());
        }
        if self.peek_char() == Some('}') {
            self.offset += 1;
            self.tokens.push(Token { kind: TokenKind::RBrace, span: span(start, self.offset) });
            return Ok(());
        }

        self.read_statement_line()?;
        self.tokens.push(Token { kind: TokenKind::Statement, span: span(start, self.offset) });
        Ok(())
    }

    fn read_identifier(&mut self) -> Result<(), WitError> {
        let start = self.offset;
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() || ch == ';' || ch == '{' || ch == '}' {
                break;
            }
            self.offset += ch.len_utf8();
        }
        if start == self.offset {
            return Err(WitError::InvalidInterface("期望标识符".to_string()));
        }
        Ok(())
    }

    fn read_statement_line(&mut self) -> Result<(), WitError> {
        while let Some(ch) = self.peek_char() {
            if ch == ';' {
                self.offset += 1;
                return Ok(());
            }
            if ch == '}' {
                return Ok(());
            }
            self.offset += ch.len_utf8();
        }
        Err(WitError::InvalidInterface("函数声明缺少 `;`".to_string()))
    }

    fn expect_char(&mut self, expected: char) -> Result<(), WitError> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(WitError::InvalidInterface(format!("缺少字符 `{expected}`"))),
        }
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn peek_starts_with(&self, literal: &str) -> bool {
        self.source[self.offset..].starts_with(literal)
    }
}

fn span(start: usize, end: usize) -> Range<usize> {
    start..end
}

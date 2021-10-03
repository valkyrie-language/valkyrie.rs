//! WAT 词法分析。

mod token;

pub use token::{Token, TokenKind};

use std::ops::Range;

use super::WatError;

pub struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn tokenize(source: &'a str) -> Result<Vec<Token>, WatError> {
        let mut lexer = Self { source: source.trim(), offset: 0, tokens: Vec::new() };
        while lexer.offset < lexer.source.len() {
            lexer.skip_ws();
            if lexer.offset >= lexer.source.len() {
                break;
            }
            lexer.lex_token()?;
        }
        lexer.tokens.push(Token::eof(lexer.source.len()));
        Ok(lexer.tokens)
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() {
                self.offset += ch.len_utf8();
            }
            else {
                break;
            }
        }
    }

    fn lex_token(&mut self) -> Result<(), WatError> {
        let start = self.offset;
        let kind = match self.peek_char() {
            Some('(') => {
                self.offset += 1;
                TokenKind::LParen
            }
            Some(')') => {
                self.offset += 1;
                TokenKind::RParen
            }
            Some('"') => {
                self.lex_string()?;
                TokenKind::StringLiteral
            }
            Some(_) => {
                self.lex_identifier()?;
                TokenKind::Identifier
            }
            None => return Ok(()),
        };
        self.tokens.push(Token { kind, span: span(start, self.offset) });
        Ok(())
    }

    fn lex_string(&mut self) -> Result<(), WatError> {
        self.expect_char('"')?;
        let mut escape = false;
        while let Some(ch) = self.next_char() {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == '"' {
                return Ok(());
            }
        }
        Err(WatError::UnterminatedString)
    }

    fn lex_identifier(&mut self) -> Result<(), WatError> {
        let start = self.offset;
        while let Some(ch) = self.peek_char() {
            if ch.is_whitespace() || ch == '(' || ch == ')' {
                break;
            }
            self.offset += ch.len_utf8();
        }
        if start == self.offset {
            return Err(WatError::InvalidModule("期望读取标记，但遇到空内容".to_string()));
        }
        Ok(())
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }

    fn next_char(&mut self) -> Option<char> {
        let ch = self.peek_char()?;
        self.offset += ch.len_utf8();
        Some(ch)
    }

    fn expect_char(&mut self, expected: char) -> Result<(), WatError> {
        match self.next_char() {
            Some(ch) if ch == expected => Ok(()),
            _ => Err(WatError::InvalidModule(format!("缺少字符 `{expected}`"))),
        }
    }
}

fn span(start: usize, end: usize) -> Range<usize> {
    start..end
}

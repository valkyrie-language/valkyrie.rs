//! MSIL 词法分析。

mod token;

pub use token::{Token, TokenKind};

pub struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    line_number: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    pub fn tokenize(source: &'a str) -> Vec<Token> {
        let mut lexer = Self { source, offset: 0, line_number: 1, tokens: Vec::new() };
        while lexer.offset < source.len() {
            let start = lexer.offset;
            while let Some(ch) = lexer.peek_char() {
                if ch == '\n' {
                    break;
                }
                lexer.offset += ch.len_utf8();
            }
            lexer.tokens.push(Token { kind: TokenKind::Line, span: start..lexer.offset, line_number: lexer.line_number });
            if lexer.peek_char() == Some('\n') {
                lexer.offset += 1;
                lexer.line_number += 1;
            }
            else if lexer.offset >= source.len() {
                break;
            }
        }
        lexer.tokens.push(Token::eof(source.len(), lexer.line_number));
        lexer.tokens
    }

    fn peek_char(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }
}

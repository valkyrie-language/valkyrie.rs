//! 词法分析入口。

pub mod token;

pub use token::{Token, TokenKind};

use crate::parser::ParseError;
use std::ops::Range;

/// 将源文本切分为词法记号流。
pub struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
}

impl<'a> Lexer<'a> {
    /// 词法分析并返回完整 token 序列。
    pub fn tokenize(source: &'a str) -> Result<Vec<Token>, ParseError> {
        let mut lexer = Self { source, offset: 0, tokens: Vec::new() };
        while lexer.offset < source.len() {
            lexer.skip_trivia();
            if lexer.offset >= source.len() {
                break;
            }
            lexer.lex_token()?;
        }
        let eof = source.len();
        lexer.tokens.push(Token { kind: TokenKind::Eof, span: span(eof, eof) });
        Ok(lexer.tokens)
    }

    fn skip_trivia(&mut self) {
        loop {
            let Some(remaining) = self.source.get(self.offset..)
            else {
                return;
            };
            if remaining.starts_with('#') {
                while let Some(ch) = self.peek_char() {
                    self.offset += ch.len_utf8();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            // `⍝`（APL 风格注释）与 `#` 一样作为行注释处理。
            if remaining.starts_with('⍝') {
                self.offset += '⍝'.len_utf8();
                while let Some(ch) = self.peek_char() {
                    self.offset += ch.len_utf8();
                    if ch == '\n' {
                        break;
                    }
                }
                continue;
            }
            // `<% ... %>` 模板指令：跳过指令内容，保留 `%>` 与下一个 `<%` 之间的代码。
            if remaining.starts_with("<%") {
                self.offset += 2;
                while let Some(ch) = self.peek_char() {
                    if ch == '%' && self.source.get(self.offset + 1..).is_some_and(|s| s.starts_with('>')) {
                        self.offset += 2;
                        break;
                    }
                    self.offset += ch.len_utf8();
                }
                continue;
            }
            let Some(ch) = self.peek_char()
            else {
                return;
            };
            if ch.is_whitespace() {
                self.offset += ch.len_utf8();
                continue;
            }
            return;
        }
    }

    fn lex_token(&mut self) -> Result<(), ParseError> {
        let start = self.offset;
        let Some(ch) = self.peek_char()
        else {
            return Ok(());
        };

        if ch == 'r' && (self.starts_with("r\"") || self.starts_with("r\"\"\"")) {
            self.lex_string(start, true)?;
            return Ok(());
        }
        if is_identifier_start(ch) {
            self.lex_identifier(start);
            return Ok(());
        }
        if ch.is_ascii_digit() {
            self.lex_number(start);
            return Ok(());
        }
        if ch == '"' {
            self.lex_string(start, false)?;
            return Ok(());
        }
        if ch == '`' {
            self.lex_backtick_symbol(start)?;
            return Ok(());
        }

        match ch {
            '(' => self.push_token(TokenKind::LParen, start, start + 1),
            ')' => self.push_token(TokenKind::RParen, start, start + 1),
            '{' => self.push_token(TokenKind::LBrace, start, start + 1),
            '}' => self.push_token(TokenKind::RBrace, start, start + 1),
            '[' => self.push_token(TokenKind::LBracket, start, start + 1),
            ']' => self.push_token(TokenKind::RBracket, start, start + 1),
            '<' => {
                if self.starts_with("<=") {
                    self.push_token(TokenKind::LessEq, start, start + 2);
                }
                else if self.starts_with("<<") {
                    self.push_token(TokenKind::Shl, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::LAngle, start, start + 1);
                }
            }
            '>' => {
                if self.starts_with(">=") {
                    self.push_token(TokenKind::GreaterEq, start, start + 2);
                }
                else if self.starts_with(">>") {
                    self.push_token(TokenKind::Shr, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::RAngle, start, start + 1);
                }
            }
            ',' => self.push_token(TokenKind::Comma, start, start + 1),
            ':' => {
                if self.starts_with("::") {
                    self.push_token(TokenKind::DoubleColon, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Colon, start, start + 1);
                }
            }
            ';' => self.push_token(TokenKind::Semicolon, start, start + 1),
            '=' => {
                if self.starts_with("==") {
                    self.push_token(TokenKind::EqEq, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Equal, start, start + 1);
                }
            }
            '-' => {
                if self.starts_with("->") {
                    self.push_token(TokenKind::Arrow, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Minus, start, start + 1);
                }
            }
            '.' => self.push_token(TokenKind::Dot, start, start + 1),
            '+' => self.push_token(TokenKind::Plus, start, start + 1),
            '*' => self.push_token(TokenKind::Star, start, start + 1),
            '/' => self.push_token(TokenKind::Slash, start, start + 1),
            '%' => self.push_token(TokenKind::Percent, start, start + 1),
            '&' => {
                if self.starts_with("&&") {
                    self.push_token(TokenKind::AndAnd, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Ampersand, start, start + 1);
                }
            }
            '|' => {
                if self.starts_with("||") {
                    self.push_token(TokenKind::OrOr, start, start + 2);
                }
                else if self.starts_with("|>") {
                    self.push_token(TokenKind::PipeGt, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Pipe, start, start + 1);
                }
            }
            '~' => self.push_token(TokenKind::Tilde, start, start + 1),
            '^' => self.push_token(TokenKind::Caret, start, start + 1),
            '@' => self.push_token(TokenKind::At, start, start + 1),
            '!' => {
                if self.starts_with("!=") {
                    self.push_token(TokenKind::NotEq, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Bang, start, start + 1);
                }
            }
            '?' => {
                if self.starts_with("?.") {
                    self.push_token(TokenKind::QuestionDot, start, start + 2);
                }
                else {
                    self.push_token(TokenKind::Question, start, start + 1);
                }
            }
            _ => {
                return Err(ParseError::invalid(format!("unexpected character '{}' at {}", ch, start)));
            }
        }
        Ok(())
    }

    fn lex_identifier(&mut self, start: usize) {
        self.offset += self.peek_char().unwrap().len_utf8();
        while let Some(ch) = self.peek_char() {
            if !is_identifier_continue(ch) {
                break;
            }
            self.offset += ch.len_utf8();
        }
        self.push_token(TokenKind::Identifier, start, self.offset);
    }

    fn lex_number(&mut self, start: usize) {
        self.offset += self.peek_char().unwrap().len_utf8();

        // 检查十六进制、二进制、八进制前缀。
        if self.peek_char() == Some('x') || self.peek_char() == Some('X') {
            self.offset += 'x'.len_utf8();
            while let Some(ch) = self.peek_char() {
                if !ch.is_ascii_hexdigit() {
                    break;
                }
                self.offset += ch.len_utf8();
            }
            self.push_token(TokenKind::IntegerLiteral, start, self.offset);
            return;
        }
        if self.peek_char() == Some('b') || self.peek_char() == Some('B') {
            self.offset += 'b'.len_utf8();
            while let Some(ch) = self.peek_char() {
                if ch != '0' && ch != '1' {
                    break;
                }
                self.offset += ch.len_utf8();
            }
            self.push_token(TokenKind::IntegerLiteral, start, self.offset);
            return;
        }
        if self.peek_char() == Some('o') || self.peek_char() == Some('O') {
            self.offset += 'o'.len_utf8();
            while let Some(ch) = self.peek_char() {
                if !('0'..='7').contains(&ch) {
                    break;
                }
                self.offset += ch.len_utf8();
            }
            self.push_token(TokenKind::IntegerLiteral, start, self.offset);
            return;
        }

        // 十进制整数部分。
        while let Some(ch) = self.peek_char() {
            if !ch.is_ascii_digit() {
                break;
            }
            self.offset += ch.len_utf8();
        }

        // 浮点数小数部分。
        if self.peek_char() == Some('.') {
            self.offset += '.'.len_utf8();
            while let Some(ch) = self.peek_char() {
                if !ch.is_ascii_digit() {
                    break;
                }
                self.offset += ch.len_utf8();
            }
            // 指数部分。
            if self.peek_char() == Some('e') || self.peek_char() == Some('E') {
                self.offset += 'e'.len_utf8();
                if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                    self.offset += 1;
                }
                while let Some(ch) = self.peek_char() {
                    if !ch.is_ascii_digit() {
                        break;
                    }
                    self.offset += ch.len_utf8();
                }
            }
            self.push_token(TokenKind::FloatLiteral, start, self.offset);
            return;
        }

        // 指数部分（无小数点）。
        if self.peek_char() == Some('e') || self.peek_char() == Some('E') {
            self.offset += 'e'.len_utf8();
            if self.peek_char() == Some('+') || self.peek_char() == Some('-') {
                self.offset += 1;
            }
            while let Some(ch) = self.peek_char() {
                if !ch.is_ascii_digit() {
                    break;
                }
                self.offset += ch.len_utf8();
            }
            self.push_token(TokenKind::FloatLiteral, start, self.offset);
            return;
        }

        self.push_token(TokenKind::IntegerLiteral, start, self.offset);
    }

    /// 词法分析反引号包裹的符号字面量。
    ///
    /// 读取从开始反引号到结束反引号之间的内容，生成 `BacktickSymbol` token。
    /// span 覆盖整个 `` `symbol` `` 文本（含反引号），解析器负责剥离反引号。
    fn lex_backtick_symbol(&mut self, start: usize) -> Result<(), ParseError> {
        self.offset += '`'.len_utf8();
        while let Some(ch) = self.peek_char() {
            if ch == '`' {
                self.offset += '`'.len_utf8();
                self.tokens.push(Token { kind: TokenKind::BacktickSymbol, span: span(start, self.offset) });
                return Ok(());
            }
            self.offset += ch.len_utf8();
        }
        Err(ParseError::invalid("unterminated backtick symbol literal"))
    }

    fn lex_string(&mut self, start: usize, is_raw: bool) -> Result<(), ParseError> {
        if is_raw {
            self.offset += 'r'.len_utf8();
        }

        let quote_count = if self.starts_with("\"\"\"") { 3 } else { 1 };
        self.offset += quote_count;

        if quote_count == 3 {
            while self.offset < self.source.len() {
                if self.source.get(self.offset..).is_some_and(|text| text.starts_with("\"\"\"")) {
                    self.offset += 3;
                    self.tokens.push(Token { kind: TokenKind::StringLiteral, span: span(start, self.offset) });
                    return Ok(());
                }

                let Some(ch) = self.peek_char()
                else {
                    break;
                };
                self.offset += ch.len_utf8();
            }
            return Err(ParseError::invalid("unterminated triple-quoted string literal"));
        }

        let mut escape = false;
        while let Some(ch) = self.peek_char() {
            self.offset += ch.len_utf8();
            if !is_raw && escape {
                escape = false;
                continue;
            }
            match ch {
                '\\' if !is_raw => escape = true,
                '"' => {
                    self.tokens.push(Token { kind: TokenKind::StringLiteral, span: span(start, self.offset) });
                    return Ok(());
                }
                _ => {}
            }
        }
        Err(ParseError::invalid("unterminated string literal"))
    }

    fn push_token(&mut self, kind: TokenKind, start: usize, end: usize) {
        self.offset = end;
        self.tokens.push(Token { kind, span: span(start, end) });
    }

    fn peek_char(&self) -> Option<char> {
        self.source.get(self.offset..)?.chars().next()
    }

    fn starts_with(&self, text: &str) -> bool {
        self.source.get(self.offset..).is_some_and(|value| value.starts_with(text))
    }
}

fn is_identifier_start(ch: char) -> bool {
    ch == '_' || ch.is_alphabetic()
}

fn is_identifier_continue(ch: char) -> bool {
    ch == '_' || ch.is_alphanumeric()
}

fn span(start: usize, end: usize) -> Range<usize> {
    start..end
}

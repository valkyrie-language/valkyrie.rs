//! WAT 语法分析。

use super::{
    WatDocument, WatError,
    lexer::{Lexer, Token, TokenKind},
};

pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(source: &'a str) -> Result<WatDocument, WatError> {
        let source = source.trim();
        if !source.starts_with('(') || !source.ends_with(')') {
            return Err(WatError::InvalidModule("顶层必须是 `(module ...)`".to_string()));
        }
        let tokens = Lexer::tokenize(source)?;
        let mut parser = Self { source, tokens, index: 0 };
        parser.parse_module()
    }

    fn parse_module(&mut self) -> Result<WatDocument, WatError> {
        let open = self.advance();
        if open.kind != TokenKind::LParen {
            return Err(WatError::InvalidModule("顶层必须是 `(module ...)`".to_string()));
        }

        let keyword = self.advance();
        if self.text(&keyword) != "module" {
            return Err(WatError::InvalidModule("顶层节点必须是 `module`".to_string()));
        }

        let module_name =
            if self.check(TokenKind::Identifier) && self.text(&self.peek()).starts_with('$') { Some(self.advance_text()) } else { None };

        let fields = self.parse_top_level_fields()?;
        self.expect(TokenKind::RParen)?;
        if !self.is_at_end() {
            return Err(WatError::InvalidModule("模块结束后仍有多余内容".to_string()));
        }
        Ok(WatDocument { module_name, fields })
    }

    fn parse_top_level_fields(&mut self) -> Result<Vec<String>, WatError> {
        let mut fields = Vec::new();
        while !self.is_at_end() && !self.check(TokenKind::RParen) {
            let start_index = self.index;
            self.expect(TokenKind::LParen)?;
            let mut depth = 1usize;
            while depth > 0 {
                match self.advance().kind {
                    TokenKind::LParen => depth += 1,
                    TokenKind::RParen => depth -= 1,
                    TokenKind::Eof => return Err(WatError::UnbalancedParentheses),
                    _ => {}
                }
            }
            let start = self.tokens[start_index].span.start;
            let end = self.tokens[self.index - 1].span.end;
            fields.push(self.source[start..end].trim().to_string());
        }
        Ok(fields)
    }

    fn text(&self, token: &Token) -> &str {
        &self.source[token.span.clone()]
    }

    fn advance_text(&mut self) -> String {
        let token = self.advance();
        self.text(&token).to_string()
    }

    fn peek(&self) -> Token {
        self.tokens.get(self.index).cloned().unwrap_or_else(|| Token::eof(self.source.len()))
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), WatError> {
        if self.check(kind) {
            self.advance();
            Ok(())
        }
        else {
            Err(WatError::InvalidModule(format!("期望记号 {kind:?}")))
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        if !self.is_at_end() {
            self.index += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

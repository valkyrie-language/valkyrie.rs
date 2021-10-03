//! WIT 语法分析。

use super::{
    WitError, WitInterface, WitPackage,
    lexer::{Lexer, TokenKind},
};

pub struct Parser<'a> {
    source: &'a str,
    tokens: Vec<super::lexer::Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    pub fn parse(source: &'a str) -> Result<WitPackage, WitError> {
        let tokens = Lexer::tokenize(source)?;
        let mut parser = Self { source, tokens, index: 0 };
        parser.parse_package()
    }

    fn parse_package(&mut self) -> Result<WitPackage, WitError> {
        let package_token = self.advance();
        if package_token.kind != TokenKind::Package {
            return Err(WitError::InvalidPackage);
        }
        let package_name = self.extract_package_name(&package_token.span)?;

        let mut package = WitPackage::new(package_name);
        while !self.is_at_end() {
            package.interfaces.push(self.parse_interface()?);
        }
        Ok(package)
    }

    fn parse_interface(&mut self) -> Result<WitInterface, WitError> {
        let interface_token = self.advance();
        if interface_token.kind != TokenKind::Interface {
            return Err(WitError::InvalidInterface("期望 `interface`".to_string()));
        }
        let name = self.extract_interface_name(&interface_token.span)?;
        let mut functions = Vec::new();
        loop {
            match self.peek_kind() {
                TokenKind::Statement => {
                    let token = self.advance();
                    let statement = self.source[token.span].trim().trim_end_matches(';').trim().to_string();
                    functions.push(statement);
                }
                TokenKind::RBrace => {
                    self.advance();
                    break;
                }
                TokenKind::Eof => return Err(WitError::InvalidInterface(format!("接口 `{name}` 没有闭合"))),
                other => return Err(WitError::InvalidInterface(format!("接口体内出现意外记号：{other:?}"))),
            }
        }
        Ok(WitInterface { name, functions })
    }

    fn extract_package_name(&self, span: &std::ops::Range<usize>) -> Result<String, WitError> {
        let text = self.source[span.clone()].trim();
        let rest = text.strip_prefix("package ").ok_or(WitError::InvalidPackage)?;
        let name = rest.strip_suffix(';').map(str::trim).ok_or(WitError::InvalidPackage)?;
        Ok(name.to_string())
    }

    fn extract_interface_name(&self, span: &std::ops::Range<usize>) -> Result<String, WitError> {
        let text = self.source[span.clone()].trim();
        let rest = text.strip_prefix("interface ").ok_or_else(|| WitError::InvalidInterface("接口头格式错误".to_string()))?;
        let name = rest.strip_suffix('{').map(str::trim).ok_or_else(|| WitError::InvalidInterface("接口头缺少 `{`".to_string()))?;
        Ok(name.to_string())
    }

    fn peek_kind(&self) -> TokenKind {
        self.tokens.get(self.index).map(|token| token.kind).unwrap_or(TokenKind::Eof)
    }

    fn advance(&mut self) -> super::lexer::Token {
        let token = self.tokens[self.index].clone();
        if !self.is_at_end() {
            self.index += 1;
        }
        token
    }

    fn is_at_end(&self) -> bool {
        self.peek_kind() == TokenKind::Eof
    }
}

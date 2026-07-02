//! Hermes recursive-descent parser (minimal schema / query subset).

use super::{
    HermesError,
    ast::{FieldDecl, FieldKeyKind, HermesDocument, HermesItem, Literal, ModelDecl, SelectQuery, StorageDecl},
    lexer::{Token, TokenKind, tokenize},
};

/// Parse Hermes source into a document.
pub fn parse(source: &str) -> Result<HermesDocument, HermesError> {
    let tokens = tokenize(source);
    Parser { source, tokens, index: 0 }.parse_document()
}

struct Parser<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    index: usize,
}

impl<'a> Parser<'a> {
    fn parse_document(&mut self) -> Result<HermesDocument, HermesError> {
        let mut doc = HermesDocument::default();
        while !self.check(TokenKind::Eof) {
            if self.check_kw("namespace") {
                if doc.namespace.is_some() {
                    return Err(HermesError::DuplicateNamespace);
                }
                doc.namespace = Some(self.parse_namespace()?);
                continue;
            }
            if self.check_kw("storage") {
                doc.items.push(HermesItem::Storage(self.parse_storage()?));
                continue;
            }
            if self.check_kw("model") {
                doc.items.push(HermesItem::Model(self.parse_model()?));
                continue;
            }
            if self.check_kw("select") {
                doc.items.push(HermesItem::Query(self.parse_select()?));
                continue;
            }
            return Err(HermesError::UnexpectedToken { found: self.peek_lexeme().to_string() });
        }
        if doc.namespace.is_none() && doc.items.is_empty() {
            return Err(HermesError::EmptyDocument);
        }
        Ok(doc)
    }

    fn parse_namespace(&mut self) -> Result<String, HermesError> {
        self.expect_kw("namespace")?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::Semicolon)?;
        Ok(name)
    }

    fn parse_storage(&mut self) -> Result<StorageDecl, HermesError> {
        self.expect_kw("storage")?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LeftBrace)?;
        let mut models = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            if self.check_kw("model") {
                models.push(self.parse_model()?);
            }
            else {
                return Err(HermesError::ExpectedKeyword("model".into()));
            }
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(StorageDecl { name, models })
    }

    fn parse_model(&mut self) -> Result<ModelDecl, HermesError> {
        self.expect_kw("model")?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LeftBrace)?;
        let mut fields = Vec::new();
        while !self.check(TokenKind::RightBrace) && !self.check(TokenKind::Eof) {
            fields.push(self.parse_field()?);
            // Trailing commas allowed.
            let _ = self.match_kind(TokenKind::Comma);
        }
        self.expect(TokenKind::RightBrace)?;
        Ok(ModelDecl { name, fields })
    }

    fn parse_field(&mut self) -> Result<FieldDecl, HermesError> {
        let (name, key) = if self.check(TokenKind::AtAtIdent) {
            let name = self.bump_lexeme();
            (name, FieldKeyKind::Primary)
        }
        else if self.check(TokenKind::AtIdent) {
            let name = self.bump_lexeme();
            (name, FieldKeyKind::Unique)
        }
        else {
            (self.expect_ident()?, FieldKeyKind::Plain)
        };
        self.expect(TokenKind::Colon)?;
        let ty = self.expect_type_name()?;
        Ok(FieldDecl { name, ty, key })
    }

    fn parse_select(&mut self) -> Result<SelectQuery, HermesError> {
        self.expect_kw("select")?;
        let columns = if self.match_kind(TokenKind::Star) {
            Vec::new()
        }
        else {
            let mut cols = vec![self.expect_ident()?];
            while self.match_kind(TokenKind::Comma) {
                cols.push(self.expect_ident()?);
            }
            cols
        };
        self.expect_kw("from")?;
        let from = self.expect_ident()?;
        let where_eq = if self.check_kw("where") {
            self.bump();
            let col = self.expect_ident()?;
            self.expect(TokenKind::Equal)?;
            let lit = self.parse_literal()?;
            Some((col, lit))
        }
        else {
            None
        };
        let limit = if self.check_kw("limit") {
            self.bump();
            Some(self.expect_u64()?)
        }
        else {
            None
        };
        let _ = self.match_kind(TokenKind::Semicolon);
        Ok(SelectQuery { columns, from, where_eq, limit })
    }

    fn parse_literal(&mut self) -> Result<Literal, HermesError> {
        if self.check(TokenKind::Integer) {
            let n: i64 = self.bump_lexeme().parse().map_err(|_| HermesError::InvalidInteger)?;
            return Ok(Literal::Integer(n));
        }
        if self.check(TokenKind::String) {
            let raw = self.bump_lexeme();
            let inner = raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(raw.as_str());
            return Ok(Literal::String(inner.to_string()));
        }
        if self.check(TokenKind::Param) {
            return Ok(Literal::Param(self.bump_lexeme()));
        }
        if self.check_kw("true") {
            self.bump();
            return Ok(Literal::Bool(true));
        }
        if self.check_kw("false") {
            self.bump();
            return Ok(Literal::Bool(false));
        }
        // Allow bare @ident as query param when lexer produced AtIdent.
        if self.check(TokenKind::AtIdent) {
            let name = self.bump_lexeme();
            return Ok(Literal::Param(format!("@{name}")));
        }
        Err(HermesError::ExpectedLiteral)
    }

    fn expect_type_name(&mut self) -> Result<String, HermesError> {
        if self.check_kw("option") {
            self.bump();
            self.expect(TokenKind::LeftAngle)?;
            let inner = self.expect_ident()?;
            self.expect(TokenKind::RightAngle)?;
            return Ok(format!("option<{inner}>"));
        }
        self.expect_ident()
    }

    fn expect_ident(&mut self) -> Result<String, HermesError> {
        if self.check(TokenKind::Ident) { Ok(self.bump_lexeme()) } else { Err(HermesError::ExpectedIdent) }
    }

    fn expect_u64(&mut self) -> Result<u64, HermesError> {
        if !self.check(TokenKind::Integer) {
            return Err(HermesError::InvalidInteger);
        }
        self.bump_lexeme().parse().map_err(|_| HermesError::InvalidInteger)
    }

    fn expect_kw(&mut self, kw: &str) -> Result<(), HermesError> {
        if self.check_kw(kw) {
            self.bump();
            Ok(())
        }
        else {
            Err(HermesError::ExpectedKeyword(kw.to_string()))
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), HermesError> {
        if self.check(kind) {
            self.bump();
            Ok(())
        }
        else {
            Err(HermesError::ExpectedToken(kind))
        }
    }

    fn check_kw(&self, kw: &str) -> bool {
        self.check(TokenKind::Ident) && self.peek_lexeme().eq_ignore_ascii_case(kw)
    }

    fn check(&self, kind: TokenKind) -> bool {
        self.peek().kind == kind
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.check(kind) {
            self.bump();
            true
        }
        else {
            false
        }
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn peek_lexeme(&self) -> &str {
        self.peek().lexeme(self.source)
    }

    fn bump(&mut self) {
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
        }
    }

    fn bump_lexeme(&mut self) -> String {
        let lex = self.peek_lexeme().to_string();
        self.bump();
        lex
    }
}

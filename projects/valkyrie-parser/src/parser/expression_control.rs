use crate::{
    ast::{
        DereferenceKind, NamePath, SubscriptItem, SubscriptKind, TermArguments, TermCallExpression, TermDereferenceExpression,
        TermDotExpression, TermExpression, TermSubscriptExpression,
    },
    lexer::{Keyword, TokenKind},
};

use super::{span, ParseError, Parser};

impl<'a> Parser<'a> {
    pub(super) fn parse_raise_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Raise)?.span.start;
        let value = Box::new(self.parse_value_expression_bp(0)?);
        let end = value.span().end;
        Ok(TermExpression::Raise { value, span: span(start, end) })
    }

    pub(super) fn parse_catch_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Catch)?.span.start;
        self.suppress_struct_constructor = true;
        let expr = Box::new(self.parse_value_expression_bp(0)?);
        self.suppress_struct_constructor = false;
        self.expect_symbol(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated catch body"));
            }
            if self.match_symbol(TokenKind::Semicolon) {
                continue;
            }
            let arm = self.parse_match_arm()?;
            arms.push(arm);
        }
        let close = self.expect_symbol(TokenKind::RBrace)?;
        Ok(TermExpression::Catch { expr, arms, span: span(start, close.span.end) })
    }

    /// 解析 lambda 表达式 `micro(params) -> return_type { body }`。
    ///
    /// 与顶层 `micro name(params) -> T { body }` 函数声明不同，lambda 作为表达式
    /// 出现在调用参数等位置时没有函数名，直接以 `micro(` 起始。
    pub(super) fn parse_lambda_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Micro)?.span.start;
        let params = self.parse_parameter_list()?;
        let return_type = if self.match_symbol(TokenKind::Arrow) { Some(self.parse_type_expression_bp(0)?) } else { None };
        let body = if self.check_symbol(TokenKind::LBrace) {
            self.parse_block_body()?
        }
        else {
            return Err(self.error_here("expected lambda body '{'"));
        };
        let end = body.span.end;
        Ok(TermExpression::Lambda { params, return_type, body: Box::new(body), span: span(start, end) })
    }

    pub(super) fn expression_postfix_binding_power(&self) -> Option<(u8, u8)> {
        match self.current().kind {
            TokenKind::Dot => Some((95, 96)),
            TokenKind::DoubleColon if self.nth_is_identifier(1) => Some((95, 96)),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LAngle) => Some((93, 94)),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LBracket) => Some((90, 91)),
            TokenKind::LParen | TokenKind::LBracket | TokenKind::LOffsetBracket => Some((90, 91)),
            TokenKind::LBrace if !self.suppress_struct_constructor => Some((85, 86)),
            _ => None,
        }
    }

    pub(super) fn parse_expression_postfix(&mut self, lhs: TermExpression) -> Result<TermExpression, ParseError> {
        match self.current().kind {
            TokenKind::LParen => self.parse_call_expression(lhs),
            TokenKind::Dot => self.parse_member_expression(lhs),
            TokenKind::LBracket => self.parse_subscript_expression(lhs, SubscriptKind::Ordinal),
            TokenKind::LOffsetBracket => self.parse_offset_subscript_expression(lhs),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LBracket) => self.parse_offset_alias_subscript_expression(lhs),
            TokenKind::DoubleColon if self.nth_is_symbol(1, TokenKind::LAngle) => self.parse_turbofish_expression(lhs),
            TokenKind::DoubleColon if self.nth_is_identifier(1) => self.parse_path_member_expression(lhs),
            TokenKind::LBrace => self.parse_construct_expression(lhs),
            _ => Err(self.error_here("expected postfix operator")),
        }
    }

    fn parse_call_expression(&mut self, callee: TermExpression) -> Result<TermExpression, ParseError> {
        let start = callee.span().start;
        self.expect_symbol(TokenKind::LParen)?;
        let args = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_value_expression_bp(0))?;
        let close = self.expect_symbol(TokenKind::RParen)?;
        Ok(TermExpression::Call(Box::new(TermCallExpression {
            callee,
            args: TermArguments { terms: args },
            span: span(start, close.span.end),
        })))
    }

    fn parse_member_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::Dot)?;
        if self.match_symbol(TokenKind::HollowDiamond) {
            let end = self.previous().span.end;
            return Ok(TermExpression::Dereference(Box::new(TermDereferenceExpression {
                base: object,
                kind: DereferenceKind::ReadOnly,
                span: span(start, end),
            })));
        }
        if self.match_symbol(TokenKind::SolidDiamond) {
            let end = self.previous().span.end;
            return Ok(TermExpression::Dereference(Box::new(TermDereferenceExpression {
                base: object,
                kind: DereferenceKind::Mutable,
                span: span(start, end),
            })));
        }
        let member = self.expect_member_name_text()?;
        let end = self.previous().span.end;
        let caller = NamePath { parts: vec![member], span: span(start, end) };
        Ok(TermExpression::DotCall(Box::new(TermDotExpression {
            base: object,
            caller,
            arguments: TermArguments { terms: vec![] },
            span: span(start, end),
        })))
    }

    fn parse_path_member_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::DoubleColon)?;
        let member = self.expect_member_name_text()?;
        let end = self.previous().span.end;
        let caller = NamePath { parts: vec![member], span: span(start, end) };
        Ok(TermExpression::DotCall(Box::new(TermDotExpression {
            base: object,
            caller,
            arguments: TermArguments { terms: vec![] },
            span: span(start, end),
        })))
    }

    fn parse_subscript_expression(&mut self, object: TermExpression, kind: SubscriptKind) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::LBracket)?;
        let index = self.parse_value_expression_bp(0)?;
        let close = self.expect_symbol(TokenKind::RBracket)?;
        Ok(TermExpression::Subscript(Box::new(TermSubscriptExpression {
            base: object,
            subscripts: vec![SubscriptItem::Index { term: index, span: span(start, close.span.end) }],
            kind,
            span: span(start, close.span.end),
        })))
    }

    fn parse_offset_subscript_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::LOffsetBracket)?;
        let index = self.parse_value_expression_bp(0)?;
        let close = self.expect_symbol(TokenKind::ROffsetBracket)?;
        Ok(TermExpression::Subscript(Box::new(TermSubscriptExpression {
            base: object,
            subscripts: vec![SubscriptItem::Index { term: index, span: span(start, close.span.end) }],
            kind: SubscriptKind::Cardinal,
            span: span(start, close.span.end),
        })))
    }

    fn parse_offset_alias_subscript_expression(&mut self, object: TermExpression) -> Result<TermExpression, ParseError> {
        let start = object.span().start;
        self.expect_symbol(TokenKind::DoubleColon)?;
        self.expect_symbol(TokenKind::LBracket)?;
        let index = self.parse_value_expression_bp(0)?;
        let close = self.expect_symbol(TokenKind::RBracket)?;
        Ok(TermExpression::Subscript(Box::new(TermSubscriptExpression {
            base: object,
            subscripts: vec![SubscriptItem::Index { term: index, span: span(start, close.span.end) }],
            kind: SubscriptKind::Cardinal,
            span: span(start, close.span.end),
        })))
    }

    fn parse_turbofish_expression(&mut self, expr: TermExpression) -> Result<TermExpression, ParseError> {
        let start = expr.span().start;
        self.expect_symbol(TokenKind::DoubleColon)?;
        let arguments = self.parse_type_argument_clause()?;
        let end = self.previous().span.end;
        Ok(TermExpression::Turbofish { expr: Box::new(expr), arguments, span: span(start, end) })
    }

    fn parse_construct_expression(&mut self, lhs: TermExpression) -> Result<TermExpression, ParseError> {
        let path = match lhs {
            TermExpression::Name { path, .. } => path,
            TermExpression::Turbofish { expr, .. } => match *expr {
                TermExpression::Name { path, .. } => path,
                _ => return Err(self.error_here("struct constructor requires a type name")),
            },
            _ => return Err(self.error_here("struct constructor requires a type name")),
        };
        let start = path.span.start;
        self.expect_symbol(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated struct constructor"));
            }
            let field_name = self.expect_identifier_text()?.to_string();
            self.expect_symbol(TokenKind::Colon)?;
            let value = self.parse_value_expression_bp(0)?;
            fields.push((field_name, value));
            if !self.match_symbol(TokenKind::Comma) {
                break;
            }
        }
        let close = self.expect_symbol(TokenKind::RBrace)?;
        Ok(TermExpression::Construct { path, fields, span: span(start, close.span.end) })
    }
}

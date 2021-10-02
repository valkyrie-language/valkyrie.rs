use crate::{
    ast::{
        ArmStatement, ArrayPattern, CaseArm, DeclarationBody, ExtractPattern, LiteralExpression, MatchObjectField, ObjectPattern,
        PatternExpression, PatternOrExpression, TermExpression, TuplePattern,
    },
    lexer::{Keyword, TokenKind},
};
use valkyrie_types::Identifier;

use super::{parse_string_literal, span, ParseError, Parser};

impl<'a> Parser<'a> {
    /// 解析 `match scrutinee { ... }` 表达式。
    pub(super) fn parse_match_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Match)?.span.start;
        self.parse_case_like_expression(start, true)
    }

    /// 解析 `case scrutinee { ... }` 语句级控制流入口。
    pub(super) fn parse_case_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Case)?.span.start;
        self.parse_case_like_expression(start, false)
    }

    /// 解析 `match` / `case` 共享的 scrutinee 与 arm 体。
    fn parse_case_like_expression(&mut self, start: usize, produce_value: bool) -> Result<TermExpression, ParseError> {
        // 抑制结构体构造 postfix，避免误吞 match 块体。
        self.suppress_struct_constructor = true;
        let scrutinee = Box::new(self.parse_expression_bp(0)?);
        self.suppress_struct_constructor = false;
        self.expect_symbol(TokenKind::LBrace)?;

        let mut arms = Vec::new();
        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated match body"));
            }
            if self.match_symbol(TokenKind::Semicolon) {
                continue;
            }
            let arm = self.parse_match_arm()?;
            arms.push(arm);
        }
        let close = self.expect_symbol(TokenKind::RBrace)?;
        let span = span(start, close.span.end);
        if produce_value {
            Ok(TermExpression::Match { scrutinee, arms, span })
        }
        else {
            Ok(TermExpression::Match { scrutinee, arms, span })
        }
    }

    pub(super) fn parse_match_arm(&mut self) -> Result<ArmStatement, ParseError> {
        let start = self.current().span.start;
        let (pattern, guard) = if self.match_token_keyword(Keyword::Else) {
            self.expect_symbol(TokenKind::Colon)?;
            (None, None)
        }
        else {
            self.expect_token_keyword(Keyword::Case)?;
            let first_pattern = self.parse_match_single_pattern()?;
            let mut patterns = vec![first_pattern];
            while self.match_symbol(TokenKind::Pipe) {
                patterns.push(self.parse_match_single_pattern()?);
            }
            let guard = if self.match_token_keyword(Keyword::If) { Some(self.parse_expression_bp(0)?) } else { None };
            self.expect_symbol(TokenKind::Colon)?;
            if patterns.len() == 1 {
                (Some(patterns.pop().expect("single match pattern")), guard)
            }
            else {
                let end = self.previous().span.end;
                (Some(PatternExpression::Or(Box::new(PatternOrExpression { patterns, span: span(start, end) }))), guard)
            }
        };
        let body = self.parse_match_arm_body()?;
        Ok(ArmStatement::Case(CaseArm { pattern, guard, body, span: span(start, self.previous().span.end) }))
    }

    fn parse_match_single_pattern(&mut self) -> Result<PatternExpression, ParseError> {
        let start = self.current().span.start;
        match self.current().kind {
            TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False) => {
                let literal = LiteralExpression::Bool(self.current().kind == TokenKind::Keyword(Keyword::True));
                let token = self.bump().clone();
                self.parse_match_literal_or_range_pattern(literal, token.span.start, token.span.end)
            }
            TokenKind::StringLiteral => {
                let token = self.bump().clone();
                let literal = LiteralExpression::String(parse_string_literal(self.slice(token.span.clone()))?);
                self.parse_match_literal_or_range_pattern(literal, token.span.start, token.span.end)
            }
            TokenKind::IntegerLiteral => {
                let token = self.bump().clone();
                let literal = LiteralExpression::Integer(self.slice(token.span.clone()).to_string());
                self.parse_match_literal_or_range_pattern(literal, token.span.start, token.span.end)
            }
            TokenKind::FloatLiteral => {
                let token = self.bump().clone();
                let literal = LiteralExpression::Float(self.slice(token.span.clone()).to_string());
                self.parse_match_literal_or_range_pattern(literal, token.span.start, token.span.end)
            }
            TokenKind::DotDot | TokenKind::DotDotEq | TokenKind::DotDotLt => self.parse_match_leading_range_pattern(),
            TokenKind::LParen => {
                let open = self.bump().clone();
                if self.match_symbol(TokenKind::RParen) {
                    Ok(PatternExpression::Literal { literal: LiteralExpression::Unit, span: span(open.span.start, self.previous().span.end) })
                }
                else {
                    let first = self.parse_match_single_pattern()?;
                    if !self.match_symbol(TokenKind::Comma) {
                        self.expect_symbol(TokenKind::RParen)?;
                        Ok(first)
                    }
                    else {
                        let mut items = vec![first];
                        while !self.check_symbol(TokenKind::RParen) {
                            items.push(self.parse_match_single_pattern()?);
                            if !self.match_symbol(TokenKind::Comma) {
                                break;
                            }
                        }
                        let close = self.expect_symbol(TokenKind::RParen)?;
                        Ok(PatternExpression::Tuple(Box::new(TuplePattern { items, span: span(open.span.start, close.span.end) })))
                    }
                }
            }
            TokenKind::LBracket => self.parse_match_array_pattern(),
            TokenKind::Identifier => {
                let name = self.parse_name_path()?;
                if name.parts.len() == 1
                    && name.parts[0].chars().next().is_some_and(|it| it.is_lowercase())
                    && self.current().kind == TokenKind::Keyword(Keyword::As)
                    && self.nth_is_type_name(1)
                    && self.peek_pattern_type_name_starts_with_uppercase()
                {
                    self.expect_token_keyword(Keyword::As)?;
                    let ty = self.parse_name_path()?;
                    return Ok(PatternExpression::TypedBind { name: name.parts[0].clone(), ty, span: span(start, self.previous().span.end) });
                }
                if self.check_symbol(TokenKind::LBrace) {
                    let (fields, rest) = self.parse_match_object_fields()?;
                    return Ok(PatternExpression::Object(Box::new(ObjectPattern {
                        name: Some(name),
                        fields,
                        rest,
                        span: span(start, self.previous().span.end),
                    })));
                }
                if self.match_symbol(TokenKind::LParen) {
                    let mut fields = Vec::new();
                    while !self.check_symbol(TokenKind::RParen) {
                        fields.push(self.parse_match_single_pattern()?);
                        if !self.match_symbol(TokenKind::Comma) {
                            break;
                        }
                    }
                    self.expect_symbol(TokenKind::RParen)?;
                    return Ok(PatternExpression::Extract(Box::new(ExtractPattern {
                        name,
                        fields,
                        span: span(start, self.previous().span.end),
                    })));
                }
                if name.parts.len() == 1 {
                    let text = &name.parts[0];
                    if text == "_" {
                        return Ok(PatternExpression::Wildcard { span: name.span.clone() });
                    }
                    if text.chars().next().map(|it| it.is_lowercase()).unwrap_or(false) {
                        return Ok(PatternExpression::Variable { name: text.clone(), span: name.span.clone() });
                    }
                }
                Ok(PatternExpression::Name { path: name, span: span(start, self.previous().span.end) })
            }
            TokenKind::LBrace => {
                let (fields, rest) = self.parse_match_object_fields()?;
                Ok(PatternExpression::Object(Box::new(ObjectPattern { name: None, fields, rest, span: span(start, self.previous().span.end) })))
            }
            _ => Err(ParseError::invalid_at("unsupported match pattern", self.current().span.clone())),
        }
    }

    fn parse_match_literal_or_range_pattern(
        &mut self,
        literal: LiteralExpression,
        start: usize,
        literal_end: usize,
    ) -> Result<PatternExpression, ParseError> {
        if matches!(self.current().kind, TokenKind::DotDot | TokenKind::DotDotEq | TokenKind::DotDotLt) {
            return self.parse_match_range_pattern(Some(literal), start);
        }
        Ok(PatternExpression::Literal { literal, span: span(start, literal_end) })
    }

    fn parse_match_leading_range_pattern(&mut self) -> Result<PatternExpression, ParseError> {
        let start = self.current().span.start;
        self.parse_match_range_pattern(None, start)
    }

    fn parse_match_range_pattern(&mut self, start_literal: Option<LiteralExpression>, start: usize) -> Result<PatternExpression, ParseError> {
        let inclusive_end = match self.current().kind {
            TokenKind::DotDotEq => {
                self.bump();
                true
            }
            TokenKind::DotDot | TokenKind::DotDotLt => {
                self.bump();
                false
            }
            _ => return Err(ParseError::invalid_at("expected range operator", self.current().span.clone())),
        };
        let end = if self.is_match_range_bound_start() { Some(self.parse_match_range_bound_literal()?) } else { None };
        Ok(PatternExpression::Range { start: start_literal, end, inclusive_end, span: span(start, self.previous().span.end) })
    }

    fn parse_match_array_pattern(&mut self) -> Result<PatternExpression, ParseError> {
        let open = self.expect_symbol(TokenKind::LBracket)?;
        let mut prefix = Vec::new();
        let mut suffix = Vec::new();
        let mut rest = None;
        let mut seen_rest = false;

        while !self.check_symbol(TokenKind::RBracket) {
            if seen_rest {
                suffix.push(self.parse_match_single_pattern()?);
            }
            else if self.match_symbol(TokenKind::DotDot) {
                seen_rest = true;
                if matches!(self.current().kind, TokenKind::Identifier) {
                    rest = Some(Identifier::new(self.expect_identifier_text()?));
                }
            }
            else {
                prefix.push(self.parse_match_single_pattern()?);
            }

            if !self.match_symbol(TokenKind::Comma) {
                break;
            }
        }

        let close = self.expect_symbol(TokenKind::RBracket)?;
        Ok(PatternExpression::Array(Box::new(ArrayPattern { prefix, rest, suffix, span: span(open.span.start, close.span.end) })))
    }

    fn parse_match_object_fields(&mut self) -> Result<(Vec<MatchObjectField>, Option<Identifier>), ParseError> {
        self.expect_symbol(TokenKind::LBrace)?;
        let mut fields = Vec::new();
        let mut rest = None;
        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated object pattern"));
            }
            if self.match_symbol(TokenKind::Ellipsis) {
                rest = Some(Identifier::new(self.expect_identifier_text()?));
                if self.match_symbol(TokenKind::Comma) {
                    continue;
                }
                break;
            }
            let field_start = self.current().span.start;
            let field_name = self.expect_identifier_text()?.to_string();
            let pattern = if self.match_symbol(TokenKind::Colon) {
                self.parse_match_single_pattern()?
            }
            else {
                PatternExpression::Variable { name: field_name.clone(), span: span(field_start, self.previous().span.end) }
            };
            fields.push(MatchObjectField { name: field_name, pattern, span: span(field_start, self.previous().span.end) });
            if !self.match_symbol(TokenKind::Comma) {
                break;
            }
        }
        self.expect_symbol(TokenKind::RBrace)?;
        Ok((fields, rest))
    }

    /// 解析 match arm 体：不要求 `{}`，以 `case`/`else`/`}` 作为终止边界。
    pub(super) fn parse_match_arm_body(&mut self) -> Result<DeclarationBody, ParseError> {
        let start = self.current().span.start;
        let mut statements = Vec::new();
        let mut tail_expression = None;

        while !self.is_match_arm_terminator() {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated match arm body"));
            }
            if self.match_symbol(TokenKind::Semicolon) {
                continue;
            }

            if matches!(self.current().kind, TokenKind::Keyword(Keyword::Let) | TokenKind::Keyword(Keyword::Mut)) {
                statements.push(self.parse_let_statement()?);
                continue;
            }
            if matches!(
                self.current().kind,
                TokenKind::Keyword(
                    Keyword::Return | Keyword::Break | Keyword::Continue | Keyword::Yield | Keyword::Resume | Keyword::Fallthrough
                )
            ) {
                statements.push(self.parse_control_flow_statement()?);
                self.match_symbol(TokenKind::Semicolon);
                continue;
            }

            let expr = self.parse_expression_bp(0)?;
            let expr_span = expr.span().clone();
            if self.match_symbol(TokenKind::Semicolon) {
                statements.push(crate::ast::FunctionStatement::Term { span: expr_span, expression: expr });
                continue;
            }
            if self.is_match_arm_terminator() {
                tail_expression = Some(expr);
                break;
            }
            statements.push(crate::ast::FunctionStatement::Term { span: expr_span, expression: expr });
        }

        let end = tail_expression
            .as_ref()
            .map(|expression| expression.span().end)
            .or_else(|| statements.last().map(|statement| statement.span().end))
            .unwrap_or(start);
        Ok(DeclarationBody { statements, tail_expression, span: span(start, end) })
    }

    fn is_match_range_bound_start(&self) -> bool {
        matches!(
            self.current().kind,
            TokenKind::Keyword(Keyword::True)
                | TokenKind::Keyword(Keyword::False)
                | TokenKind::StringLiteral
                | TokenKind::IntegerLiteral
                | TokenKind::FloatLiteral
        )
    }

    fn parse_match_range_bound_literal(&mut self) -> Result<LiteralExpression, ParseError> {
        match self.current().kind {
            TokenKind::Keyword(Keyword::True) | TokenKind::Keyword(Keyword::False) => {
                let value = self.current().kind == TokenKind::Keyword(Keyword::True);
                self.bump();
                Ok(LiteralExpression::Bool(value))
            }
            TokenKind::StringLiteral => {
                let token = self.bump().clone();
                Ok(LiteralExpression::String(parse_string_literal(self.slice(token.span.clone()))?))
            }
            TokenKind::IntegerLiteral => {
                let token = self.bump().clone();
                Ok(LiteralExpression::Integer(self.slice(token.span.clone()).to_string()))
            }
            TokenKind::FloatLiteral => {
                let token = self.bump().clone();
                Ok(LiteralExpression::Float(self.slice(token.span.clone()).to_string()))
            }
            _ => Err(ParseError::invalid_at("unsupported range bound", self.current().span.clone())),
        }
    }

    fn peek_pattern_type_name_starts_with_uppercase(&self) -> bool {
        self.tokens
            .get(self.index + 1)
            .filter(|token| matches!(token.kind, TokenKind::Identifier | TokenKind::Keyword(_) | TokenKind::BacktickSymbol))
            .and_then(|token| self.slice(token.span.clone()).chars().next())
            .is_some_and(|ch| ch.is_uppercase())
    }
}

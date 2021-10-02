use crate::{
    ast::{
        BreakStatement, ContinueStatement, DeclarationBody, FallthroughStatement, FunctionStatement, IfStatement, LetStatement,
        LoopInStatement, LoopStatement, PatternExpression, ResumeStatement, ReturnStatement, TermExpression, TuplePattern, WhileStatement,
        YieldFromStatement, YieldStatement,
    },
    lexer::{Keyword, TokenKind},
};
use valkyrie_types::Identifier;

use super::{expression_requires_statement, span, ParseError, Parser};

impl<'a> Parser<'a> {
    pub(super) fn parse_return_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let start = self.expect_token_keyword(Keyword::Return)?.span.start;
        let value = if self.is_expression_terminator() { None } else { Some(Box::new(self.parse_value_expression_bp(0)?)) };
        let end = value.as_ref().map_or(self.previous().span.end, |expr| expr.span().end);
        Ok(FunctionStatement::Return(ReturnStatement { value: value.map(|value| *value), span: span(start, end) }))
    }

    pub(super) fn parse_break_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let start = self.expect_token_keyword(Keyword::Break)?.span.start;
        let label = if self.check_symbol(TokenKind::Apostrophe) { Some(Identifier::new(&self.parse_label_name()?)) } else { None };
        let value = if self.is_expression_terminator() { None } else { Some(Box::new(self.parse_value_expression_bp(0)?)) };
        let end = value.as_ref().map_or(self.previous().span.end, |expr| expr.span().end);
        Ok(FunctionStatement::Break(BreakStatement { label, value: value.map(|value| *value), span: span(start, end) }))
    }

    pub(super) fn parse_continue_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let start = self.expect_token_keyword(Keyword::Continue)?.span.start;
        let label = if self.check_symbol(TokenKind::Apostrophe) { Some(Identifier::new(&self.parse_label_name()?)) } else { None };
        let end = self.previous().span.end;
        Ok(FunctionStatement::Continue(ContinueStatement { label, span: span(start, end) }))
    }

    pub(super) fn parse_yield_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let start = self.expect_token_keyword(Keyword::Yield)?.span.start;
        if self.match_identifier_text_eq("from") {
            let value = self.parse_value_expression_bp(0)?;
            let end = value.span().end;
            return Ok(FunctionStatement::YieldFrom(YieldFromStatement { value, span: span(start, end) }));
        }

        let value = if self.is_expression_terminator() { None } else { Some(self.parse_value_expression_bp(0)?) };
        let end = value.as_ref().map_or(self.previous().span.end, |expr| expr.span().end);
        Ok(FunctionStatement::Yield(YieldStatement { value, span: span(start, end) }))
    }

    pub(super) fn parse_resume_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let start = self.expect_token_keyword(Keyword::Resume)?.span.start;
        let value = self.parse_value_expression_bp(0)?;
        let end = value.span().end;
        Ok(FunctionStatement::Resume(ResumeStatement { value: Some(value), span: span(start, end) }))
    }

    pub(super) fn is_expression_terminator(&self) -> bool {
        matches!(self.current().kind, TokenKind::Semicolon | TokenKind::RBrace | TokenKind::Comma | TokenKind::RParen | TokenKind::RBracket)
            || matches!(
                self.current().kind,
                TokenKind::Keyword(
                    Keyword::Case
                        | Keyword::Else
                        | Keyword::Let
                        | Keyword::Mut
                        | Keyword::Return
                        | Keyword::Break
                        | Keyword::Continue
                        | Keyword::Yield
                        | Keyword::Resume
                        | Keyword::Fallthrough
                )
            )
    }

    pub(super) fn parse_control_flow_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        match self.current().kind {
            TokenKind::Keyword(Keyword::Return) => self.parse_return_statement(),
            TokenKind::Keyword(Keyword::Break) => self.parse_break_statement(),
            TokenKind::Keyword(Keyword::Continue) => self.parse_continue_statement(),
            TokenKind::Keyword(Keyword::Yield) => self.parse_yield_statement(),
            TokenKind::Keyword(Keyword::Resume) => self.parse_resume_statement(),
            TokenKind::Keyword(Keyword::Fallthrough) => self.parse_fallthrough_statement(),
            _ => Err(self.error_here("expected control flow statement")),
        }
    }

    pub(super) fn parse_fallthrough_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let span = self.expect_token_keyword(Keyword::Fallthrough)?.span;
        Ok(FunctionStatement::Fallthrough(FallthroughStatement { span }))
    }

    /// 解析 `if condition { then } else { else }` 表达式。
    ///
    /// `else` 分支可选；当存在时既可以是块体，也可以是嵌套的 `if` 表达式。
    pub(super) fn parse_if_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::If)?.span.start;
        // 抑制结构体构造 postfix，避免误吞 if 块体。
        self.suppress_struct_constructor = true;
        let condition = self.parse_expression_bp(0)?;
        self.suppress_struct_constructor = false;
        let then_body = Box::new(self.parse_block_body()?);
        let else_body = if self.match_token_keyword(Keyword::Else) {
            if self.check_token_keyword(Keyword::If) {
                let nested = self.parse_if_expression()?;
                let nested_span = nested.span().clone();
                Some(Box::new(DeclarationBody { statements: Vec::new(), tail_expression: Some(nested), span: nested_span }))
            }
            else {
                Some(Box::new(self.parse_block_body()?))
            }
        }
        else {
            None
        };
        let end = else_body.as_ref().map_or(then_body.span.end, |body| body.span.end);
        Ok(TermExpression::If(Box::new(IfStatement {
            condition,
            then_body: *then_body,
            else_body: else_body.map(|b| *b),
            span: span(start, end),
        })))
    }

    /// 解析 `loop` 表达式。
    ///
    /// 支持两种语法：
    /// - 无限循环：`loop { body }`
    /// - 迭代循环：`loop pattern in iterator { body }`
    pub(super) fn parse_loop_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::Loop)?.span.start;
        self.parse_loop_expression_after_keyword(start, None)
    }

    pub(super) fn parse_loop_expression_after_keyword(
        &mut self,
        start: usize,
        label: Option<Identifier>,
    ) -> Result<TermExpression, ParseError> {
        if self.check_symbol(TokenKind::LBrace) {
            let body = self.parse_block_body()?;
            let end = body.span.end;
            if let Some(label) = label {
                return Ok(TermExpression::LoopIn(Box::new(LoopInStatement {
                    label: Some(label),
                    pattern: None,
                    iterator: None,
                    condition: None,
                    body,
                    span: span(start, end),
                })));
            }
            return Ok(TermExpression::Loop(Box::new(LoopStatement { body, span: span(start, end) })));
        }

        let checkpoint = self.index;
        if matches!(self.current().kind, TokenKind::Identifier | TokenKind::LParen) {
            if let Ok(pattern) = self.parse_pattern_expression() {
                if self.match_token_keyword(Keyword::In) {
                    // 抑制结构体构造 postfix，避免误吞 loop 块体。
                    self.suppress_struct_constructor = true;
                    let iterator = Some(self.parse_expression_bp(0)?);
                    self.suppress_struct_constructor = false;
                    let body = self.parse_block_body()?;
                    let end = body.span.end;
                    return Ok(TermExpression::LoopIn(Box::new(LoopInStatement {
                        label,
                        pattern: Some(pattern),
                        iterator,
                        condition: None,
                        body,
                        span: span(start, end),
                    })));
                }
            }
            self.index = checkpoint;
        }

        Err(ParseError::invalid_at("loop 只支持 `loop {}` 或 `loop pattern in iterator {}`", self.current().span.clone()))
    }

    /// 解析 `while condition { body }` 表达式。
    ///
    /// `while condition { body }` 复用 `TermExpression::Loop`。
    pub(super) fn parse_while_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.expect_token_keyword(Keyword::While)?.span.start;
        self.parse_while_expression_after_keyword(start, None)
    }

    pub(super) fn parse_while_expression_after_keyword(
        &mut self,
        start: usize,
        label: Option<Identifier>,
    ) -> Result<TermExpression, ParseError> {
        // 抑制结构体构造 postfix，避免误吞 while 块体。
        self.suppress_struct_constructor = true;
        let condition = Some(self.parse_expression_bp(0)?);
        self.suppress_struct_constructor = false;
        let body = self.parse_block_body()?;
        let end = body.span.end;
        Ok(TermExpression::While(Box::new(WhileStatement { label, condition, body, span: span(start, end) })))
    }

    pub(super) fn parse_labeled_loop_expression(&mut self) -> Result<TermExpression, ParseError> {
        let start = self.current().span.start;
        let label = Identifier::new(&self.parse_label_name()?);
        self.expect_symbol(TokenKind::Colon)?;
        if self.check_token_keyword(Keyword::Loop) {
            self.expect_token_keyword(Keyword::Loop)?;
            self.parse_loop_expression_after_keyword(start, Some(label))
        }
        else if self.check_token_keyword(Keyword::While) {
            self.expect_token_keyword(Keyword::While)?;
            self.parse_while_expression_after_keyword(start, Some(label))
        }
        else {
            Err(ParseError::invalid_at("label 只允许附着在 `loop` 或 `while` 上", self.current().span.clone()))
        }
    }

    /// 判断当前 token 是否为 match arm 体的终止边界。
    pub(super) fn is_match_arm_terminator(&self) -> bool {
        if self.check_symbol(TokenKind::RBrace) || self.is_eof() {
            return true;
        }
        if matches!(self.current().kind, TokenKind::Keyword(crate::lexer::Keyword::Case | crate::lexer::Keyword::Else)) {
            return true;
        }
        false
    }

    pub(super) fn parse_pattern_expression(&mut self) -> Result<PatternExpression, ParseError> {
        match self.current().kind {
            TokenKind::Identifier => {
                let name = self.expect_identifier_text()?.to_string();
                let span = self.previous().span.clone();
                if name == "_" {
                    Ok(PatternExpression::Wildcard { span })
                }
                else {
                    Ok(PatternExpression::Variable { name, span })
                }
            }
            TokenKind::LParen => {
                let open = self.expect_symbol(TokenKind::LParen)?;
                let items = self.parse_comma_separated_until(TokenKind::RParen, |parser| parser.parse_pattern_expression())?;
                let close = self.expect_symbol(TokenKind::RParen)?;
                Ok(PatternExpression::Tuple(Box::new(TuplePattern { items, span: span(open.span.start, close.span.end) })))
            }
            _ => Err(self.error_here("expected pattern")),
        }
    }

    pub(super) fn parse_block_body(&mut self) -> Result<DeclarationBody, ParseError> {
        let open = self.expect_symbol(TokenKind::LBrace)?;
        let mut statements = Vec::new();
        let mut tail_expression = None;

        while !self.check_symbol(TokenKind::RBrace) {
            if self.is_eof() {
                return Err(ParseError::invalid("unterminated block body"));
            }
            if self.match_symbol(TokenKind::Semicolon) {
                continue;
            }

            if self.check_token_keyword(Keyword::Let) || self.check_token_keyword(Keyword::Mut) {
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
            if self.match_symbol(TokenKind::Semicolon) {
                statements.push(FunctionStatement::Term { span: expr.span().clone(), expression: expr });
                continue;
            }

            // `if`/`loop`/`match` 等控制流表达式可作为语句使用，不需要分号终止。
            if matches!(expr, TermExpression::If(_) | TermExpression::Loop(_) | TermExpression::Match { .. }) {
                statements.push(FunctionStatement::Term { span: expr.span().clone(), expression: expr });
                continue;
            }

            // 换行隐式终止：当下一个 token 是新语句起始关键字时，当前表达式作为语句结束。
            if self.is_statement_start() {
                statements.push(FunctionStatement::Term { span: expr.span().clone(), expression: expr });
                continue;
            }

            if !self.check_symbol(TokenKind::RBrace) {
                return Err(self.error_here("expected ';' or '}' after expression"));
            }

            if expression_requires_statement(&expr) {
                statements.push(FunctionStatement::Term { span: expr.span().clone(), expression: expr });
            }
            else {
                tail_expression = Some(expr);
            }
            break;
        }

        let close = self.expect_symbol(TokenKind::RBrace)?;
        Ok(DeclarationBody { statements, tail_expression, span: span(open.span.end, close.span.start) })
    }

    pub(super) fn parse_let_statement(&mut self) -> Result<FunctionStatement, ParseError> {
        let start = self.current().span.start;
        let saw_let = self.match_token_keyword(Keyword::Let);
        let is_mutable = self.match_token_keyword(Keyword::Mut);
        if !saw_let && !is_mutable {
            return Err(self.error_here("expected let binding"));
        }

        let pattern = self.parse_pattern_expression()?;
        let ty = if self.match_symbol(TokenKind::Colon) { Some(self.parse_type_expression_bp(0)?) } else { None };
        let initializer = if self.match_symbol(TokenKind::Equal) { Some(self.parse_expression_bp(0)?) } else { None };
        // 接受 `;`、`}`、EOF 或新语句起始关键字作为隐式终止符。
        if !self.match_symbol(TokenKind::Semicolon) && !self.check_symbol(TokenKind::RBrace) && !self.is_eof() && !self.is_statement_start() {
            return Err(self.error_here("expected ';' or statement boundary after let binding"));
        }

        let end = self.previous().span.end;
        Ok(FunctionStatement::Let(LetStatement { is_mutable, pattern, ty, initializer, span: span(start, end) }))
    }
}

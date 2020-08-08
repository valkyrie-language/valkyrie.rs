use super::*;
use crate::{table::TupleNode, utils::parse_expression_body};
use valkyrie_ast::{
    CallNode, ExpressionContext, ExpressionNode, LambdaCallNode, LambdaDotNode, LambdaNode, NewConstructNode, PrettyPrint,
    StatementType::Expression,
};

impl ThisParser for PrefixNode {
    fn parse(_: ParseState) -> ParseResult<Self> {
        unreachable!()
    }

    fn as_lisp(&self) -> Lisp {
        Lisp::operator(self.operator.kind.as_str(), &[self.base.as_lisp()])
    }
}

impl ThisParser for InfixNode {
    fn parse(_: ParseState) -> ParseResult<Self> {
        unreachable!()
    }

    fn as_lisp(&self) -> Lisp {
        Lisp::operator(self.operator.kind.as_str(), &[self.lhs.as_lisp(), self.rhs.as_lisp()])
    }
}

impl ThisParser for PostfixNode {
    fn parse(_: ParseState) -> ParseResult<Self> {
        unreachable!()
    }

    fn as_lisp(&self) -> Lisp {
        Lisp::operator(self.operator.kind.as_str(), &[self.base.as_lisp()])
    }
}

impl ThisParser for ExpressionNode {
    fn parse(input: ParseState) -> ParseResult<Self> {
        parse_expression_node(input, ExpressionContext::default())
    }

    fn as_lisp(&self) -> Lisp {
        self.body.as_lisp()
    }
}

impl ThisParser for ExpressionBody {
    fn parse(input: ParseState) -> ParseResult<Self> {
        parse_expression_body(input, ExpressionContext::default())
    }

    fn as_lisp(&self) -> Lisp {
        match self {
            Self::Placeholder => Lisp::Keyword("placeholder".into()),
            Self::Prefix(v) => v.as_lisp(),
            Self::Binary(v) => v.as_lisp(),
            Self::Suffix(v) => v.as_lisp(),
            Self::Number(v) => v.as_lisp(),
            Self::Symbol(v) => v.as_lisp(),
            Self::String(v) => v.as_lisp(),
            Self::Table(v) => v.as_lisp(),
            Self::Apply(v) => v.as_lisp(),
            Self::ApplyDot(v) => v.as_lisp(),
            Self::Subscript(v) => v.as_lisp(),
            Self::GenericCall(v) => v.as_lisp(),
            Self::LambdaCall(v) => v.as_lisp(),
            Self::LambdaDot(v) => v.as_lisp(),
            Self::New(v) => v.as_lisp(),
        }
    }
}

impl ExpressionStream {
    /// term (~ infix ~ term)*
    /// 1 + (1 + +3? + 4)
    pub fn parse(input: ParseState, ctx: ExpressionContext) -> ParseResult<Vec<ExpressionStream>> {
        let mut stream = Vec::with_capacity(4);
        let (state, _) = input.match_fn(|s| parse_term(s, &mut stream, ctx))?;
        let (state, _) = state.match_repeats(|s| parse_infix_term(s, &mut stream, ctx))?;
        state.finish(stream)
    }
}

/// `~ infix ~ term`
#[inline(always)]
fn parse_infix_term<'i>(
    input: ParseState<'i>,
    stream: &mut Vec<ExpressionStream>,
    ctx: ExpressionContext,
) -> ParseResult<'i, ()> {
    let (state, infix) = ValkyrieInfix::parse(input.skip(ignore), ctx.type_level)?;
    stream.push(ExpressionStream::Infix(infix));
    let (state, _) = state.skip(ignore).match_fn(|s| parse_term(s, stream, ctx))?;
    state.finish(())
}

/// `( ~ term ~ )`
pub fn parse_group(input: ParseState, ctx: ExpressionContext) -> ParseResult<Vec<ExpressionStream>> {
    let (state, _) = input.match_char('(')?;
    let (state, group) = state.skip(ignore).match_fn(|s| ExpressionStream::parse(s, ctx))?;
    let (state, _) = state.skip(ignore).match_char(')')?;
    // Only join the global stream after all success
    state.finish(group)
}

/// `(~ prefix)* ~ value (~ suffix)*`
fn parse_term<'i>(state: ParseState<'i>, stream: &mut Vec<ExpressionStream>, ctx: ExpressionContext) -> ParseResult<'i, ()> {
    let (state, _) = state.match_repeats(|s| parse_prefix(s, stream))?;
    let (state, _) = parse_expr_value(state, stream, ctx)?;
    let (state, _) = state.match_repeats(|s| parse_suffix(s, stream))?;
    state.finish(())
}

#[inline(always)]
fn parse_prefix<'a>(input: ParseState<'a>, stream: &mut Vec<ExpressionStream>) -> ParseResult<'a, ()> {
    let (state, prefix) = input.skip(ignore).match_fn(ValkyriePrefix::parse)?;
    stream.push(ExpressionStream::Prefix(prefix));
    state.finish(())
}

#[inline(always)]
fn parse_suffix<'a>(input: ParseState<'a>, stream: &mut Vec<ExpressionStream>) -> ParseResult<'a, ()> {
    let (state, suffix) = input.skip(ignore).match_fn(ValkyrieSuffix::parse)?;
    stream.push(ExpressionStream::Postfix(suffix));
    state.finish(())
}

#[inline]
fn parse_expr_value<'a>(
    input: ParseState<'a>,
    stream: &mut Vec<ExpressionStream>,
    ctx: ExpressionContext,
) -> ParseResult<'a, ()> {
    let (state, term) = input
        .skip(ignore)
        .begin_choice()
        .or_else(|s| parse_group(s, ctx).map_inner(ExpressionStream::Group))
        .or_else(|s| parse_value(s, ctx.allow_curly).map_inner(ExpressionStream::Term))
        .end_choice()?;

    stream.push(term);
    state.finish(())
}

pub enum NormalPostfixCall {
    Apply(ApplyCallNode),
    ApplyDot(Box<ApplyDotNode>),
    View(Box<SubscriptNode>),
    Generic(GenericCallNode),
    Lambda(Box<LambdaCallNode>),
    LambdaDot(Box<LambdaDotNode>),
}

#[inline]
pub fn parse_value(input: ParseState, allow_curly: bool) -> ParseResult<ExpressionNode> {
    let (state, mut base) = input
        .begin_choice()
        .or_else(|s| NewConstructNode::parse(s).map_inner(|s| ExpressionBody::New(Box::new(s))))
        .or_else(|s| NamePathNode::parse(s).map_inner(Into::into))
        .or_else(|s| NumberLiteralNode::parse(s).map_inner(Into::into))
        .or_else(|s| StringLiteralNode::parse(s).map_inner(Into::into))
        .or_else(|s| TableNode::parse(s).map_inner(Into::into))
        .or_else(|s| TupleNode::parse(s).map_inner(|s| ExpressionBody::Table(Box::new(s.as_table()))))
        .end_choice()?;
    let (state, rest) = match allow_curly {
        true => state.match_repeats(NormalPostfixCall::parse_allow_curly),
        false => state.match_repeats(NormalPostfixCall::parse),
    }?;
    for caller in rest {
        match caller {
            NormalPostfixCall::Apply(v) => base = ExpressionNode::call_apply(base, v),
            NormalPostfixCall::ApplyDot(v) => base = ExpressionBody::ApplyDot(v.rebase(base)),
            NormalPostfixCall::View(v) => base = ExpressionBody::Subscript(v.rebase(base)),
            NormalPostfixCall::Generic(v) => base = ExpressionNode::call_generic(base, v),
            NormalPostfixCall::Lambda(v) => base = ExpressionBody::LambdaCall(v.rebase(base)),
            NormalPostfixCall::LambdaDot(v) => base = ExpressionBody::LambdaDot(v.rebase(base)),
        }
    }
    state.finish(base)
}

impl NormalPostfixCall {
    fn parse(input: ParseState) -> ParseResult<Self> {
        input
            .skip(ignore)
            .begin_choice()
            .or_else(|s| ApplyCallNode::parse(s).map_inner(|s| NormalPostfixCall::Apply(Box::new(s))))
            .or_else(|s| ApplyDotNode::parse(s).map_inner(|s| NormalPostfixCall::ApplyDot(Box::new(s))))
            .or_else(|s| SubscriptNode::parse(s).map_inner(|s| NormalPostfixCall::View(Box::new(s))))
            .or_else(|s| GenericCallNode::parse(s).map_inner(|s| NormalPostfixCall::Generic(Box::new(s))))
            .end_choice()
    }
    fn parse_allow_curly(input: ParseState) -> ParseResult<Self> {
        input
            .skip(ignore)
            .begin_choice()
            .or_else(|s| ApplyCallNode::parse(s).map_inner(|s| NormalPostfixCall::Apply(Box::new(s))))
            .or_else(|s| ApplyDotNode::parse(s).map_inner(|s| NormalPostfixCall::ApplyDot(Box::new(s))))
            .or_else(|s| SubscriptNode::parse(s).map_inner(|s| NormalPostfixCall::View(Box::new(s))))
            .or_else(|s| GenericCallNode::parse(s).map_inner(|s| NormalPostfixCall::Generic(Box::new(s))))
            .or_else(|s| LambdaCallNode::parse(s).map_inner(|s| NormalPostfixCall::Lambda(Box::new(s))))
            .or_else(|s| LambdaDotNode::parse(s).map_inner(|s| NormalPostfixCall::LambdaDot(Box::new(s))))
            .end_choice()
    }
}

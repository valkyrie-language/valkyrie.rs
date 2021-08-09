use crate::helpers::ProgramState;
use nyar_error::{NyarError, Result, SourceID};
use pratt::{Affix, PrattParser, Precedence};
use std::str::FromStr;
use valkyrie_ast::*;
mod call_dot;
mod call_dot_closure;
mod call_dot_match;
mod call_generic;
mod control_flow;
mod operators;

impl crate::ExpressionRootNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<StatementKind> {
        let expr = self.main_expression.build(ctx)?;
        let eos = self.eos.is_some();
        let ex = ExpressionNode { omit: eos, body: expr, span: self.span.clone() };
        Ok(StatementKind::Expression(Box::new(ex)))
    }
}

impl crate::MainExpressionNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let mut stream = vec![];
        let (head, rest) = self.main_term.split_first().expect("at least one term");
        head.push_tokens(&mut stream, ctx)?;
        for (infix, rhs) in self.main_infix.iter().zip(rest.iter()) {
            stream.push(TokenStream::Infix(infix.as_operator()));
            rhs.push_tokens(&mut stream, ctx)?;
        }
        let mut parser = ExpressionResolver;
        let expr = parser.parse(stream.into_iter())?;
        Ok(expr)
    }
}
impl crate::InlineExpressionNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let mut stream = vec![];
        let (head, rest) = self.inline_term.split_first().expect("at least one term");
        head.push_tokens(&mut stream, ctx)?;
        for (infix, rhs) in self.main_infix.iter().zip(rest.iter()) {
            stream.push(TokenStream::Infix(infix.as_operator()));
            rhs.push_tokens(&mut stream, ctx)?;
        }
        let mut parser = ExpressionResolver;
        let expr = parser.parse(stream.into_iter())?;
        Ok(expr)
    }
}
impl crate::TypeExpressionNode {
    pub fn build_external(&self, file: SourceID) -> Result<ExpressionKind> {
        let mut stream = ProgramState::new(file);
        self.build(&mut stream)
    }
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let mut stream = vec![];
        let (head, rest) = self.type_term.split_first().expect("at least one term");
        head.push_tokens(&mut stream, ctx)?;
        for (infix, rhs) in self.type_infix.iter().zip(rest.iter()) {
            stream.push(TokenStream::Infix(infix.as_operator()));
            rhs.push_tokens(&mut stream, ctx)?;
        }
        let mut parser = ExpressionResolver;
        let expr = parser.parse(stream.into_iter())?;
        Ok(expr)
    }
}

impl crate::MainTermNode {
    fn push_tokens(&self, stream: &mut Vec<TokenStream>, ctx: &mut ProgramState) -> Result<()> {
        for i in &self.main_prefix {
            stream.push(TokenStream::Prefix(i.as_operator()))
        }
        let main = self.main_factor.build(ctx)?;
        stream.push(TokenStream::Term(main));
        for i in &self.main_suffix_term {
            stream.push(i.as_token(ctx)?)
        }
        Ok(())
    }
}
impl crate::InlineTermNode {
    fn push_tokens(&self, stream: &mut Vec<TokenStream>, ctx: &mut ProgramState) -> Result<()> {
        for i in &self.main_prefix {
            stream.push(TokenStream::Prefix(i.as_operator()))
        }
        let main = self.main_factor.build(ctx)?;
        stream.push(TokenStream::Term(main));
        for i in &self.inline_suffix_term {
            stream.push(i.as_token(ctx)?)
        }
        Ok(())
    }
}
impl crate::TypeTermNode {
    fn push_tokens(&self, stream: &mut Vec<TokenStream>, ctx: &mut ProgramState) -> Result<()> {
        for i in &self.type_prefix {
            stream.push(TokenStream::Prefix(i.as_operator()))
        }
        let main = self.main_factor.build(ctx)?;
        stream.push(TokenStream::Term(main));
        for i in &self.type_suffix_term {
            stream.push(i.as_token(ctx)?)
        }
        Ok(())
    }
}

struct ExpressionResolver;

#[derive(Debug)]
enum TokenStream {
    Prefix(OperatorNode),
    Infix(OperatorNode),
    Term(ExpressionKind),
    Postfix(OperatorNode),
    Subscript(SubscriptCallNode),
    Generic(GenericCallNode),
    Apply(ApplyCallNode),
    Dot(DotCallNode),
    DotMatch(MatchCallNode),
    DotClosure(ClosureCallNode),
}

impl<I> PrattParser<I> for ExpressionResolver
where
    I: Iterator<Item = TokenStream>,
{
    type Error = NyarError;
    type Input = TokenStream;
    type Output = ExpressionKind;

    fn query(&mut self, input: &Self::Input) -> Result<Affix> {
        let affix = match input {
            TokenStream::Prefix(v) => Affix::Prefix(v.kind.precedence()),
            TokenStream::Infix(v) => Affix::Infix(v.kind.precedence(), v.kind.associativity()),
            TokenStream::Term(_) => Affix::Nilfix,
            TokenStream::Postfix(v) => Affix::Postfix(v.kind.precedence()),
            _ => Affix::Postfix(Precedence(10000)),
        };
        Ok(affix)
    }

    fn primary(&mut self, input: Self::Input) -> Result<Self::Output> {
        match input {
            TokenStream::Term(v) => Ok(v),
            _ => unreachable!(),
        }
    }

    fn infix(&mut self, lhs: Self::Output, op: Self::Input, rhs: Self::Output) -> Result<Self::Output> {
        match op {
            TokenStream::Infix(v) => Ok(BinaryNode { infix: v, lhs, rhs }.into()),
            _ => unreachable!(),
        }
    }

    fn prefix(&mut self, op: Self::Input, rhs: Self::Output) -> Result<Self::Output> {
        match op {
            TokenStream::Prefix(v) => Ok(UnaryNode { operator: v, base: rhs }.into()),
            _ => unreachable!(),
        }
    }

    fn postfix(&mut self, lhs: Self::Output, op: Self::Input) -> Result<Self::Output> {
        match op {
            TokenStream::Postfix(v) => Ok(UnaryNode { operator: v, base: lhs }.into()),
            TokenStream::Subscript(call) => Ok(call.with_base(lhs).into()),
            TokenStream::Apply(call) => Ok(call.with_base(lhs).into()),
            TokenStream::Dot(call) => Ok(call.with_base(lhs).into()),
            TokenStream::Generic(call) => Ok(call.with_base(lhs).into()),
            TokenStream::DotMatch(call) => Ok(call.with_base(lhs).into()),
            TokenStream::DotClosure(call) => Ok(call.with_base(lhs).into()),
            _ => unreachable!(),
        }
    }
}

impl crate::MainFactorNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        match self {
            Self::Leading(v) => v.build(ctx),
            Self::GroupFactor(v) => v.main_expression.build(ctx),
            Self::NewStatement(v) => v.build(ctx).map(Into::into),
            Self::ObjectStatement(v) => v.build(ctx).map(Into::into),
            Self::DefineLambda(v) => v.build(ctx).map(Into::into),
            Self::TryStatement(v) => v.build(ctx).map(Into::into),
            Self::MatchExpression(v) => v.build(ctx).map(Into::into),
            Self::SwitchStatement(v) => v.build(ctx).map(Into::into),
        }
    }
}
impl crate::TypeFactorNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        match self {
            Self::Leading(v) => v.build(ctx),
            Self::TypeExpression(v) => v.build(ctx),
        }
    }
}

impl crate::MainSuffixTermNode {
    fn as_token(&self, ctx: &mut ProgramState) -> Result<TokenStream> {
        let token = match self {
            Self::InlineSuffixTerm(v) => v.as_token(ctx)?,
            Self::DotMatchCall(v) => TokenStream::DotMatch(v.build(ctx)?),
            Self::DotClosureCall(v) => TokenStream::DotClosure(v.build(ctx)?),
            Self::TupleCall(v) => TokenStream::Apply(v.build(ctx)?),
        };
        Ok(token)
    }
}

impl crate::InlineSuffixTermNode {
    fn as_token(&self, ctx: &mut ProgramState) -> Result<TokenStream> {
        let token = match self {
            Self::MainSuffix(v) => TokenStream::Postfix(v.as_operator()),
            Self::RangeCall(v) => TokenStream::Subscript(v.build(ctx)?),
            Self::InlineTupleCall(v) => TokenStream::Apply(v.build(ctx)?),
            Self::DotCall(v) => TokenStream::Dot(v.build(ctx)?),
            Self::GenericCall(v) => TokenStream::Generic(v.build(ctx)?),
        };
        Ok(token)
    }
}

impl crate::TypeSuffixTermNode {
    fn as_token(&self, ctx: &mut ProgramState) -> Result<TokenStream> {
        let token = match self {
            Self::GenericHide(v) => TokenStream::Generic(v.build_call(ctx)?),
            Self::TypeSuffix(v) => TokenStream::Postfix(v.as_operator()),
        };
        Ok(token)
    }
}

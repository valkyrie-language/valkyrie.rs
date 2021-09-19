use super::*;
use crate::{traits::YggdrasilNodeExtension, SignNode};
use std::{str::FromStr, sync::Arc};
use valkyrie_ast::NullNode;
use valkyrie_types::{Identifier, NyarError};
use yggdrasil_rt::YggdrasilNode;

// A number literal.
// #[derive(Debug, Clone, Eq, Hash)]
// pub struct IntegerNode {}

//     ⍚(_*[0-9A-F])* # hex
// |   ⍙(_*[0-7])*       # octal
// |   ⍜(_*[01])*        # binary

impl<'i> crate::NumberNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let n = match self {
            Self::Decimal(v) => v.build(ctx)?,
            Self::DecimalX(v) => v.build(ctx)?,
        };
        Ok(n)
    }
}

impl<'i> crate::DecimalNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let mut n = NumberLiteralNode::new(10);
        n.set_span(ctx.file.with_range(self.get_range32()));
        n.set_integer(&self.lhs().get_str(), ctx.file, self.lhs().get_range().start)?;
        if let Some(s) = &self.rhs() {
            n.set_decimal(s.get_str(), ctx.file, s.get_range().start)?
        }
        if let Some(s) = &self.unit() {
            n.unit = Some(s.build(ctx.file))
        }
        if let Some(s) = &self.shift() {
            match &self.sign() {
                Some(SignNode::Netative(_)) => n.shift = -s.parse::<isize>(ctx)?,
                _ => n.shift = s.parse::<isize>(ctx)?,
            }
        }
        n.set_dot(self.dot().is_some());
        Ok(n.into())
    }
}

impl<'i> crate::DecimalXNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let mut n = NumberLiteralNode::new(self.base().as_base(ctx)?);
        n.set_span(ctx.file.with_range(self.get_range32()));
        n.set_integer(self.lhs().get_str(), ctx.file, self.lhs().get_range().start as usize)?;
        if let Some(s) = &self.rhs() {
            n.set_decimal(s.get_str(), ctx.file, s.get_range().start as usize)?
        }
        if let Some(s) = &self.unit() {
            n.unit = Some(s.build(ctx.file))
        }
        if let Some(s) = &self.shift() {
            match &self.sign() {
                Some(SignNode::Netative(_)) => n.shift = -s.parse::<isize>(ctx)?,
                _ => n.shift = s.parse::<isize>(ctx)?,
            }
        }
        n.set_dot(self.dot().is_some());
        Ok(n.into())
    }
}

impl<'i> crate::IntegerNode<'i> {
    // pub(crate) fn build(&self) -> NumberLiteralNode {
    //     NumberLiteralNode::new(10, self.get_range32())
    // }
    pub(crate) fn as_identifier(&self, ctx: &mut ProgramState) -> IdentifierNode {
        let text: String = self.get_chars().filter(|c| c.is_digit(10)).collect();
        IdentifierNode { name: Identifier::new(&text), span: ctx.file.with_range(self.get_range32()), shadow_index: 0 }
    }
    pub(crate) fn as_base(&self, ctx: &mut ProgramState) -> Result<u32> {
        let span = ctx.file.with_range(self.get_range32());
        match u32::from_str(self.get_str()) {
            Ok(o) if o >= 2 && o <= 36 => Ok(o),
            Ok(_) => Err(NyarError::syntax_error(format!("Currently only `2 ⩽ base ⩽ 36` is supported"), span)),
            Err(e) => Err(NyarError::syntax_error(e.to_string(), span)),
        }
    }
    pub(crate) fn parse<T>(&self, ctx: &mut ProgramState) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::error::Error,
    {
        let span = ctx.file.with_range(self.get_range32());
        match T::from_str(self.get_str()) {
            Ok(o) => Ok(o),
            Err(e) => Err(NyarError::syntax_error(e.to_string(), span)),
        }
    }
}

impl<'i> crate::SpecialNode<'i> {
    pub(crate) fn build(&self) -> ExpressionKind {
        match self.get_str() {
            "false" => BooleanNode { value: false, span: self.get_range32() }.into(),
            "true" => BooleanNode { value: true, span: self.get_range32() }.into(),
            "∞" => NullNode { nil: true, span: self.get_range32() }.into(),
            "∅" | "nil" => NullNode { nil: true, span: self.get_range32() }.into(),
            _ => unimplemented!("Unknown special value: {}", self.get_str()),
        }
    }
}

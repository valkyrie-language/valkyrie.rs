use crate::{helpers::ProgramState, utils::build_annotation_terms};
use nyar_error::{Result, SyntaxError};
use std::num::NonZeroU64;
use valkyrie_ast::*;

mod bytes;
mod create_lambda;
mod create_new;
mod create_object;
mod create_try;
mod identifier;
mod number;
mod procedural;
mod range;
mod string;
mod tuple;

impl crate::LeadingNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExpressionKind> {
        let value = match self {
            Self::Special(v) => v.build(),
            Self::Number(v) => v.build(ctx)?.into(),
            Self::Slot(v) => v.build(ctx)?.into(),
            Self::Namepath(v) => v.build(ctx).into(),
            Self::ProceduralCall(v) => v.build(ctx).into(),
            Self::RangeLiteral(v) => v.build(ctx)?.into(),
            Self::TupleLiteralStrict(v) => v.build(ctx)?.into(),
            Self::TextLiteral(v) => v.build(ctx).into(),
        };
        Ok(value)
    }
}

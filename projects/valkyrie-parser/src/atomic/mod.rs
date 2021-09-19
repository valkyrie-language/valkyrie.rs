use crate::{
    helpers::ProgramState,
    traits::YggdrasilNodeExtension,
    utils::{build_annotation_terms, Ast2Hir},
};
use std::{num::NonZeroU64, sync::Arc};
use valkyrie_ast::*;
use valkyrie_types::{Result, SourceID, SyntaxError};
use yggdrasil_rt::YggdrasilNode;

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

impl<'i> crate::LeadingNode<'i> {
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

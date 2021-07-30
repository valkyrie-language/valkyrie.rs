use crate::{
    helpers::ProgramState,
    utils::{build_annotation_terms, build_annotation_terms_mix, Ast2Hir},
};
use nyar_error::Result;
use valkyrie_ast::*;
mod annotation;
mod import;
mod namespace;

impl crate::ProgramNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ProgramRoot> {
        let mut statements = vec![];
        for node in &self.statement {
            match node.build(ctx) {
                Ok(o) => statements.extend(o),
                Err(e) => ctx.add_error(e),
            }
        }
        Ok(ProgramRoot { statements })
    }
}

impl crate::StatementNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<Option<StatementKind>> {
        let value = match self {
            Self::DefineNamespace(v) => v.build(ctx).into(),
            Self::DefineClass(v) => v.build(ctx)?.into(),
            Self::DefineEnumerate(v) => v.build(ctx)?.into(),
            Self::DefineFunction(v) => v.build(ctx)?.into(),
            Self::DefineVariable(v) => v.build(ctx)?.into(),
            Self::DefineTrait(v) => v.build(ctx)?.into(),
            Self::DefineExtends(v) => v.build(ctx)?.into(),
            Self::DefineUnion(v) => v.build(ctx)?.into(),
            Self::ControlFlow(v) => v.build(ctx)?.into(),
            Self::DefineImport(v) => v.build(ctx)?.into(),
            Self::ForStatement(v) => v.build(ctx)?.into(),
            Self::WhileStatement(v) => v.build(ctx)?.into(),
            Self::ExpressionRoot(v) => v.build(ctx)?.into(),
            Self::Eos(_) => return Ok(None),
        };
        Ok(Some(value))
    }
}

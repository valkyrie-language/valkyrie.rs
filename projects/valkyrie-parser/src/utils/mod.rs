use crate::{helpers::ProgramState, AnnotationTermMixNode, AnnotationTermNode, DefineConstraintNode, ModifierAheadNode};
use nyar_error::Result;
use valkyrie_ast::{AttributeList, ConstraintDeclaration, ModifierList};

pub(crate) trait Ast2Hir<Out> {
    fn to_hir(&self, ctx: &mut ProgramState) -> Out;
}
pub(crate) fn build_constraint(this: &Option<DefineConstraintNode>, ctx: &mut ProgramState) -> ConstraintDeclaration {
    match this {
        Some(s) => s.build(ctx),
        None => ConstraintDeclaration::default(),
    }
}

pub(crate) fn build_modifier_ahead(this: &[ModifierAheadNode], ctx: &mut ProgramState) -> ModifierList {
    ModifierList { terms: this.iter().map(|s| s.build(ctx)).collect() }
}

pub(crate) fn build_annotation_terms(this: &[AnnotationTermNode], ctx: &mut ProgramState) -> AttributeList {
    let mut terms = Vec::with_capacity(this.len());
    for term in this {
        terms.push(term.build(ctx))
    }
    AttributeList { terms: terms.into_iter().map(|v| v.terms).flatten().collect() }
}

pub(crate) fn build_annotation_terms_mix(this: &[AnnotationTermMixNode], ctx: &mut ProgramState) -> Result<AttributeList> {
    let mut terms = vec![];
    for term in this {
        terms.push(term.build(ctx))
    }
    Ok(AttributeList { terms: terms.into_iter().map(|v| v.terms).flatten().collect() })
}

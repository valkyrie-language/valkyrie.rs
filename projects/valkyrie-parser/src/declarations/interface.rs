use super::*;

impl<'i> crate::DefineTraitNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<TraitDeclaration> {
        Ok(TraitDeclaration {
            kind: self.kw_trait().build(),
            name: self.identifier().build(ctx.file),
            generics: self.define_generic().as_ref().map(|s| s.build(ctx)),
            implements: self.type_hint().build(ctx),
            body: self.trait_block().build(ctx),
            span: Default::default(),
        })
    }
}

impl<'i> crate::KwTraitNode<'i> {
    pub(crate) fn build(&self) -> TraitKind {
        TraitKind::Trait
    }
}
impl<'i> crate::TraitBlockNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Vec<TraitTerm> {
        let mut terms = Vec::with_capacity(self.trait_term().len());
        for term in &self.trait_term() {
            match term.build(ctx) {
                Ok(o) => terms.extend(o),
                Err(e) => ctx.add_error(e),
            }
        }
        terms
    }
}

impl<'i> crate::TraitTermNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<Option<TraitTerm>> {
        let item = match self {
            Self::ProceduralCall(v) => v.build(ctx).into(),
            Self::DefineField(v) => v.build(ctx)?.into(),
            Self::DefineMethod(v) => v.build(ctx)?.into(),
            Self::EosFree(_) => return Ok(None),
        };
        Ok(Some(item))
    }
}

use super::*;

impl<'i> crate::DefineTraitNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<TraitDeclaration> {
        Ok(TraitDeclaration {
            keyword: self.kw_trait().get_range32(),
            name: self.identifier().build(ctx.file),
            generics: self.define_generic().as_ref().map(|s| s.build(ctx)),
            implements: self.type_hint().and_then(|e| e.build(ctx)),
            body: self.trait_block().build(ctx),
            span: self.get_range32(),
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

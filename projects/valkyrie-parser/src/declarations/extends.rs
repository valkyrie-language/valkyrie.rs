use super::*;

impl<'i> crate::DefineExtendsNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ImplementsStatement> {
        Ok(ImplementsStatement {
            keyword: self.kw_extends().get_range32(),
            annotations: self.annotation_head().annotations(ctx),
            target: self.namepath().build(ctx).into(),
            implements: self.type_hint().and_then(|x| x.build(ctx)),
            body: self.trait_block().build(ctx),
            span: self.get_range32(),
        })
    }
}

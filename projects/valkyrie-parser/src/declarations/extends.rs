use super::*;

impl<'i> crate::DefineExtendsNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ExtendsStatement> {
        Ok(ExtendsStatement { implements: self.type_hint().build(ctx), body: self.trait_block().build(ctx) })
    }
}

use super::*;

impl<'i> crate::LoopEachStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopEach> {
        Ok(LoopEach {
            pattern: self.let_pattern().build(ctx)?,
            iterator: Default::default(),
            condition: self.if_guard().build(ctx),
            label: None,
            body: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::LoopStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopRepeat> {
        Ok(LoopRepeat { label: None, terms: vec![] })
    }
}

use super::*;

impl<'i> crate::ForStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ForLoop> {
        Ok(ForLoop {
            pattern: self.let_pattern().build(ctx)?,
            iterator: Default::default(),
            condition: self.if_guard().build(ctx),
            label: None,
            body: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

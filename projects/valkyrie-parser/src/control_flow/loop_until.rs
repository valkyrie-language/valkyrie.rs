use super::*;

impl<'i> crate::LoopUntilStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopUntil> {
        let kind = self.inline_expression().build(ctx)?;
        Ok(LoopUntil {
            keyword: self.kw_until().get_range32(),
            condition: UntilConditionNode::Expression(kind),
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

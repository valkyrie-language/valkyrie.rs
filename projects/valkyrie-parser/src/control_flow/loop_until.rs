use super::*;

impl<'i> crate::LoopUntilStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopUntil> {
        let start = self.kw_loop().get_range32().start;
        let end = self.kw_until().get_range32().end;
        let kind = self.inline_expression().build(ctx)?;
        Ok(LoopUntil {
            keyword: start..end,
            condition: UntilConditionNode::Expression(kind),
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

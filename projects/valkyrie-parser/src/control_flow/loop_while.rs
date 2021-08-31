use super::*;

impl<'i> crate::LoopWhileStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopWhile> {
        Ok(LoopWhile {
            keyword: self.kw_while().get_range32(),
            condition: ConditionNode::Unconditional,
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::LoopUntilStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopWhile> {
        Ok(LoopWhile {
            keyword: self.kw_until().get_range32(),
            condition: ConditionNode::Unconditional,
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

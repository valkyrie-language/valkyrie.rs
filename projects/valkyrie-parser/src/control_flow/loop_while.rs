use super::*;

impl<'i> crate::LoopWhileStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopWhile> {
        let kind = self.inline_expression().build(ctx)?;
        Ok(LoopWhile {
            keyword: self.kw_while().get_range32(),
            condition: WhileConditionNode::Expression(kind),
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

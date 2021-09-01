use super::*;

impl<'i> crate::LoopWhileStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopWhile> {
        let start = self.kw_loop().get_range32().start;
        let end = self.kw_while().get_range32().end;
        let kind = self.inline_expression().build(ctx)?;
        Ok(LoopWhile {
            keyword: start..end,
            condition: WhileConditionNode::Expression(kind),
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

use super::*;

impl<'i> crate::WhileStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<WhileLoop> {
        Ok(WhileLoop {
            kind: self.kw_while().build(),
            condition: WhileConditionNode::Unconditional,
            then: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::KwWhileNode<'i> {
    pub(crate) fn build(&self) -> WhileLoopKind {
        match self {
            Self::Until(_) => WhileLoopKind::Until,
            Self::While(_) => WhileLoopKind::While,
        }
    }
}

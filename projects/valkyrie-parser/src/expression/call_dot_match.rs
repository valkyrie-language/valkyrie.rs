use super::*;

impl<'i> crate::DotMatchCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<MatchCallNode> {
        let monadic = self.op_and_then().is_some();
        Ok(MatchCallNode {
            monadic,
            base: Default::default(),
            kind: self.kw_match().build(),
            patterns: self.match_block().build(ctx),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::MainSuffixTerm0Node<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<MatchCallNode> {
        todo!()
    }
}

impl<'i> crate::MainSuffixTerm1Node<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ClosureCallNode> {
        todo!()
    }
}

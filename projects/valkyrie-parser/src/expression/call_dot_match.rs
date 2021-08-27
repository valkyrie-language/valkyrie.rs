use super::*;

impl crate::DotMatchCallNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<MatchCallNode> {
        let monadic = self.op_and_then.is_some();

        Ok(MatchCallNode {
            monadic,
            base: Default::default(),
            kind: self.kw_match.build(),
            patterns: self.match_block.build(ctx),
            span: self.get_range32(),
        })
    }
}

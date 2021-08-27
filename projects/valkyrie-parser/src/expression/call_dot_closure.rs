use super::*;
impl<'i> crate::DotClosureCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ClosureCallNode> {
        let monadic = self.op_and_then().is_some();
        Ok(ClosureCallNode {
            monadic,
            base: Default::default(),
            trailing: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

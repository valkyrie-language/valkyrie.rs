use super::*;

impl<'i> crate::ProceduralCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ProceduralNode {
        ProceduralNode {
            kind: Default::default(),
            path: self.namepath().build(ctx),
            arguments: Default::default(),
            domain: None,
            span: ctx.file.with_range(self.get_range32()),
        }
    }
}

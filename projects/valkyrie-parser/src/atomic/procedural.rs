use super::*;

impl crate::ProceduralCallNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ProceduralNode {
        ProceduralNode {
            kind: Default::default(),
            path: self.namepath.build(ctx),
            arguments: Default::default(),
            domain: None,
            span: ctx.file.with_range(self.span.clone()),
        }
    }
}

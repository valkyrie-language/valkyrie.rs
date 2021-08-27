use super::*;

impl<'i> crate::JumpLabelNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> LabelNode {
        match &self.identifier() {
            Some(s) => LabelNode::Named(s.build(ctx.file)),
            None => LabelNode::Nearest,
        }
    }
}

use super::*;

impl<'i> crate::ObjectStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ConstructObjectNode> {
        Ok(ConstructObjectNode {
            base_classes: None,
            bounds: self.type_hint().and_then(|x| x.build(ctx)),
            span: self.get_range32(),
        })
    }
}

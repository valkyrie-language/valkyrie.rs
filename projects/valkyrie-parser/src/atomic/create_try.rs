use super::*;

impl<'i> crate::TryStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<TryStatement> {
        let handler = match &self.type_expression() {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        Ok(TryStatement { handler, body: self.continuation().build(ctx), span: self.get_range32() })
    }
}

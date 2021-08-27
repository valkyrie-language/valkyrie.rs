use super::*;

impl<'i> crate::DefineLambdaNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LambdaNode> {
        let returns = self.function_middle().returns(ctx)?;
        Ok(LambdaNode {
            annotations: build_annotation_terms(&self.annotation_term(), ctx).into(),
            generics: self.function_middle().generics(ctx),
            parameters: self.function_middle().parameters(ctx),
            returns,
            body: self.continuation().build(ctx),
            span: self.span().clone(),
        })
    }
}

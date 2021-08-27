use super::*;

impl<'i> crate::DefineConstraintNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ConstraintDeclaration {
        ConstraintDeclaration {
            annotations: self.annotation_head().annotations(ctx),
            generics: self.generics(ctx),
            terms: vec![],
        }
    }
    fn generics(&self, ctx: &mut ProgramState) -> Vec<IdentifierNode> {
        match &self.constraint_parameters() {
            Some(s) => s.identifier().iter().map(|x| x.build(ctx.file)).collect(),
            None => vec![],
        }
    }
}

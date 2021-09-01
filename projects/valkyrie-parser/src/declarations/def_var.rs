use super::*;

impl<'i> crate::DefineVariableNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LetBindNode> {
        Ok(LetBindNode {
            pattern: CasePattern::Tuple(Box::new(TuplePatternNode {
                bind: None,
                name: None,
                terms: vec![],
                span: Default::default(),
            })),
            type_hint: self.type_hint().build(ctx),
            body: self.parameter_default().build(ctx),
            span: self.get_range32(),
        })
    }
}

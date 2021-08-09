use super::*;

impl crate::GenericCallNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<GenericCallNode> {
        let monadic = self.op_and_then.is_some();
        let term = GenericCallTerm::Generic(self.generic_terms.build(ctx)?);
        Ok(GenericCallNode { monadic, base: Default::default(), term, span: self.span.clone() })
    }
}
impl crate::GenericHideNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<GenericCallTerm> {
        Ok(GenericCallTerm::Generic(self.generic_terms.build(ctx)?))
    }
    pub(crate) fn build_call(&self, ctx: &mut ProgramState) -> Result<GenericCallNode> {
        Ok(GenericCallNode { monadic: false, base: Default::default(), term: self.build(ctx)?, span: self.span.clone() })
    }
}

impl crate::GenericTermsNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ArgumentsList> {
        let mut list = ArgumentsList::new(self.generic_pair.len());
        for x in &self.generic_pair {
            match x.build(ctx) {
                Ok(o) => list.terms.push(o),
                Err(e) => ctx.add_error(e),
            }
        }
        Ok(list)
    }
}
impl crate::GenericPairNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ArgumentTerm> {
        Ok(ArgumentTerm {
            modifiers: Default::default(),
            key: self.get_key(ctx),
            value: self.type_expression.build(ctx)?,
            span: ctx.file.with_range(self.span.clone()),
        })
    }
    fn get_key(&self, ctx: &mut ProgramState) -> ArgumentKey {
        match &self.identifier {
            Some(v) => ArgumentKey::Symbol(v.build(ctx.file)),
            None => ArgumentKey::Nothing,
        }
    }
}

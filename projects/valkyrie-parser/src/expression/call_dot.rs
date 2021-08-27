use super::*;

impl<'i> crate::DotCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<DotCallNode> {
        let monadic = self.op_and_then().is_some();
        Ok(DotCallNode { monadic, base: Default::default(), term: self.dot_call_item().build(ctx)?, span: self.get_range32() })
    }
}

impl<'i> crate::DotCallItemNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<DotCallTerm> {
        match self {
            Self::Namepath(v) => Ok(DotCallTerm::Symbol(v.build(ctx))),
            Self::Integer(v) => {
                let u = usize::from_str(v.get_str())?;
                Ok(DotCallTerm::index(u))
            }
        }
    }
}

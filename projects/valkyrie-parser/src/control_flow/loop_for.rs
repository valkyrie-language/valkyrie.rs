use super::*;

impl<'i> crate::LoopEachStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopEach> {
        let start = self.kw_loop().get_range32().start;
        let end = match self.kw_each() {
            Some(s) => s.get_range32().end,
            None => self.kw_loop().get_range32().end,
        };
        Ok(LoopEach {
            keyword: start..end,
            pattern: self.let_pattern().build(ctx)?,
            iterator: Default::default(),
            condition: self.if_guard().build(ctx),
            label: None,
            body: self.continuation().build(ctx),
            span: self.get_range32(),
        })
    }
}

impl<'i> crate::LoopStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LoopRepeat> {
        Ok(LoopRepeat { keyword: self.kw_loop().get_range32(), label: None, terms: vec![] })
    }
}

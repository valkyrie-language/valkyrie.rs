use super::*;
use crate::{traits::YggdrasilNodeExtension, utils::Ast2Hir, TupleLiteralNode};

impl<'i> crate::TupleLiteralStrictNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<TupleNode> {
        let mut terms = vec![];
        for term in &self.tuple_pair() {
            match term.to_hir(ctx) {
                Ok(o) => terms.push(o),
                Err(e) => ctx.add_error(e),
            }
        }
        Ok(TupleNode { kind: TupleKind::Tuple, terms: ArgumentsList { terms }, span: self.get_range32() })
    }
}

impl<'i> Ast2Hir<TupleNode> for crate::TupleLiteralNode<'i> {
    fn to_hir(&self, ctx: &mut ProgramState) -> TupleNode {
        TupleNode { kind: Default::default(), terms: self.tuple_terms().to_hir(ctx), span: self.get_range32() }
    }
}

impl<'i> Ast2Hir<ArgumentsList> for crate::TupleTermsNode<'i> {
    fn to_hir(&self, ctx: &mut ProgramState) -> ArgumentsList {
        let mut list = ArgumentsList::new(self.tuple_pair().len());
        for term in &self.tuple_pair() {
            match term.to_hir(ctx) {
                Ok(o) => list.terms.push(o),
                Err(e) => ctx.add_error(e),
            }
        }
        list
    }
}
impl<'i> Ast2Hir<ArgumentsList> for TupleLiteralNode<'i> {
    fn to_hir(&self, ctx: &mut ProgramState) -> ArgumentsList {
        let mut out = ArgumentsList::new(self.tuple_terms().tuple_pair().len());
        for pair in &self.tuple_terms().tuple_pair() {
            match pair.to_hir(ctx) {
                Ok(o) => out += o,
                Err(e) => *ctx += e,
            }
        }
        out
    }
}

impl<'i> crate::TuplePairNode<'i> {
    pub(crate) fn to_hir(&self, ctx: &mut ProgramState) -> Result<ArgumentTerm> {
        let key = match &self.tuple_key() {
            Some(v) => v.build(ctx),
            None => ArgumentKey::Nothing,
        };
        Ok(ArgumentTerm {
            modifiers: Default::default(),
            key,
            value: self.main_expression.build(ctx)?,
            span: ctx.file.with_range(self.get_range32()),
        })
    }
}

impl<'i> crate::TupleKeyNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> ArgumentKey {
        match self {
            Self::Identifier(v) => ArgumentKey::Symbol(v.build(ctx.file)),
            Self::TextRaw(v) => ArgumentKey::Symbol(v.build_id(ctx)),
            Self::Integer(v) => {
                ctx.add_error(
                    SyntaxError::new("tuple key cannot be a number")
                        .with_hint("Expect a symbol")
                        .with_range(&v.get_range32())
                        .with_file(ctx.file),
                );

                ArgumentKey::Nothing
            }
        }
    }
}

impl<'i> crate::TupleCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ApplyCallNode> {
        let monadic = self.op_and_then().is_some();
        let arguments = match &self.tuple_literal() {
            Some(s) => s.to_hir(ctx),
            None => ArgumentsList { terms: vec![] },
        };
        Ok(ApplyCallNode { monadic, caller: Default::default(), arguments, body: None, span: self.get_range32() })
    }
}
impl<'i> crate::InlineTupleCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ApplyCallNode> {
        let monadic = self.op_and_then().is_some();
        let arguments = self.tuple_literal().to_hir(ctx);
        Ok(ApplyCallNode { monadic, caller: Default::default(), arguments, body: None, span: self.get_range32() })
    }
}

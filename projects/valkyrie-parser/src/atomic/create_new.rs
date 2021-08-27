use super::*;
use crate::utils::Ast2Hir;
use yggdrasil_rt::YggdrasilNode;

impl<'i> crate::NewStatementNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<ConstructNewNode> {
        let namepath = self.namepath().build(ctx);
        let generics = match &self.generic_hide() {
            Some(s) => vec![s.build(ctx)?],
            None => vec![],
        };
        Ok(ConstructNewNode {
            annotations: self.annotations(ctx),
            namepath,
            generics,
            arguments: self.tuple_literal().as_ref().map(|s| s.tuple_terms().to_hir(ctx)).unwrap_or_default(),
            body: self.new_block().as_ref().map(|s| s.build(ctx)).unwrap_or_default(),
            span: self.get_range(),
        })
    }
    fn annotations(&self, ctx: &mut ProgramState) -> AnnotationNode {
        let mut out = AnnotationNode::default();
        for term in &self.modifier_ahead {
            out.modifiers.terms.push(term.build(ctx))
        }
        out
    }
}

impl<'i> crate::NewBlockNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Vec<CollectorTerm> {
        let mut olistt = Vec::with_capacity(self.new_pair().len());
        for pair in &self.new_pair() {
            match pair.build(ctx) {
                Ok(o) => olistt.push(o),
                Err(e) => ctx.add_error(e),
            }
        }

        olistt
    }
}

impl<'i> crate::NewPairNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<CollectorTerm> {
        let item = self.main_expression().build(ctx)?;

        Ok(CollectorTerm::Item(item))
    }
}

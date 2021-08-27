use super::*;
use valkyrie_ast::helper::ValkyrieNode;

impl<'i> crate::RangeLiteralNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeNode> {
        let mut value = RangeNode { kind: RangeKind::Ordinal, terms: vec![], span: Default::default() };
        match self {
            Self::RangeLiteralIndex0(v) => {
                for term in &v.subscript_axis() {
                    match term.build(ctx) {
                        Ok(o) => value.terms.push(o),
                        Err(e) => ctx.add_error(e),
                    }
                }
                value.span = v.get_range32()
            }
            Self::RangeLiteralIndex1(v) => {
                value.kind = RangeKind::Offset;
                for term in &v.subscript_axis() {
                    match term.build(ctx) {
                        Ok(o) => value.terms.push(o),
                        Err(e) => ctx.add_error(e),
                    }
                }
                value.span = v.get_range32()
            }
        }
        Ok(value)
    }
}

impl<'i> crate::SubscriptAxisNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeTermNode> {
        match self {
            Self::SubscriptOnly(v) => v.build(ctx),
            Self::SubscriptRange(v) => v.build(ctx),
        }
    }
}

impl<'i> crate::SubscriptOnlyNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeTermNode> {
        self.index().build(ctx).map(|v| RangeTermNode::Index { span: ctx.file.with_range(v.get_range()), index: v })
    }
}

impl<'i> crate::SubscriptRangeNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeTermNode> {
        let head = match &self.head() {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        let tail = match &self.tail() {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        let step = match &self.step() {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        Ok(RangeTermNode::Range { head, tail, step })
    }
}
impl<'i> crate::RangeCallNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<SubscriptCallNode> {
        let monadic = self.op_and_then().is_some();
        let terms = self.range_literal().build(ctx)?.terms;
        Ok(SubscriptCallNode {
            kind: RangeKind::Ordinal,
            base: ExpressionKind::Placeholder,
            monadic,
            terms,
            span: self.get_range32(),
        })
    }
}

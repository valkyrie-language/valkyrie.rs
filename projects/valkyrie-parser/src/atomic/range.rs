use super::*;
use valkyrie_ast::helper::ValkyrieNode;

impl crate::RangeLiteralNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeNode> {
        let mut value = RangeNode { kind: RangeKind::Ordinal, terms: vec![], span: Default::default() };
        match self {
            Self::RangeLiteralIndex0(v) => {
                for term in &v.subscript_axis {
                    match term.build(ctx) {
                        Ok(o) => value.terms.push(o),
                        Err(e) => ctx.add_error(e),
                    }
                }
                value.span = v.span.clone()
            }
            Self::RangeLiteralIndex1(v) => {
                value.kind = RangeKind::Offset;
                for term in &v.subscript_axis {
                    match term.build(ctx) {
                        Ok(o) => value.terms.push(o),
                        Err(e) => ctx.add_error(e),
                    }
                }
                value.span = v.span.clone()
            }
        }
        Ok(value)
    }
}

impl crate::SubscriptAxisNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeTermNode> {
        match self {
            Self::SubscriptOnly(v) => v.build(ctx),
            Self::SubscriptRange(v) => v.build(ctx),
        }
    }
}

impl crate::SubscriptOnlyNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeTermNode> {
        self.index.build(ctx).map(|v| RangeTermNode::Index { span: ctx.file.with_range(v.get_range()), index: v })
    }
}

impl crate::SubscriptRangeNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<RangeTermNode> {
        let head = match &self.head {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        let tail = match &self.tail {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        let step = match &self.step {
            Some(s) => Some(s.build(ctx)?),
            None => None,
        };
        Ok(RangeTermNode::Range { head, tail, step })
    }
}
impl crate::RangeCallNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<SubscriptCallNode> {
        let monadic = self.op_and_then.is_some();
        let terms = self.range_literal.build(ctx)?.terms;
        Ok(SubscriptCallNode {
            kind: RangeKind::Ordinal,
            base: ExpressionKind::Placeholder,
            monadic,
            terms,
            span: self.span.clone(),
        })
    }
}

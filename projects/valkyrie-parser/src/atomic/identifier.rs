use super::*;
use nyar_error::SourceID;
use std::sync::Arc;

impl crate::NamepathNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> NamePathNode {
        NamePathNode::from_iter(self.identifier.iter().map(|v| v.build(ctx.file)))
            .with_span(ctx.file.with_range(self.span.clone()))
    }
}

impl crate::NamepathFreeNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> NamePathNode {
        NamePathNode::from_iter(self.identifier.iter().map(|v| v.build(ctx.file)))
            .with_span(ctx.file.with_range(self.span.clone()))
    }
}
impl crate::IdentifierNode {
    pub fn build(&self, file: SourceID) -> IdentifierNode {
        match self {
            Self::IdentifierBare(v) => {
                IdentifierNode { name: Arc::from(v.text.as_str()), span: file.with_range(v.span.clone()) }
            }
            Self::IdentifierRaw(v) => {
                IdentifierNode { name: Arc::from(v.identifier_raw_text.text.as_str()), span: file.with_range(v.span.clone()) }
            }
        }
    }
}

impl crate::SlotNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LambdaSlotNode> {
        Ok(LambdaSlotNode { level: self.op_slot.span.len(), item: self.item(ctx)?, span: self.span.clone() })
    }
    fn item(&self, ctx: &mut ProgramState) -> Result<LambdaSlotItem> {
        match &self.slot_item {
            Some(s) => s.build(ctx),
            None => return Ok(LambdaSlotItem::SelfType),
        }
    }
}

impl crate::SlotItemNode {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LambdaSlotItem> {
        let value = match self {
            Self::Identifier(v) => LambdaSlotItem::Named(v.build(ctx.file)),
            Self::Integer(v) => match NonZeroU64::new(v.parse::<u64>(ctx)?) {
                Some(s) => LambdaSlotItem::Index(s),
                None => LambdaSlotItem::MetaType,
            },
        };
        Ok(value)
    }
}

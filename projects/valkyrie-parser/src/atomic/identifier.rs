use super::*;
use nyar_error::SourceID;
use std::sync::Arc;
use yggdrasil_rt::YggdrasilNode;

impl<'i> crate::NamepathNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> NamePathNode {
        NamePathNode::from_iter(self.identifier().iter().map(|v| v.build(ctx.file)))
            .with_span(ctx.file.with_range(self.get_range32()))
    }
}

impl<'i> crate::NamepathFreeNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> NamePathNode {
        NamePathNode::from_iter(self.identifier().iter().map(|v| v.build(ctx.file)))
            .with_span(ctx.file.with_range(self.get_range32()))
    }
}
impl<'i> crate::IdentifierNode<'i> {
    pub fn build(&self, file: SourceID) -> IdentifierNode {
        match self {
            Self::IdentifierBare(v) => {
                IdentifierNode { name: Arc::from(v.get_text().as_str()), span: file.with_range(v.get_range32().clone()) }
            }
            Self::IdentifierRaw(v) => IdentifierNode {
                name: Arc::from(v.identifier_raw_text().get_text().as_str()),
                span: file.with_range(v.get_range32().clone()),
            },
        }
    }
}

impl<'i> crate::SlotNode<'i> {
    pub(crate) fn build(&self, ctx: &mut ProgramState) -> Result<LambdaSlotNode> {
        Ok(LambdaSlotNode { level: self.op_slot().get_range32(), item: self.item(ctx)?, span: self.get_range32() })
    }
    fn item(&self, ctx: &mut ProgramState) -> Result<LambdaSlotItem> {
        match &self.slot_item() {
            Some(s) => s.build(ctx),
            None => return Ok(LambdaSlotItem::SelfType),
        }
    }
}

impl<'i> crate::SlotItemNode<'i> {
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

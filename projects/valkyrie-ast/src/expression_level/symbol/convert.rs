use super::*;

impl ValkyrieNode for BooleanNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl ValkyrieNode for NullNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl ValkyrieNode for OutputNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl ValkyrieNode for LambdaSlotNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl FromIterator<IdentifierNode> for NamePathNode {
    fn from_iter<T: IntoIterator<Item = IdentifierNode>>(iter: T) -> Self {
        Self { path: iter.into_iter().collect(), span: Default::default() }
    }
}

use super::*;

impl Default for ImportTermNode {
    fn default() -> Self {
        Self::Group(Default::default())
    }
}
// impl ValkyrieNode for ImportTermNode {
//     fn get_range(&self) -> Range<u32> {
//         match self {
//             Self::Group(v) => {v.get_range()}
//             Self::All(v) => {v.get_range()}
//             Self::Alias(v) => {v.get_range()}
//         }
//     }
// }
impl ValkyrieNode for ImportAllNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl ValkyrieNode for ImportAliasNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl FromIterator<IdentifierNode> for ImportGroupNode {
    fn from_iter<T: IntoIterator<Item = IdentifierNode>>(iter: T) -> Self {
        Self { path: iter.into_iter().collect(), terms: vec![] }
    }
}
impl FromIterator<ImportTermNode> for ImportGroupNode {
    fn from_iter<T: IntoIterator<Item = ImportTermNode>>(iter: T) -> Self {
        Self { path: vec![], terms: iter.into_iter().collect() }
    }
}

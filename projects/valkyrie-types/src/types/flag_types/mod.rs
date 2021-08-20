use super::*;

pub struct ValkyrieFlags {
    pub flags_name: Identifier,
    pub flags: IndexMap<Arc<str>, ValkyrieSemanticNumbers>,
}
impl AddAssign<ValkyrieFlags> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieFlags) {
        self.items.insert(rhs.flags_name.clone(), ModuleItem::Resource(rhs));
    }
}

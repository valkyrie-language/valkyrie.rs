use super::*;

pub struct ValkyrieEnumeration {
    pub enumeration_name: Identifier,
    pub indexes: IndexMap<Arc<str>, ValkyrieSemanticNumbers>,
}

pub struct ValkyrieSemanticNumbers {
    pub term_name: Arc<str>,
}

impl AddAssign<ValkyrieEnumeration> for ResolveState {
    fn add_assign(&mut self, rhs: ValkyrieEnumeration) {
        self.items.insert(rhs.enumeration_name.clone(), ModuleItem::Structure(rhs));
    }
}

use super::*;

mod codegen;

/// abstract class with closed childrens
#[derive(Clone)]
pub struct ValkyrieUnite {
    /// The full name path of the union
    pub unite_name: Identifier,
    pub variants: IndexMap<Arc<str>, ValkyrieVariant>,
}

impl ValkyrieUnite {
    pub fn new(name: Identifier) -> Self {
        Self { unite_name: name, variants: Default::default() }
    }
}

impl Debug for ValkyrieUnite {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Union").field("name", &self.unite_name).field("variants", &self.variants.values()).finish()
    }
}

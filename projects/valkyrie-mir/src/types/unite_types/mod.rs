use super::*;
use valkyrie_error::SourceSpan;

mod codegen;

/// abstract class with closed childrens
#[derive(Clone)]
pub struct ValkyrieUnite {
    /// The full name path of the union
    pub unite_name: WasmIdentifier,
    pub variants: IndexMap<Arc<str>, ValkyrieVariant>,
    pub source: SourceSpan,
}

impl ValkyrieUnite {
    pub fn new(name: WasmIdentifier) -> Self {
        Self { unite_name: name, variants: Default::default(), source: Default::default() }
    }
}

impl Debug for ValkyrieUnite {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Union").field("name", &self.unite_name).field("variants", &self.variants.values()).finish()
    }
}

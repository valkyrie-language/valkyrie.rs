use super::*;

mod codegen;

#[derive(Clone)]
pub struct ValkyrieVariant {
    /// The full name path of the variant item
    pub variant_name: Arc<str>,
    /// The alias name in wasi
    pub wasi_alias: Arc<str>,
    /// The following fields belonging to an independent type
    pub type_alias: Identifier,
    pub fields: IndexMap<Arc<str>, ValkyrieField>,
}

impl Debug for ValkyrieVariant {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Variant")
            .field("name", &self.variant_name)
            .field("wasi", &self.wasi_alias)
            .field("fields", &self.fields.values())
            .finish()
    }
}

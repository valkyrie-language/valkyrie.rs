use super::*;

mod codegen;

#[derive(Clone)]
pub struct ValkyrieVariant {
    /// The full name path of the variant item
    pub variant_name: Identifier,
    /// The alias name in wasi
    pub wasi_alias: Identifier,
    /// The following fields belonging to an independent type
    pub type_alias: WasmIdentifier,
    pub fields: IndexMap<Identifier, ValkyrieField>,
    pub source: SourceSpan,
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

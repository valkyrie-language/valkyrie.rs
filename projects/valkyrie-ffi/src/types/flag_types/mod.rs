use super::*;

mod codegen;

#[derive(Debug)]
pub struct ValkyrieFlagation {
    pub flags_name: WasmIdentifier,
    pub flags: IndexMap<Identifier, ValkyrieSemanticNumber>,
}

use super::*;

mod codegen;

#[derive(Debug)]
pub struct ValkyrieFlagation {
    pub flags_name: Identifier,
    pub flags: IndexMap<Arc<str>, ValkyrieSemanticNumber>,
}

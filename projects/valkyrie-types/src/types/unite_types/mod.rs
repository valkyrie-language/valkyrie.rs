use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    ModuleItem, ResolveState, ValkyrieField,
};
use indexmap::IndexMap;
use nyar_wasm::{DependentGraph, Identifier};
use std::{
    fmt::{Debug, Formatter},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_ast::{ClassTerm, UnionDeclaration, UnionTerm, VariantDeclaration};

mod codegen;

/// abstract class with closed childrens
#[derive(Clone)]
pub struct ValkyrieUnite {
    /// The full name path of the union
    pub unite_name: Identifier,
    pub variants: IndexMap<Arc<str>, ValkyrieUniteItem>,
}

impl ValkyrieUnite {
    pub fn new(name: Identifier) -> Self {
        Self { unite_name: name, variants: Default::default() }
    }
}

#[derive(Clone)]
pub struct ValkyrieUniteItem {
    /// The full name path of the variant item
    pub variant_name: Arc<str>,
    /// The alias name in wasi
    pub wasi_alias: Arc<str>,
    /// The following fields belonging to an independent type
    pub type_alias: Identifier,
    pub fields: IndexMap<Arc<str>, ValkyrieField>,
}

impl Debug for ValkyrieUnite {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Union").field("name", &self.unite_name).field("variants", &self.variants.values()).finish()
    }
}

impl Debug for ValkyrieUniteItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Variant")
            .field("name", &self.variant_name)
            .field("wasi", &self.wasi_alias)
            .field("fields", &self.fields.values())
            .finish()
    }
}

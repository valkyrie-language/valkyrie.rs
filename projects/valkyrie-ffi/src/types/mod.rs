use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    NamespaceItem, ResolveContext, ValkyrieField, ValkyrieSemanticNumber, ValkyrieVariant,
};
use indexmap::IndexMap;
use std::{
    fmt::{Debug, Formatter},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_lir::{DependentGraph, WasiEnumeration, WasiFlags, WasiSemanticIndex, WasiVariantItem, WasmIdentifier};
use valkyrie_types::{Identifier, SourceSpan};

pub mod encoding_type;
pub mod enumeration_types;
pub mod flag_types;
pub mod trait_types;
pub mod unite_types;
pub mod variant_type;

#[derive(Clone, Debug, Hash)]
pub enum ValkyrieType {
    Boolean,
    Integer { bits: usize },
    Unsigned { bits: usize },
    Float { bits: usize },
    Unicode,
    Unsolved(WasmIdentifier),
}

impl ValkyrieType {}

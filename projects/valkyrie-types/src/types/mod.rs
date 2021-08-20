use crate::{helpers::Hir2Mir, ModuleItem, ResolveState, ValkyrieSemanticNumbers};
use indexmap::IndexMap;
use nyar_wasm::Identifier;
use std::{ops::AddAssign, sync::Arc};
use valkyrie_ast::TraitDeclaration;

pub mod enumeration_types;
pub mod flag_types;
pub mod trait_types;

#[derive(Clone, Debug, Hash)]
pub enum ValkyrieType {
    Boolean,
    Integer { bits: usize },
    Unsigned { bits: usize },
    Float { bits: usize },
    Unicode,
    Unsolved(Identifier),
}

impl ValkyrieType {}

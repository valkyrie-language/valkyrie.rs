use crate::{helpers::Hir2Mir, ResolveState};
use nyar_wasm::Identifier;
use std::sync::{Arc, Mutex};
use valkyrie_ast::{FlagDeclaration, TraitDeclaration};

pub mod flag_types;
pub mod trait_types;

#[derive(Clone, Debug, Hash)]
pub enum ValkyrieType {
    Boolean,
    Integer,
    Unsolved(Identifier),
}

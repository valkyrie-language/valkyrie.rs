use crate::{helpers::Hir2Mir, ResolveContext, ValkyrieFunction};
use nyar_wasm::Identifier;
use std::sync::{Arc, Mutex};
use valkyrie_ast::{FlagDeclaration, TraitDeclaration};

pub mod flag_types;
pub mod trait_types;

#[derive(Clone, Debug)]
pub enum ValkyrieType {
    Boolean,
    Integer,
    Function(Arc<Mutex<ValkyrieFunction>>),
    Unsourced(Identifier),
}

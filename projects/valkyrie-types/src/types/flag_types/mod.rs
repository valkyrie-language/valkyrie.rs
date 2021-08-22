use super::*;
use crate::helpers::Mir2Lir;
use nyar_wasm::{DependentGraph, WasiFlags};

mod codegen;

#[derive(Debug)]
pub struct ValkyrieFlagation {
    pub flags_name: Identifier,
    pub flags: IndexMap<Arc<str>, ValkyrieSemanticNumber>,
}

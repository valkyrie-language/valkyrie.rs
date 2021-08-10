use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    ModuleItem, ResolveState, ValkyrieType,
};
use indexmap::IndexMap;
use nyar_wasm::{DependentGraph, Identifier, WasiExport, WasiFunction, WasiImport};
use std::{ops::AddAssign, sync::Arc};
use valkyrie_ast::FunctionDeclaration;
mod arithmetic;
mod stage1_mir;
mod stage2_lir;

/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Debug)]
pub struct ValkyrieFunction {
    pub function_name: Identifier,
    /// The WASI import symbol if exists
    pub wasi_import: Option<WasiImport>,
    /// The WASI export symbol if exists
    pub wasi_export: Option<WasiExport>,
    /// The input output signature of the function
    pub signature: FunctionSignature,
}

#[derive(Clone, Debug)]
pub struct FunctionParameter {
    pub name: Arc<str>,
    pub r#type: ValkyrieType,
}

#[derive(Clone, Default, Debug)]
pub struct FunctionSignature {
    pub positional: IndexMap<Arc<str>, FunctionParameter>,
    pub mixed: IndexMap<Arc<str>, FunctionParameter>,
    pub named: IndexMap<Arc<str>, FunctionParameter>,
    pub output: Vec<ValkyrieType>,
}

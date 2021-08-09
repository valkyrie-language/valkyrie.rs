use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    ModuleItem, ResolveContext, ValkyrieType,
};
use nyar_wasm::{DependentGraph, Identifier, WasiExport, WasiFunction, WasiImport};
use std::ops::AddAssign;
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

#[derive(Clone, Default, Debug)]
pub struct FunctionSignature {
    pub inputs: Vec<ValkyrieType>,
    pub output: Vec<ValkyrieType>,
}

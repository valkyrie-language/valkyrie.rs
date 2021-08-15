use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    ModuleItem, ResolveState, ValkyrieType,
};
use im::HashMap;
use indexmap::IndexMap;
use nyar_wasm::{DependentGraph, Identifier, WasiExport, WasiFunction, WasiImport};
use ordered_float::NotNan;
use std::{
    hash::{Hash, Hasher},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_ast::FunctionDeclaration;
mod arithmetic;
mod stage1_mir;
mod stage2_lir;

/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Debug)]
pub struct ValkyrieImportFunction {
    /// The unique identifier of the function
    pub function_name: Identifier,
    pub wasi_import: WasiImport,
    pub signature: FunctionSignature,
}
/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Debug)]
pub struct ValkyrieNativeFunction {
    /// The unique identifier of the function
    pub function_name: Identifier,
    /// The WASI export symbol if exists
    pub wasi_export: Option<WasiExport>,
    /// The input output signature of the function
    pub overloads: HashMap<NotNan<f64>, FunctionSignature>,
}

#[derive(Clone, Debug, Hash)]
pub struct FunctionParameter {
    pub name: Arc<str>,
    pub r#type: ValkyrieType,
}

#[derive(Clone, Default, Debug)]
pub struct FunctionSignature {
    pub positional: IndexMap<Arc<str>, FunctionParameter>,
    pub mixed: IndexMap<Arc<str>, FunctionParameter>,
    pub named: IndexMap<Arc<str>, FunctionParameter>,
    pub output: Vec<FunctionParameter>,
}

impl Hash for FunctionSignature {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for i in self.positional.values() {
            i.hash(state);
        }
        for (key, value) in self.mixed.iter() {
            key.hash(state);
            value.hash(state);
        }
        for (key, value) in self.named.iter() {
            key.hash(state);
            value.hash(state);
        }
        for i in &self.output {
            i.hash(state);
        }
    }
}

use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    ModuleItem, ResolveState, ValkyrieType,
};
use indexmap::IndexMap;
use nyar_wasm::{DependentGraph, Identifier, WasiExport, WasiFunction, WasiImport};
use ordered_float::NotNan;
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    ops::AddAssign,
    sync::Arc,
};

mod arithmetic;
mod stage1_mir;
mod stage2_lir;

/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieImportFunction {
    /// The unique identifier of the function
    pub function_name: Identifier,
    pub wasi_import: WasiImport,
    pub signature: FunctionSignature,
}

/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieNativeFunction {
    /// The unique identifier of the function
    pub function_name: Identifier,
    /// The WASI export symbol if exists
    pub wasi_export: Option<WasiExport>,
    /// The input output signature of the function
    pub overloads: BTreeMap<NotNan<f64>, FunctionInstance>,
}

#[derive(Clone, Default, Debug)]
pub struct FunctionSignature {
    pub positional: IndexMap<Arc<str>, FunctionParameter>,
    pub mixed: IndexMap<Arc<str>, FunctionParameter>,
    pub named: BTreeMap<Arc<str>, FunctionParameter>,
    pub output: Vec<FunctionParameter>,
}

#[derive(Clone, Debug, Hash)]
pub struct FunctionParameter {
    /// The parameter name of the function
    pub name: Arc<str>,
    /// The type hint of the function
    pub r#type: ValkyrieType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionInstance {
    pub signature: FunctionSignature,
    pub body: FunctionBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FunctionBody {
    pub assembly: String,
}

impl Default for FunctionInstance {
    fn default() -> Self {
        Self { signature: Default::default(), body: FunctionBody { assembly: "".to_string() } }
    }
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

impl PartialEq<Self> for FunctionSignature {
    fn eq(&self, other: &Self) -> bool {
        return false;
    }
}

impl Eq for FunctionSignature {}

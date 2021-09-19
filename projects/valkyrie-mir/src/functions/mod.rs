use crate::{
    helpers::{Hir2Mir, Mir2Lir},
    NamespaceItem, ResolveContext, ValkyrieType,
};
use indexmap::IndexMap;
use ordered_float::NotNan;
use std::{
    collections::BTreeMap,
    hash::{Hash, Hasher},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_lir::{DependentGraph, WasiExport, WasiFunction, WasiImport, WasmIdentifier};
use valkyrie_types::Identifier;

mod arithmetic;
mod stage1_mir;
mod stage2_lir;

/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieImportFunction {
    /// The unique identifier of the function
    pub function_name: WasmIdentifier,
    pub wasi_import: WasiImport,
    pub signature: FunctionSignature,
}

/// The [function](), [`external` function](), [`extension` function](), [`overload` function] in valkyrie language
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValkyrieNativeFunction {
    /// The unique identifier of the function
    pub function_name: WasmIdentifier,
    /// The WASI export symbol if exists
    pub wasi_export: Option<WasiExport>,
    /// The input output signature of the function
    pub overloads: BTreeMap<NotNan<f64>, FunctionInstance>,
}

#[derive(Clone, Default, Debug)]
pub struct FunctionSignature {
    pub positional: IndexMap<Identifier, FunctionParameter>,
    pub mixed: IndexMap<Identifier, FunctionParameter>,
    pub named: BTreeMap<Identifier, FunctionParameter>,
    pub output: Vec<FunctionParameter>,
}

#[derive(Clone, Debug, Hash)]
pub struct FunctionParameter {
    /// The parameter name of the function
    pub name: Identifier,
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

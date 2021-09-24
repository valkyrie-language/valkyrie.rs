use crate::{
    functions::{FunctionBody, FunctionInstance},
    helpers::Mir2Lir,
    modules::{NamespaceItem, ResolveContext},
    ValkyrieImportFunction,
};
use indexmap::IndexMap;
use ordered_float::NotNan;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_ast::{helper::WrapDisplay, MethodDeclaration};
use valkyrie_lir::{DependentGraph, WasiImport, WasiResource, WasiType, WasmIdentifier};
use valkyrie_types::{Identifier, Result};

mod codegen;
mod display;

#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieResource {
    pub resource_name: WasmIdentifier,
    /// The wasi import/export name
    pub wasi_import: WasiImport,
    pub imports: IndexMap<Identifier, ValkyrieImportFunction>,
}

/// A primitive
#[derive(Clone, Eq, PartialEq)]
pub struct ValkyriePrimitive {
    /// The name of the primitive
    pub primitive_name: WasmIdentifier,
    /// primitive type had no fields, only primitive type wrapper
    pub wrapper: WasiType,
    pub imports: IndexMap<Identifier, ValkyrieImportFunction>,
    pub methods: IndexMap<Identifier, ValkyrieMethod>,
    pub from: Vec<ValkyrieFrom>,
    pub into: Vec<ValkyrieInto>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieClass {
    pub class_name: WasmIdentifier,
    pub primitive: Option<WasmIdentifier>,
    pub fields: IndexMap<Identifier, ValkyrieField>,
    pub imports: IndexMap<Identifier, ValkyrieImportFunction>,
    pub methods: IndexMap<Identifier, ValkyrieMethod>,
    pub from: Vec<ValkyrieFrom>,
    pub into: Vec<ValkyrieInto>,
}

impl Hash for ValkyrieClass {
    /// ```wat
    /// $type-id = package::module::name
    ///          + Generic Types
    /// ```
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.class_name.hash(state);
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieField {
    /// The name of the field
    pub field_name: Identifier,
    /// The WASI name of the field
    pub wasi_alias: Identifier,
}

impl AddAssign<ValkyrieField> for ValkyrieClass {
    fn add_assign(&mut self, rhs: ValkyrieField) {
        self.fields.insert(rhs.field_name.clone(), rhs);
    }
}
#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieMethod {
    /// The name of the field
    pub method_name: Identifier,
    /// The WASI name of the field
    pub wasi_alias: Identifier,

    pub overloads: BTreeMap<NotNan<f64>, FunctionInstance>,
}

// up_cast
// down_cast
// explicit_cast
#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieFrom {
    pub from: WasmIdentifier,
    pub implicit: bool,
    pub action: FunctionBody,
    pub exception: Option<WasmIdentifier>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieInto {
    pub into: WasmIdentifier,
    pub action: FunctionBody,
    pub exception: Option<WasmIdentifier>,
}

impl ValkyrieClass {
    pub fn get_name(&self) -> String {
        self.class_name.to_string()
    }
    // pub fn get_field(&self, name: &str) -> Option<&ValkyrieField> {
    //     self.fields.get(name)
    // }
    // pub fn add_field(&mut self, field: ValkyrieField) -> Result<()> {
    //     let name = field.name();
    //     let span = field.get_span();
    //     match self.fields.insert(field.name(), field) {
    //         Some(old) => Err(NyarError::duplicate_key(name, old.get_span(), span)),
    //         None => Ok(()),
    //     }
    // }
    // pub fn get_fields(&self) -> Values<String, ValkyrieField> {
    //     self.fields.values()
    // }
    // pub fn add_method(&mut self, method: MethodDefinition) -> Result<()> {
    //     let name = method.name();
    //     let span = method.get_span();
    //     match self.methods.insert(method.name(), method) {
    //         Some(old) => Err(NyarError::duplicate_key(name, old.get_span(), span)),
    //         None => Ok(()),
    //     }
    // }
}

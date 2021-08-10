use crate::{
    helpers::Hir2Mir,
    modules::{ModuleItem, ResolveState},
};
use indexmap::IndexMap;
use nyar_error::Result;
use nyar_wasm::{Identifier, WasiExport, WasiImport, WasiResource};
use std::{
    fmt::{Debug, Formatter},
    hash::{Hash, Hasher},
    ops::AddAssign,
    sync::Arc,
};
use valkyrie_ast::{helper::WrapDisplay, ClassDeclaration, ClassTerm, FieldDeclaration, MethodDeclaration};

mod display;
mod stage1_mir;
mod stage2_lir;

#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieClass {
    pub(crate) symbol: Identifier,
    /// The wasi import/export name
    pub wasi_import: Option<WasiImport>,
    pub(crate) fields: IndexMap<Arc<str>, ValkyrieField>,
    pub(crate) methods: IndexMap<Arc<str>, ValkyrieMethod>,
}

impl Hash for ValkyrieClass {
    /// ```wat
    /// $type-id = package::module::name
    ///          + Generic Types
    /// ```
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct ValkyrieField {
    /// The name of the field
    pub field_name: Arc<str>,
    /// The WASI name of the field
    pub wasi_alias: Arc<str>,
}

/// A method belongs to a class or a trait
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ValkyrieMethod {
    /// The name of the method
    pub method_name: Arc<str>,
    /// The WASI import symbol if exists
    pub wasi_import: Option<WasiImport>,
    /// The WASI export symbol if exists
    pub wasi_export: Option<WasiExport>,
}

impl AddAssign<ValkyrieField> for ValkyrieClass {
    fn add_assign(&mut self, rhs: ValkyrieField) {
        self.fields.insert(rhs.field_name.clone(), rhs);
    }
}

impl AddAssign<ValkyrieMethod> for ValkyrieClass {
    fn add_assign(&mut self, rhs: ValkyrieMethod) {
        self.methods.insert(rhs.method_name.clone(), rhs);
    }
}

impl ValkyrieClass {
    pub fn new(symbol: Identifier) -> Self {
        Self { symbol, wasi_import: None, fields: Default::default(), methods: Default::default() }
    }
    pub fn get_name(&self) -> String {
        self.symbol.to_string()
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

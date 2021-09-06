use crate::{
    Identifier, STRING_POOL, Spur, ValkyrieTypeGraph,
    field_data::{IntoField, ValkyrieFieldData},
    string_pool::NamePath,
};
use indexmap::IndexMap;
use std::sync::{Arc, LockResult, RwLock, RwLockReadGuard, RwLockWriteGuard};

pub trait IntoRowType {
    fn get_type_name(&self) -> NamePath;
    fn get_type_kind(&self) -> ValkyrieRowKind {
        ValkyrieRowKind::Class { r#abstract: false }
    }
    fn into_row(self) -> ValkyrieRowType
    where
        Self: Sized,
    {
        ValkyrieRowType { data: Arc::new(RwLock::new(self.into_row_data())) }
    }
    fn into_row_data(self) -> ValkyrieRowData
    where
        Self: Sized,
    {
        let name = self.get_type_name();
        let kind = self.get_type_kind();
        ValkyrieRowData { name, kind, parents: vec![], instance_of: vec![], fields: Default::default() }
    }
}

#[derive(Clone)]
pub struct ValkyrieRowType {
    data: Arc<RwLock<ValkyrieRowData>>,
}

pub struct ValkyrieRowData {
    pub kind: ValkyrieRowKind,
    pub name: NamePath,
    pub parents: Vec<NamePath>,
    pub instance_of: Vec<NamePath>,
    pub fields: IndexMap<Identifier, ValkyrieFieldData>,
}

pub enum ValkyrieRowKind {
    Primitive {
        // bits
        size: usize,
        mapping: Spur,
    },
    // Normal class with all fields filled and all methods implemented
    Class {
        // Allowing some methods not to be implemented
        r#abstract: bool,
    },
    // Field is not allowed
    //
    // Can have auto-implemented macros
    Trait {
        /// Whether to require rewriting in inheritance
        mandatory_implementation: bool,
        /// Whether to define the macro for automatically derived interfaces
        automation_implementation: Option<NamePath>,
    },
    // enumerate number
    Enums,
    // bit-width number
    Flags,
    Unity,
    Variant,
}

impl<'a> IntoRowType for &'a str {
    fn get_type_name(&self) -> NamePath {
        let name = STRING_POOL.encode_string(self);
        NamePath::from(name)
    }
}

impl IntoRowType for NamePath {
    fn get_type_name(&self) -> NamePath {
        *self
    }
}

impl ValkyrieRowType {
    pub fn get_data(&self) -> LockResult<RwLockReadGuard<'_, ValkyrieRowData>> {
        self.data.read()
    }
    pub fn mut_data(&self) -> LockResult<RwLockWriteGuard<'_, ValkyrieRowData>> {
        self.data.write()
    }

    pub fn get_name(&self) -> NamePath {
        match self.data.read() {
            Ok(o) => o.name,
            Err(e) => {
                tracing::error!("Failed to acquire lock: {:?}", e);
                NamePath::default()
            }
        }
    }
    pub fn add_parent(&self, parent: impl IntoRowType) {
        let name = parent.get_type_name();
        match self.data.write() {
            Ok(mut o) => o.parents.push(name),
            Err(e) => {
                tracing::error!("Failed to acquire lock: {:?}", e);
            }
        }
    }
    pub fn add_instance_of(&self, subtyping: impl IntoRowType) {
        let name = subtyping.get_type_name();
        match self.data.write() {
            Ok(mut o) => o.instance_of.push(name),
            Err(e) => {
                tracing::error!("Failed to acquire lock: {:?}", e);
            }
        }
    }
    pub fn add_field(&self, field: impl IntoField) {
        let name = field.as_field_name();
        match self.data.write() {
            Ok(mut o) => {
                o.fields.insert(name, field.into_field_data());
            }
            Err(e) => {
                tracing::error!("Failed to acquire lock: {:?}", e);
            }
        }
    }

    pub fn inherit_order(&self) -> Vec<NamePath> {
        let mut order = Vec::with_capacity(16);
        match self.data.read() {
            Ok(o) => {
                order.extend(o.parents.iter());
                order.extend(o.instance_of.iter());
            }
            Err(e) => {
                tracing::error!("Failed to acquire lock: {:?}", e);
            }
        }
        order
    }
}

impl ValkyrieRowData {
    pub fn inherit_order(&self, graph: &ValkyrieTypeGraph) -> Vec<&NamePath> {
        let mut order = Vec::with_capacity(self.parents.len());
        order.extend(self.parents.iter());
        order.extend(self.instance_of.iter());
        order
    }
}

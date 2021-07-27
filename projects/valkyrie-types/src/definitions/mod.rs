use crate::{
    backends::ConvertTo,
    types::{atomic_type::ValkyrieDocument, ValkyrieMetaType},
    InitializeType, ValkyrieEnumerate, ValkyrieError, ValkyrieID, ValkyrieString,
};
use nyar_error::{FileSpan, Success, Validation};
use nyar_wasm::FieldType;
use std::{
    collections::BTreeMap,
    fmt::{Debug, Formatter},
    ops::Range,
    sync::Arc,
};
use valkyrie_ast::{ExpressionKind, ExpressionNode, FlagDeclaration, FlagKind, IdentifierNode, NamePathNode};

pub mod enumerates;
pub mod fields;
pub mod instances;
pub mod interfaces;
pub mod methods;
pub mod names;
pub mod properties;

pub struct Valhalla {
    /// Current working namespace
    scope: ValkyrieID,
    items: BTreeMap<String, ValhallaItem>,
}

pub struct ValhallaBuilder {}

pub enum ValhallaItem {
    Enumerate(ValkyrieEnumerate),
}
impl Valhalla {
    pub(crate) fn load_ns(&self, o: NamePathNode) -> Result<ValkyrieID, ValkyrieError> {
        todo!()
    }
    pub(crate) fn load_id(&self, o: IdentifierNode) -> ValkyrieID {
        todo!()
    }
}

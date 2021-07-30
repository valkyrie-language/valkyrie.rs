pub mod boolean;
#[cfg(feature = "polars")]
pub mod data_frame;
pub mod images;
pub mod json_like;
pub mod list;
pub mod module_name;
pub mod pointer;
pub mod primitive;
pub mod result;

pub mod texts;

use crate::{types::ValkyrieMetaType, ValkyrieType, ValkyrieValue};
use indexmap::IndexMap;
use shredder::Gc;
use std::{
    collections::{hash_map::DefaultHasher, BTreeMap, BTreeSet},
    fmt::Debug,
    hash::{Hash, Hasher},
    ops::Not,
    sync::Arc,
};

/// The display style of a token
pub enum TokenType {
    /// A keyword
    Keyword,
    /// `a, b, c`
    Punctuation,
    /// `a + b`
    Operator,
    /// `structure StructureName`
    Structure,
    /// `class ClassName`
    Class,
    /// `enumerate EnumerateName`
    Enumerate,
    /// `union UnionName`
    Union,
    /// `unite UniteName`
    Unite,
    /// `variant VariantName`
    Variant,
    /// `interfaces InterfaceName`
    Interface,
    /// `trait TraitName`
    Trait,
    /// `let variable`
    Variable,
    /// `let mut variable`
    VariableMutable,
    /// `method(parameter)`
    Parameter,
    /// `method(mut parameter)`
    ParameterMutable,
    /// `method(self)`
    ParameterSelf,
    /// `method(mut self)`
    ParameterSelfMutable,
    /// `constant`
    Constant,
}

#![feature(associated_type_defaults)]
#![feature(extend_one)]
#![feature(pattern)]

mod frontends;
mod functions;
mod helpers;
mod modules;
mod string_pool;
mod structures;
mod types;

pub use crate::{
    functions::{ValkyrieImportFunction, ValkyrieNativeFunction},
    modules::{NamespaceItem, ResolveContext, ValkyrieModule},
    string_pool::{variable::Variable, FileName, Identifier, Location, NamePath, StringPool, STRING_POOL},
    structures::{ValkyrieClass, ValkyrieField, ValkyrieFrom, ValkyrieInto, ValkyrieMethod, ValkyriePrimitive},
    types::{
        encoding_type::ValkyrieSemanticNumber, enumeration_types::ValkyrieEnumeration, flag_types::ValkyrieFlagation,
        unite_types::ValkyrieUnite, variant_type::ValkyrieVariant, ValkyrieType,
    },
};

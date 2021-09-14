
#![feature(associated_type_defaults)]
#![feature(extend_one)]
#![feature(pattern)]

mod string_pool;
mod frontends;
mod functions;
mod helpers;
mod modules;
mod structures;
mod types;

pub use crate::{
    string_pool::{FileName, Identifier, Location, NamePath, STRING_POOL, StringPool, variable::Variable},
    functions::{ValkyrieImportFunction, ValkyrieNativeFunction},
    modules::{NamespaceItem, ResolveContext, ValkyrieModule},
    structures::{ValkyrieClass, ValkyrieField, ValkyrieFrom, ValkyrieInto, ValkyrieMethod, ValkyriePrimitive},
    types::{
        encoding_type::ValkyrieSemanticNumber, enumeration_types::ValkyrieEnumeration, flag_types::ValkyrieFlagation,
        unite_types::ValkyrieUnite, variant_type::ValkyrieVariant, ValkyrieType,
    },
};

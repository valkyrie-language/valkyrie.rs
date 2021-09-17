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
    string_pool::{FileName, Identifier, Location, NamePath, STRING_POOL, StringPool, variable::Variable},
    structures::{ValkyrieClass, ValkyrieField, ValkyrieFrom, ValkyrieInto, ValkyrieMethod, ValkyriePrimitive},
    types::{
        ValkyrieType, encoding_type::ValkyrieSemanticNumber, enumeration_types::ValkyrieEnumeration,
        flag_types::ValkyrieFlagation, unite_types::ValkyrieUnite, variant_type::ValkyrieVariant,
    },
};

#![feature(associated_type_defaults)]
#![feature(extend_one)]

mod frontends;
mod functions;
mod helpers;
mod modules;
mod structures;
mod types;

pub use crate::{
    functions::{ValkyrieImportFunction, ValkyrieNativeFunction},
    modules::{ModuleItem, ResolveContext, ValkyrieModule},
    structures::{PrimitiveType, ValkyrieClass, ValkyrieField, ValkyrieFrom, ValkyrieInto, ValkyrieMethod, ValkyriePrimitive},
    types::{
        encoding_type::ValkyrieSemanticNumber, enumeration_types::ValkyrieEnumeration, flag_types::ValkyrieFlagation,
        unite_types::ValkyrieUnite, variant_type::ValkyrieVariant, ValkyrieType,
    },
};

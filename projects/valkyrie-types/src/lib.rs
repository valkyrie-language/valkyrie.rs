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
    modules::{ModuleItem, ResolveState, ValkyrieModule},
    structures::{ValkyrieClass, ValkyrieField, ValkyrieMethod},
    types::{
        enumeration_types::{ValkyrieEnumeration, ValkyrieSemanticNumber},
        flag_types::ValkyrieFlagation,
        unite_types::{ValkyrieUnite, ValkyrieUniteItem},
        ValkyrieType,
    },
};

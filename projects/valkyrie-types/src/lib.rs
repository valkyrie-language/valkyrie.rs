#![feature(associated_type_defaults)]
#![feature(extend_one)]

mod frontends;
mod functions;
mod helpers;
mod modules;
mod structures;
mod types;
mod variants;

pub use crate::{
    functions::{ValkyrieImportFunction, ValkyrieNativeFunction},
    modules::{ModuleItem, ResolveState, ValkyrieModule},
    structures::{ValkyrieClass, ValkyrieField, ValkyrieMethod},
    types::{
        enumeration_types::{ValkyrieEnumeration, ValkyrieSemanticNumbers},
        flag_types::ValkyrieFlags,
        ValkyrieType,
    },
    variants::{ValkyrieUnionItem, ValkyrieUnite},
};

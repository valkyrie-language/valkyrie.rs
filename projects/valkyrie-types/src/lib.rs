#![feature(trivial_bounds)]
#![feature(allocator_api)]
#![feature(never_type)]
#![feature(unboxed_closures)]
#![feature(lazy_cell)]
#![feature(extend_one)]
#![feature(associated_type_defaults)]

pub mod helpers;
pub mod projects;

mod builtin;
mod collection;
mod functions;
mod modifiers;
mod modules;
// #[cfg(test)]
mod backends;
mod definitions;

mod packages;
pub mod structures;
pub mod testing;
pub mod third_party;
mod types;
mod utils;
mod values;

pub use self::{
    builtin::{
        images::ValkyrieImage,
        result::{ValkyrieFailure, ValkyrieSuccess},
        texts::{StringID, ValkyrieString},
        TokenType,
    },
    collection::dict::ValkyrieDict,
    definitions::{enumerates::ValkyrieEnumerate, fields::ValkyrieField, interfaces::ValkyrieInterface, names::ValkyrieName},
    functions::{ValkyrieFunction, ValkyrieFunctionType, ValkyrieMonomorphicFunction},
    modifiers::{FeatureType, InitializeType, MutableType},
    modules::{ModuleItem, ModuleResolver, ValkyrieModule},
    packages::ids::{ValkyrieID, ValkyrieUniverse},
    types::{
        atomic_type::ValkyrieAtomicType, literal_type::ValkyrieLiteralType, union_type::ValkyrieUnionType,
        variant_type::ValkyrieVariantType, ValkyrieType,
    },
    values::{symbols::ValkyrieSymbol, ValkyrieValue},
};
pub(crate) use self::{modules::HIR, values::symbols::AsSymbol};
pub use nyar_error::{
    Failure, FileCache, FileID, MissingError, NyarError as ValkyrieError, Result as ValkyrieResult, RuntimeError, Success,
    SyntaxError,
};
pub use shredder::Gc;
pub use structures::ValkyrieStructure;
pub use valkyrie_ast::ValkyrieOperator;
pub use valkyrie_parser::{ProgramContext, StringFormatterBuilder};

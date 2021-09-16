#![feature(pattern)]

mod string_pool;

pub use crate::string_pool::{FileName, Identifier, Location, NamePath, STRING_POOL, StringPool, variable::Variable};
pub use valkyrie_error::{NyarError, NyarErrorKind, Result, SyntaxError};

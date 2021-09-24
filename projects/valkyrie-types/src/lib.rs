#![feature(pattern)]

mod bindings;
mod source_pool;
mod string_pool;
mod string_pool2;

pub use crate::string_pool::{variable::Variable, FileName, Location, NamePath};
pub use string_pool2::identifier::Identifier;
pub use valkyrie_error::{
    third_party, Failure, ForeignInterfaceError, NyarError, NyarErrorKind, ReportKind, Result, SourceCache, SourceID,
    SourceSpan, Success, SyntaxError, Validation,
};
// use crate::string_pool2::StringContext;

// crate::bindings::export!(StringContext with_types_in bindings);

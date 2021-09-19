#![feature(pattern)]

mod string_pool;

pub use crate::string_pool::{
    identifier::Identifier, variable::Variable, FileName, Location, NamePath, StringPool, STRING_POOL,
};
pub use valkyrie_error::{
    third_party, Failure, ForeignInterfaceError, NyarError, NyarErrorKind, ReportKind, Result, SourceCache, SourceID,
    SourceSpan, Success, SyntaxError, Validation,
};

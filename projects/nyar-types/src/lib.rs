#![warn(missing_docs)]

//! Minimal shared types used by the current Rust bootstrap path.

pub use self::{
    errors::{NyarError, NyarErrorKind},
    source::{Location, Position, SourceID, SourceSpan},
    symbols::{Identifier, NamePath, QualifiedName, SymbolIdentity},
};
mod errors;
mod source;
mod symbols;

/// Stable capability tag shared across analyzers, planners and backends.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CapabilityTag(String);

impl CapabilityTag {
    /// Creates a new capability tag.
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Returns the tag as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for CapabilityTag {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for CapabilityTag {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl std::fmt::Display for CapabilityTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

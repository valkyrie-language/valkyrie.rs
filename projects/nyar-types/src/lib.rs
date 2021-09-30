#![warn(missing_docs)]

//! Minimal shared types used by the current Rust bootstrap path.

use std::fmt;

/// Fully qualified name in different languages.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QualifiedName {
    parts: Vec<String>,
}

impl QualifiedName {
    /// Creates a qualified name from explicit parts.
    pub fn new(parts: Vec<String>) -> Self {
        Self { parts }
    }

    /// Returns the parts as string slices.
    pub fn parts(&self) -> Vec<&str> {
        self.parts.iter().map(String::as_str).collect()
    }
}

impl From<&str> for QualifiedName {
    fn from(value: &str) -> Self {
        let parts: Vec<String> = if value.is_empty() { Vec::new() } else { value.split("::").map(|s| s.to_string()).collect() };
        Self { parts }
    }
}

impl From<String> for QualifiedName {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}

impl fmt::Display for QualifiedName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.parts.join("::"))
    }
}

/// Minimal shared error kind for compatibility during bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NyarErrorKind {
    key: String,
    message: String,
}

impl NyarErrorKind {
    /// Creates a new error kind.
    pub fn new(key: impl Into<String>, message: impl Into<String>) -> Self {
        Self { key: key.into(), message: message.into() }
    }

    /// Returns the stable diagnostic key.
    pub fn key(&self) -> &str {
        &self.key
    }
}

impl fmt::Display for NyarErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

/// Minimal shared error wrapper for compatibility during bootstrap.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NyarError {
    /// Numeric error code.
    pub code: u32,
    /// Structured error payload.
    pub kind: NyarErrorKind,
}

impl NyarError {
    /// Creates a new shared error.
    pub fn new(code: u32, key: impl Into<String>, message: impl Into<String>) -> Self {
        Self { code, kind: NyarErrorKind::new(key, message) }
    }
}

//! Module and documentation definitions for HIR.

use super::{
    type_family::{HirTypeFamily, HirTypeFunction},
    HirEnum, HirFlags, HirFunction, HirImpl, HirSingleton, HirStatement, HirStruct, HirTrait, HirWidget,
};
use crate::NamePath;

/// A module in HIR.
///
/// Modules are the top-level organizational unit in Valkyrie,
/// containing functions, types, and other definitions.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirModule {
    /// The fully qualified name path of the module.
    pub name: NamePath,
    /// Documentation for the module.
    pub doc: HirDocumentation,
    /// Import statements in this module.
    pub imports: Vec<NamePath>,
    /// Nested submodules.
    pub submodules: Vec<HirModule>,
    /// Functions defined in this module.
    pub functions: Vec<HirFunction>,
    /// Structs defined in this module.
    pub structs: Vec<HirStruct>,
    /// Enums defined in this module.
    pub enums: Vec<HirEnum>,
    /// Flags types defined in this module.
    pub flags: Vec<HirFlags>,
    /// Traits defined in this module.
    pub traits: Vec<HirTrait>,
    /// Impl blocks defined in this module.
    pub impls: Vec<HirImpl>,
    /// Type functions defined in this module.
    pub type_functions: Vec<HirTypeFunction>,
    /// Type families defined in this module.
    pub type_families: Vec<HirTypeFamily>,
    /// Widgets defined in this module.
    pub widgets: Vec<HirWidget>,
    /// Singletons defined in this module.
    pub singletons: Vec<HirSingleton>,
    /// Top-level statements in this module.
    pub statements: Vec<HirStatement>,
}

/// Documentation for HIR items.
///
/// Documentation is stored as a collection of lines, typically
/// extracted from doc comments (`///` or `/** */`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirDocumentation {
    /// The documentation lines.
    pub lines: Vec<String>,
}

impl HirDocumentation {
    /// Creates documentation from multiple lines.
    pub fn from_lines(lines: Vec<String>) -> Self {
        Self { lines }
    }

    /// Creates documentation from a single line.
    pub fn from_single(line: impl Into<String>) -> Self {
        Self { lines: vec![line.into()] }
    }

    /// Returns true if the documentation is empty.
    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }
}

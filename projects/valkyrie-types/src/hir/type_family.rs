//! Type function and type family definitions for HIR.

use super::{HirBlock, HirDocumentation, HirGeneric, HirParam, HirType};
use crate::Identifier;

/// A type function in HIR.
///
/// Type functions are functions that operate on types rather than values.
/// They take types as parameters and return types as results.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirTypeFunction {
    /// The name of the type function.
    pub name: Identifier,
    /// Documentation for the type function.
    pub doc: HirDocumentation,
    /// Generic parameters for the type function.
    pub generics: Vec<HirGeneric>,
    /// Parameters of the type function.
    pub params: Vec<HirParam>,
    /// The return type of the type function.
    pub return_type: HirType,
    /// The body of the type function.
    pub body: HirBlock,
}

/// A type family in HIR.
///
/// Type families define a set of related types indexed by types.
/// They allow for type-level computation and ad-hoc polymorphism.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirTypeFamily {
    /// The name of the type family.
    pub name: Identifier,
    /// Documentation for the type family.
    pub doc: HirDocumentation,
    /// The cases of the type family.
    ///
    /// Each case maps an input type to an output type.
    pub cases: Vec<(HirType, HirType)>,
}

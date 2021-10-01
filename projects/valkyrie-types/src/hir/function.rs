//! Function definitions for HIR.

use super::{GenericType, HirAttribute, HirBlock, HirParam, HirVisibility, ValkyrieType};
use crate::{Identifier, SourceSpan};

/// A function in HIR.
///
/// Functions are the primary unit of computation in Valkyrie.
/// They can be standalone or methods within a struct/class.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HirFunction {
    /// The name of the function.
    pub name: Identifier,
    /// Documentation for the function.
    pub doc: super::HirDocumentation,
    /// Attributes applied to the function (e.g., `@inline`, `@deprecated`).
    pub annotations: Vec<HirAttribute>,
    /// Generic parameters for the function.
    pub generics: Vec<GenericType>,
    /// Parameters of the function.
    pub params: Vec<HirParam>,
    /// The return type of the function.
    pub return_type: ValkyrieType,
    /// The function body block.
    pub body: HirBlock,
    /// The source span for error reporting.
    pub span: SourceSpan,
    /// Visibility of the function.
    pub visibility: HirVisibility,
    /// Whether this function is abstract (has no body implementation).
    ///
    /// Abstract functions are declared without a body in abstract classes
    /// and must be implemented by concrete subclasses.
    pub is_abstract: bool,
    /// Whether this function is final (cannot be overridden).
    ///
    /// Final methods cannot be overridden by subclasses.
    /// This is useful for methods that should have fixed behavior.
    pub is_final: bool,
}

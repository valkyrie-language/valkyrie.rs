use crate::{ExpressionNode, IdentifierNode};
use core::ops::Range;

/// `vow T {} fun f() { return T }`
///
///
/// ```vk
/// vouch T, U {
///     return T
///     effect [Asynchronous, Logging]
/// }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuaranteeNode {
    /// The generics of this guarantee
    pub generics: Vec<IdentifierNode>,
    /// The return type of this guarantee
    pub return_type: Option<ExpressionNode>,
    /// The effects of this guarantee
    pub effect_type: Option<ExpressionNode>,
    /// The body of this guarantee
    pub span: Range<u32>,
}

/// `[Asynchronous<T>, Logging]`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EffectTypeNode {
    /// The generics of this effect
    pub effect_type: Vec<ExpressionNode>,
    /// The range of the node
    pub span: Range<u32>,
}

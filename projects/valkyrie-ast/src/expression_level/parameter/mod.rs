mod display;

use super::*;

/// The kind of the parameter node
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterKind {
    /// `a: Type = null`
    Expression,
    /// `T: Trait = ()`
    Generic,
}

/// `micro f(t: Type = default)` or `class F⦓T: Trait = Default⦔`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParametersList {
    /// The kind of the parameter node
    pub kind: ParameterKind,
    /// The raw string of the number.
    pub terms: Vec<ParameterTerm>,
}

/// `T: Type = type_expression`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParameterTerm {
    /// `<`
    ///
    /// The type on the left cannot be called by name, but the type on the right can be called by name.
    LMark,
    /// `>`
    ///
    /// The type on the left can be called by name, and the type on the right must be called by name.
    RMark,
    /// `#annotation modifier a: Type = default`
    Single {
        /// The modifiers apply on the parameter
        annotations: AnnotationNode,
        /// The name
        key: IdentifierNode,
        /// The type boundary of the parameter
        bound: Option<ExpressionKind>,
        /// The default value of the parameter
        default: Option<ExpressionKind>,
    },
    /// `#annotation modifier ..list: Type`
    UnpackList {
        /// The modifiers apply on the parameter
        modifiers: ModifierList,
        /// The list receiver name
        key: IdentifierNode,
        /// The type boundary of the parameter
        bound: Option<ExpressionKind>,
    },
    /// `#annotation modifier ...dict: Type`
    UnpackDict {
        /// The modifiers apply on the parameter
        modifiers: ModifierList,
        /// The dict receiver name
        key: IdentifierNode,
        /// The type boundary of the parameter
        bound: Option<ExpressionKind>,
    },
}

impl ParametersList {
    /// Create a new parameter list
    pub fn new(capacity: usize, kind: ParameterKind) -> Self {
        Self { kind, terms: Vec::with_capacity(capacity) }
    }
}

impl Default for ParameterKind {
    fn default() -> Self {
        Self::Expression
    }
}
impl Default for ParametersList {
    fn default() -> Self {
        Self { kind: ParameterKind::default(), terms: vec![] }
    }
}

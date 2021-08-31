use super::*;
mod display;
mod iters;

/// The flags kind
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SemanticKind {
    /// `enumerate`, exclusive encoding number
    Enumerate,
    /// `flags`, juxtapose encoding number
    Flags,
}

/// a number that encodes special semantics
///
/// `enumerate Bit(8bits): Trait { FlagA, FlagB }`
///
/// `flags Bit(8bits): Trait { FlagA, FlagB }`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticNumber {
    /// The kind of the flag statement
    pub kind: SemanticKind,
    /// The annotations of this flag.
    pub annotations: AnnotationNode,
    /// The name of the flag.
    pub name: IdentifierNode,
    /// `flags Flag = u32`
    pub layout: Option<ExpressionKind>,
    /// `flags Flag: Trait`
    pub implements: Option<ExpressionKind>,
    /// `flags Flag { FlagA, FlagB }`
    pub body: Vec<FlagTerm>,
    /// The range of the node.
    pub span: Range<u32>,
}

/// Valid terms in the flags statement
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlagTerm {
    /// `@macro`
    Macro(ProceduralNode),
    /// `Name = number`
    Encode(EncodeDeclaration),
    /// `method() { }`
    Method(MethodDeclaration),
}

/// `Name = 0x00`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EncodeDeclaration {
    /// The documentation for this field.
    pub annotations: AnnotationNode,
    /// The identifier of the field.
    pub name: IdentifierNode,
    /// The value of the field if exists.
    pub value: Option<ExpressionKind>,
    /// The range of the node.
    pub span: Range<u32>,
}

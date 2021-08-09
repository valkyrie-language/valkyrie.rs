use super::*;

mod display;

mod iters;

/// `union Name(Super): Trait {}`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnionDeclaration {
    /// The annotations of this union
    pub annotations: AnnotationNode,
    /// The range of the number.
    pub name: IdentifierNode,
    /// `union A(Union)`, the super unions
    pub inherits: Vec<ExpressionKind>,
    /// `union A: Debug { }`, the trait bounds
    pub implements: Option<ExpressionKind>,
    /// The variants of this union
    pub body: Vec<UnionTerm>,
    /// The text range of the statement
    pub span: Range<u32>,
}

/// `Variant { }`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariantDeclaration {
    /// The name of variants
    pub name: IdentifierNode,
    /// The annotations of the variant declaration.
    pub annotations: AnnotationNode,
    /// The main body of the variant
    pub body: Vec<ClassTerm>,
    /// The range of the node
    pub span: Range<u32>,
}

/// A valid term in the union declaration.
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnionTerm {
    /// `@procedural`
    Macro(ProceduralNode),
    /// `Variant { }`
    Variant(VariantDeclaration),
    /// `method()`
    Method(MethodDeclaration),
}

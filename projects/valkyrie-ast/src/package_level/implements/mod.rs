use super::*;

/// `extends path::A: Debug {}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImplementsStatement {
    pub keyword: Range<u32>,
    /// `implements A: Debug { }`, the annotations
    pub annotations: AnnotationNode,
    /// `implements A: Debug { }`, the trait bounds
    pub target: NamePathNode,
    /// `implements A: Debug { }`, the trait bounds
    pub implements: Option<ExpressionKind>,
    /// The additional methods
    pub body: Vec<TraitTerm>,
    pub span: Range<u32>,
}

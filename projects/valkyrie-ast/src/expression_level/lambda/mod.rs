use super::*;

mod display;

/// `lambda(args) { ... }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LambdaNode {
    /// The modifiers of the node.
    pub annotations: AnnotationNode,
    /// Thy type parameters of this function
    pub generics: ParametersList,
    /// Thy value parameters of this function
    pub parameters: ParametersList,
    /// `: ReturnType / [EffectType]`
    pub returns: FunctionReturnNode,
    /// `{ body }`
    pub body: StatementBlock,
    /// The range of the declaration.
    pub span: Range<u32>,
}

impl ValkyrieNode for LambdaNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

/// `object.{ lambda(args), ... }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClosureCallNode {
    /// Weather it is a monadic call
    pub monadic: bool,
    /// called
    pub base: ExpressionKind,
    /// trailing closure
    pub trailing: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

impl ValkyrieNode for ClosureCallNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl ClosureCallNode {
    /// Replace placeholder with actual expression
    pub fn with_base(self, base: ExpressionKind) -> Self {
        Self { base, ..self }
    }
}

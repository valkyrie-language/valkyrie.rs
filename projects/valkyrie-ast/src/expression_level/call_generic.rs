use super::*;

/// `A⦓T⦔, A⟨T⟩, A::<T>`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GenericCallNode {
    /// `this?::<T>`
    pub monadic: bool,
    /// `Base::<T>, ::<T as Trait>`
    pub base: ExpressionKind,
    /// `A::<T>::Associated::<T as Trait>`
    pub term: GenericCallTerm,
    /// The range of the node
    pub span: Range<u32>,
}

/// Call with static method
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GenericCallTerm {
    /// `T::Associated`
    Associated(IdentifierNode),
    /// `f::<T, U>`
    Generic(ArgumentsList),
}

impl ValkyrieNode for GenericCallNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

impl GenericCallNode {
    /// Replace placeholder with actual expression
    pub fn with_base(self, base: ExpressionKind) -> Self {
        Self { base, ..self }
    }
}

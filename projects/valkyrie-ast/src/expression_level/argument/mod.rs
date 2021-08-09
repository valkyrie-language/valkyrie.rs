use super::*;
use crate::StatementBlock;
use core::ops::AddAssign;

mod arithmetic;
mod display;

/// `a + b, c: d, ..e`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgumentsList {
    /// The raw string of the number.
    pub terms: Vec<ArgumentTerm>,
}

impl Debug for ArgumentsList {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_list().entries(self.terms.iter()).finish()
    }
}

/// `#annotation mut this: null`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgumentTerm {
    /// The modifier conditions
    pub modifiers: ModifierList,
    /// The key of the argument
    pub key: ArgumentKey,
    /// The value of the argument
    pub value: ExpressionKind,
    /// The range of the node
    pub span: SourceSpan,
}

/// The key of the argument
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ArgumentKey {
    /// `a + b`
    Nothing,
    /// `key: a + b`
    Symbol(IdentifierNode),
}

impl ArgumentsList {
    /// Create a new `ArgumentsList` with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self { terms: Vec::with_capacity(capacity) }
    }
    /// Create a new `ArgumentsList` with the given capacity.
    pub fn get<T: ?Sized>(&self, argument: &T) -> Option<&ArgumentTerm>
    where
        ArgumentTerm: PartialEq<T>,
    {
        self.terms.iter().filter(|&x| x.eq(argument)).next()
    }
}

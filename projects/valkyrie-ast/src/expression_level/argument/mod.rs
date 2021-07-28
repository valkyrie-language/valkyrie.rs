use super::*;
use crate::StatementBlock;
use core::ops::AddAssign;

mod display;

/// `a + b, c: d, ..e`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArgumentsList {
    /// The raw string of the number.
    pub terms: Vec<ArgumentTerm>,
}

impl AddAssign<ArgumentTerm> for ArgumentsList {
    fn add_assign(&mut self, rhs: ArgumentTerm) {
        self.terms.push(rhs)
    }
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
}

use super::*;

mod display;

/// The receiver type of the tuple literal
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TupleKind {
    /// `(a, b, ..c)`
    Tuple,
    /// `⦃a: 1, b, ..c⦄`
    Set,
}

/// `(tuple, ), (named: tuple, expression)`
#[derive(Clone, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TupleNode {
    ///  The kind of tuple.
    pub kind: TupleKind,
    /// The raw string of the number.
    pub terms: ArgumentsList,
    /// The range of the number.
    pub span: Range<u32>,
}

impl Debug for TupleNode {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        let w = &mut match self.kind {
            TupleKind::Tuple => f.debug_struct("Tuple"),
            TupleKind::Set => f.debug_struct("Set"),
        };
        w.field("terms", &self.terms);
        w.finish()
    }
}

impl ValkyrieNode for TupleNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl Default for TupleKind {
    fn default() -> Self {
        Self::Tuple
    }
}

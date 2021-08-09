#[cfg(feature = "pretty-print")]
mod display;

use super::*;

/// The literal of array
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RangeKind {
    /// `[1, 2:3, 4:5:6]`
    Ordinal,
    /// `⁅1, 2:3, 4:5:6⁆`
    Offset,
}

/// `[0, [], [:], [::]]`
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RangeNode {
    ///  The kind of tuple.
    pub kind: RangeKind,
    /// Terms
    pub terms: Vec<RangeTermNode>,
    /// The range of the number.
    pub span: Range<u32>,
}
impl ValkyrieNode for RangeNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
/// `[index], ⁅start : end : step⁆`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RangeTermNode {
    /// The index kind
    Index {
        /// The index of range
        index: ExpressionKind,
        /// The range of the number.
        span: SourceSpan,
    },
    /// The range
    Range {
        /// The first element in range
        head: Option<ExpressionKind>,
        /// The middle element in range
        tail: Option<ExpressionKind>,
        /// The
        step: Option<ExpressionKind>,
    },
}

impl Default for RangeKind {
    fn default() -> Self {
        Self::Ordinal
    }
}

impl RangeNode {
    /// Convert to tuple if possible
    pub fn as_tuple(&self) -> Option<TupleNode> {
        let mut terms = Vec::with_capacity(self.terms.len());
        for term in &self.terms {
            terms.push(term.as_tuple()?)
        }
        Some(TupleNode { kind: TupleKind::Tuple, terms: ArgumentsList { terms }, span: self.span.clone() })
    }
}

impl RangeTermNode {
    /// Convert to tuple item if possible
    pub fn as_tuple(&self) -> Option<ArgumentTerm> {
        match self {
            RangeTermNode::Index { index, span } => Some(ArgumentTerm {
                modifiers: Default::default(),
                key: ArgumentKey::Nothing,
                value: index.clone(),
                span: *span,
            }),
            RangeTermNode::Range { .. } => None,
        }
    }
}

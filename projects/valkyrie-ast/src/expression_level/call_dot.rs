use super::*;
use core::num::NonZeroU64;

/// `a.method.1`
///
/// ```v
/// f { a + b }
/// f(0, key: 1, ..list)
/// f(0, key: 1, ..list) { a + b }
///
/// this.m { a + b }
/// this.m(0, key: 1, ..list)
/// this.m(0, key: 1, ..list) { a + b }
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotCallNode {
    /// Weather it is a monadic call
    pub monadic: bool,
    /// The caller of argument
    pub base: ExpressionKind,
    /// The call arguments
    pub term: DotCallTerm,
    /// The range of the number.
    pub span: Range<u32>,
}

/// `a.method.1`
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DotCallTerm {
    /// `caller.0`
    MetaType,
    /// `caller.1`
    Integer(NonZeroU64),
    /// `caller.module::method`
    Symbol(NamePathNode),
}

impl Debug for DotCallTerm {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::MetaType => f.write_str("MetaType"),
            Self::Integer(v) => Display::fmt(v, f),
            Self::Symbol(v) => Display::fmt(v, f),
        }
    }
}

impl ValkyrieNode for DotCallNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
impl DotCallNode {
    /// Replace placeholder with actual expression
    pub fn with_base(self, base: ExpressionKind) -> Self {
        Self { base, ..self }
    }
}

impl DotCallTerm {
    /// Create a call index from rust index
    pub fn index(u: usize) -> Self {
        match NonZeroU64::new(u as u64) {
            Some(n) => Self::Integer(n),
            None => Self::MetaType,
        }
    }
}

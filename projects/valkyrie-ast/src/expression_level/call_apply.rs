use super::*;

/// `f(a + b, c: d, ..e) { a + b }`
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

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyCallNode {
    /// Weather it is a monadic call
    pub monadic: bool,
    /// The caller of argument
    pub caller: ExpressionKind,
    /// The raw string of the number.
    pub arguments: ArgumentsList,
    /// The raw string of the number.
    pub body: Option<StatementBlock>,
    /// The range of the number.
    pub span: Range<u32>,
}

impl ValkyrieNode for ApplyCallNode {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

impl ApplyCallNode {
    /// Replace placeholder with actual expression
    pub fn with_base(self, base: ExpressionKind) -> Self {
        Self { caller: base, ..self }
    }
}

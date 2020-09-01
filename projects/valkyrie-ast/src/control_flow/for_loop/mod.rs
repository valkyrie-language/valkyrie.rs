use super::*;
mod display;

/// `for ... in ... if ... {...} else {...}`
///
/// ```vk
/// for i in 0..10 if condition {
///     ...
/// }
/// else {
///     ...
/// }
/// ```
///
/// ```vk
/// let i = 1;
/// let j = 1;
/// let mut i, mut j;
/// let [a, b]
/// let (a, b)
/// ```
///
/// ```vk
/// for i in range;
/// for i, j in range;
/// for mut i, mut j in range
/// for [table] in
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ForLoop {
    /// `for pattern`
    pub pattern: PatternType,
    /// `in iterator`
    pub iterator: ExpressionNode,
    /// `if condition`
    pub condition: Option<PatternCondition>,
    /// `{ body }`
    pub body: StatementBlock,
    /// `else { body }`
    pub r#else: Option<ElsePart>,
    /// The range of the node
    pub span: Range<u32>,
}

use super::*;

#[cfg(feature = "pretty-print")]
mod display;

/// `guard a > 0 { ... }`
///
/// The else block must use control.
///
/// ```vk
/// 
/// assert a > 0 {
///     panic!("a must be greater than 0");
/// }
///
/// assert let Some(a) = b {
///     panic!("b must be Some");
/// }
///
///
///
/// if a < 0 {
///     return error;
/// }
/// do_something_else();
/// ```
///
/// `guard let Failure(error) = e if xxx then { ... }`
///
/// The else block must use control.
///
/// ```vk
/// if e.is_failure() {
///     let error = x.as_failure();
///     return error;
/// }
/// do_something_else();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GuardStatement {
    /// The condition to check
    pub negative: bool,
    /// The condition to check
    pub condition: ExpressionNode,
    /// same as if condition
    pub main_body: GuardPattern,
    /// The range of the node
    pub span: Range<u32>,
}
impl ValkyrieNode for GuardStatement {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}
/// `guard <CONDITION> then { ... } else { ... }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GuardPattern {
    /// Same as if condition
    Expression(ExpressionNode),
    /// Same as if !condition
    List(ElseStatement),
    /// Same as if !condition
    Dict(ElseStatement),
}

impl GuardStatement {}

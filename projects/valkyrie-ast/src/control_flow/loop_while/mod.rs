use super::*;
use crate::ForLoop;

mod display;

#[doc = include_str!("readme.md")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WhileLoop {
    /// The kind of while loop, including `while` and `until`
    pub kind: WhileLoopKind,
    /// The condition of the loop
    pub condition: WhileConditionNode,
    /// The main body of the loop
    pub then: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

/// The kind of while loop, including `while` and `until`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WhileLoopKind {
    /// `while condition {}`
    While,
    /// `while !condition {}`
    Until,
}

/// `while true`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WhileConditionNode {
    /// `while { }`
    Unconditional,
    /// `while true {}`
    Expression(ExpressionNode),
    /// `while let Some(_) = ... {}`
    Case(PatternNode),
}

impl ValkyrieNode for WhileLoop {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

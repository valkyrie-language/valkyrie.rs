use super::*;
use crate::LoopEach;

mod display;

#[doc = include_str!("readme.md")]
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LoopWhile {
    /// The kind of while loop, including `while` and `until`
    pub keyword: Range<u32>,
    /// The condition of the loop
    pub condition: WhileConditionNode,
    /// The main body of the loop
    pub then: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

/// `while true`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WhileConditionNode {
    /// `while true {}`
    Expression(ExpressionKind),
    /// `while let Some(_) = ... {}`
    LetCase(CasePattern),
}

impl ValkyrieNode for LoopWhile {
    fn get_range(&self) -> Range<u32> {
        self.span.clone()
    }
}

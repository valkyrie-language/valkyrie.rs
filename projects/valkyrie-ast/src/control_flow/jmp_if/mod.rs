use super::*;
#[cfg(feature = "pretty-print")]
mod display;

/// `if a {1} else if b {2} else if c {3} else {4}`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfStatement {
    /// The case branches to check
    pub branches: Vec<IfBranchNode>,
    /// The default branch if all cases fail
    pub else_body: Option<ElseStatement>,
    /// The range of the node
    pub span: Range<u32>,
}
impl ValkyrieNode for IfStatement {
    fn get_range(&self) -> Range<usize> {
        Range { start: self.span.start as usize, end: self.span.end as usize }
    }
}
/// `a > 0 then { ... }`
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IfBranchNode {
    /// The condition to check
    pub condition: ExpressionNode,
    /// The body of the statement
    pub body: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

/// Helper function to format the body of an if statement
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElseStatement {
    /// The main body of the statement
    pub body: StatementBlock,
    /// The range of the node
    pub span: Range<u32>,
}

/// `@jmp 1 if a > 0`, *MIR*
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JumpStatement {
    /// Jump target id of imports fsm
    pub target: i32,
    /// Jump condition if exists
    pub condition: Option<ExpressionNode>,
}

/// `@br if a > 0`, *MIR*
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BreakStatement {
    /// Break condition if exists
    pub condition: Option<ExpressionNode>,
}

impl IfStatement {
    /// Make the if statement into equivalent switch statement
    pub fn as_switch(&self) -> SwitchStatement {
        todo!()
    }
}

impl WhileConditionNode {
    /// Weather it is a unconditional loop
    pub fn is_empty(&self) -> bool {
        match self {
            WhileConditionNode::Unconditional => true,
            _ => false,
        }
    }
}

use super::*;

mod display;

/// always equivalent to a statement that returns `( )`, and cannot be used as an `rvalue`.
#[derive(Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ControlNode {
    /// The type of control flow
    pub kind: ControlKind,
    /// In theory, all jumps need a destination
    pub label: LabelNode,
    /// The label of the control flow
    pub expression: Option<ExpressionKind>,
    /// The range of the node
    pub span: Range<u32>,
}

/// `goto ^label`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LabelNode {
    /// Find the nearest loop
    Nearest,
    /// Find the loop with the given label
    Named(IdentifierNode),
}

/// The control flow keywords
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ControlKind {
    /// `goto ^label`, equivalent to `call/cc`, can't construct from frontend
    Goto,
    /// `raise ^label`
    Raise,
    /// `break ^label`
    Break,
    /// `continue ^label`, equivalent to `goto`
    Continue,
    /// `fallthrough ^label`
    Fallthrough,
    /// `fallthrough! ^label`
    FallthroughUnchecked,
    /// `return ^LABEL`
    Return,
    /// `resume ^LABEL`
    Resume,
    /// `yield ^LABEL`
    YieldReturn,
    /// `yield break ^LABEL`, `yield! ^LABEL`
    YieldBreak,
    /// `yield from iterator`, `yield* iterator`
    YieldFrom,
    /// `let v = yield wait`, `let v = yield?`
    YieldSend,
    /// `let v = await AWAITABLE`
    Await,
    /// `await? AWAITABLE`
    AwaitNever,
    /// `violence { await! AWAITABLE }`
    AwaitBlockOn,
}

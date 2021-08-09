pub mod annotations;
pub mod argument;
pub mod call_apply;
pub mod call_dot;
pub mod call_subscript;
pub mod ctor;
mod dispatch;
pub mod lambda;
pub mod number;
pub mod operators;
pub mod parameter;
pub mod range;
pub mod string_template;
pub mod symbol;
pub mod tuple;

pub mod call_generic;

mod display;
use crate::{helper::ValkyrieNode, *};
use alloc::{
    boxed::Box,
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::{
    fmt::{Debug, Display, Formatter, Write},
    num::{NonZeroU64, NonZeroUsize},
    ops::Range,
};
use deriver::From;
#[cfg(feature = "lispify")]
use lispify::{Lisp, Lispify};
use nyar_error::{
    third_party::{Associativity, Precedence},
    NyarError, ReportKind, SourceID, SourceSpan, SyntaxError,
};
#[cfg(feature = "pretty-print")]
use pretty_print::{
    helpers::{KAndRBracket, PrettySequence},
    PrettyPrint, PrettyProvider, PrettyTree,
};
/// The ast node for an expression
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionNode {
    /// Weather to omit to results
    pub omit: bool,
    /// The expression body
    pub body: ExpressionKind,
    /// The range of the node
    pub span: Range<u32>,
}

/// Temporary node for use in the parser
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct TypingExpression {
    /// The type level expression body
    pub body: ExpressionKind,
    /// The range of the node
    pub span: Range<u32>,
}

/// Environment for expression parsing
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExpressionContext {
    /// Weather the expression is on type level
    pub type_level: bool,
    /// Weather the expression allow new line in postfix calls
    pub allow_newline: bool,
    /// Weather the expression allow curly braces in postfix calls
    pub allow_curly: bool,
}

/// The base expression type
#[derive(Clone, PartialEq, Eq, Hash, From)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ExpressionKind {
    /// - Placeholder expression
    Placeholder,
    /// - Atomic expression
    Null(Box<NullNode>),
    /// - Atomic expression
    Boolean(Box<BooleanNode>),
    /// - Atomic expression
    Lambda(Box<LambdaNode>),
    /// - Atomic expression
    Slot(Box<LambdaSlotNode>),
    /// - Atomic expression
    Symbol(Box<NamePathNode>),
    /// - Atomic expression
    Number(Box<NumberLiteralNode>),
    /// - Atomic expression
    Text(Box<StringTextNode>),
    /// - Atomic expression
    String(Box<StringLiteralNode>),
    /// - Atomic expression
    Formatted(Box<FormatterNode>),
    /// - Atomic expression
    New(Box<ConstructNewNode>),
    /// - Atomic expression
    Object(Box<ConstructObjectNode>),
    /// - Compound expression
    Unary(Box<UnaryNode>),
    /// - Compound expression
    Infix(Box<BinaryNode>),
    /// - Compound expression
    Tuple(Box<TupleNode>),
    /// - Compound expression
    Array(Box<RangeNode>),
    /// - Standalone expression
    If(Box<IfStatement>),
    /// - Standalone expression
    IfLet(Box<GuardStatement>),
    /// - Standalone expression
    Switch(Box<SwitchStatement>),
    /// - Standalone expression
    Match(Box<MatchStatement>),
    /// - Standalone expression
    Try(Box<TryStatement>),
    /// `@procedural() { }`
    Procedural(Box<ProceduralNode>),
    /// - Postfix expression
    ApplyCall(Box<ApplyCallNode>),
    /// - Postfix expression
    ClosureCall(Box<ClosureCallNode>),
    /// - Postfix expression
    SubscriptCall(Box<SubscriptCallNode>),
    /// - Postfix expression
    GenericCall(Box<GenericCallNode>),
    /// - Postfix expression
    DotCall(Box<DotCallNode>),
    /// - Postfix expression
    DotMatchCall(Box<MatchCallNode>),
    /// - REPL Reference
    OutputReference(Box<OutputNode>),
}

impl Default for ExpressionContext {
    fn default() -> Self {
        ExpressionContext { type_level: false, allow_newline: true, allow_curly: false }
    }
}

impl ExpressionContext {
    /// A type level expression
    pub fn in_type() -> Self {
        ExpressionContext { type_level: true, allow_newline: true, allow_curly: false }
    }
    /// A term level expression
    pub fn in_free() -> Self {
        ExpressionContext { type_level: false, allow_newline: true, allow_curly: true }
    }
    /// In a repl statement
    pub fn in_repl() -> Self {
        ExpressionContext { type_level: false, allow_newline: false, allow_curly: true }
    }
}

impl TypingExpression {
    /// Convert this node into a full expression node
    #[allow(clippy::wrong_self_convention)]
    pub fn as_normal(self) -> ExpressionNode {
        ExpressionNode { omit: true, body: self.body, span: self.span }
    }
}

impl ExpressionKind {
    /// Build a new binary expression
    pub fn binary(o: OperatorNode, lhs: Self, rhs: Self) -> Self {
        Self::Infix(Box::new(BinaryNode { infix: o, lhs, rhs }))
    }
    /// Build a new prefix expression
    pub fn prefix(o: OperatorNode, rhs: Self) -> Self {
        Self::Unary(Box::new(UnaryNode { operator: o, base: rhs }))
    }
    /// Build a new suffix expression
    pub fn suffix(o: OperatorNode, lhs: Self) -> Self {
        Self::Unary(Box::new(UnaryNode { operator: o, base: lhs }))
    }
}

impl ExpressionKind {
    /// Try to convert this expression into a text node
    pub fn as_text(&self) -> Option<&StringTextNode> {
        match self {
            ExpressionKind::Text(v) => Some(v),
            ExpressionKind::String(v) => match v.handler {
                None => Some(&v.literal),
                Some(_) => None,
            },
            _ => None,
        }
    }
}

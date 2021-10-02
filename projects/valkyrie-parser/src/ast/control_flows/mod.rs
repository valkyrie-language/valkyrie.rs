use crate::{DeclarationBody, PatternExpression, TermExpression};
use std::ops::Range;
use valkyrie_types::Identifier;

pub mod if_statements;
pub mod loop_statements;
pub mod until_statements;
pub mod while_statements;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContinueStatement {
    /// Optional continue label.
    pub label: Option<Identifier>,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BreakStatement {
    /// Optional break label.
    pub label: Option<Identifier>,
    /// Optional break value.
    pub value: Option<TermExpression>,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YieldStatement {
    /// Optional yielded value.
    pub value: Option<TermExpression>,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YieldFromStatement {
    /// Source expression delegated by `yield from`.
    pub value: TermExpression,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReturnStatement {
    /// Optional return value.
    pub value: Option<TermExpression>,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeStatement {
    /// Optional return value.
    pub value: Option<TermExpression>,
    /// Source span of the expression.
    pub span: Range<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallthroughStatement {
    /// 源码范围。
    pub span: Range<usize>,
}

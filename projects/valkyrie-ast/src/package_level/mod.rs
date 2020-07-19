pub mod classes;
mod dispatch;
pub mod function;
pub mod import;
pub mod license;
pub mod namespace;
use crate::{
    expression_level::ExpressionNode,
    package_level::{classes::ClassDeclarationNode, namespace::NamespaceDeclarationNode},
    ExpressionBody, ExpressionContext, ForLoopNode, FunctionDeclarationNode, IdentifierNode, ImportStatementNode, NamePathNode,
    WhileLoopNode,
};
use alloc::{boxed::Box, string::String, vec::Vec};
use core::{
    fmt::{Debug, Display, Formatter, Write},
    ops::Range,
};
use indentation::{IndentDisplay, IndentFormatter};

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StatementNode {
    pub r#type: StatementType,
    pub eos: bool,
    pub range: Range<usize>,
}

/// The top level elements in script mode.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StatementType {
    Nothing,
    Namespace(Box<NamespaceDeclarationNode>),
    Import(Box<ImportStatementNode>),
    Class(Box<ClassDeclarationNode>),
    Function(Box<FunctionDeclarationNode>),
    While(Box<WhileLoopNode>),
    For(Box<ForLoopNode>),
    Expression(Box<ExpressionNode<{ ExpressionContext::Term }>>),
}

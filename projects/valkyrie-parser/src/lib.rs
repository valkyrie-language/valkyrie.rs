#![doc = include_str!("readme.md")]
#![warn(missing_docs)]
#![feature(box_patterns)]

/// Parser-facing AST node family.
pub mod ast;
/// Lexical analysis entry points and token definitions.
pub mod lexer;
/// Source-to-AST parsing entry points.
pub mod parser;

pub use ast::{
    Annotations, AttributeArgument, AttributeDeclaration, AttributeItem, AttributeList, BinaryOperator, ClassDeclaration, DeclarationBody,
    FunctionDeclaration, FunctionParameter, FunctionStatement, GenericParameterDeclaration, ImplyAssociatedConstBinding,
    ImplyAssociatedTypeBinding, ImplyDeclaration, InheritanceItem, LetStatement, LiteralExpression, NamePath, NamespaceDeclaration, ObjectBody,
    ObjectFieldDeclaration, ObjectMethodDeclaration, PatternExpression, RootStatement, RowMethodTypeExpression, StringLiteral, StringSegment,
    SubscriptKind, TermExpression, TraitAssociatedConstDeclaration, TraitAssociatedTypeDeclaration, TraitDeclaration, TypeExpression, TypePath,
    UnaryOperator, UniteDeclaration, UniteVariantDeclaration, UsingStatement, ValkyrieRoot, WhereConstraintDeclaration,
};
pub use parser::{AstParser, ParseError};

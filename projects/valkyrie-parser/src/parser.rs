//! Valkyrie Language Parser
//!
//! This module provides a parser for the Valkyrie programming language
//! using the chumsky parser combinator library.

use chumsky::prelude::*;
// SimpleSpan is available through prelude
use nyar_ast::*;
use nyar_core::*;
use nyar_error::{errors::NyarErrorKind, NyarError, SourceSpan};

// Type alias for Span
use tracing::debug;

/// Main grammar parser for Valkyrie language
#[derive(Debug, Clone)]
pub struct ValkyrieGrammar {
    /// Enable error recovery during parsing
    pub error_recovery: bool,
}

impl Default for ValkyrieGrammar {
    fn default() -> Self {
        Self { error_recovery: true }
    }
}

impl ValkyrieGrammar {
    pub fn new(error_recovery: bool) -> Self {
        Self { error_recovery }
    }

    /// Parse source code into an AST
    pub fn parse(&self, source: &str, _file_id: FileId) -> Result<Program, Vec<NyarError>> {
        let parser = Self::build_parser();

        match parser.parse(source) {
            Ok(program) => {
                debug!("Successfully parsed program with {} statements", program.statements.len());
                Ok(program)
            }
            Err(errors) => {
                let diagnostics = errors
                    .into_iter()
                    .map(|e| {
                        let msg = format!("Parse error: {:?}", e);
                        NyarError::from(NyarErrorKind::SyntaxError { details: msg.clone() }, msg)
                    })
                    .collect();
                Err(diagnostics)
            }
        }
    }

    /// Build the main parser
    fn build_parser() -> impl Parser<char, Program, Error = Simple<char>> + Clone {
        let statement = Self::statement_parser();

        statement.repeated().then_ignore(end()).map_with_span(|statements, span| Program {
            statements,
            span: SourceSpan::new(span.start.into(), span.end - span.start),
        })
    }

    /// Parse an expression
    fn expression_parser() -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
        recursive(|expr| {
            let primary =
                choice((Self::literal_parser(), Self::identifier_parser(), expr.clone().delimited_by(just('('), just(')'))));

            let binary = primary
                .clone()
                .then(
                    choice((
                        just('+').to(BinaryOperator::Add),
                        just('-').to(BinaryOperator::Subtract),
                        just('*').to(BinaryOperator::Multiply),
                        just('/').to(BinaryOperator::Divide),
                        text::keyword("==").to(BinaryOperator::Equal),
                        text::keyword("!=").to(BinaryOperator::NotEqual),
                        just('<').to(BinaryOperator::Less),
                        just('>').to(BinaryOperator::Greater),
                        text::keyword("<=").to(BinaryOperator::LessEqual),
                        text::keyword(">=").to(BinaryOperator::GreaterEqual),
                    ))
                    .then(expr)
                    .repeated(),
                )
                .foldl(|left, (op, right)| {
                    Expression::Binary(BinaryExpression {
                        left: Box::new(left),
                        operator: op,
                        right: Box::new(right),
                        span: SourceSpan::new(0.into(), 0),
                    })
                });

            let unary = choice((just('-').to(UnaryOperator::Minus), just('!').to(UnaryOperator::Not)))
                .repeated()
                .then(binary)
                .foldr(|op, expr| {
                    Expression::Unary(UnaryExpression {
                        operator: op,
                        operand: Box::new(expr),
                        span: SourceSpan::new(0.into(), 0),
                    })
                });

            unary
        })
    }

    /// Parse a statement
    fn statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        recursive(|_stmt| {
            choice((
                Self::variable_declaration_parser(),
                Self::function_declaration_parser(),
                Self::if_statement_parser(),
                Self::while_statement_parser(),
                Self::break_statement_parser(),
                Self::continue_statement_parser(),
                Self::return_statement_parser(),
                Self::expression_statement_parser(),
            ))
        })
    }

    /// Parse literals
    fn literal_parser() -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
        choice((
            text::int(10).map(|s: String| s.parse().unwrap()).map_with_span(|n, span: std::ops::Range<usize>| {
                Expression::Literal(LiteralExpression {
                    value: Literal::Integer(n),
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            }),
            text::int(10)
                .chain::<char, _, _>(just('.'))
                .chain::<char, String, _>(text::digits(10))
                .collect::<String>()
                .map(|s| s.parse().unwrap_or(0.0))
                .map_with_span(|f, span: std::ops::Range<usize>| {
                    Expression::Literal(LiteralExpression {
                        value: Literal::Float(f),
                        span: SourceSpan::new(span.start.into(), span.end - span.start),
                    })
                }),
            just('"').ignore_then(filter(|c| *c != '"').repeated()).then_ignore(just('"')).collect::<String>().map_with_span(
                |s, span: std::ops::Range<usize>| {
                    Expression::Literal(LiteralExpression {
                        value: Literal::String(s),
                        span: SourceSpan::new(span.start.into(), span.end - span.start),
                    })
                },
            ),
            text::keyword("true").map_with_span(|_, span: std::ops::Range<usize>| {
                Expression::Literal(LiteralExpression {
                    value: Literal::Boolean(true),
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            }),
            text::keyword("false").map_with_span(|_, span: std::ops::Range<usize>| {
                Expression::Literal(LiteralExpression {
                    value: Literal::Boolean(false),
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            }),
            text::keyword("null").map_with_span(|_, span: std::ops::Range<usize>| {
                Expression::Literal(LiteralExpression {
                    value: Literal::Null,
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            }),
        ))
    }

    /// Parse identifiers
    fn identifier_parser() -> impl Parser<char, Expression, Error = Simple<char>> + Clone {
        text::ident().map_with_span(|name, span: std::ops::Range<usize>| {
            Expression::Identifier(IdentifierExpression {
                name,
                span: SourceSpan::new(span.start.into(), span.end - span.start),
            })
        })
    }

    /// Parse type expressions
    fn type_expression_parser() -> impl Parser<char, TypeExpression, Error = Simple<char>> + Clone {
        text::ident().map_with_span(|name, span: std::ops::Range<usize>| {
            TypeExpression::Named(NamedType { name, span: SourceSpan::new(span.start.into(), span.end - span.start) })
        })
    }

    /// Parse variable declaration: let [mut] name [: type] [= value];
    fn variable_declaration_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        text::keyword("let")
            .ignore_then(text::keyword("mut").or_not().map(|m| m.is_some()))
            .then(text::ident())
            .then(just(':').ignore_then(Self::type_expression_parser()).or_not())
            .then(just('=').ignore_then(Self::expression_parser()).or_not())
            .then_ignore(just(';'))
            .map_with_span(|(((mutable, name), type_annotation), initializer), span| {
                Statement::VariableDeclaration(VariableDeclaration {
                    name,
                    mutable,
                    type_annotation,
                    initializer,
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            })
    }

    /// Parse expression statements
    fn expression_statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        Self::expression_parser().then_ignore(just(';').or_not()).map_with_span(|expression, span| {
            Statement::Expression(ExpressionStatement {
                expression,
                span: SourceSpan::new(span.start.into(), span.end - span.start),
            })
        })
    }

    /// Parse function declarations
    fn function_declaration_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        text::keyword("fn")
            .ignore_then(text::ident())
            .then_ignore(just('('))
            .then(
                text::ident()
                    .then_ignore(just(':'))
                    .then(Self::type_expression_parser())
                    .map_with_span(|(name, type_annotation), span| Parameter {
                        name,
                        type_annotation,
                        span: SourceSpan::new(span.start.into(), span.end - span.start),
                    })
                    .separated_by(just(',')),
            )
            .then_ignore(just(')'))
            .then(just(':').ignore_then(Self::type_expression_parser()).or_not())
            .then(Self::block_statement_parser())
            .map_with_span(|(((name, parameters), return_type), body), span| {
                Statement::FunctionDeclaration(FunctionDeclaration {
                    name,
                    parameters,
                    return_type,
                    body,
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            })
    }

    /// Parse if statements
    fn if_statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        recursive(|if_stmt| {
            text::keyword("if")
                .ignore_then(Self::expression_parser())
                .then(Self::block_statement_parser())
                .then(
                    text::keyword("else")
                        .ignore_then(choice((
                            if_stmt.map(Box::new),
                            Self::block_statement_parser().map(|block| Box::new(Statement::Block(block))),
                        )))
                        .or_not(),
                )
                .map_with_span(|((condition, then_branch), else_branch), span| {
                    Statement::If(IfStatement {
                        condition,
                        then_branch,
                        else_branch,
                        span: SourceSpan::new(span.start.into(), span.end - span.start),
                    })
                })
        })
    }

    /// Parse while statements
    fn while_statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        text::keyword("while").ignore_then(Self::expression_parser()).then(Self::block_statement_parser()).map_with_span(
            |(condition, body), span| {
                Statement::While(WhileStatement {
                    condition,
                    body,
                    span: SourceSpan::new(span.start.into(), span.end - span.start),
                })
            },
        )
    }

    /// Parse return statements
    fn return_statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        text::keyword("return").ignore_then(Self::expression_parser().or_not()).then_ignore(just(';').or_not()).map_with_span(
            |value, span| {
                Statement::Return(ReturnStatement { value, span: SourceSpan::new(span.start.into(), span.end - span.start) })
            },
        )
    }

    /// Parse block statements
    fn block_statement_parser() -> impl Parser<char, BlockStatement, Error = Simple<char>> + Clone {
        Self::statement_parser().repeated().delimited_by(just('{'), just('}')).map_with_span(|statements, span| {
            BlockStatement { statements, span: SourceSpan::new(span.start.into(), span.end - span.start) }
        })
    }

    /// Parse break statements
    fn break_statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        text::keyword("break").then_ignore(just(';').or_not()).map_with_span(|_, span: std::ops::Range<usize>| {
            Statement::Break(BreakStatement { span: SourceSpan::new(span.start.into(), span.end - span.start) })
        })
    }

    /// Parse continue statements
    fn continue_statement_parser() -> impl Parser<char, Statement, Error = Simple<char>> + Clone {
        text::keyword("continue").then_ignore(just(';').or_not()).map_with_span(|_, span: std::ops::Range<usize>| {
            Statement::Continue(ContinueStatement { span: SourceSpan::new(span.start.into(), span.end - span.start) })
        })
    }
}

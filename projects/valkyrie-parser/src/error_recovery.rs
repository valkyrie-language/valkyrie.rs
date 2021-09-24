//! Error recovery strategies for the Valkyrie parser
//!
//! This module provides error recovery mechanisms for graceful parsing
//! when encountering syntax errors.

use chumsky::prelude::*;
use nyar_error::{errors::NyarErrorKind, NyarError};
use std::collections::HashSet;

/// Recovery strategy for different error contexts
#[derive(Debug, Clone, PartialEq)]
pub enum RecoveryStrategy {
    /// Skip to the next synchronization point
    SkipToSync,
    /// Insert a missing token
    InsertToken(char),
    /// Replace current token with expected
    ReplaceToken(char),
    /// Continue parsing with assumption
    Continue,
}

/// Error recovery context
#[derive(Debug, Clone)]
pub struct RecoveryContext {
    /// Current parsing context
    pub context: ParsingContext,
    /// Expected characters at this point
    pub expected: HashSet<char>,
    /// Current position in input
    pub position: usize,
}

/// Parsing context for error recovery
#[derive(Debug, Clone, PartialEq)]
pub enum ParsingContext {
    /// Top-level program
    Program,
    /// Statement context
    Statement,
    /// Expression context
    Expression,
    /// Function declaration
    FunctionDeclaration,
    /// Variable declaration
    VariableDeclaration,
    /// Block statement
    Block,
    /// Parameter list
    ParameterList,
    /// Argument list
    ArgumentList,
}

impl RecoveryContext {
    /// Create a new recovery context
    pub fn new(context: ParsingContext) -> Self {
        Self { context, expected: HashSet::new(), position: 0 }
    }

    /// Add expected character
    pub fn expect(&mut self, ch: char) {
        self.expected.insert(ch);
    }

    /// Determine recovery strategy based on context and error
    pub fn recovery_strategy(&self, found: Option<char>) -> RecoveryStrategy {
        match (&self.context, found) {
            // In expression context, try to continue or skip
            (ParsingContext::Expression, Some(';')) => RecoveryStrategy::SkipToSync,
            (ParsingContext::Expression, Some('}')) => RecoveryStrategy::SkipToSync,

            // In statement context, skip to next statement
            (ParsingContext::Statement, Some(';')) => RecoveryStrategy::Continue,
            (ParsingContext::Statement, _) => RecoveryStrategy::SkipToSync,

            // In function declaration, skip to end of function
            (ParsingContext::FunctionDeclaration, Some('}')) => RecoveryStrategy::Continue,
            (ParsingContext::FunctionDeclaration, _) => RecoveryStrategy::SkipToSync,

            // In block context, skip to next statement or end of block
            (ParsingContext::Block, Some('}')) => RecoveryStrategy::Continue,
            (ParsingContext::Block, _) => RecoveryStrategy::SkipToSync,

            // Default: skip to synchronization point
            _ => RecoveryStrategy::SkipToSync,
        }
    }
}

/// Synchronization points for error recovery
pub fn sync_points() -> HashSet<char> {
    let mut points = HashSet::new();
    points.insert(';');
    points.insert('{');
    points.insert('}');
    points.insert('\n');
    points
}

/// Create a recovery parser that skips to synchronization points
pub fn recovery_parser() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    let sync_chars = sync_points();

    filter(move |c: &char| !sync_chars.contains(c)).repeated().ignored()
}

/// Create an error recovery parser for statements
pub fn statement_recovery() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    choice((just(';').ignored(), just('}').ignored(), just('\n').ignored(), recovery_parser()))
}

/// Create an error recovery parser for expressions
pub fn expression_recovery() -> impl Parser<char, (), Error = Simple<char>> + Clone {
    choice((just(';').ignored(), just(',').ignored(), just(')').ignored(), just('}').ignored(), recovery_parser()))
}

/// Convert parser errors to compiler diagnostics
pub fn convert_parse_errors(errors: Vec<Simple<char>>) -> Vec<NyarError> {
    errors
        .into_iter()
        .map(|error| {
            let message = match error.reason() {
                chumsky::error::SimpleReason::Unexpected => {
                    format!("Unexpected character at position {}", error.span().start)
                }
                chumsky::error::SimpleReason::Unclosed { span, delimiter } => {
                    format!("Unclosed delimiter '{}' opened at position {}", delimiter, span.start)
                }
                chumsky::error::SimpleReason::Custom(msg) => msg.clone(),
            };

            NyarError::new(NyarErrorKind::SyntaxError { details: message.clone() }, message)
        })
        .collect()
}

//! Valkyrie Language Parser
//!
//! This crate provides a parser for the Valkyrie programming language,
//! built on top of the nyar platform using the chumsky parsing library.

use nyar_ast::*;
use nyar_core::FileId;
use nyar_error::{errors::NyarErrorKind, NyarError, SourceSpan};
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};
use tracing::{debug, trace};

pub mod error_recovery;
pub mod parser;

pub use error_recovery::*;
pub use parser::*;

/// Main parser interface for Valkyrie language
#[derive(Debug, Clone)]
pub struct ValkyrieParser {
    /// Enable error recovery during parsing
    pub error_recovery: bool,
    /// Enable incremental parsing optimizations
    pub incremental: bool,
}

impl Default for ValkyrieParser {
    fn default() -> Self {
        Self { error_recovery: true, incremental: false }
    }
}

impl ValkyrieParser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable or disable error recovery
    pub fn with_error_recovery(mut self, enabled: bool) -> Self {
        self.error_recovery = enabled;
        self
    }

    /// Enable or disable incremental parsing
    pub fn with_incremental(mut self, enabled: bool) -> Self {
        self.incremental = enabled;
        self
    }

    /// Parse source code into an AST
    pub fn parse(&self, source: &str, file_id: FileId) -> Result<Program, Vec<NyarError>> {
        trace!("Starting parse for file {:?}", file_id);

        // Parse source code directly using chumsky parser combinators
        let parser = ValkyrieGrammar::new(self.error_recovery);
        let ast = parser.parse(source, file_id)?;
        debug!("Successfully parsed AST with {} statements", ast.statements.len());

        Ok(ast)
    }

    // Tokenization is no longer needed - parsing directly from source
}

/// Parse a string into a Valkyrie AST
pub fn parse_string(source: &str) -> Result<Program, Vec<NyarError>> {
    let file_id = FileId::from(0); // Use dummy file ID for string parsing
    ValkyrieParser::new().parse(source, file_id)
}

/// Parse a file into a Valkyrie AST
pub fn parse_file(path: &std::path::Path) -> Result<Program, Vec<NyarError>> {
    let source = std::fs::read_to_string(path).map_err(|e| {
        vec![NyarError::new(
            NyarErrorKind::IoError { details: format!("Failed to read file: {}", e) },
            format!("Failed to read file: {}", e),
        )]
    })?;

    let mut hasher = DefaultHasher::new();
    path.to_string_lossy().as_ref().hash(&mut hasher);
    let file_id = FileId::from(hasher.finish() as u32);
    ValkyrieParser::new().parse(&source, file_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_program() {
        let result = parse_string("");
        assert!(result.is_ok());
        let program = result.unwrap();
        assert!(program.statements.is_empty());
    }

    #[test]
    fn test_simple_expression() {
        let result = parse_string("42");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parser_creation() {
        let parser = ValkyrieParser::new().with_error_recovery(true).with_incremental(false);

        assert!(parser.error_recovery);
        assert!(!parser.incremental);
    }
}

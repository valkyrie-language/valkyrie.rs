#![doc = include_str!("readme.md")]

/// `AST -> HIR -> MIR -> LIR` compiler pipeline entry points.
pub mod ast_to_hir;

pub use ast_to_hir::{AstToHir, CaptureAnalyzer, ValkyrieCompiler};

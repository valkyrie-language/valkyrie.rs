//! `valkyrie-compiler` 的 `HIR` 入口。
//!
//! 语义归属仍然在编译器主链中，这里只把实际承载类型
//! 收口到一个稳定入口，避免上层直接散落依赖 `valkyrie_types::hir::*`。
//! 控制流统一设计见 `control_flow.next.md`。

/// `AST -> HIR` 前的结构预校验。
pub mod ast_validation;
/// 闭包捕获分析。
pub mod capture_analysis;
/// `HIR` 控制流语义校验。
pub mod control_flow_validation;
/// 语义诊断工具。
pub mod diagnostics;
/// `AST -> HIR` lowering 与编译 facade。
pub mod lowering;
/// 名义类型相关能力。
pub mod nominal;
/// 重载解析相关能力。
pub mod overload;
/// Row 类型相关能力。
pub mod row;
/// Trait 系统相关能力。
pub mod trait_system;
/// `AST` 类型到 `HIR` 类型的 lowering。
pub mod type_lowering;
/// 统一类型判定入口。
pub mod type_relation;

pub(crate) use ast_validation::validate_ast_root;
pub use capture_analysis::CaptureAnalyzer;
pub use control_flow_validation::validate_control_flow_module;
pub use lowering::{AstToHir, ValkyrieCompiler};
pub(crate) use type_lowering::{lower_type_expression, render_type_expression, validate_type_expression, BuiltinTypeAliasScope};
pub use valkyrie_types::hir::*;

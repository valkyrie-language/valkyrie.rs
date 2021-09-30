//! `valkyrie-compiler` 的 `HIR` 入口。
//!
//! 语义归属仍然在编译器主链中，这里只把实际承载类型
//! 收口到一个稳定入口，避免上层直接散落依赖 `valkyrie_types::hir::*`。

pub mod diagnostics;
pub mod nominal;
pub mod overload;
pub mod row;
pub mod trait_system;

pub use valkyrie_types::hir::*;

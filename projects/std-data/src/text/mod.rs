//! `std-data` 文本格式：统一为 **lexer → parser → 模型**，不含文本格式化输出。
//! 格式化职责在 [`nyar_language::text`]。

/// `Bash` 文本格式模型。
pub mod bash;

/// `C` 文本格式模型（legend / legacy-vm 子集）。
pub mod c;

/// `Lua` 文本格式模型。
pub mod lua;

/// `MSIL` 文本格式模型与解析能力。
pub mod msil;

/// `PowerShell` 文本格式模型。
pub mod powershell;

/// `Tcl` 文本格式模型。
pub mod tcl;

/// AWSL 模板源码的词法、语法与 `AST` facade。
pub mod awsl;

/// Notedown 统一文档 IR（Valkyrie 文档语义）。
pub mod notedown;

/// Markdown 文档 AST 与 Notedown 桥接。
pub mod markdown;

/// Valkyrie 源码文本的词法、语法与 `AST` facade。
pub mod valkyrie;

/// `VON` 文本格式模型、解析器与 `serde` 支持。
pub mod von;

/// `WAT` 文本格式模型。
pub mod wat;

/// `WIT` 文本格式模型。
pub mod wit;

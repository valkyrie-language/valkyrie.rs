//! `vcc-data` 文本格式：二进制 spy 与 MSIL 等工具向格式。
//! 语言 parse/format 权威在 `oak-*` 与 `nyar-language`，不在此 crate 重复实现。

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

/// Notedown 统一文档 IR（Valkyrie 文档语义）。
pub mod notedown;
/// Markdown 文档 AST 与 Notedown 桥接。
pub mod markdown;

/// Valkyrie 源码文本 facade（重导出 `oak-valkyrie`）。
pub mod valkyrie;

/// `WAT` 文本格式模型。
pub mod wat;
/// `WIT` 文本格式模型。
pub mod wit;

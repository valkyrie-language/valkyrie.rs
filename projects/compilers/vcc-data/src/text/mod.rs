//! `vcc-data` 文本格式：MSIL 等与二进制 spy / PE 打包耦合的工具向格式。
//! 语言 parse/format 权威在 `oak-*` 与 `nyar-language`，不在此 crate 重复实现。

/// `MSIL` 文本格式模型与解析能力。
pub mod msil;

//! 标准格式编解码层。
//!
//! 这里只承载二进制与文本格式的模型、编码器、解码器与解析器，
//! 不承载编译流程编排、语义分析或后端执行职责。

#![feature(box_patterns)]
#![warn(missing_docs)]

pub mod binary;
/// Hermes schema / query frontend (Atlas default truth source).
pub mod hermes;
/// SQL AST + dialect printer (SQL **materialization** for Atlas data access).
pub mod sql;
pub mod text;

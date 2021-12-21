//! 二进制与数据平面格式编解码层。
//!
//! 承载 PE / WASM / DEX / MSIL 等工具链格式，以及 Hermes / SQL 数据访问前端。
//! 语言 parse/format 权威在 `oak-*` 与 `nyar-language`，不在此 crate 重复实现。

#![warn(missing_docs)]

pub mod binary;
/// Hermes schema / query frontend (Atlas default truth source).
pub mod hermes;
/// SQL AST + dialect printer (SQL **materialization** for Atlas data access).
pub mod sql;
pub mod text;

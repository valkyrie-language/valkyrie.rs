//! 共享二进制格式层。
//!
//! 这里只承载可跨后端复用的容器模型，避免把具体编码器实现耦合到某个后端 crate。

#![warn(missing_docs)]

pub mod binary;
pub mod text;

pub use self::binary::coff::{CoffHeader, CoffMachine, CoffObject, CoffObjectWriter, CoffRelocation, CoffRelocationKind, CoffSection, CoffSymbol};

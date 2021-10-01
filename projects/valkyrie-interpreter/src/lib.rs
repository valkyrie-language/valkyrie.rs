#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod wasi;

pub use wasi::WasiRuntime;

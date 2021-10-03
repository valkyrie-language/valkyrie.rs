//! 最小 ELF64 可执行镜像（ET_EXEC，x86-64）与 ET_DYN 共享库（AArch64）。

mod builder;
mod layout;
mod read;
mod shared_writer;
mod writer;

pub use builder::NativeElfImageBuilder;
pub use read::{Elf64Header, Elf64ParseError, parse_elf64};
pub use shared_writer::{SharedElfImage, SharedElfWriter, SharedObjectExport};
pub use writer::{NativeElfImage, NativeElfWriter};

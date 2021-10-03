#![doc = include_str!("readme.md")]

mod abi_msvc;
mod abi_sysv;
mod encode;
mod instruction;
mod layout;
mod reg;

pub use abi_msvc::{MsvcFunctionBuilder, SHADOW_STACK_SIZE};
pub use abi_sysv::SysvFunctionBuilder;
pub use encode::{EncodedModule, X64Encoder, X64Fixup, X64FixupKind, apply_fixups};
pub use instruction::{ConditionCode, X64Instruction};
pub use layout::ObjectLayout;
pub use reg::Reg64;

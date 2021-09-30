#![doc = include_str!("readme.md")]

/// `SSA`-based `MIR` main representation.
pub mod ssa;

pub use ssa::{
    MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirField, MirFunction, MirInstruction, MirInstructionKind, MirLowerer, MirModule,
    MirOperand, MirStruct, MirTerminator, MirValue, MirValueOrigin, MirValueRef,
};

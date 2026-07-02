//! Driver-side backend executable contracts (backend-private).
//!
//! **Rule**: backend lowering code must depend on driver-owned executable views and helpers,
//! not on `nyar-language` MIR types.

pub use nyar_types::{
    Block, BlockRef, CarrierTable, CaseArm, CaseChain, Constant, Continuation, Diagnostic, DispatchKind, EffectKind, ExecutableFunction,
    FrameLayout, FrameSlot, Instruction, InstructionKind, IntrinsicBinaryOp, IntrinsicBitwiseOp, IntrinsicCompareOp, IntrinsicOpcode, Operand,
    ReceiverPassingKind, StorageKind, SuspendLoweringPlan, SuspendPoint, SuspendState, Terminator, Value, ValueOrigin, ValueRef,
};

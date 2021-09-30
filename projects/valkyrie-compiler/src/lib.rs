#![doc = include_str!("readme.md")]
#![warn(missing_docs)]
#![feature(new_range_api)]

pub mod backend_boundary;
pub mod derive;
pub mod hir;
pub mod lir;
pub mod mir;
pub mod module;
pub mod pipeline;
pub mod type_checker;
/// Typing helpers such as linearization and semantic inheritance analysis.
pub mod typing;

pub use hir::*;
pub use lir::{
    lower_dispatch_kind, LirBlock, LirDispatchKind, LirFunction, LirLowerer, LirModule, LirOperand, LirOperation, LirOperationKind,
    LirTargetLane, LirTerminator,
};
pub use mir::{
    MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirFunction, MirInstruction, MirInstructionKind, MirLowerer, MirModule, MirOperand,
    MirTerminator, MirValue, MirValueOrigin, MirValueRef,
};
pub use pipeline::{AstToHir, CaptureAnalyzer, ValkyrieCompiler};

#![doc = include_str!("readme.md")]
#![warn(missing_docs)]
#![feature(new_range_api)]

pub mod backend_boundary;
pub mod derive;
pub mod hir;
pub mod lir;
pub mod mir;
pub mod module;
pub mod nyar_backend_bridge;
pub mod nyar_bridge;
pub mod type_checker;
/// Typing helpers such as linearization and semantic inheritance analysis.
pub mod typing;
/// 跨 `HIR / MIR / LIR` 的编译器一致性校验入口。
pub mod validation;

pub use hir::{CaptureAnalyzer, *};
pub use lir::{
    lower_dispatch_kind, lower_effect_kind, LirBlock, LirDispatchKind, LirEffectKind, LirFunction, LirLowerer, LirModule, LirOperand,
    LirOperation, LirOperationKind, LirTargetLane, LirTerminator,
};
pub use mir::{
    MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind, MirLowerer, MirModule,
    MirOperand, MirTerminator, MirValue, MirValueOrigin, MirValueRef,
};
pub use nyar::{
    self, ArtifactKind, ArtifactPartitionPlan, ArtifactPolicy, ArtifactSet, CanonicalAbi, CanonicalArch, CanonicalSpecification,
    CanonicalTarget, CanonicalTargetParseError, CanonicalVendor, CompilationOptions, EntryPolicy, PlanningInput, ProgramFacts, PublishFormat,
    RunnerFamily, RunnerSelector, TargetBackendFamily, TargetHostKind, TargetMode, TargetProfile, WrapStrategy,
};
pub use nyar_backend_bridge::{
    collect_wasm_imports, lower_lir_to_jvm_class, lower_lir_to_msil, lower_lir_to_native_assembly, lower_lir_to_wasm_module,
    lower_to_driver_input, resolve_wasm_import, write_clr_msil_sidecar, ClrLirLoweringLane, JvmLirLoweringLane, NativeLirLoweringLane,
    WasmImport, WasmLirLoweringLane,
};
pub use nyar_bridge::{
    hir_module_to_analysis_artifact, hir_module_to_artifact_plan, hir_module_to_object_algebraic_program, hir_module_to_program_facts,
};
pub use validation::ControlFlowScheduler;

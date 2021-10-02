#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod abstractions;
pub mod backends;
pub mod lanes;
pub mod packaging;
pub mod planning;
pub mod selection;
pub mod target_profile;

pub use self::{
    abstractions::{
        ArtifactFormat, ArtifactKind, BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, ByteOrder, CanonicalAbi, CanonicalArch,
        CanonicalSpecification, CanonicalTarget, CanonicalTargetParseError, CanonicalVendor, ObjectKind, TargetFamily,
    },
    backends::{clr::ClrImageKind, BackendDescriptor, CompilationOptions, TargetCodeGenBackend},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::{ArtifactDescriptor, ArtifactSet, OutputSpec, TargetLane},
    planning::{ArtifactPartition, ArtifactPartitionPlan, PartitionBackendRequirement, PlanningInput},
    selection::{BackendCandidate, BackendSelector},
    target_profile::{
        ArtifactPolicy, EntryPolicy, PublishFormat, RunnerFamily, RunnerSelector, TargetBackendFamily, TargetHostKind, TargetMode,
        TargetProfile, WrapStrategy,
    },
};
pub use nyar_analyzer::{EntryContract, ExportContract, FunctionAnalysis, ImportContract, ProgramFacts, RuntimeRequirement};
pub use nyar_optimizer::{
    EGraphSnapshot, FutamuraProjectionFamily, HostProjectionBoundary, ObjectAlgebraicBuilder, ObjectAlgebraicDimension,
    ObjectAlgebraicInterpreter, ObjectAlgebraicProgram, OptimizationRequest, OptimizationResult, OptimizationSession, ProjectionPlan,
    ProjectionPolicy, ReferenceManagement, RewritePhase, RewriteRule, RewriteTheory,
};
pub use nyar_types::{CapabilityTag, Identifier, NamePath, QualifiedName, SymbolIdentity};

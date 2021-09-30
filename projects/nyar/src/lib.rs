#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod abstractions;
pub mod backends;
pub mod lanes;
pub mod packaging;
pub mod selection;
pub mod target_profile;

pub use abstractions::{
    ArtifactFormat, ArtifactKind, BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, ByteOrder, CanonicalAbi, CanonicalArch,
    CanonicalSpecification, CanonicalTarget, CanonicalTargetParseError, CanonicalVendor, ObjectKind, TargetFamily,
};
pub use backends::{clr::ClrImageKind, BackendDescriptor, CompilationOptions, TargetCodeGenBackend};
pub use lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor};
pub use packaging::{ArtifactDescriptor, ArtifactSet, OutputSpec, TargetLane};
pub use selection::{BackendCandidate, BackendSelector};
pub use target_profile::{
    ArtifactPolicy, EntryPolicy, PublishFormat, RunnerFamily, RunnerSelector, TargetBackendFamily, TargetHostKind, TargetMode, TargetProfile,
    WrapStrategy,
};

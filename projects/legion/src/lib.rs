#![warn(missing_docs)]

pub mod cmds;
pub mod manifest;
pub mod planner;

pub use cmds::spy::{run as run_spy, SpyMode, SpyOptions, SpyTargetOptions, SpyTargetPlatform};
pub use manifest::{
    AutoLinkConfig, BuildTargetSpec, DependencySpec, ProjectManifest, PublishTargetSpec, RunnerBinding, WorkspaceDefaults, WorkspaceManifest,
};
pub use planner::{BuildPlan, BuildRequest, LegionWorkspace, PlannedDependency, PlannedHostContract, PlannedHostProvider, PlannedProject};
pub use valkyrie_compiler::{
    ArtifactPolicy, CanonicalAbi, CanonicalArch, CanonicalSpecification, CanonicalTarget, CanonicalTargetParseError, CanonicalVendor,
    EntryPolicy, PublishFormat, RunnerFamily, RunnerSelector, TargetBackendFamily, TargetHostKind, TargetMode, TargetProfile, WrapStrategy,
};
pub use von_parser::{
    from_str as parse_von, from_value as parse_von_value, to_string as write_von, to_string_pretty as write_von_pretty, VonError,
    VonParseError, VonParser, VonValue,
};

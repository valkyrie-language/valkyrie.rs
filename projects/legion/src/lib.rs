#![warn(missing_docs)]

pub mod artifact_formats;
pub mod cache;
pub mod cmds;
pub mod forward;
pub mod manifest;
pub mod planner;
pub mod unity_export;

pub use artifact_formats::{artifact_format_from_extension, artifact_format_slug, artifact_formats_for_publish_format};
pub use cmds::{
    doc::generate_for_project,
    spy::{SpyMode, SpyOptions, SpyTargetOptions, SpyTargetPlatform, run as run_spy},
};
pub use manifest::{
    AutoLinkConfig, BuildPluginSpec, BuildTargetSpec, DependencySpec, ProjectManifest, PublishTargetSpec, RunnerBinding, WorkspaceDefaults,
    WorkspaceManifest,
};

/// On-disk layout for Legion projects (`legion.von` / `legions.von` / `legion-lock.von` / `~/.valkyrie`).
pub const LEGION_PROJECT_LAYOUT: nyar_package_manager::ProjectLayout = nyar_package_manager::ProjectLayout {
    package_manifest: "legion.von",
    workspace_manifest: "legions.von",
    ignore_file: "legion.ignore",
    lockfile: "legion-lock.von",
    home_dirname: ".valkyrie",
    home_env: "VALKYRIE_HOME",
    token_env_vars: &["VALKYRIE_TOKEN", "LEGION_TOKEN"],
    entry_aliases: bootstrap_entry_aliases,
};

/// Node bootstrap / npm publish: `legion.mjs` is canonical.
///
/// Accept historical `legion_legion.*` and multi-partition `legion__main_legion.*`.
pub fn bootstrap_entry_aliases(physical_entry: &str) -> &'static [&'static str] {
    match physical_entry {
        "legion_legion.mjs" | "legion__main_legion.mjs" => &["legion.mjs"],
        "legion_legion.wasm" | "legion__main_legion.wasm" => &["legion.wasm"],
        _ => &[],
    }
}

pub use nyar_language::{
    ArtifactPolicy, CanonicalAbi, CanonicalArch, CanonicalSpecification, CanonicalTarget, CanonicalTargetParseError, CanonicalVendor,
    EntryPolicy, PublishFormat, RunnerFamily, RunnerSelector, TargetHostKind, TargetMode, TargetProfile, WrapStrategy,
    formatter::{to_string as write_von, to_string_indented as write_von_indented},
};
pub use planner::{
    BuildPlan, BuildRequest, LegionWorkspace, PlannedDependency, PlannedHostContract, PlannedHostProvider, PlannedProject,
    PlannedSemanticSourceGroup, collect_project_v_files, collect_test_build_sources, collect_test_v_files,
};
pub use std_data::text::von::{VonError, VonParseError, VonParser, VonValue, from_str as parse_von, from_value as parse_von_value};

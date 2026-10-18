//! Migrated from `src/planner.rs` (`exclusion_contract_tests`).





use legion_workspace::planner::*;
use std::fs;
use std::path::Path;
use legion_workspace::manifest::BuildTargetSpec;












#[test]
fn collect_source_files_includes_root_v_without_manifest_entry() {
let directory = tempfile::tempdir().expect("project directory");
fs::write(directory.path().join("solution.v"), "[export] micro solve() -> unit {}").expect("solution");
let files = collect_source_files(directory.path()).expect("collect root sources");
assert_eq!(files, vec![directory.path().join("solution.v")]);
}

#[test]
fn exclusions_use_normalized_project_relative_identity() {
let directory = tempfile::tempdir().expect("project directory");
fs::create_dir_all(directory.path().join("source/nested/source")).expect("sources");
let excluded = directory.path().join("source/skip.v");
let retained = directory.path().join("source/nested/source/skip.v");
fs::write(&excluded, "").expect("source");
fs::write(&retained, "").expect("source");
let target = BuildTargetSpec { exclude_files: vec!["source/skip.v".to_owned()], ..BuildTargetSpec::default() };
let files = vec![path_for_local_fs(&excluded), path_for_local_fs(&retained)];
let result = filter_excluded_sources(files, &canonicalize_lossy(directory.path()), &target);
assert_eq!(result, vec![path_for_local_fs(&retained)]);
}

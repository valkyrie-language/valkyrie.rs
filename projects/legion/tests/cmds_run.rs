mod support;

use legion::{
    CanonicalTarget,
    cmds::{
        build::{BuildArgs, run as run_build},
        run::{RunArgs, run as run_project},
    },
};
use std::{
    env,
    path::{Path, PathBuf},
    process::ExitCode,
};
use support::{
    create_local_package_project,
    run_fixture::{
        RunFixtureContext, can_run_all_runtime_targets, can_run_fixture_context, collect_run_fixture_cases, verify_run_fixture,
        verify_run_fixture_for_context,
    },
};

#[test]
fn runs_declared_target_fixtures() {
    let fixtures_root = run_fixture_root();
    let fixtures = collect_run_fixture_cases(&fixtures_root)
        .into_iter()
        .filter(|fixture| !explicit_interop_fixture_paths().contains(fixture))
        .collect::<Vec<_>>();
    assert!(!fixtures.is_empty(), "no run fixtures found under '{}'", fixtures_root.display());

    if !can_run_all_runtime_targets(&fixtures) {
        return;
    }

    for fixture in &fixtures {
        verify_run_fixture(fixture);
    }
}

#[test]
fn runs_jvm_interop_fixture() {
    let context = RunFixtureContext::new(&["jvm"]);
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("jvm_interop_stdout.valkyrie"), &context);
}

#[test]
fn runs_wasm_interop_fixture() {
    let context = RunFixtureContext::new(&["node"]);
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("wasm_interop_stdout.valkyrie"), &context);
}

#[test]
fn runs_wasi_component_clock_fixture() {
    let context = RunFixtureContext::new(&["wasi"]);
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("wasi_component_clock.valkyrie"), &context);
}

#[test]
fn runs_multiple_main_stdout_fixture() {
    let context = RunFixtureContext::new(&["node"]);
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("multiple_main_stdout.valkyrie"), &context);
}

#[test]
fn runs_clr_local_package_fixture() {
    let context = RunFixtureContext::new(&["clr"]).local_package().with_manifest(
        r#"{
    name: "local-package-minimal",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        }
    ]
}
"#,
    );
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("local_package_minimal.valkyrie"), &context);
}

#[test]
fn runs_clr_script_fixture() {
    let context = RunFixtureContext::new(&["clr"]).script().with_manifest(
        r#"{
    name: "script-minimal",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        }
    ]
}
"#,
    );
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("script_minimal.valkyrie"), &context);
}

#[test]
fn runs_clr_nested_workspace_fixture() {
    let context = RunFixtureContext::new(&["clr"]).nested_workspace_member().with_manifest(
        r#"{
    name: "nested-workspace-minimal",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        }
    ]
}
"#,
    );
    if !can_run_fixture_context(&context) {
        return;
    }

    verify_run_fixture_for_context(&run_fixture_root().join("nested_workspace_minimal.valkyrie"), &context);
}

#[test]
fn falls_back_to_local_package_when_unregistered() {
    let fixture = create_local_package_project(
        "legion-run-local-package",
        r#"{
            target: "clr",
            msil: true
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("local-package");

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();
    assert_eq!(build_status, ExitCode::SUCCESS);

    let run_status = run_project(&RunArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        runner: Vec::new(),
        artifact: None,
        dry_run: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
}

fn run_fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").join("fixtures").join("cmds_run")
}

fn explicit_interop_fixture_paths() -> Vec<PathBuf> {
    let root = run_fixture_root();
    vec![
        root.join("jvm_interop_stdout.valkyrie"),
        root.join("wasm_interop_stdout.valkyrie"),
        root.join("wasi_component_clock.valkyrie"),
        root.join("local_package_minimal.valkyrie"),
        root.join("script_minimal.valkyrie"),
        root.join("nested_workspace_minimal.valkyrie"),
    ]
}

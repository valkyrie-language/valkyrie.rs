mod support;

use legion::{
    cmds::build::{run, BuildArgs},
    CanonicalTarget,
};
use std::{fs, process::ExitCode};
use support::{
    create_local_package_project, create_smoke_project, create_smoke_project_with_build, create_smoke_project_with_manifest,
    create_smoke_project_with_source,
};

#[test]
fn builds_minimal_clr_project() {
    let fixture = create_smoke_project("legion-build");
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
    assert!(output_dir.join("app.runtimeconfig.json").exists());
}

#[test]
fn builds_migrated_test_clr_smoke_project() {
    let fixture = create_smoke_project_with_manifest(
        "legion-build-migrated-clr-smoke",
        r#"{
    name: "test_clr_smoke",
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
        r#"[main]
micro main(): i32 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr-smoke");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("test_clr_smoke.exe").exists());
    assert!(output_dir.join("test_clr_smoke.msil").exists());
    assert!(output_dir.join("run-contract.txt").exists());
}

#[test]
fn prefers_named_main_when_multiple_functions_have_main_attribute() {
    let fixture = create_smoke_project_with_source(
        "legion-build-entry",
        r#"[main]
micro helper() -> i64 {
    return 1;
}

[main]
micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let run_contract = fs::read_to_string(output_dir.join("run-contract.txt")).unwrap();
    assert!(run_contract.contains("logical_entry: \"main\""));
}

#[test]
fn builds_clr_project_with_tuple_pattern_let_and_loop_in() {
    let fixture = create_smoke_project_with_source(
        "legion-build-pattern",
        r#"micro main() -> i64 {
    let pair = ((1, 2), 3);
    let ((x, _), y) = pair;
    let pairs = [((4, 5), 6)];
    loop ((a, _), b) in pairs {
        return x + y + a + b;
    }
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
    assert!(output_dir.join("app.msil").exists());
}

#[test]
fn builds_node_wasm_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-node",
        r#"{
            target: "node"
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-node");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("node").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.wasm").exists());
    assert!(output_dir.join("app.mjs").exists());
    let run_contract = fs::read_to_string(output_dir.join("run-contract.txt")).unwrap();
    assert!(run_contract.contains("physical_entry: \"app.mjs\""));
    assert!(run_contract.contains("invocation: \"node\""));
}

#[test]
fn builds_wasi_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-wasi",
        r#"{
            target: "wasi"
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-wasi");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasi").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.wasm").exists());
    let run_contract = fs::read_to_string(output_dir.join("run-contract.txt")).unwrap();
    assert!(run_contract.contains("physical_entry: \"app.wasm\""));
    assert!(run_contract.contains("invocation: \"wasmtime\""));
}

#[test]
fn builds_native_msvc_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-native",
        r#"{
            target: "native"
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-native");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("native").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.obj").exists());
}

#[test]
fn builds_migrated_test_wasm_minimal_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-migrated-wasm-minimal",
        r#"{
            target: "wasm32-unknown-web-webassembly"
        }"#,
        r#"namespace test;

[main]
micro main(): unit {
    var _ = hello()
}

micro hello(): i32 {
    return 42
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-web-wasm");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasm32-unknown-web-webassembly").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.wasm").exists());
    assert!(output_dir.join("app.mjs").exists());
    assert!(output_dir.join("run-contract.txt").exists());
}

#[test]
fn builds_migrated_test_wasm_hello_project() {
    let source = r#"[main]
micro main(): i32 {
    return 42;
}
"#;
    let node_fixture = create_smoke_project_with_build(
        "legion-build-migrated-wasm-hello-node",
        r#"{
            target: "node"
        }"#,
        source,
    );
    let node_output_dir = node_fixture.project_dir.join("dist").join("custom-node");
    let node_status = run(&BuildArgs {
        project_dir: node_fixture.project_dir.clone(),
        target: CanonicalTarget::parse("node").unwrap(),
        output_dir: Some(node_output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(node_status, ExitCode::SUCCESS);
    assert!(node_output_dir.join("app.wasm").exists());
    assert!(node_output_dir.join("app.mjs").exists());

    let wasi_fixture = create_smoke_project_with_build(
        "legion-build-migrated-wasm-hello-wasi",
        r#"{
            target: "wasi"
        }"#,
        source,
    );
    let wasi_output_dir = wasi_fixture.project_dir.join("dist").join("custom-wasi");
    let wasi_status = run(&BuildArgs {
        project_dir: wasi_fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasi").unwrap(),
        output_dir: Some(wasi_output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(wasi_status, ExitCode::SUCCESS);
    assert!(wasi_output_dir.join("app.wasm").exists());
    assert!(wasi_output_dir.join("run-contract.txt").exists());
}

#[test]
fn builds_local_package_when_project_is_not_registered_in_workspace_members() {
    let fixture = create_local_package_project(
        "legion-build-local-package",
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
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
    assert!(output_dir.join("run-contract.txt").exists());
}

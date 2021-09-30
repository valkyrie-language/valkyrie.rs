mod support;

use legion::{
    cmds::build::{run, BuildArgs},
    CanonicalTarget,
};
use std::{fs, process::ExitCode};
use support::{create_smoke_project, create_smoke_project_with_build, create_smoke_project_with_source};

#[test]
fn builds_minimal_clr_project() {
    let fixture = create_smoke_project("legion-build");
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status =
        run(&BuildArgs { project_dir: fixture.project_dir.clone(), target: CanonicalTarget::clr(), output_dir: Some(output_dir.clone()) })
            .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
    assert!(output_dir.join("app.runtimeconfig.json").exists());
}

#[test]
fn prefers_named_main_when_multiple_functions_have_main_attribute() {
    let fixture = create_smoke_project_with_source(
        "legion-build-entry",
        "[main]\nmicro helper() -> i64 {\n    return 1;\n}\n\n[main]\nmicro main() -> i64 {\n    return 0;\n}\n",
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status =
        run(&BuildArgs { project_dir: fixture.project_dir.clone(), target: CanonicalTarget::clr(), output_dir: Some(output_dir.clone()) })
            .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let run_contract = fs::read_to_string(output_dir.join("run-contract.txt")).unwrap();
    assert!(run_contract.contains("logical_entry: \"main\""));
}

#[test]
fn builds_clr_project_with_tuple_pattern_let_and_loop_in() {
    let fixture = create_smoke_project_with_source(
        "legion-build-pattern",
        "micro main() -> i64 {\n    let pair = ((1, 2), 3);\n    let ((x, _), y) = pair;\n    let pairs = [((4, 5), 6)];\n    loop ((a, _), b) in pairs {\n        return x + y + a + b;\n    }\n    return 0;\n}\n",
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status =
        run(&BuildArgs { project_dir: fixture.project_dir.clone(), target: CanonicalTarget::clr(), output_dir: Some(output_dir.clone()) })
            .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
    assert!(output_dir.join("app.msil").exists());
}

#[test]
fn builds_node_wasm_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-node",
        "{\n            target: \"node\"\n        }",
        "micro main() -> i64 {\n    return 0;\n}\n",
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-node");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("node").unwrap(),
        output_dir: Some(output_dir.clone()),
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
        "{\n            target: \"wasi\"\n        }",
        "micro main() -> i64 {\n    return 0;\n}\n",
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-wasi");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasi").unwrap(),
        output_dir: Some(output_dir.clone()),
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
        "{\n            target: \"native\"\n        }",
        "micro main() -> i64 {\n    return 0;\n}\n",
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-native");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("native").unwrap(),
        output_dir: Some(output_dir.clone()),
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.obj").exists());
}

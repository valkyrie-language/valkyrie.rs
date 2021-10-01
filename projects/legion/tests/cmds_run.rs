mod support;

use legion::{
    cmds::{
        build::{run as run_build, BuildArgs},
        run::{run as run_project, RunArgs},
    },
    CanonicalTarget,
};
use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};
use support::{create_local_package_project, create_smoke_project, create_smoke_project_with_manifest, create_smoke_project_with_source};
use tempfile::tempdir;

fn workspace_repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("valkyrie.v")
}

#[test]
fn runs_minimal_clr_project_in_dry_run_mode() {
    let fixture = create_smoke_project("legion-run");
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
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
        dry_run: true,
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
    assert!(output_dir.join("run-contract.txt").exists());
}

#[test]
fn runs_clr_tuple_pattern_project_in_dry_run_mode() {
    let fixture = create_smoke_project_with_source(
        "legion-run-pattern",
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

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
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
        dry_run: true,
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
}

#[test]
fn runs_minimal_clr_project() {
    let fixture = create_smoke_project("legion-run-real");
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
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
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
}

#[test]
fn runs_migrated_test_clr_smoke_project() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-clr-smoke",
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

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();
    assert_eq!(build_status, ExitCode::SUCCESS);

    let run_status = run_project(&RunArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir),
        workspace: false,
        runner: Vec::new(),
        artifact: None,
        dry_run: false,
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
}

#[test]
fn runs_migrated_test_local_var_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-local-var",
        r#"{
    name: "test_local_var",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let x: i32 = 42
    if x != 42 {
        return 1
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-local-var", CanonicalTarget::clr()),
        ("jvm-local-var", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-local-var", CanonicalTarget::parse("node").unwrap()),
        ("wasi-local-var", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_loop_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-loop",
        r#"{
    name: "test_loop",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let i: i32 = 0
    while i < 3 {
        i = i + 1
    }
    if i != 3 {
        return 1
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-loop", CanonicalTarget::clr()),
        ("jvm-loop", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-loop", CanonicalTarget::parse("node").unwrap()),
        ("wasi-loop", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_function_call_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-func-call",
        r#"{
    name: "test_function_call",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let res: i32 = compute(10)
    if res != 55 {
        return 1
    }
    return 0
}

micro compute(n: i32) -> i32 {
    let sum: i32 = 0
    let i: i32 = 1
    while i <= n {
        sum = sum + i
        i = i + 1
    }
    return sum
}
"#,
    );
    let cases = [
        ("clr-func-call", CanonicalTarget::clr()),
        ("jvm-func-call", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-func-call", CanonicalTarget::parse("node").unwrap()),
        ("wasi-func-call", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_store_subscript_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-store-subscript",
        r#"{
    name: "test_store_subscript",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let mut values: [i32] = [10, 20, 30]
    if values[1] != 20 {
        return 1
    }
    values[1] = 42
    if values[1] != 42 {
        return 2
    }
    if values[0] + values[1] + values[2] != 82 {
        return 3
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-store-subscript", CanonicalTarget::clr()),
        ("jvm-store-subscript", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-store-subscript", CanonicalTarget::parse("node").unwrap()),
        ("wasi-store-subscript", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_if_while_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-if-while",
        r#"{
    name: "test_if_while",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let i: i32 = 0
    let sum: i32 = 0
    while i < 5 {
        if i == 3 {
            sum = sum + 10
        }
        else {
            sum = sum + i
        }
        i = i + 1
    }
    if sum != 17 {
        return 1
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-if-while", CanonicalTarget::clr()),
        ("jvm-if-while", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-if-while", CanonicalTarget::parse("node").unwrap()),
        ("wasi-if-while", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_minimal_func_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-minimal-func",
        r#"{
    name: "test_minimal_func",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let value: i32 = answer()
    if value != 42 {
        return 1
    }
    return 0
}

micro answer(): i32 {
    return 42
}
"#,
    );
    let cases = [
        ("clr-minimal-func", CanonicalTarget::clr()),
        ("jvm-minimal-func", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-minimal-func", CanonicalTarget::parse("node").unwrap()),
        ("wasi-minimal-func", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_operator_expr_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-operator-expr",
        r#"{
    name: "test_operator_expr",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"[main]
micro main(): i32 {
    let a: i32 = 10
    let b: i32 = 3
    if a + b != 13 {
        return 1
    }
    if a - b != 7 {
        return 2
    }
    if a * b != 30 {
        return 3
    }
    if a / b != 3 {
        return 4
    }
    if a % b != 1 {
        return 5
    }
    if !(a > b) {
        return 6
    }
    if a < b {
        return 7
    }
    if a == b {
        return 8
    }
    if a != 10 {
        return 9
    }
    if -a != -10 {
        return 10
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-operator-expr", CanonicalTarget::clr()),
        ("jvm-operator-expr", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-operator-expr", CanonicalTarget::parse("node").unwrap()),
        ("wasi-operator-expr", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_control_flow_smoke_project_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-control-flow-smoke",
        r#"{
    name: "test_control_flow_smoke",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"micro counted(limit: i32): i32 {
    let total: i32 = 0
    let i: i32 = 0
    while i < limit {
        total = total + 1
        i = i + 1
    }
    return total
}

micro branching(limit: i32): i32 {
    let sum: i32 = 0
    let i: i32 = 0
    while i < limit {
        if i == 2 {
            sum = sum + 10
        }
        else {
            sum = sum + i
        }
        i = i + 1
    }
    return sum
}

micro infinite(limit: i32): i32 {
    let current: i32 = 0
    loop {
        if current >= limit {
            break
        }
        current = current + 1
    }
    return current
}

[main]
micro main(): i32 {
    let limit: i32 = 4
    if counted(limit) != 4 {
        return 1
    }
    if branching(limit) != 14 {
        return 2
    }
    if infinite(limit) != 4 {
        return 3
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-control-flow-smoke", CanonicalTarget::clr()),
        ("jvm-control-flow-smoke", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-control-flow-smoke", CanonicalTarget::parse("node").unwrap()),
        ("wasi-control-flow-smoke", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_nested_loop_control_flow_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-nested-loop-control-flow",
        r#"{
    name: "test_nested_loop_control_flow",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"micro nested(limit: i32): i32 {
    let outer: i32 = 0
    let total: i32 = 0
    while outer < limit {
        let inner: i32 = 0
        while inner < 5 {
            inner = inner + 1
            if inner == 2 {
                continue
            }
            if outer == 2 {
                if inner == 4 {
                    break
                }
            }
            total = total + outer + inner
        }
        outer = outer + 1
    }
    return total
}

[main]
micro main(): i32 {
    if nested(4) != 63 {
        return 1
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-nested-loop-control-flow", CanonicalTarget::clr()),
        ("jvm-nested-loop-control-flow", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-nested-loop-control-flow", CanonicalTarget::parse("node").unwrap()),
        ("wasi-nested-loop-control-flow", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_nested_early_return_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-nested-early-return",
        r#"{
    name: "test_nested_early_return",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"micro find_target(limit: i32): i32 {
    let outer: i32 = 0
    while outer < limit {
        let inner: i32 = 0
        while inner < 4 {
            if outer == 1 {
                if inner == 2 {
                    return outer * 10 + inner
                }
            }
            inner = inner + 1
        }
        outer = outer + 1
    }
    return 0
}

[main]
micro main(): i32 {
    if find_target(3) != 12 {
        return 1
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-nested-early-return", CanonicalTarget::clr()),
        ("jvm-nested-early-return", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-nested-early-return", CanonicalTarget::parse("node").unwrap()),
        ("wasi-nested-early-return", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_mixed_complex_control_flow_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-mixed-complex-control-flow",
        r#"{
    name: "test_mixed_complex_control_flow",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"micro search(limit: i32): i32 {
    let outer: i32 = 0
    loop {
        if outer >= limit {
            break
        }

        let inner: i32 = 0
        while inner < 6 {
            inner = inner + 1
            if inner == 2 {
                continue
            }
            if outer == 1 {
                if inner == 5 {
                    break
                }
            }
            if outer == 2 {
                if inner == 4 {
                    return outer * 10 + inner
                }
            }
        }

        outer = outer + 1
    }

    return -1
}

[main]
micro main(): i32 {
    if search(5) != 24 {
        return 1
    }
    if search(2) != -1 {
        return 2
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-mixed-complex-control-flow", CanonicalTarget::clr()),
        ("jvm-mixed-complex-control-flow", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-mixed-complex-control-flow", CanonicalTarget::parse("node").unwrap()),
        ("wasi-mixed-complex-control-flow", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_loop_scope_control_flow_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-loop-scope-control-flow",
        r#"{
    name: "test_loop_scope_control_flow",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"micro scoped(limit: i32): i32 {
    let outer: i32 = 0
    let total: i32 = 0
    while outer < limit {
        let inner: i32 = 0
        while inner < 5 {
            inner = inner + 1
            if inner == 1 {
                continue
            }
            if inner == 4 {
                break
            }
            total = total + outer * 10 + inner
        }
        total = total + 100 + outer
        outer = outer + 1
    }
    return total
}

[main]
micro main(): i32 {
    if scoped(3) != 378 {
        return 1
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-loop-scope-control-flow", CanonicalTarget::clr()),
        ("jvm-loop-scope-control-flow", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-loop-scope-control-flow", CanonicalTarget::parse("node").unwrap()),
        ("wasi-loop-scope-control-flow", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_migrated_test_array_and_control_flow_combo_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-migrated-array-control-flow-combo",
        r#"{
    name: "test_array_control_flow_combo",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        },
        {
            target: "jvm-openjdk-unknown-managed"
        },
        {
            target: "node"
        },
        {
            target: "wasi"
        }
    ]
}
"#,
        r#"micro classify(value: i32): i32 {
    if value == 0 {
        return 5
    }
    if value == 2 {
        return 20
    }
    return value + 1
}

micro accumulate(limit: i32): i32 {
    let mut values: [i32] = [0, 0, 0, 0, 0]
    let index: i32 = 0
    loop {
        if index >= limit {
            break
        }
        if index == 1 {
            values[index] = 50
            index = index + 1
            continue
        }
        if index == 4 {
            break
        }
        values[index] = classify(index)
        index = index + 1
    }

    let cursor: i32 = 0
    let total: i32 = 0
    while cursor < 5 {
        if cursor == 3 {
            cursor = cursor + 1
            continue
        }
        total = total + values[cursor]
        cursor = cursor + 1
    }
    return total
}

[main]
micro main(): i32 {
    if accumulate(5) != 75 {
        return 1
    }
    if accumulate(2) != 55 {
        return 2
    }
    return 0
}
"#,
    );
    let cases = [
        ("clr-array-control-flow-combo", CanonicalTarget::clr()),
        ("jvm-array-control-flow-combo", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-array-control-flow-combo", CanonicalTarget::parse("node").unwrap()),
        ("wasi-array-control-flow-combo", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        let build_status = run_build(&BuildArgs {
            project_dir: fixture.project_dir.clone(),
            target: target.clone(),
            output_dir: Some(output_dir.clone()),
            workspace: false,
        })
        .unwrap();
        assert_eq!(build_status, ExitCode::SUCCESS, "build failed for {}", dir_name);

        let run_status = run_project(&RunArgs {
            project_dir: fixture.project_dir.clone(),
            target,
            output_dir: Some(output_dir),
            workspace: false,
            runner: Vec::new(),
            artifact: None,
            dry_run: false,
        })
        .unwrap();
        assert_eq!(run_status, ExitCode::SUCCESS, "run failed for {}", dir_name);
    }
}

#[test]
fn runs_local_package_when_project_is_not_registered_in_workspace_members() {
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
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
}

#[test]
fn runs_std_fs_smoke_project_on_clr() {
    let project_dir = workspace_repo_root().join("examples").join("test.fs");
    let temp_dir = tempdir().unwrap();
    let output_dir = temp_dir.path().join("test-fs-clr");
    let smoke_file = project_dir.join("test.fs.smoke.txt");
    let _ = fs::remove_file(&smoke_file);

    let build_status = run_build(&BuildArgs {
        project_dir: project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
    })
    .unwrap();
    assert_eq!(build_status, ExitCode::SUCCESS);

    let run_status = run_project(&RunArgs {
        project_dir: project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir),
        workspace: false,
        runner: Vec::new(),
        artifact: None,
        dry_run: false,
    })
    .unwrap();
    assert_eq!(run_status, ExitCode::SUCCESS);
    assert_eq!(fs::read_to_string(smoke_file).unwrap(), "fs smoke ok");
}

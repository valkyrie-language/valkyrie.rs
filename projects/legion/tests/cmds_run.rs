mod support;

use legion::{
    cmds::{
        build::{run as run_build, BuildArgs},
        run::{run as run_project, RunArgs},
    },
    CanonicalTarget,
};
use std::{path::Path, process::ExitCode};
use support::{create_local_package_project, create_smoke_project_with_manifest, create_smoke_project_with_source};

fn assert_delegated_dry_run_succeeds(fixture: &support::SmokeProject, output_dir: &Path, target: CanonicalTarget) {
    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: target.clone(),
        output_dir: Some(output_dir.to_path_buf()),
        workspace: false,
    })
    .unwrap();
    assert_eq!(build_status, ExitCode::SUCCESS);

    let run_status = run_project(&RunArgs {
        project_dir: fixture.project_dir.clone(),
        target,
        output_dir: Some(output_dir.to_path_buf()),
        workspace: false,
        runner: Vec::new(),
        artifact: None,
        dry_run: true,
    })
    .unwrap();
    assert_eq!(run_status, ExitCode::SUCCESS);
}

#[test]
fn runs_minimal_project_in_dry_run_mode_across_clr_jvm_node_and_wasi() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run",
        r#"{
    name: "app",
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
    return 0;
}
"#,
    );
    let cases = [
        ("custom-clr", CanonicalTarget::clr()),
        ("custom-jvm", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("custom-node", CanonicalTarget::parse("node").unwrap()),
        ("custom-wasi", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        assert_delegated_dry_run_succeeds(&fixture, &output_dir, target);
        assert!(output_dir.join("run-contract.txt").exists(), "run contract missing for {}", dir_name);
    }
}

#[test]
fn delegates_default_runtime_templates_across_clr_jvm_node_and_wasi_in_dry_run_mode() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-runtime-delegation",
        r#"{
    name: "runtime_delegation",
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
    return 0;
}
"#,
    );

    let cases = [
        ("clr-runtime-delegation", CanonicalTarget::clr()),
        ("jvm-runtime-delegation", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-runtime-delegation", CanonicalTarget::parse("node").unwrap()),
        ("wasi-runtime-delegation", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        assert_delegated_dry_run_succeeds(&fixture, &output_dir, target);
        assert!(output_dir.join("run-contract.txt").exists(), "run contract missing for {}", dir_name);
    }
}

#[test]
fn delegates_migrated_test_subscript_across_clr_jvm_node_and_wasi_in_dry_run_mode() {
    let fixture = create_smoke_project_with_manifest(
        "legion-run-subscript",
        r#"{
    name: "test_subscript",
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
micro main(args: [utf16]): i32 {
    if len(args) > 0 {
        let first: utf16 = args[0];
        let copied: utf16 = first;
        let _ = copied;
    }
    return 0;
}
"#,
    );

    let cases = [
        ("clr-subscript", CanonicalTarget::clr()),
        ("jvm-subscript", CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap()),
        ("node-subscript", CanonicalTarget::parse("node").unwrap()),
        ("wasi-subscript", CanonicalTarget::parse("wasi").unwrap()),
    ];

    for (dir_name, target) in cases {
        let output_dir = fixture.project_dir.join("dist").join(dir_name);
        assert_delegated_dry_run_succeeds(&fixture, &output_dir, target);
        assert!(output_dir.join("run-contract.txt").exists(), "run contract missing for {}", dir_name);
    }
}

#[test]
#[ignore = "feature-oriented smoke moved to compiler/interpreter regression suites"]
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
#[ignore = "redundant migrated smoke; keep thin legion dry-run coverage only"]
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
        dry_run: true,
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
}

#[test]
#[ignore = "runtime smoke moved to valkyrie-interpreter/tests"]
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

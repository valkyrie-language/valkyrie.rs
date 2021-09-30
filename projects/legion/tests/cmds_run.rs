mod support;

use legion::{
    cmds::{
        build::{run as run_build, BuildArgs},
        run::{run as run_project, RunArgs},
    },
    CanonicalTarget,
};
use std::process::ExitCode;
use support::{create_smoke_project, create_smoke_project_with_source};

#[test]
fn runs_minimal_clr_project_in_dry_run_mode() {
    let fixture = create_smoke_project("legion-run");
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
    })
    .unwrap();
    assert_eq!(build_status, ExitCode::SUCCESS);

    let run_status = run_project(&RunArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
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
        "micro main() -> i64 {\n    let pair = ((1, 2), 3);\n    let ((x, _), y) = pair;\n    let pairs = [((4, 5), 6)];\n    loop ((a, _), b) in pairs {\n        return x + y + a + b;\n    }\n    return 0;\n}\n",
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");

    let build_status = run_build(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
    })
    .unwrap();
    assert_eq!(build_status, ExitCode::SUCCESS);

    let run_status = run_project(&RunArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        runner: Vec::new(),
        artifact: None,
        dry_run: true,
    })
    .unwrap();

    assert_eq!(run_status, ExitCode::SUCCESS);
    assert!(output_dir.join("app.exe").exists());
}

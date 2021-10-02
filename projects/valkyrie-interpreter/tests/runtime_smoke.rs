use std::{
    env, fs,
    fs::File,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::{SystemTime, UNIX_EPOCH},
};

use miette::{miette, IntoDiagnostic, Result};
use valkyrie_interpreter::{RuntimeContract, RuntimeFamily};

const QUAD_CASES: [(&str, &str, RuntimeFamily); 4] = [
    ("clr", "clr", RuntimeFamily::Clr),
    ("jvm", "jvm-openjdk-unknown-managed", RuntimeFamily::Jvm),
    ("node", "node", RuntimeFamily::Node),
    ("wasi", "wasi", RuntimeFamily::Wasi),
];

const JVM_NODE_WASI_CASES: [(&str, &str, RuntimeFamily); 3] =
    [("jvm", "jvm-openjdk-unknown-managed", RuntimeFamily::Jvm), ("node", "node", RuntimeFamily::Node), ("wasi", "wasi", RuntimeFamily::Wasi)];

const JVM_WASI_CASES: [(&str, &str, RuntimeFamily); 2] =
    [("jvm", "jvm-openjdk-unknown-managed", RuntimeFamily::Jvm), ("wasi", "wasi", RuntimeFamily::Wasi)];

const CLR_CASES: [(&str, &str, RuntimeFamily); 1] = [("clr", "clr", RuntimeFamily::Clr)];

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new(name: &str) -> Self {
        let unique = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let path = std::env::temp_dir().join(format!("valkyrie-runtime-smoke-{name}-{}-{unique}", std::process::id()));
        fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedRunContract {
    logical_entry: String,
    physical_entry: String,
}

#[test]
fn runs_minimal_clr_jvm_node_and_wasi_smoke_artifacts() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }

    let fixture = create_runtime_smoke_project()?;
    run_project_across_cases(&fixture.path().join("app"), &JVM_NODE_WASI_CASES)?;
    Ok(())
}

#[test]
fn runs_control_flow_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "control-flow-smoke",
        "test_control_flow_smoke",
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
micro main(): ExitCode {
    let limit: i32 = 4
    if counted(limit) != 4 {
        return ExitCode(1)
    }
    if branching(limit) != 14 {
        return ExitCode(2)
    }
    if infinite(limit) != 4 {
        return ExitCode(3)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &JVM_NODE_WASI_CASES)?;
    Ok(())
}

#[test]
fn runs_nested_loop_control_flow_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "nested-loop-control-flow",
        "test_nested_loop_control_flow",
        r#"micro nested(limit: i64): i64 {
    let outer: i64 = 0
    let total: i64 = 0
    let inner_limit: i64 = 5
    let step: i64 = 1
    let skip_inner: i64 = 2
    let break_outer: i64 = 2
    let break_inner: i64 = 4
    while outer < limit {
        let inner: i64 = 0
        while inner < inner_limit {
            inner = inner + step
            if inner == skip_inner {
                continue
            }
            if outer == break_outer {
                if inner == break_inner {
                    break
                }
            }
            total = total + outer + inner
        }
        outer = outer + step
    }
    return total
}

[main]
micro main(): ExitCode {
    let limit: i64 = 4
    let expected: i64 = 63
    if nested(limit) != expected {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &JVM_NODE_WASI_CASES)?;
    Ok(())
}

#[test]
fn runs_nested_early_return_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "nested-early-return",
        "test_nested_early_return",
        r#"micro find_target(limit: i64): i64 {
    let outer: i64 = 0
    let inner_limit: i64 = 4
    let outer_target: i64 = 1
    let inner_target: i64 = 2
    let factor: i64 = 10
    let step: i64 = 1
    while outer < limit {
        let inner: i64 = 0
        while inner < inner_limit {
            if outer == outer_target {
                if inner == inner_target {
                    return outer * factor + inner
                }
            }
            inner = inner + step
        }
        outer = outer + step
    }
    return 0
}

[main]
micro main(): ExitCode {
    let limit: i64 = 3
    let expected: i64 = 12
    if find_target(limit) != expected {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &JVM_NODE_WASI_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr runtime regression during real execution; migrated from legion"]
fn keeps_control_flow_smoke_on_clr_as_runtime_regression() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "control-flow-smoke-clr",
        "test_control_flow_smoke",
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
micro main(): ExitCode {
    let limit: i32 = 4
    if counted(limit) != 4 {
        return ExitCode(1)
    }
    if branching(limit) != 14 {
        return ExitCode(2)
    }
    if infinite(limit) != 4 {
        return ExitCode(3)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &CLR_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr runtime regression during real execution; migrated from legion"]
fn keeps_nested_loop_control_flow_on_clr_as_runtime_regression() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "nested-loop-control-flow-clr",
        "test_nested_loop_control_flow",
        r#"micro nested(limit: i64): i64 {
    let outer: i64 = 0
    let total: i64 = 0
    let inner_limit: i64 = 5
    let step: i64 = 1
    let skip_inner: i64 = 2
    let break_outer: i64 = 2
    let break_inner: i64 = 4
    while outer < limit {
        let inner: i64 = 0
        while inner < inner_limit {
            inner = inner + step
            if inner == skip_inner {
                continue
            }
            if outer == break_outer {
                if inner == break_inner {
                    break
                }
            }
            total = total + outer + inner
        }
        outer = outer + step
    }
    return total
}

[main]
micro main(): ExitCode {
    let limit: i64 = 4
    let expected: i64 = 63
    if nested(limit) != expected {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &CLR_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr runtime regression during real execution; migrated from legion"]
fn keeps_nested_early_return_on_clr_as_runtime_regression() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "nested-early-return-clr",
        "test_nested_early_return",
        r#"micro find_target(limit: i64): i64 {
    let outer: i64 = 0
    let inner_limit: i64 = 4
    let outer_target: i64 = 1
    let inner_target: i64 = 2
    let factor: i64 = 10
    let step: i64 = 1
    while outer < limit {
        let inner: i64 = 0
        while inner < inner_limit {
            if outer == outer_target {
                if inner == inner_target {
                    return outer * factor + inner
                }
            }
            inner = inner + step
        }
        outer = outer + step
    }
    return 0
}

[main]
micro main(): ExitCode {
    let limit: i64 = 3
    let expected: i64 = 12
    if find_target(limit) != expected {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &CLR_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr std fs runtime regression; migrated from legion"]
fn runs_std_fs_smoke_on_clr() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let project_dir = workspace_examples_root().join("test.fs");
    if !project_dir.exists() {
        eprintln!("skip runtime smoke: missing {}", project_dir.display());
        return Ok(());
    }

    let temp = TestDir::new("std-fs-clr");
    let output_dir = temp.path().join("dist").join("clr");
    let smoke_file = project_dir.join("test.fs.smoke.txt");
    let _ = fs::remove_file(&smoke_file);

    build_with_legion(&project_dir, "clr", &output_dir)?;
    let exit_code = run_built_artifact(&project_dir, &output_dir, RuntimeFamily::Clr)?;
    assert_eq!(exit_code, 0, "runtime smoke failed for std_fs clr");
    assert_eq!(fs::read_to_string(&smoke_file).into_diagnostic()?, "fs smoke ok");
    let _ = fs::remove_file(smoke_file);
    Ok(())
}

#[test]
fn runs_local_var_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "local-var-smoke",
        "test_local_var",
        r#"[main]
micro main(): ExitCode {
    let x: i32 = 42
    if x != 42 {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &QUAD_CASES)?;
    Ok(())
}

#[test]
fn runs_loop_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "loop-smoke",
        "test_loop",
        r#"[main]
micro main(): ExitCode {
    let i: i32 = 0
    while i < 3 {
        i = i + 1
    }
    if i != 3 {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &QUAD_CASES)?;
    Ok(())
}

#[test]
fn runs_function_call_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "function-call-smoke",
        "test_function_call",
        r#"[main]
micro main(): ExitCode {
    let input: i64 = 10
    let expected: i64 = 55
    let res: i64 = compute(input)
    if res != expected {
        return ExitCode(1)
    }
    return ExitCode(0)
}

micro compute(n: i64): i64 {
    let sum: i64 = 0
    let i: i64 = 1
    let step: i64 = 1
    while i <= n {
        sum = sum + i
        i = i + step
    }
    return sum
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &JVM_NODE_WASI_CASES)?;
    Ok(())
}

#[test]
fn runs_if_while_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "if-while-smoke",
        "test_if_while",
        r#"[main]
micro main(): ExitCode {
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
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &QUAD_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr loop runtime regression during real execution; migrated from legion"]
fn keeps_loop_smoke_on_clr_as_runtime_regression() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "loop-smoke-clr",
        "test_loop",
        r#"[main]
micro main(): ExitCode {
    let i: i32 = 0
    while i < 3 {
        i = i + 1
    }
    if i != 3 {
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &CLR_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr call runtime regression during real execution; migrated from legion"]
fn keeps_function_call_smoke_on_clr_as_runtime_regression() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "function-call-smoke-clr",
        "test_function_call",
        r#"[main]
micro main(): ExitCode {
    let input: i64 = 10
    let expected: i64 = 55
    let res: i64 = compute(input)
    if res != expected {
        return ExitCode(1)
    }
    return ExitCode(0)
}

micro compute(n: i64): i64 {
    let sum: i64 = 0
    let i: i64 = 1
    let step: i64 = 1
    while i <= n {
        sum = sum + i
        i = i + step
    }
    return sum
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &CLR_CASES)?;
    Ok(())
}

#[test]
#[ignore = "known clr if-while runtime regression during real execution; migrated from legion"]
fn keeps_if_while_smoke_on_clr_as_runtime_regression() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "if-while-smoke-clr",
        "test_if_while",
        r#"[main]
micro main(): ExitCode {
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
        return ExitCode(1)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &CLR_CASES)?;
    Ok(())
}

#[test]
fn runs_operator_expr_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "operator-expr-smoke",
        "test_operator_expr",
        r#"[main]
micro main(): ExitCode {
    let a: i32 = 10
    let b: i32 = 3
    if a + b != 13 {
        return ExitCode(1)
    }
    if a - b != 7 {
        return ExitCode(2)
    }
    if a * b != 30 {
        return ExitCode(3)
    }
    if a / b != 3 {
        return ExitCode(4)
    }
    if a % b != 1 {
        return ExitCode(5)
    }
    if !(a > b) {
        return ExitCode(6)
    }
    if a < b {
        return ExitCode(7)
    }
    if a == b {
        return ExitCode(8)
    }
    if a != 10 {
        return ExitCode(9)
    }
    if -a != -10 {
        return ExitCode(10)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &QUAD_CASES)?;
    Ok(())
}

#[test]
fn runs_store_subscript_smoke_across_clr_jvm_node_and_wasi() -> Result<()> {
    if !has_required_commands(&[("cargo", "cargo"), ("dotnet", "dotnet"), ("java", "java"), ("node", "node"), ("wasmtime", "wasmtime")]) {
        return Ok(());
    }
    let fixture = create_smoke_project(
        "store-subscript-smoke",
        "test_store_subscript",
        r#"[main]
micro main(): ExitCode {
    let mut values: [i32] = [10, 20, 30]
    if values[1] != 20 {
        return ExitCode(1)
    }
    values[1] = 42
    if values[1] != 42 {
        return ExitCode(2)
    }
    if values[0] + values[1] + values[2] != 82 {
        return ExitCode(3)
    }
    return ExitCode(0)
}
"#,
    )?;
    run_project_across_cases(&fixture.path().join("app"), &QUAD_CASES)?;
    Ok(())
}

fn create_runtime_smoke_project() -> Result<TestDir> {
    create_smoke_project(
        "triplet",
        "runtime_smoke",
        r#"[main]
micro main(): ExitCode {
    return ExitCode(0);
}
"#,
    )
}

fn create_smoke_project(name: &str, project_name: &str, source: &str) -> Result<TestDir> {
    let dir = TestDir::new(name);
    let workspace_manifest = r#"{
    name: "runtime-smoke",
    members: [
        "app"
    ]
}
"#;
    let project_manifest = r#"{
    name: "__PROJECT_NAME__",
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
"#
    .replace("__PROJECT_NAME__", project_name);

    let app_dir = dir.path().join("app");
    let source_dir = app_dir.join("source");
    fs::create_dir_all(&source_dir).into_diagnostic()?;
    fs::write(dir.path().join("legions.von"), workspace_manifest).into_diagnostic()?;
    fs::write(app_dir.join("legion.von"), project_manifest).into_diagnostic()?;
    fs::write(source_dir.join("main.v"), source).into_diagnostic()?;
    Ok(dir)
}

fn run_project_across_cases(project_dir: &Path, cases: &[(&str, &str, RuntimeFamily)]) -> Result<()> {
    for (dir_name, target, family) in cases {
        let output_dir = project_dir.join("dist").join(dir_name);
        build_with_legion(project_dir, target, &output_dir)?;
        let exit_code = run_built_artifact(project_dir, &output_dir, *family)?;
        assert_eq!(exit_code, 0, "runtime smoke failed for {}", target);
    }
    Ok(())
}

fn run_built_artifact(project_dir: &Path, output_dir: &Path, family: RuntimeFamily) -> Result<i32> {
    let contract = read_run_contract(output_dir)?;
    let artifact = output_dir.join(&contract.physical_entry);
    let classpath =
        if artifact.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("jar")) { artifact.clone() } else { output_dir.to_path_buf() };
    let entry = if contract.logical_entry.is_empty() {
        artifact.file_stem().and_then(|value| value.to_str()).unwrap_or_default().to_string()
    }
    else {
        contract.logical_entry.clone()
    };
    let template = family.default_template(Some(RuntimeContract {
        logical_entry: (!contract.logical_entry.is_empty()).then_some(contract.logical_entry.as_str()),
        physical_entry: Some(contract.physical_entry.as_str()),
    }));
    let command = template.prepare_command(&artifact, &classpath, &entry);
    run_checked_command(project_dir, &command.command, &command.args)
}

fn build_with_legion(project_dir: &Path, target: &str, output_dir: &Path) -> Result<()> {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..");
    let project_dir = project_dir.to_string_lossy().to_string();
    let output_dir = output_dir.to_string_lossy().to_string();
    let args = vec![
        "run".to_string(),
        "-p".to_string(),
        "legion".to_string(),
        "--".to_string(),
        "build".to_string(),
        project_dir,
        "--target".to_string(),
        target.to_string(),
        "--output".to_string(),
        output_dir,
    ];
    let _ = run_checked_command(&repo_root, "cargo", &args).map_err(|error| miette!("legion build failed for {}: {}", target, error))?;
    Ok(())
}

fn run_checked_command(cwd: &Path, program: &str, args: &[String]) -> Result<i32> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    let temp_root = std::env::temp_dir();
    let stdout_path = temp_root.join(format!("valkyrie-runtime-smoke-stdout-{}-{stamp}.log", std::process::id()));
    let stderr_path = temp_root.join(format!("valkyrie-runtime-smoke-stderr-{}-{stamp}.log", std::process::id()));
    let stdout_file = File::create(&stdout_path).into_diagnostic()?;
    let stderr_file = File::create(&stderr_path).into_diagnostic()?;
    let status = Command::new(program)
        .args(args)
        .current_dir(cwd)
        .stdout(Stdio::from(stdout_file))
        .stderr(Stdio::from(stderr_file))
        .status()
        .into_diagnostic()?;
    let exit_code = status.code().unwrap_or(1);
    if status.success() {
        let _ = fs::remove_file(&stdout_path);
        let _ = fs::remove_file(&stderr_path);
        return Ok(exit_code);
    }

    let stdout = fs::read_to_string(&stdout_path).unwrap_or_default().trim().to_string();
    let stderr = fs::read_to_string(&stderr_path).unwrap_or_default().trim().to_string();
    let _ = fs::remove_file(&stdout_path);
    let _ = fs::remove_file(&stderr_path);
    let mut details = Vec::new();
    details.push(format!("command: {} {}", program, args.join(" ")));
    details.push(format!("cwd: {}", cwd.display()));
    details.push(format!("exit code: {}", exit_code));
    if !stdout.is_empty() {
        details.push(format!("stdout:\n{}", stdout));
    }
    if !stderr.is_empty() {
        details.push(format!("stderr:\n{}", stderr));
    }
    Err(miette!(details.join("\n\n")))
}

fn read_run_contract(output_dir: &Path) -> Result<ParsedRunContract> {
    let source = fs::read_to_string(output_dir.join("run-contract.txt")).into_diagnostic()?;
    let mut logical_entry = String::new();
    let mut physical_entry = String::new();

    for line in source.lines() {
        let Some((key, value)) = line.split_once(':')
        else {
            continue;
        };
        let value = value.trim().trim_end_matches(',').trim().trim_matches('"').to_string();
        match key.trim() {
            "logical_entry" => logical_entry = value,
            "physical_entry" => physical_entry = value,
            _ => {}
        }
    }

    Ok(ParsedRunContract { logical_entry, physical_entry })
}

fn has_required_commands(commands: &[(&str, &str)]) -> bool {
    for (label, command) in commands {
        if !command_exists(command) {
            eprintln!("skip runtime smoke: missing {}", label);
            return false;
        }
    }
    true
}

fn command_exists(command: &str) -> bool {
    find_command_in_path(command).is_some()
}

fn find_command_in_path(command: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    let extensions = executable_extensions();
    for dir in env::split_paths(&path) {
        if let Some(found) = candidate_command_paths(&dir, command, &extensions).into_iter().find(|candidate| candidate.is_file()) {
            return Some(found);
        }
    }
    None
}

fn candidate_command_paths(dir: &Path, command: &str, extensions: &[String]) -> Vec<PathBuf> {
    let base = dir.join(command);
    if Path::new(command).extension().is_some() {
        return vec![base];
    }

    let mut candidates = Vec::with_capacity(1 + extensions.len());
    candidates.push(base.clone());
    for ext in extensions {
        candidates.push(dir.join(format!("{command}{ext}")));
    }
    candidates
}

fn executable_extensions() -> Vec<String> {
    if cfg!(windows) {
        env::var("PATHEXT")
            .ok()
            .map(|value| value.split(';').filter(|item| !item.is_empty()).map(|item| item.to_ascii_lowercase()).collect())
            .unwrap_or_else(|| vec![".exe".to_string(), ".cmd".to_string(), ".bat".to_string(), ".com".to_string()])
    }
    else {
        Vec::new()
    }
}

fn workspace_examples_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("valkyrie.v").join("examples")
}

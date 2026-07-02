//! 测试构建与执行会话。

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Instant,
};

use miette::{Result, miette};
use nvm::{NyarVm, Value};
use nyar_language::{CanonicalTarget, RunnerFamily};
use nyar_runner::{RuntimeContract as InterpreterRuntimeContract, RuntimeFamily as InterpreterRuntimeFamily};

use crate::{
    cmds::{build::compile_plan, report::TestResultEntry},
    manifest::RunnerBinding,
    planner::{BuildRequest, LegionWorkspace},
};

use super::{
    discover::{DiscoveredFunction, discover_project_tests},
    targets::parse_target_label,
};

/// 单次测试执行结果。
#[derive(Debug, Clone)]
pub struct RunOutcome {
    pub success: bool,
    pub is_compile_error: bool,
    pub error: Option<String>,
}

/// NyarVM 测试会话：一次编译，多次运行。
pub struct NyarTestSession {
    bytecode: Vec<u8>,
    /// 仅存在于 suspend_runtime sidecar、未编入 `.nyar` 的测试名。
    suspend_only_tests: BTreeMap<String, ()>,
}

/// 外部 target 测试会话：产物落盘后复用。
pub struct ExternalTestSession {
    pub target: String,
    pub output_dir: PathBuf,
    pub artifact_paths: BTreeMap<String, PathBuf>,
}

/// 编译 nyar 测试会话。
pub fn compile_nyar_test_session(workspace: &LegionWorkspace, project_dir: &Path) -> Result<NyarTestSession, String> {
    let target = parse_target_label("nyar").map_err(|e| e.to_string())?;
    let output_dir = project_dir.join(".cache").join("test").join("nyar");
    let request = BuildRequest { project_dir: project_dir.to_path_buf(), target, output_dir: Some(output_dir.clone()) };
    let (plan, _) = workspace.build_test_plan(&request).map_err(|e| e.to_string())?;
    compile_plan(&plan, false).map_err(|e| e.to_string())?;

    let nyar_path = find_nyar_artifact(&output_dir, &plan.project.name).ok_or_else(|| "未找到 .nyar 产物".to_string())?;
    let bytecode = fs::read(&nyar_path).map_err(|e| format!("读取 nyar 产物失败: {e}"))?;
    let suspend_only_tests = collect_suspend_only_test_names(&output_dir);
    Ok(NyarTestSession { bytecode, suspend_only_tests })
}

/// 在 nyar 会话中执行单个测试函数。
pub fn run_nyar_test_in_session(session: &NyarTestSession, function_name: &str) -> RunOutcome {
    if session.suspend_only_tests.contains_key(function_name) {
        return RunOutcome {
            success: false,
            is_compile_error: false,
            error: Some("suspend runtime 测试需在外部 target（clr/jvm/wasm）上运行".into()),
        };
    }

    let mut vm = NyarVm::new();
    vm.register_native("panic", |args| {
        let message = args.first().map(|value| format!("{value:?}")).unwrap_or_else(|| "panic".into());
        Err(nvm::NyarRuntimeError::ModuleLoad(message))
    });
    vm.register_native("is_null", |args| {
        let is_null = matches!(args.first(), Some(Value::Null) | None);
        Ok(Value::Bool(is_null))
    });
    vm.register_native("unwrap_null", |args| match args.first() {
        Some(Value::Null) | None => Err(nvm::NyarRuntimeError::ModuleLoad("unwrap_null on null".into())),
        Some(value) => Ok(value.clone()),
    });
    let loaded = match vm.load(&session.bytecode) {
        Ok(module) => module,
        Err(error) => return RunOutcome { success: false, is_compile_error: true, error: Some(error.to_string()) },
    };
    match vm.run(&loaded, function_name, Vec::<Value>::new()) {
        Ok(_) => RunOutcome { success: true, is_compile_error: false, error: None },
        Err(error) => RunOutcome { success: false, is_compile_error: false, error: Some(error.to_string()) },
    }
}

/// 编译外部 target 测试会话。
pub fn compile_external_test_session(
    workspace: &LegionWorkspace,
    project_dir: &Path,
    target_label: &str,
) -> Result<ExternalTestSession, String> {
    let target = parse_target_label(target_label).map_err(|e| e.to_string())?;
    let output_dir = project_dir.join(".cache").join("test").join(target_label);
    let request = BuildRequest { project_dir: project_dir.to_path_buf(), target, output_dir: Some(output_dir.clone()) };
    let (plan, _) = workspace.build_test_plan(&request).map_err(|e| e.to_string())?;
    compile_plan(&plan, false).map_err(|e| e.to_string())?;

    let mut artifact_paths = BTreeMap::new();
    collect_artifacts_recursive(&output_dir, &output_dir, &mut artifact_paths);
    Ok(ExternalTestSession { target: target_label.to_string(), output_dir, artifact_paths })
}

/// 在外部会话中执行单个测试函数。
pub fn run_external_test_in_session(
    workspace: &LegionWorkspace,
    session: &ExternalTestSession,
    function_name: &str,
    cli_runners: &[String],
    verbose: bool,
) -> RunOutcome {
    let Some(artifact) = resolve_external_test_artifact_path(session, function_name)
    else {
        return RunOutcome {
            success: false,
            is_compile_error: true,
            error: Some(format!("未找到测试 `{function_name}` 对应的产物。{}", describe_runnable_artifacts(session))),
        };
    };

    let Ok(canonical) = parse_target_label(&session.target)
    else {
        return RunOutcome { success: false, is_compile_error: true, error: Some(format!("未知目标 {}", session.target)) };
    };

    let runner_family = runner_family_for_label(&session.target);
    let template = match resolve_runner_template(workspace, &canonical, runner_family, cli_runners, function_name) {
        Ok(t) => t,
        Err(error) => return RunOutcome { success: false, is_compile_error: true, error: Some(error.to_string()) },
    };

    let placeholders = build_placeholders(&session.output_dir, &artifact, runner_family, function_name);
    let command = expand_placeholders(&template.command, &placeholders);
    let args: Vec<String> = template.args.iter().map(|a| expand_placeholders(a, &placeholders)).collect();

    let output = Command::new(&command).args(&args).stdout(Stdio::piped()).stderr(Stdio::piped()).output();

    match output {
        Ok(output) if output.status.success() => {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            if !stderr.trim().is_empty() {
                return RunOutcome { success: false, is_compile_error: false, error: Some(stderr) };
            }
            if verbose {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if !stdout.trim().is_empty() {
                    println!("      stdout: {}", stdout.trim_end());
                }
            }
            RunOutcome { success: true, is_compile_error: false, error: None }
        }
        Ok(output) => {
            let mut error = String::from_utf8_lossy(&output.stderr).to_string();
            if error.trim().is_empty() {
                error = String::from_utf8_lossy(&output.stdout).to_string();
            }
            RunOutcome { success: false, is_compile_error: false, error: Some(error) }
        }
        Err(error) => RunOutcome { success: false, is_compile_error: false, error: Some(error.to_string()) },
    }
}

/// 为项目在多 target 上运行全部测试。
pub fn run_tests_for_project(
    workspace: &LegionWorkspace,
    project_dir: &Path,
    filter: Option<&str>,
    targets: &[String],
    cli_runners: &[String],
    verbose: bool,
) -> (usize, usize, usize, Vec<TestResultEntry>) {
    let test_dir = project_dir.join("test");
    if !test_dir.exists() {
        println!("  无 test/ 目录，跳过");
        return (0, 0, 0, Vec::new());
    }

    let functions = discover_project_tests(project_dir);
    if functions.is_empty() {
        println!("  未发现 [test] 标注的测试函数");
        return (0, 0, 0, Vec::new());
    }

    let mut total_passed = 0usize;
    let mut total_failed = 0usize;
    let mut total_skipped = 0usize;
    let mut results = Vec::new();

    for target in targets {
        println!("  --- {target} ---");
        let is_nyar = target.eq_ignore_ascii_case("nyar");

        let nyar_session = if is_nyar { Some(compile_nyar_test_session(workspace, project_dir)) } else { None };
        let external_session = if !is_nyar { Some(compile_external_test_session(workspace, project_dir, target)) } else { None };

        for tf in &functions {
            if filter.is_some_and(|f| !tf.name.to_ascii_lowercase().contains(&f.to_ascii_lowercase())) {
                if verbose {
                    println!("    - {} ... 已跳过（过滤）", tf.name);
                }
                total_skipped += 1;
                results.push(TestResultEntry {
                    name: tf.name.clone(),
                    status: "skip".into(),
                    error: Some("被过滤器排除".into()),
                    target: target.clone(),
                });
                continue;
            }

            if tf.is_benchmark {
                if verbose {
                    println!("    - {} ... 已跳过（[benchmark]）", tf.name);
                }
                total_skipped += 1;
                results.push(TestResultEntry {
                    name: tf.name.clone(),
                    status: "skip".into(),
                    error: Some("[benchmark] 标注".into()),
                    target: target.clone(),
                });
                continue;
            }

            if tf.compile_only {
                if verbose {
                    println!("    - {} ... 已跳过（compile_only）", tf.name);
                }
                total_skipped += 1;
                results.push(TestResultEntry {
                    name: tf.name.clone(),
                    status: "skip".into(),
                    error: Some("compile_only 夹具（见 nyar-language 编译测试）".into()),
                    target: target.clone(),
                });
                continue;
            }

            if !tf.is_test {
                continue;
            }

            print!("    {} ... ", tf.name);

            let outcome = if is_nyar {
                match &nyar_session {
                    Some(Ok(session)) => run_nyar_test_in_session(session, &tf.name),
                    Some(Err(error)) => RunOutcome { success: false, is_compile_error: true, error: Some(error.clone()) },
                    None => RunOutcome { success: false, is_compile_error: true, error: Some("无 nyar 会话".into()) },
                }
            }
            else {
                match &external_session {
                    Some(Ok(session)) => run_external_test_in_session(workspace, session, &tf.name, cli_runners, verbose),
                    Some(Err(error)) => RunOutcome { success: false, is_compile_error: true, error: Some(error.clone()) },
                    None => RunOutcome { success: false, is_compile_error: true, error: Some("无外部会话".into()) },
                }
            };

            if outcome.success {
                println!("ok");
                total_passed += 1;
                results.push(TestResultEntry { name: tf.name.clone(), status: "pass".into(), error: None, target: target.clone() });
            }
            else if outcome.is_compile_error {
                println!("COMPILE ERROR");
                if let Some(error) = &outcome.error {
                    println!("      {error}");
                }
                total_failed += 1;
                results.push(TestResultEntry {
                    name: tf.name.clone(),
                    status: "compile_error".into(),
                    error: outcome.error,
                    target: target.clone(),
                });
            }
            else {
                println!("FAILED");
                if let Some(error) = &outcome.error {
                    println!("      {error}");
                }
                total_failed += 1;
                results.push(TestResultEntry { name: tf.name.clone(), status: "fail".into(), error: outcome.error, target: target.clone() });
            }
        }
    }

    (total_passed, total_failed, total_skipped, results)
}

/// 为项目在多 target 上运行基准测试。
pub fn bench_project(
    workspace: &LegionWorkspace,
    project_dir: &Path,
    project_name: &str,
    runs: usize,
    targets: &[String],
    verbose: bool,
) -> Vec<crate::cmds::report::BenchResultRow> {
    let functions: Vec<DiscoveredFunction> = discover_project_tests(project_dir).into_iter().filter(|f| f.is_benchmark).collect();
    if functions.is_empty() {
        println!("  未发现 [benchmark] 标注的函数");
        return Vec::new();
    }

    let mut results = Vec::new();
    for target in targets {
        let is_nyar = target.eq_ignore_ascii_case("nyar");
        for tf in &functions {
            let mut compile_times = Vec::new();
            let mut runtime_times = Vec::new();

            for _ in 0..runs {
                let compile_start = Instant::now();
                let session_result: Result<BenchSession, String> = if is_nyar {
                    compile_nyar_test_session(workspace, project_dir).map(BenchSession::Nyar)
                }
                else {
                    compile_external_test_session(workspace, project_dir, target).map(BenchSession::External)
                };
                let compile_ms = compile_start.elapsed().as_secs_f64() * 1000.0;

                let Ok(session) = session_result
                else {
                    continue;
                };

                let runtime_start = Instant::now();
                let outcome = match &session {
                    BenchSession::Nyar(s) => run_nyar_test_in_session(s, &tf.name),
                    BenchSession::External(s) => run_external_test_in_session(workspace, s, &tf.name, &[], verbose),
                };
                let runtime_ms = runtime_start.elapsed().as_secs_f64() * 1000.0;

                if !outcome.success && verbose {
                    if let Some(error) = &outcome.error {
                        println!("      bench {} 失败: {error}", tf.name);
                    }
                }

                compile_times.push(compile_ms);
                runtime_times.push(runtime_ms);
            }

            if !compile_times.is_empty() && !runtime_times.is_empty() {
                let compile_avg = compile_times.iter().sum::<f64>() / compile_times.len() as f64;
                let runtime_avg = runtime_times.iter().sum::<f64>() / runtime_times.len() as f64;
                results.push(crate::cmds::report::BenchResultRow {
                    project: project_name.to_string(),
                    test: tf.name.clone(),
                    target: target.clone(),
                    compile_ms: compile_avg,
                    runtime_ms: runtime_avg,
                });
            }
        }
    }
    results
}

enum BenchSession {
    Nyar(NyarTestSession),
    External(ExternalTestSession),
}

fn find_nyar_artifact(output_dir: &Path, project_name: &str) -> Option<PathBuf> {
    let preferred = output_dir.join(format!("{project_name}.nyar"));
    if preferred.exists() {
        return Some(preferred);
    }
    let functions_partition = output_dir.join(format!("{project_name}__functions.nyar"));
    if functions_partition.exists() {
        return Some(functions_partition);
    }

    let mut found = None;
    let Ok(entries) = fs::read_dir(output_dir)
    else {
        return None;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "nyar") {
            if path.file_name().is_some_and(|name| name.to_string_lossy().contains("__functions")) {
                return Some(path);
            }
            found = Some(path);
        }
    }
    found
}

fn collect_suspend_only_test_names(output_dir: &Path) -> BTreeMap<String, ()> {
    let mut names = BTreeMap::new();
    let Ok(entries) = fs::read_dir(output_dir)
    else {
        return names;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "json")
            && path.file_name().is_some_and(|name| name.to_string_lossy().contains(".suspend_runtime."))
        {
            if let Ok(body) = fs::read_to_string(&path) {
                extract_suspend_symbols(&body, &mut names);
            }
        }
    }
    names
}

fn extract_suspend_symbols(body: &str, names: &mut BTreeMap<String, ()>) {
    for segment in body.split("\"symbol\":\"").skip(1) {
        let Some(end) = segment.find('"')
        else {
            continue;
        };
        let symbol = &segment[..end];
        if let Some(short) = symbol.rsplit("::").next() {
            names.insert(short.to_string(), ());
        }
    }
}

fn collect_artifacts_recursive(root: &Path, dir: &Path, map: &mut BTreeMap<String, PathBuf>) {
    let Ok(entries) = fs::read_dir(dir)
    else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_artifacts_recursive(root, &path, map);
            continue;
        }
        if let Ok(rel) = path.strip_prefix(root) {
            let key = rel.to_string_lossy().replace('\\', "/");
            map.insert(key, path.clone());
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                map.insert(name.to_string(), path);
            }
        }
    }
}

fn runnable_extensions(target: &str) -> &'static [&'static str] {
    match target.to_ascii_lowercase().as_str() {
        "clr" => &[".exe"],
        "jvm" => &[".jar"],
        "node" => &[".mjs", ".js"],
        "nyar" => &[".nyar"],
        _ => &[],
    }
}

/// 解析外部测试产物路径。
pub fn resolve_external_test_artifact_path(session: &ExternalTestSession, function_name: &str) -> Option<PathBuf> {
    let extensions = runnable_extensions(&session.target);
    let runnable: Vec<_> = session
        .artifact_paths
        .iter()
        .filter(|(key, _)| extensions.iter().any(|ext| key.to_ascii_lowercase().ends_with(ext)))
        .map(|(_, path)| path.clone())
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect();

    if runnable.len() == 1 {
        return runnable.into_iter().next();
    }

    let short = get_short_function_name(function_name);
    let base = sanitize_test_artifact_name(&short);
    for extension in extensions {
        let file_name = format!("{base}{extension}");
        if let Some(path) = session.artifact_paths.get(&file_name) {
            return Some(path.clone());
        }
    }

    for extension in extensions {
        let suffix = format!("{base}__suspend{extension}");
        if let Some(path) = session.artifact_paths.get(&suffix) {
            return Some(path.clone());
        }
    }

    for (key, path) in &session.artifact_paths {
        if extensions.iter().any(|ext| key.to_ascii_lowercase().ends_with(ext)) && key.to_ascii_lowercase().contains(&base.to_ascii_lowercase())
        {
            return Some(path.clone());
        }
    }

    for (key, path) in &session.artifact_paths {
        if extensions.iter().any(|ext| key.to_ascii_lowercase().ends_with(ext)) && key.contains("__functions") {
            return Some(path.clone());
        }
    }

    None
}

fn describe_runnable_artifacts(session: &ExternalTestSession) -> String {
    let extensions = runnable_extensions(&session.target);
    let names: Vec<_> =
        session.artifact_paths.keys().filter(|key| extensions.iter().any(|ext| key.to_ascii_lowercase().ends_with(ext))).cloned().collect();
    if names.is_empty() { "无可执行产物".into() } else { format!("可执行产物：{}", names.join(", ")) }
}

fn get_short_function_name(function_name: &str) -> String {
    function_name.rsplit('.').next().unwrap_or(function_name).to_string()
}

fn sanitize_test_artifact_name(name: &str) -> String {
    if name.trim().is_empty() {
        return "Module".into();
    }
    let chars: String = name.chars().map(|ch| if ch.is_ascii_alphanumeric() || ch == '_' { ch } else { '_' }).collect();
    if chars.chars().next().is_some_and(|c| c.is_ascii_digit()) { format!("M_{chars}") } else { chars }
}

#[derive(Debug, Clone)]
struct RunnerTemplate {
    command: String,
    args: Vec<String>,
}

fn runner_family_for_label(label: &str) -> RunnerFamily {
    match label.to_ascii_lowercase().as_str() {
        "clr" => RunnerFamily::Clr,
        "jvm" => RunnerFamily::Jvm,
        "node" => RunnerFamily::Node,
        "nyar" => RunnerFamily::NyarVm,
        "wasi" => RunnerFamily::Wasi,
        _ => RunnerFamily::NyarVm,
    }
}

fn resolve_runner_template(
    workspace: &LegionWorkspace,
    canonical_target: &CanonicalTarget,
    runner_target: RunnerFamily,
    cli_runner_overrides: &[String],
    function_name: &str,
) -> Result<RunnerTemplate> {
    let default = default_runner_template(runner_target, function_name);

    if let Some(command) = parse_runner_overrides(cli_runner_overrides)?.remove(&runner_target) {
        return Ok(RunnerTemplate { command, args: default.args });
    }

    if let Some(workspace_manifest) = &workspace.workspace_manifest {
        if let Some(binding) = workspace_manifest.runner.iter().find(|binding| runner_binding_matches(binding, runner_target, canonical_target))
        {
            return Ok(RunnerTemplate { command: binding.command.clone(), args: binding.args.clone() });
        }
    }

    if let Ok(command) = std::env::var(format!("LEGION_RUNNER_{}", runner_target.as_str().to_ascii_uppercase())) {
        if !command.trim().is_empty() {
            return Ok(RunnerTemplate { command, args: default.args });
        }
    }

    Ok(default)
}

fn runner_binding_matches(binding: &RunnerBinding, runner_target: RunnerFamily, canonical_target: &CanonicalTarget) -> bool {
    binding.target.matches(runner_target, canonical_target)
}

fn parse_runner_overrides(values: &[String]) -> Result<BTreeMap<RunnerFamily, String>> {
    let mut overrides = BTreeMap::new();
    for item in values {
        let Some((target, command)) = item.split_once('=')
        else {
            return Err(miette!("invalid runner override '{item}': expected target=command"));
        };
        let family = target.trim().parse::<RunnerFamily>().map_err(|error| miette!("invalid runner override '{item}': {error}"))?;
        overrides.insert(family, command.trim().to_string());
    }
    Ok(overrides)
}

fn default_runner_template(target: RunnerFamily, function_name: &str) -> RunnerTemplate {
    let contract = InterpreterRuntimeContract { logical_entry: Some(function_name), physical_entry: Some(function_name), wasi_p3: false };
    let family = match target {
        RunnerFamily::Clr => InterpreterRuntimeFamily::Clr,
        RunnerFamily::Jvm => InterpreterRuntimeFamily::Jvm,
        RunnerFamily::Node => InterpreterRuntimeFamily::Node,
        RunnerFamily::Windows => InterpreterRuntimeFamily::Windows,
        RunnerFamily::Wasi => InterpreterRuntimeFamily::Wasi,
        RunnerFamily::NyarVm => InterpreterRuntimeFamily::NyarVm,
    };
    let template = family.default_template(Some(contract));
    RunnerTemplate { command: template.command, args: template.args }
}

fn build_placeholders(output_dir: &Path, artifact: &Path, runner_target: RunnerFamily, function_name: &str) -> BTreeMap<&'static str, String> {
    let mut values = BTreeMap::new();
    let artifact_text = strip_verbatim_prefix(&artifact.to_string_lossy()).to_string();
    values.insert("artifact", artifact_text.clone());
    let classpath = if artifact.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("jar")) {
        artifact_text.clone()
    }
    else {
        strip_verbatim_prefix(&output_dir.to_string_lossy()).to_string()
    };
    values.insert("classpath", classpath);
    let entry = if runner_target == RunnerFamily::Jvm { function_name.to_string() } else { function_name.to_string() };
    values.insert("entry", entry);
    values
}

fn strip_verbatim_prefix(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

fn expand_placeholders(template: &str, placeholders: &BTreeMap<&'static str, String>) -> String {
    placeholders.iter().fold(template.to_string(), |current, (key, value)| current.replace(&format!("{{{key}}}"), value))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_artifact_name() {
        assert_eq!(sanitize_test_artifact_name("add_two"), "add_two");
        assert_eq!(sanitize_test_artifact_name("1bad"), "M_1bad");
        assert_eq!(sanitize_test_artifact_name("a-b"), "a_b");
    }

    #[test]
    fn short_function_name() {
        assert_eq!(get_short_function_name("mod.add"), "add");
        assert_eq!(get_short_function_name("add"), "add");
    }
}

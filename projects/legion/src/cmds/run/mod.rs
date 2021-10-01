use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::Args;
use miette::{miette, IntoDiagnostic, Result};
use serde::{Deserialize, Serialize};
use valkyrie_compiler::{CanonicalTarget, RunnerFamily};
use valkyrie_interpreter::WasiRuntime;

use crate::{
    manifest::RunnerBinding,
    planner::{BuildRequest, LegionWorkspace},
};

/// `legion run` 的命令参数。
#[derive(Debug, Clone, Args)]
pub struct RunArgs {
    /// 项目目录，默认当前目录。
    #[arg(value_name = "project-dir", default_value = ".")]
    pub project_dir: PathBuf,
    /// 目标平台。
    #[arg(long, default_value = "clr")]
    pub target: CanonicalTarget,
    /// 输出目录。
    #[arg(short = 'o', long = "output")]
    pub output_dir: Option<PathBuf>,
    /// 强制按 workspace 成员解析；若项目未注册则直接报错，不回退到 package 模式。
    #[arg(long, default_value_t = false)]
    pub workspace: bool,
    /// 覆盖 runner 命令，例如 `clr=C:\dotnet\dotnet.exe`。
    #[arg(long = "runner", value_name = "target=command")]
    pub runner: Vec<String>,
    /// 直接指定产物路径，跳过默认探测。
    #[arg(long, value_name = "artifact")]
    pub artifact: Option<PathBuf>,
    /// 只打印即将执行的命令，不真正启动。
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct RunContract {
    pub(crate) logical_entry: String,
    pub(crate) physical_entry: String,
    pub(crate) invocation: String,
    pub(crate) validate: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunnerTemplate {
    target: RunnerFamily,
    command: String,
    args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RunCommand {
    target: RunnerFamily,
    artifact: PathBuf,
    command: String,
    args: Vec<String>,
}

/// 执行 `legion run`。
pub fn run(args: &RunArgs) -> Result<ExitCode> {
    let workspace = LegionWorkspace::discover(&args.project_dir)?;
    let request = BuildRequest { project_dir: args.project_dir.clone(), target: args.target.clone(), output_dir: args.output_dir.clone() };
    let (plan, fallback_to_package) =
        if args.workspace { (workspace.build_plan(&request)?, false) } else { workspace.build_plan_with_local_fallback(&request)? };

    let command = plan_run_command(
        &workspace,
        &plan.output_dir,
        &plan.project.name,
        &plan.project.build_target.target,
        &args.runner,
        args.artifact.as_deref(),
    )?;

    println!("project: {}", plan.project.name);
    println!("target: {}", plan.project.build_target.target);
    println!("output: {}", plan.output_dir.display());
    if fallback_to_package {
        println!("mode: package");
    }
    else {
        println!("mode: workspace");
    }
    println!("artifact: {}", command.artifact.display());
    println!("runner: {}", command.command);
    println!("args: {}", shell_join(&command.args));

    if args.dry_run {
        println!("run status: dry-run");
        return Ok(ExitCode::SUCCESS);
    }

    let status = Command::new(&command.command)
        .args(command.args.iter().map(OsString::from))
        .current_dir(&plan.project.manifest_dir)
        .status()
        .into_diagnostic()
        .map_err(|error| error.wrap_err(format!("failed to start runner '{}'", command.command)))?;

    Ok(exit_code_from_status(status.code()))
}

fn plan_run_command(
    workspace: &LegionWorkspace,
    output_dir: &Path,
    project_name: &str,
    canonical_target: &CanonicalTarget,
    cli_runner_overrides: &[String],
    artifact_override: Option<&Path>,
) -> Result<RunCommand> {
    let runner_target = runner_target_for(canonical_target);
    let run_contract = read_run_contract(output_dir)?;
    let artifact = match artifact_override {
        Some(path) => path.to_path_buf(),
        None => discover_artifact(output_dir, project_name, runner_target, run_contract.as_ref())?,
    };

    if !artifact.exists() {
        return Err(miette!("artifact does not exist: {}", artifact.display()));
    }

    let mut runner = resolve_runner(workspace, canonical_target, runner_target, cli_runner_overrides)?;
    apply_run_contract_to_runner(&mut runner, runner_target, &artifact, run_contract.as_ref());
    let placeholders = build_placeholders(output_dir, &artifact, runner_target, run_contract.as_ref())?;
    let command = expand_placeholders(&runner.command, &placeholders);
    let args = expand_runner_args(&runner.args, &placeholders);

    Ok(RunCommand { target: runner_target, artifact, command, args })
}

fn apply_run_contract_to_runner(
    runner: &mut RunnerTemplate,
    runner_target: RunnerFamily,
    _artifact: &Path,
    run_contract: Option<&RunContract>,
) {
    match runner_target {
        RunnerFamily::Clr if uses_default_clr_runner(runner) => {
            runner.command = "dotnet".to_string();
            runner.args = vec!["exec".to_string(), "{artifact}".to_string()];
        }
        RunnerFamily::Jvm if uses_default_jvm_runner(runner) => {
            if run_contract.is_some() {
                runner.command = "java".to_string();
                runner.args = vec!["-jar".to_string(), "{artifact}".to_string()];
            }
        }
        RunnerFamily::Wasi if uses_default_wasi_runner(runner) => {
            runner.command = WasiRuntime::LAUNCHER.to_string();
            runner.args = match run_contract {
                Some(contract) if !contract.logical_entry.is_empty() && !contract.logical_entry.eq_ignore_ascii_case("_start") => {
                    vec!["--invoke".to_string(), "{entry}".to_string(), "{artifact}".to_string()]
                }
                _ => vec!["{artifact}".to_string()],
            };
        }
        _ => {}
    }
}

fn uses_default_clr_runner(runner: &RunnerTemplate) -> bool {
    runner.target == RunnerFamily::Clr && runner.command == "dotnet" && runner.args == ["exec".to_string(), "{artifact}".to_string()]
}

fn uses_default_jvm_runner(runner: &RunnerTemplate) -> bool {
    runner.target == RunnerFamily::Jvm
        && runner.command == "java"
        && runner.args == ["-cp".to_string(), "{classpath}".to_string(), "{entry}".to_string()]
}

fn uses_default_wasi_runner(runner: &RunnerTemplate) -> bool {
    runner.target == RunnerFamily::Wasi
        && runner.command == "wasmtime"
        && runner.args == ["--invoke".to_string(), "main".to_string(), "{artifact}".to_string()]
}

fn resolve_runner(
    workspace: &LegionWorkspace,
    canonical_target: &CanonicalTarget,
    runner_target: RunnerFamily,
    cli_runner_overrides: &[String],
) -> Result<RunnerTemplate> {
    if let Some(command) = parse_runner_overrides(cli_runner_overrides)?.remove(&runner_target) {
        let mut template = default_runner(runner_target);
        template.command = command;
        return Ok(template);
    }

    if let Some(workspace_manifest) = &workspace.workspace_manifest {
        if let Some(binding) = workspace_manifest.runner.iter().find(|binding| runner_binding_matches(binding, runner_target, canonical_target))
        {
            return Ok(RunnerTemplate { target: runner_target, command: binding.command.clone(), args: binding.args.clone() });
        }
    }

    if let Ok(command) = std::env::var(format!("LEGION_RUNNER_{}", runner_target.as_str().to_ascii_uppercase())) {
        if !command.trim().is_empty() {
            let mut template = default_runner(runner_target);
            template.command = command;
            return Ok(template);
        }
    }

    Ok(default_runner(runner_target))
}

fn parse_runner_overrides(values: &[String]) -> Result<BTreeMap<RunnerFamily, String>> {
    let mut overrides = BTreeMap::new();
    for item in values {
        let Some((target, command)) = item.split_once('=')
        else {
            return Err(miette!("invalid runner override '{}': expected target=command", item));
        };
        let family = target.trim().parse::<RunnerFamily>().map_err(|error| miette!("invalid runner override '{}': {}", item, error))?;
        overrides.insert(family, command.trim().to_string());
    }
    Ok(overrides)
}

fn runner_binding_matches(binding: &RunnerBinding, runner_target: RunnerFamily, canonical_target: &CanonicalTarget) -> bool {
    binding.target.matches(runner_target, canonical_target)
}

fn default_runner(target: RunnerFamily) -> RunnerTemplate {
    match target {
        RunnerFamily::Clr => RunnerTemplate {
            target: RunnerFamily::Clr,
            command: "dotnet".to_string(),
            args: vec!["exec".to_string(), "{artifact}".to_string()],
        },
        RunnerFamily::Jvm => RunnerTemplate {
            target: RunnerFamily::Jvm,
            command: "java".to_string(),
            args: vec!["-cp".to_string(), "{classpath}".to_string(), "{entry}".to_string()],
        },
        RunnerFamily::Node => RunnerTemplate { target: RunnerFamily::Node, command: "node".to_string(), args: vec!["{artifact}".to_string()] },
        RunnerFamily::Windows => RunnerTemplate { target: RunnerFamily::Windows, command: "{artifact}".to_string(), args: Vec::new() },
        RunnerFamily::Wasi => RunnerTemplate {
            target: RunnerFamily::Wasi,
            command: WasiRuntime::LAUNCHER.to_string(),
            args: vec!["--invoke".to_string(), "main".to_string(), "{artifact}".to_string()],
        },
    }
}

fn runner_target_for(canonical_target: &CanonicalTarget) -> RunnerFamily {
    canonical_target.to_profile(None).runner_family()
}

fn build_placeholders(
    output_dir: &Path,
    artifact: &Path,
    runner_target: RunnerFamily,
    run_contract: Option<&RunContract>,
) -> Result<BTreeMap<&'static str, String>> {
    let mut values = BTreeMap::new();
    let artifact_text = strip_verbatim_prefix(artifact.to_string_lossy().as_ref()).to_owned();
    values.insert("artifact", artifact_text.clone());

    let classpath = if artifact.extension().is_some_and(|ext| ext.eq_ignore_ascii_case("jar")) {
        artifact_text.clone()
    }
    else {
        strip_verbatim_prefix(output_dir.to_string_lossy().as_ref()).to_owned()
    };
    values.insert("classpath", classpath);

    let entry = match run_contract {
        Some(contract) if runner_target == RunnerFamily::Jvm && !contract.logical_entry.is_empty() => contract.logical_entry.clone(),
        Some(contract) if !contract.physical_entry.is_empty() => contract.physical_entry.clone(),
        Some(contract) if !contract.logical_entry.is_empty() => contract.logical_entry.clone(),
        _ if runner_target == RunnerFamily::Jvm => artifact
            .file_stem()
            .and_then(|stem| stem.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| miette!("cannot infer JVM entry from {}", artifact.display()))?,
        _ => artifact.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default().to_string(),
    };
    values.insert("entry", entry);
    Ok(values)
}

fn expand_runner_args(args: &[String], placeholders: &BTreeMap<&'static str, String>) -> Vec<String> {
    args.iter().map(|value| expand_placeholders(value, placeholders)).collect()
}

fn expand_placeholders(template: &str, placeholders: &BTreeMap<&'static str, String>) -> String {
    placeholders.iter().fold(template.to_string(), |current, (key, value)| current.replace(&format!("{{{}}}", key), value))
}

fn read_run_contract(output_dir: &Path) -> Result<Option<RunContract>> {
    let path = output_dir.join("run-contract.txt");
    if !path.exists() {
        return Ok(None);
    }

    let source = fs::read_to_string(&path)
        .into_diagnostic()
        .map_err(|error| error.wrap_err(format!("failed to read run contract '{}'", path.display())))?;
    let mut logical_entry = String::new();
    let mut physical_entry = String::new();
    let mut invocation = String::new();
    let mut validate = String::new();

    for line in source.lines() {
        let Some((key, value)) = line.split_once(':')
        else {
            continue;
        };
        let value = parse_von_scalar(value);
        match key.trim() {
            "logical_entry" => logical_entry = value,
            "physical_entry" => physical_entry = value,
            "invocation" => invocation = value,
            "validate" => validate = value,
            _ => {}
        }
    }

    Ok(Some(RunContract { logical_entry, physical_entry, invocation, validate }))
}

/// 解析 `run-contract.txt` 中的简单 `VON` 标量字符串。
fn parse_von_scalar(value: &str) -> String {
    value.trim().trim_end_matches(',').trim().trim_matches('"').to_string()
}

fn discover_artifact(
    output_dir: &Path,
    project_name: &str,
    runner_target: RunnerFamily,
    run_contract: Option<&RunContract>,
) -> Result<PathBuf> {
    let mut files = collect_files(output_dir)?;
    files.sort();

    if let Some(contract) = run_contract {
        if let Some(path) = find_contract_artifact(&files, &contract.physical_entry) {
            return Ok(path);
        }
    }

    let preferred = preferred_extensions(runner_target);
    for extension in preferred {
        if let Some(path) = files.iter().find(|path| has_extension(path, extension) && matches_project_name(path, project_name)).cloned() {
            return Ok(path);
        }
    }

    for extension in preferred {
        if let Some(path) = files.iter().find(|path| has_extension(path, extension)).cloned() {
            return Ok(path);
        }
    }

    Err(miette!("no runnable artifact found in '{}' for target '{}'", output_dir.display(), runner_target))
}

fn collect_files(dir: &Path) -> Result<Vec<PathBuf>> {
    if !dir.exists() {
        return Err(miette!("output directory does not exist: {}", dir.display()));
    }

    let mut files = Vec::new();
    for entry in fs::read_dir(dir).into_diagnostic().map_err(|error| error.wrap_err(format!("failed to list '{}'", dir.display())))? {
        let entry = entry.into_diagnostic().map_err(|error| error.wrap_err(format!("failed to read '{}'", dir.display())))?;
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }
    Ok(files)
}

fn find_contract_artifact(files: &[PathBuf], physical_entry: &str) -> Option<PathBuf> {
    if physical_entry.is_empty() {
        return None;
    }

    files.iter().find_map(|path| {
        let file_name = path.file_name()?.to_str()?;
        let stem = path.file_stem()?.to_str()?;
        if file_name.eq_ignore_ascii_case(physical_entry) || stem.eq_ignore_ascii_case(physical_entry) {
            Some(path.clone())
        }
        else {
            None
        }
    })
}

fn preferred_extensions(target: RunnerFamily) -> &'static [&'static str] {
    match target {
        RunnerFamily::Clr => &["dll", "exe"],
        RunnerFamily::Jvm => &["jar", "class"],
        RunnerFamily::Node => &["mjs", "js", "wasm"],
        RunnerFamily::Windows => &["exe"],
        RunnerFamily::Wasi => &["wasm"],
    }
}

fn matches_project_name(path: &Path, project_name: &str) -> bool {
    let Some(stem) = path.file_stem().and_then(|value| value.to_str())
    else {
        return false;
    };
    stem.eq_ignore_ascii_case(project_name)
        || stem.replace('_', ".").eq_ignore_ascii_case(project_name)
        || stem.replace('.', "_").eq_ignore_ascii_case(project_name)
}

fn has_extension(path: &Path, expected: &str) -> bool {
    path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case(expected))
}

fn shell_join(args: &[String]) -> String {
    if args.is_empty() {
        return String::new();
    }

    args.iter()
        .map(|value| {
            if value.contains(' ') {
                format!("\"{}\"", value)
            }
            else {
                value.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn exit_code_from_status(code: Option<i32>) -> ExitCode {
    match code {
        Some(value) if (0..=255).contains(&value) => ExitCode::from(value as u8),
        Some(_) => ExitCode::from(1),
        None => ExitCode::from(1),
    }
}

/// 移除 Windows extended-length path 前缀 `\\?\`，避免 Node/wasmtime 等外部工具无法解析。
fn strip_verbatim_prefix(path: &str) -> &str {
    path.strip_prefix(r"\\?\").unwrap_or(path)
}

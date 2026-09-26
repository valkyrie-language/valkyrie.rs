use std::{
    collections::BTreeMap,
    ffi::OsString,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode},
};

use clap::Args;
use miette::{IntoDiagnostic, Result, WrapErr, miette};
use nyar_language::{CanonicalAbi, CanonicalTarget, RunnerFamily};
use nyar_runner::{RuntimeContract as InterpreterRuntimeContract, RuntimeFamily as InterpreterRuntimeFamily};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    cmds::build::{BuildArgs, run as run_build},
    manifest::RunnerBinding,
    planner::{BuildPlan, BuildRequest, LegionWorkspace, ProjectResolutionMode},
};

const EXECUTION_MANIFEST_FILE_NAME: &str = "run-contracts.txt";
const LEGACY_RUN_CONTRACT_FILE_NAME: &str = "run-contract.txt";
const HOST_SELECTION_FILE_NAME: &str = "host-selection.txt";
/// 编译计划快照文件名，由构建流程写入，不属于交付产物。
const COMPILE_PLAN_SNAPSHOT_FILE_NAME: &str = "compile-plan.txt";
/// 后端执行请求快照文件名，由构建流程写入，不属于交付产物。
const BACKEND_REQUEST_SNAPSHOT_FILE_NAME: &str = "backend-request.txt";
/// 后端执行结果快照文件名，由构建流程写入，不属于交付产物。
const BACKEND_RESULT_SNAPSHOT_FILE_NAME: &str = "backend-result.txt";
const EXECUTION_MANIFEST_SCHEMA_VERSION: u32 = 1;

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
    /// 输出调试用构建副产物。
    #[arg(long, default_value_t = false)]
    pub debug_artifacts: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunContract {
    pub logical_entry: String,
    pub physical_entry: String,
    pub invocation: String,
    pub validate: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionManifest {
    pub schema_version: u32,
    pub project_name: String,
    pub target: String,
    pub inputs: Vec<ExecutionInputDigest>,
    pub artifacts: Vec<ExecutionArtifactDigest>,
    pub run_contracts: Vec<RunContract>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionInputDigest {
    pub role: String,
    pub path: String,
    pub hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionArtifactDigest {
    pub path: String,
    pub hash: String,
}

impl ExecutionManifest {
    /// 将 execution manifest 写入输出目录中的 `run-contracts.txt`。
    pub fn write_to_output_dir(&self, output_dir: &Path) -> Result<()> {
        let manifest_path = output_dir.join(EXECUTION_MANIFEST_FILE_NAME);
        let content =
            crate::write_von_indented(self).wrap_err_with(|| format!("序列化 execution manifest 失败 {}", manifest_path.display()))?;
        fs::write(&manifest_path, content)
            .into_diagnostic()
            .map_err(|error| error.wrap_err(format!("写入 execution manifest 失败 {}", manifest_path.display())))?;

        if let Some(primary_contract) = self.run_contracts.first() {
            primary_contract.write_legacy_to_output_dir(output_dir)?;
        }
        else {
            RunContract::remove_legacy_from_output_dir(output_dir)?;
        }

        Ok(())
    }

    /// 从输出目录读取 execution manifest。
    pub fn read_from_output_dir(output_dir: &Path) -> Result<Option<Self>> {
        Self::read_from_path(&output_dir.join(EXECUTION_MANIFEST_FILE_NAME))
    }

    /// 从指定路径读取 execution manifest。
    pub fn read_from_path(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }
        let source = fs::read_to_string(path)
            .into_diagnostic()
            .map_err(|error| error.wrap_err(format!("failed to read execution manifest '{}'", path.display())))?;
        crate::parse_von::<Self>(&source)
            .map(Some)
            .map_err(|error| miette!("failed to parse execution manifest '{}': {}", path.display(), error))
    }

    /// 删除输出目录中的 execution manifest。
    pub fn remove_from_output_dir(output_dir: &Path) -> Result<()> {
        let manifest_path = output_dir.join(EXECUTION_MANIFEST_FILE_NAME);
        if manifest_path.exists() {
            fs::remove_file(&manifest_path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("删除 execution manifest 失败 {}", manifest_path.display())))?;
        }
        RunContract::remove_legacy_from_output_dir(output_dir)?;
        Ok(())
    }

    /// 基于当前 `BuildPlan` 与运行契约生成 execution manifest。
    pub fn from_build_plan(plan: &BuildPlan, contracts: &[RunContract]) -> Result<Self> {
        Ok(Self {
            schema_version: EXECUTION_MANIFEST_SCHEMA_VERSION,
            project_name: plan.project.name.clone(),
            target: plan.project.build_target.target.to_string(),
            inputs: collect_execution_input_digests(plan)?,
            artifacts: collect_execution_artifact_digests(&plan.output_dir)?,
            run_contracts: contracts.to_vec(),
        })
    }

    /// 校验当前 execution manifest 是否仍与 `BuildPlan` 及产物目录匹配。
    pub fn is_fresh_for_plan(&self, plan: &BuildPlan) -> Result<bool> {
        if self.schema_version != EXECUTION_MANIFEST_SCHEMA_VERSION {
            return Ok(false);
        }
        if self.project_name != plan.project.name {
            return Ok(false);
        }
        if self.target != plan.project.build_target.target.to_string() {
            return Ok(false);
        }
        if self.inputs != collect_execution_input_digests(plan)? {
            return Ok(false);
        }
        if self.run_contracts.is_empty() || self.artifacts.is_empty() {
            return Ok(false);
        }

        for artifact in &self.artifacts {
            let artifact_path = plan.output_dir.join(PathBuf::from(&artifact.path));
            if !artifact_path.exists() {
                return Ok(false);
            }
            if hash_file_sha256(&artifact_path)? != artifact.hash {
                return Ok(false);
            }
        }

        Ok(true)
    }
}

impl RunContract {
    /// 从输出目录读取主运行契约。
    pub fn read_from_output_dir(output_dir: &Path) -> Result<Option<Self>> {
        Ok(Self::read_all_from_output_dir(output_dir)?.into_iter().next())
    }

    /// 从输出目录读取运行契约列表。
    pub fn read_all_from_output_dir(output_dir: &Path) -> Result<Vec<Self>> {
        if let Some(manifest) = ExecutionManifest::read_from_output_dir(output_dir)? {
            return Ok(manifest.run_contracts);
        }

        Ok(Self::read_legacy_from_output_dir(output_dir)?.into_iter().collect())
    }

    fn matches_artifact(&self, artifact: &Path) -> bool {
        let Some(file_name) = artifact.file_name().and_then(|value| value.to_str())
        else {
            return false;
        };
        if file_name.eq_ignore_ascii_case(&self.physical_entry) {
            return true;
        }
        crate::bootstrap_entry_aliases(&self.physical_entry).iter().any(|alias| file_name.eq_ignore_ascii_case(alias))
    }

    fn write_legacy_to_output_dir(&self, output_dir: &Path) -> Result<()> {
        let contract_path = output_dir.join(LEGACY_RUN_CONTRACT_FILE_NAME);
        let content = crate::write_von_indented(self).wrap_err_with(|| format!("序列化运行契约失败 {}", contract_path.display()))?;
        fs::write(&contract_path, content)
            .into_diagnostic()
            .map_err(|error| error.wrap_err(format!("写入运行契约失败 {}", contract_path.display())))
    }

    fn read_legacy_from_output_dir(output_dir: &Path) -> Result<Option<Self>> {
        Self::read_legacy_from_path(&output_dir.join(LEGACY_RUN_CONTRACT_FILE_NAME))
    }

    fn read_legacy_from_path(path: &Path) -> Result<Option<Self>> {
        if !path.exists() {
            return Ok(None);
        }

        let source = fs::read_to_string(path)
            .into_diagnostic()
            .map_err(|error| error.wrap_err(format!("failed to read run contract '{}'", path.display())))?;
        crate::parse_von::<Self>(&source).map(Some).map_err(|error| miette!("failed to parse run contract '{}': {}", path.display(), error))
    }

    fn remove_legacy_from_output_dir(output_dir: &Path) -> Result<()> {
        let contract_path = output_dir.join(LEGACY_RUN_CONTRACT_FILE_NAME);
        if contract_path.exists() {
            fs::remove_file(&contract_path)
                .into_diagnostic()
                .map_err(|error| error.wrap_err(format!("删除运行契约失败 {}", contract_path.display())))?;
        }
        Ok(())
    }
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
    let workspace =
        if args.workspace { LegionWorkspace::discover(&args.project_dir)? } else { LegionWorkspace::discover_for_project(&args.project_dir)? };
    let request = BuildRequest { project_dir: args.project_dir.clone(), target: args.target.clone(), output_dir: args.output_dir.clone() };
    let (plan, resolution_mode) = if args.workspace {
        (workspace.build_plan(&request)?, ProjectResolutionMode::Workspace)
    }
    else {
        workspace.build_plan_with_local_fallback(&request)?
    };

    let execution_manifest = ensure_execution_manifest(args, &plan)?;
    let command = plan_run_command(
        &workspace,
        &plan.output_dir,
        &plan.project.name,
        &plan.project.build_target.target,
        &args.runner,
        &execution_manifest.run_contracts,
        args.artifact.as_deref(),
    )?;

    println!("project: {}", plan.project.name);
    println!("target: {}", plan.project.build_target.target);
    println!("output: {}", plan.output_dir.display());
    match resolution_mode {
        ProjectResolutionMode::Workspace => {
            println!("mode: workspace");
        }
        ProjectResolutionMode::Package => {
            println!("mode: package");
        }
        ProjectResolutionMode::Script => {
            println!("mode: script");
        }
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
    run_contracts: &[RunContract],
    artifact_override: Option<&Path>,
) -> Result<RunCommand> {
    let runner_target = runner_target_for(canonical_target);
    let artifact = match artifact_override {
        Some(path) => path.to_path_buf(),
        None => discover_artifact(output_dir, project_name, runner_target, run_contracts.first())?,
    };

    if !artifact.exists() {
        return Err(miette!("artifact does not exist: {}", artifact.display()));
    }

    let run_contract = run_contracts.iter().find(|contract| contract.matches_artifact(&artifact)).or_else(|| run_contracts.first());
    let runner = resolve_runner(workspace, canonical_target, runner_target, cli_runner_overrides, run_contract)?;
    let placeholders = build_placeholders(output_dir, &artifact, runner_target, run_contract)?;
    let command = expand_placeholders(&runner.command, &placeholders);
    let args = expand_runner_args(&runner.args, &placeholders);

    Ok(RunCommand { target: runner_target, artifact, command, args })
}

fn ensure_execution_manifest(args: &RunArgs, plan: &BuildPlan) -> Result<ExecutionManifest> {
    if let Some(manifest) = ExecutionManifest::read_from_output_dir(&plan.output_dir)? {
        if manifest.is_fresh_for_plan(plan)? {
            println!("build status: reused");
            return Ok(manifest);
        }
        println!("build status: stale");
    }
    else {
        println!("build status: missing");
    }

    let build_status = run_build(&BuildArgs {
        project_dir: args.project_dir.clone(),
        target: args.target.clone(),
        output_dir: args.output_dir.clone(),
        workspace: args.workspace,
        debug_artifacts: args.debug_artifacts,
    })?;
    if build_status != ExitCode::SUCCESS {
        return Err(miette!("build returned non-zero status"));
    }

    let manifest = ExecutionManifest::read_from_output_dir(&plan.output_dir)?
        .ok_or_else(|| miette!("missing execution manifest after build: {}", plan.output_dir.join(EXECUTION_MANIFEST_FILE_NAME).display()))?;
    if !manifest.is_fresh_for_plan(plan)? {
        return Err(miette!("execution manifest is still stale after build"));
    }

    Ok(manifest)
}

fn resolve_runner(
    workspace: &LegionWorkspace,
    canonical_target: &CanonicalTarget,
    runner_target: RunnerFamily,
    cli_runner_overrides: &[String],
    run_contract: Option<&RunContract>,
) -> Result<RunnerTemplate> {
    let mut default_template = default_runner_template(runner_target, run_contract);
    if runner_target == RunnerFamily::Wasi && is_wasip3_target(canonical_target) {
        inject_wasmtime_p3_flag(&mut default_template.args);
    }

    if let Some(command) = parse_runner_overrides(cli_runner_overrides)?.remove(&runner_target) {
        let mut template = default_template.clone();
        template.command = command;
        return Ok(template);
    }

    if let Some(workspace_manifest) = &workspace.workspace_manifest {
        if let Some(binding) = workspace_manifest.runner.iter().find(|binding| runner_binding_matches(binding, runner_target, canonical_target))
        {
            let mut template = RunnerTemplate { target: runner_target, command: binding.command.clone(), args: binding.args.clone() };
            if runner_target == RunnerFamily::Wasi && is_wasip3_target(canonical_target) {
                inject_wasmtime_p3_flag(&mut template.args);
            }
            return Ok(template);
        }
    }

    if let Ok(command) = std::env::var(format!("LEGION_RUNNER_{}", runner_target.as_str().to_ascii_uppercase())) {
        if !command.trim().is_empty() {
            let mut template = default_template.clone();
            template.command = command;
            return Ok(template);
        }
    }

    Ok(default_template)
}

fn is_wasip3_target(canonical_target: &CanonicalTarget) -> bool {
    matches!(canonical_target.abi, Some(CanonicalAbi::WasiP3))
        || canonical_target.to_string().contains("wasip3")
        || canonical_target.to_profile(None).capability_tags.iter().any(|tag| tag == "wasip3")
}

fn inject_wasmtime_p3_flag(args: &mut Vec<String>) {
    if args.windows(2).any(|pair| pair[0] == "-S" && pair[1] == "p3") {
        return;
    }
    let insert_at = args.len().saturating_sub(1);
    args.insert(insert_at, "-S".to_string());
    args.insert(insert_at + 1, "p3".to_string());
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

fn default_runner_template(target: RunnerFamily, run_contract: Option<&RunContract>) -> RunnerTemplate {
    let template = runtime_family_for(target).default_template(interpreter_runtime_contract(run_contract));
    RunnerTemplate { target, command: template.command, args: template.args }
}

fn runner_target_for(canonical_target: &CanonicalTarget) -> RunnerFamily {
    canonical_target.to_profile(None).runner_family()
}

fn runtime_family_for(target: RunnerFamily) -> InterpreterRuntimeFamily {
    match target {
        RunnerFamily::Clr => InterpreterRuntimeFamily::Clr,
        RunnerFamily::Jvm => InterpreterRuntimeFamily::Jvm,
        RunnerFamily::Node => InterpreterRuntimeFamily::Node,
        RunnerFamily::Windows => InterpreterRuntimeFamily::Windows,
        RunnerFamily::Wasi => InterpreterRuntimeFamily::Wasi,
        RunnerFamily::NyarVm => InterpreterRuntimeFamily::NyarVm,
    }
}

fn interpreter_runtime_contract(run_contract: Option<&RunContract>) -> Option<InterpreterRuntimeContract<'_>> {
    run_contract.map(|contract| InterpreterRuntimeContract {
        logical_entry: (!contract.logical_entry.is_empty()).then_some(contract.logical_entry.as_str()),
        physical_entry: (!contract.physical_entry.is_empty()).then_some(contract.physical_entry.as_str()),
        wasi_p3: contract.validate.contains("-S p3") || contract.validate.contains(" p3"),
    })
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

    for candidate in crate::bootstrap_entry_aliases(physical_entry).iter().chain(std::iter::once(&physical_entry)) {
        if let Some(path) = files.iter().find_map(|path| {
            let file_name = path.file_name()?.to_str()?;
            let stem = path.file_stem()?.to_str()?;
            if file_name.eq_ignore_ascii_case(candidate) || stem.eq_ignore_ascii_case(candidate) { Some(path.clone()) } else { None }
        }) {
            return Some(path);
        }
    }
    None
}

fn preferred_extensions(target: RunnerFamily) -> &'static [&'static str] {
    match target {
        RunnerFamily::Clr => &["dll", "exe"],
        RunnerFamily::Jvm => &["jar", "class"],
        RunnerFamily::Node => &["mjs", "js", "wasm"],
        RunnerFamily::Windows => &["exe"],
        RunnerFamily::Wasi => &["wasi", "wasm"],
        RunnerFamily::NyarVm => &["nyar"],
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

    args.iter().map(|value| if value.contains(' ') { format!("\"{}\"", value) } else { value.clone() }).collect::<Vec<_>>().join(" ")
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

fn collect_execution_input_digests(plan: &BuildPlan) -> Result<Vec<ExecutionInputDigest>> {
    let mut inputs = Vec::new();
    for source_path in &plan.project.source_files {
        inputs.push(ExecutionInputDigest {
            role: "source".to_string(),
            path: normalized_path_text(source_path),
            hash: hash_file_sha256(source_path)?,
        });
    }

    inputs.push(ExecutionInputDigest {
        role: "project-manifest".to_string(),
        path: normalized_path_text(&plan.project.manifest_path),
        hash: hash_file_sha256(&plan.project.manifest_path)?,
    });

    let workspace_manifest_path = plan.workspace_root.join("legions.von");
    if workspace_manifest_path.exists() {
        inputs.push(ExecutionInputDigest {
            role: "workspace-manifest".to_string(),
            path: normalized_path_text(&workspace_manifest_path),
            hash: hash_file_sha256(&workspace_manifest_path)?,
        });
    }

    inputs.sort_by(|left, right| left.role.cmp(&right.role).then(left.path.cmp(&right.path)));
    Ok(inputs)
}

fn collect_execution_artifact_digests(output_dir: &Path) -> Result<Vec<ExecutionArtifactDigest>> {
    let mut files = collect_files(output_dir)?;
    files.retain(|path| !is_execution_metadata_file(path));
    files.sort();

    files
        .into_iter()
        .map(|path| Ok(ExecutionArtifactDigest { path: relative_output_path(output_dir, &path), hash: hash_file_sha256(&path)? }))
        .collect()
}

fn is_execution_metadata_file(path: &Path) -> bool {
    path.file_name().and_then(|value| value.to_str()).is_some_and(|value| {
        value.eq_ignore_ascii_case(EXECUTION_MANIFEST_FILE_NAME)
            || value.eq_ignore_ascii_case(LEGACY_RUN_CONTRACT_FILE_NAME)
            || value.eq_ignore_ascii_case(HOST_SELECTION_FILE_NAME)
            || value.eq_ignore_ascii_case(COMPILE_PLAN_SNAPSHOT_FILE_NAME)
            || value.eq_ignore_ascii_case(BACKEND_REQUEST_SNAPSHOT_FILE_NAME)
            || value.eq_ignore_ascii_case(BACKEND_RESULT_SNAPSHOT_FILE_NAME)
    })
}

fn relative_output_path(output_dir: &Path, path: &Path) -> String {
    path.strip_prefix(output_dir).unwrap_or(path).to_string_lossy().replace('\\', "/")
}

fn normalized_path_text(path: &Path) -> String {
    strip_verbatim_prefix(path.to_string_lossy().as_ref()).to_owned()
}

fn hash_file_sha256(path: &Path) -> Result<String> {
    let bytes = fs::read(path).into_diagnostic().map_err(|error| error.wrap_err(format!("failed to read '{}'", path.display())))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex_encode(&hasher.finalize()))
}

fn hex_encode(bytes: &[u8]) -> String {
    let mut text = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut text, "{byte:02x}");
    }
    text
}

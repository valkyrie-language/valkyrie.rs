// `legion build` 命令。
//
// 这里只负责装配与落盘；目标解释、后端注册和 lowering 均交给 `emitter`。

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{Arc, mpsc},
    thread,
};
use legion_workspace::manifest::{ProjectArtifactKind, ProjectManifest};
use legion_workspace::planner::{BuildPlan, BuildRequest, WorkspaceResolver};
use legion_workspace::source_snapshot::compile_source_snapshot;

use clap::Args;
use emitter::DriverRunContract;
use miette::{IntoDiagnostic, Report, Result, WrapErr, miette};
use nyar_language::{
    ArtifactSet, CanonicalSpecification, CanonicalTarget, compile_source_groups_to_artifacts,
    nyar::ClrSuspendStrategy,
};
use serde::Serialize;

use crate::{
    cache::{
        CompilationCache, cache_root_for, collect_build_bundle, compute_artifact_hash, store_cached_build,
    },
    cmds::{
        project_input::resolve_project_path,
        run::{ExecutionManifest, RunContract},
        source_hygiene,
    },
    script,
    unity_export, write_von_indented,
};

/// `legion build` 的命令参数。
#[derive(Debug, Clone, Args)]
pub struct BuildArgs {
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
    /// 输出调试用构建副产物。
    #[arg(long, default_value_t = false)]
    pub debug_artifacts: bool,
}

/// 执行 `legion build`。
pub fn run(args: &BuildArgs) -> Result<ExitCode> {
    if args.workspace {
        return run_workspace_members_in_parallel(args);
    }

    let project_input = resolve_project_path(&args.project_dir)?;
    let workspace = WorkspaceResolver::discover_for_project(&project_input)?;
    if workspace.is_workspace_only_root(&project_input) {
        return run_workspace_members_in_parallel(args);
    }

    let request = BuildRequest { project_dir: project_input, target: args.target.clone(), output_dir: args.output_dir.clone() };
    let plan = workspace.build_plan(&request)?;

    println!("workspace: {}", crate::cmds::path_for_cli_log(&plan.workspace_root));
    println!("mode: workspace");
    println!("project: {}", plan.project.name);
    println!("target: {}", plan.project.build_target.target);
    println!("output: {}", crate::cmds::path_for_cli_log(&plan.output_dir));
    println!("sources: {}", plan.project.source_files.len());
    println!("host contracts: {}", plan.project.host_contracts.len());
    println!("selected host providers: {}", plan.project.selected_host_providers.len());

    if plan.project.source_files.is_empty() {
        return Err(miette!("没有找到任何源码文件"));
    }

    if args.debug_artifacts {
        write_host_selection_spec(&plan.output_dir, &plan.project.selected_host_providers)?;
    }
    else {
        remove_host_selection_spec(&plan.output_dir)?;
    }

    let report = compile_plan(&plan, true)?;
    println!("backend status: compiled");
    let target_profile = plan.project.build_target.target.to_profile(None);
    println!("host kind: {:?}", target_profile.host_kind);
    println!("publish format: {}", target_profile.artifact_policy.default_publish_format);
    if let Some(entry_symbol) = &report.entry_symbol {
        println!("entry: {}", entry_symbol);
    }
    print_artifacts(&plan.output_dir, &report.artifacts);
    let manifest_source = if script::is_script_path(&plan.project.manifest_path) {
        let script_source = fs::read_to_string(&plan.project.manifest_path)
            .into_diagnostic()
            .wrap_err_with(|| format!("读取单脚本失败：{}", plan.project.manifest_path.display()))?;
        script::extract_embedded_manifest(&script_source, &plan.project.manifest_path)
            .map_err(|error| Report::from(error))?
            .ok_or_else(|| miette!("单脚本 `{}` 缺少内嵌 `# ```legion` 块", plan.project.manifest_path.display()))?
    }
    else {
        fs::read_to_string(&plan.project.manifest_path)
            .into_diagnostic()
            .wrap_err_with(|| format!("读取项目清单失败：{}", plan.project.manifest_path.display()))?
    };
    let project_manifest = ProjectManifest::parse(&manifest_source)?;
    if let Some(plugin) = &project_manifest.build_plugin {
        unity_export::export_unity_project(&plan, &report, plugin)?;
        println!("unity export: {}", plugin.kind);
    }

    Ok(ExitCode::SUCCESS)
}

fn run_workspace_members_in_parallel(args: &BuildArgs) -> Result<ExitCode> {
    let workspace = WorkspaceResolver::discover(&args.project_dir)?;
    let requested_target = args.target.clone();
    let members = workspace
        .member_manifest_dirs()
        .into_iter()
        .filter(|project_dir| {
            let request =
                BuildRequest { project_dir: project_dir.clone(), target: requested_target.clone(), output_dir: args.output_dir.clone() };
            match workspace.build_plan(&request) {
                Ok(_) => true,
                Err(legion_workspace::planner::PlannerError::MissingBuildTarget { .. }) => {
                    eprintln!("workspace: skipping {} (target not declared)", project_dir.display());
                    false
                }
                Err(error) => {
                    eprintln!("workspace: retaining {} after planner error: {}", project_dir.display(), error);
                    true
                }
            }
        })
        .collect::<Vec<_>>();
    if members.is_empty() {
        return Err(miette!("workspace 没有可构建成员"));
    }

    let worker_count = thread::available_parallelism().map(|value| value.get()).unwrap_or(4).min(members.len());
    println!("workspace mode: 并行构建 {} 个成员 (workers={worker_count})", members.len());

    let queue = Arc::new(std::sync::Mutex::new(members));
    let (tx, rx) = mpsc::channel::<(PathBuf, std::result::Result<ExitCode, String>)>();
    let mut handles = Vec::new();

    for _ in 0..worker_count {
        let queue = Arc::clone(&queue);
        let tx = tx.clone();
        let target = args.target.clone();
        let output = args.output_dir.clone();
        let debug_artifacts = args.debug_artifacts;
        // Backend lowering can recurse deeply over large workspaces. Keep the
        // debug seed usable without relying on a release build's larger stack.
        handles.push(
            thread::Builder::new()
                .name("legion-build-worker".into())
                .stack_size(64 * 1024 * 1024)
                .spawn(move || {
                    loop {
                        let next = {
                            let mut guard = queue.lock().expect("workspace queue poisoned");
                            guard.pop()
                        };
                        let Some(project_dir) = next
                        else {
                            break;
                        };

                        let member_args = BuildArgs {
                            project_dir: project_dir.clone(),
                            target: target.clone(),
                            output_dir: output.clone(),
                            workspace: false,
                            debug_artifacts,
                        };
                        let result = match run(&member_args) {
                            Ok(code) => Ok(code),
                            Err(error) => Err(error.to_string()),
                        };
                        let _ = tx.send((project_dir, result));
                    }
                })
                .map_err(|error| miette!("failed to spawn workspace build worker: {error}"))?,
        );
    }
    drop(tx);

    let mut failures = Vec::new();
    for (project_dir, result) in rx {
        match result {
            Ok(code) if code == ExitCode::SUCCESS => {}
            Ok(_) => failures.push(format!("{}: exited with failure code", project_dir.display())),
            Err(error) => failures.push(format!("{}: {}", project_dir.display(), error)),
        }
    }
    for handle in handles {
        let _ = handle.join();
    }

    if failures.is_empty() {
        println!("workspace 构建完成：全部成功");
        Ok(ExitCode::SUCCESS)
    }
    else {
        for failure in &failures {
            eprintln!("workspace build failure: {failure}");
        }
        Ok(ExitCode::FAILURE)
    }
}

/// 编译给定 `BuildPlan`，返回驱动编译报告。
pub(crate) fn compile_plan(plan: &BuildPlan, verbose: bool) -> Result<emitter::DriverCompileReport> {
    if plan.project.source_files.is_empty() {
        return Err(miette!("没有找到任何源码文件"));
    }

    // Source hygiene runs on every build (including cache hits): size + encoding.
    let hygiene = source_hygiene::scan_build_sources(&plan.project.source_files)?;
    source_hygiene::require_clean(&hygiene)?;

    let cache = CompilationCache::open(cache_root_for(&plan.workspace_root));
    let canonical_triple = plan.project.build_target.target.as_canonical_str();
    let manifest_files = plan
        .project
        .semantic_source_groups
        .iter()
        .map(|group| group.manifest_dir.join("legion.von"))
        .chain(std::iter::once(plan.project.manifest_path.clone()))
        .collect::<Vec<_>>();
    let ir_hash = compute_artifact_hash(
        &plan.project.source_files,
        &manifest_files,
        &canonical_triple,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        plan.project.build_target.runtime_async,
    )
    .map_err(|error| miette!("{error}"))?;

    // WASI 与 Node 同为 wasm32，但宿主不同：Node 走 `env.*`，WASI 走 guest/host_contract。
    // 模板 `<% match arch %>` 因此对 WASI 使用伪架构键 `"wasi"`，避免误选 wasm32/Node 分支。
    let arch = if plan.project.build_target.target.specification == CanonicalSpecification::Wasi {
        "wasi"
    }
    else {
        plan.project.build_target.target.arch.as_str()
    };
    let source_groups = compile_source_snapshot(&plan.project.semantic_source_groups)?;
    if verbose {
        println!("frontend: semantic source groups={}", plan.project.semantic_source_groups.len());
    }
    let target_profile = plan.project.build_target.target.to_profile(None);
    let clr_suspend_strategy = ClrSuspendStrategy::from_runtime_async_flag(plan.project.build_target.runtime_async);
    let wasm_package_kind = wasm_package_kind_for_manifest(plan.project.artifact_kind);
    fs::create_dir_all(&plan.output_dir).into_diagnostic().wrap_err_with(|| format!("创建输出目录失败 {}", plan.output_dir.display()))?;
    let report = compile_source_groups_to_artifacts(
        &nyar_language::ValkyrieCompiler::default(),
        &source_groups,
        arch,
        plan.project.build_target.target.clone(),
        clr_suspend_strategy,
        wasm_package_kind,
        &plan.output_dir,
        &plan.project.name,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        target_profile.artifact_policy.generate_runtime_config,
    )?;
    if verbose {
        println!("compiler: canonical artifact set ready");
    }

    // Write execution manifest before collecting the artifact-set so restore can run.
    if !report.run_contracts.is_empty() {
        write_execution_manifest(plan, &report.run_contracts)?;
    }
    else {
        ExecutionManifest::remove_from_output_dir(&plan.output_dir)?;
    }

    if let Ok(bundle) = collect_build_bundle(&plan.output_dir, &report) {
        if let Err(error) = store_cached_build(&cache, &plan.project.name, &canonical_triple, &ir_hash, &bundle) {
            if verbose {
                println!("cache: store failed ({error})");
            }
        }
        else if verbose {
            println!("cache: stored (artifact-set)");
        }
    }

    Ok(report)
}

fn wasm_package_kind_for_manifest(artifact_kind: ProjectArtifactKind) -> emitter::nyar_backend_wasi::WasmPackageKind {
    match artifact_kind {
        ProjectArtifactKind::Library => emitter::nyar_backend_wasi::WasmPackageKind::Library,
        ProjectArtifactKind::Binary => emitter::nyar_backend_wasi::WasmPackageKind::Binary,
    }
}

pub(super) fn print_artifacts(output_dir: &Path, artifacts: &ArtifactSet) {
    for artifact in &artifacts.artifacts {
        let artifact_path = output_dir.join(&artifact.path);
        if !artifact_path.is_file() {
            println!("artifact: missing {}", artifact.path);
            continue;
        }
        println!("artifact: {}", artifact_path.display());
    }
}

fn write_execution_manifest(plan: &legion_workspace::planner::BuildPlan, specs: &[DriverRunContract]) -> Result<()> {
    let contracts = specs
        .iter()
        .map(|spec| RunContract {
            logical_entry: spec.logical_entry.clone(),
            physical_entry: spec.physical_entry.clone(),
            invocation: spec.invocation.clone(),
            validate: spec.validate.clone(),
        })
        .collect::<Vec<_>>();
    ExecutionManifest::from_build_plan(plan, &contracts)?.write_to_output_dir(&plan.output_dir)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct HostSelectionEntry {
    contract: String,
    provider: String,
    source_file: String,
    line: usize,
}

fn write_host_selection_spec(output_dir: &Path, providers: &[legion_workspace::planner::PlannedHostProvider]) -> Result<()> {
    let output_path = output_dir.join("host-selection.txt");
    let entries: Vec<HostSelectionEntry> = providers
        .iter()
        .map(|item| HostSelectionEntry {
            contract: item.contract.clone(),
            provider: item.symbol.clone(),
            source_file: item.source_file.display().to_string(),
            line: item.line,
        })
        .collect();
    let content = write_von_indented(&entries).wrap_err_with(|| format!("序列化 host 选择结果失败 {}", output_path.display()))?;
    fs::create_dir_all(output_dir).into_diagnostic().map_err(|error| error.wrap_err(format!("创建输出目录失败 {}", output_dir.display())))?;
    fs::write(&output_path, content)
        .into_diagnostic()
        .map_err(|error| error.wrap_err(format!("写入 host 选择结果失败 {}", output_path.display())))
}

fn remove_host_selection_spec(output_dir: &Path) -> Result<()> {
    let output_path = output_dir.join("host-selection.txt");
    if output_path.exists() {
        fs::remove_file(&output_path)
            .into_diagnostic()
            .map_err(|error| error.wrap_err(format!("删除 host 选择结果失败 {}", output_path.display())))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use emitter::{ArtifactPartition, DriverCompileReport};
    use nyar_language::nyar::{BinaryArch, BinaryFlavor, BinaryTarget, HostProjectionBoundary, TargetFamily, TargetLane};

    fn partition_artifact_name(base_name: &str, partition: &ArtifactPartition, partition_count: usize) -> String {
        if partition_count <= 1 {
            return base_name.to_string();
        }

        let suffix = partition
            .name
            .rsplit("::")
            .next()
            .unwrap_or(partition.name.as_str())
            .chars()
            .map(|ch| if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' { ch } else { '_' })
            .collect::<String>();
        format!("{base_name}__{suffix}")
    }

    fn merge_partition_reports(reports: Vec<(String, DriverCompileReport)>, primary_partition_name: Option<&str>) -> DriverCompileReport {
        let mut merged = DriverCompileReport::default();
        let mut primary_contracts = Vec::new();
        let mut secondary_contracts = Vec::new();
        for (partition_name, report) in reports {
            merged.artifacts.artifacts.extend(report.artifacts.artifacts);
            if merged.entry_symbol.is_none() {
                merged.entry_symbol = report.entry_symbol.clone();
            }
            if primary_partition_name == Some(partition_name.as_str()) {
                primary_contracts.extend(report.run_contracts);
            }
            else {
                secondary_contracts.extend(report.run_contracts);
            }
        }
        merged.run_contracts = primary_contracts;
        merged.run_contracts.extend(secondary_contracts);
        merged
    }

    fn demo_partition(name: &str) -> ArtifactPartition {
        ArtifactPartition {
            fragment: nyar_language::Identifier::new("functions"),
            entry_operation: None,
            backend_name: "wasm-binary".to_string(),
            interpreter: nyar_language::Identifier::new("wasm.module"),
            name: name.to_string(),
            exported_operations: Vec::new(),
            lane: TargetLane::Wasm,
            binary_target: BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native),
            input_kind: None,
            clr_suspend_strategy: nyar::ClrSuspendStrategy::default(),
            host_boundary: HostProjectionBoundary::WasmJsGlue,
            reference_management: nyar_language::ReferenceManagement::HostGc,
            capabilities: Vec::new(),
            runtime_requirements: Vec::new(),
        }
    }

    #[test]
    fn partition_artifact_name_aligns_with_export_partition_suffix() {
        let partition = demo_partition("demo::export__unity_editor");
        assert_eq!(partition_artifact_name("valkyrie.unity", &partition, 2), "valkyrie.unity__export__unity_editor");
    }

    #[test]
    fn partition_artifact_name_adds_dimension_suffix_for_multi_partition_plan() {
        let partition = demo_partition("demo::host-interop");
        assert_eq!(partition_artifact_name("demo", &partition, 3), "demo__host-interop");
        assert_eq!(partition_artifact_name("demo", &partition, 1), "demo");
    }

    #[test]
    fn merge_partition_reports_prefers_suspend_partition_run_contract_when_entry_is_suspend() {
        let mut suspend = DriverCompileReport::default();
        suspend.run_contracts = vec![DriverRunContract {
            logical_entry: "main".to_string(),
            physical_entry: "demo__suspend.jar".to_string(),
            invocation: "java".to_string(),
            validate: "java -jar demo__suspend.jar".to_string(),
        }];

        let mut functions = DriverCompileReport::default();
        functions.run_contracts = vec![DriverRunContract {
            logical_entry: "helper".to_string(),
            physical_entry: "demo__functions.jar".to_string(),
            invocation: "java".to_string(),
            validate: "java -jar demo__functions.jar".to_string(),
        }];

        let merged = merge_partition_reports(
            vec![("demo::functions".to_string(), functions), ("demo::suspend".to_string(), suspend)],
            Some("demo::suspend"),
        );

        assert_eq!(merged.run_contracts.first().expect("run contract").physical_entry, "demo__suspend.jar");
    }

    #[test]
    fn merge_partition_reports_prefers_primary_partition_run_contract() {
        let mut primary = DriverCompileReport::default();
        primary.run_contracts = vec![DriverRunContract {
            logical_entry: "main".to_string(),
            physical_entry: "demo__functions.mjs".to_string(),
            invocation: "node".to_string(),
            validate: "node demo__functions.mjs".to_string(),
        }];

        let mut secondary = DriverCompileReport::default();
        secondary.run_contracts = vec![DriverRunContract {
            logical_entry: "_start".to_string(),
            physical_entry: "demo__host-interop.wasm".to_string(),
            invocation: "wasmtime".to_string(),
            validate: "wasmtime demo__host-interop.wasm".to_string(),
        }];

        let merged = merge_partition_reports(
            vec![("demo::host-interop".to_string(), secondary), ("demo::functions".to_string(), primary)],
            Some("demo::functions"),
        );

        assert_eq!(merged.run_contracts.first().expect("run contract").physical_entry, "demo__functions.mjs");
    }
}

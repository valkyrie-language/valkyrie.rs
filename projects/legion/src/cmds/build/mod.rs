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

use clap::Args;
use emitter::{
    DriverRunContract, FragmentSubmission, FrontendBuildBundle, LoweredBackendInput, PlannedArtifactPartitionsView, bundled_backend_registry,
    compile_frontend_bundle_with_bundled_backends,
};
use miette::{IntoDiagnostic, NamedSource, Report, Result, WrapErr, miette};
use nyar_language::{
    ArtifactKind, ArtifactPartitionPlan, ArtifactSet, CanonicalSpecification, CanonicalTarget, FrontendBuildOutput,
    assemble_fragment_submission,
    nyar::{
        ClrSuspendStrategy, HostProjectionBoundary, TargetBackendFamily, TargetLane, VmSuspendStrategy, projection_policy_for_target_profile,
    },
    plan_artifacts_from_build_output,
};
use serde::Serialize;
use std_data::text::valkyrie::tgrammar::{TgIf, TgLoop, TgMatch, TgNode, TgRoot, parse_tgrammar_fragment};

use crate::{
    cache::{
        CompilationCache, cache_root_for, collect_build_bundle, compile_frontend_with_cache, compile_semantic_source_groups,
        compute_artifact_hash, store_cached_build, try_restore_cached_build,
    },
    cmds::{
        run::{ExecutionManifest, RunContract},
        source_hygiene,
    },
    manifest::ProjectManifest,
    planner::{BuildPlan, BuildRequest, LegionWorkspace, ProjectResolutionMode},
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

    let workspace = LegionWorkspace::discover_for_project(&args.project_dir)?;
    if workspace.is_workspace_only_root(&args.project_dir) {
        return run_workspace_members_in_parallel(args);
    }

    let request = BuildRequest { project_dir: args.project_dir.clone(), target: args.target.clone(), output_dir: args.output_dir.clone() };
    let (plan, resolution_mode) = workspace.build_plan_with_local_fallback(&request)?;

    println!("workspace: {}", plan.workspace_root.display());
    match resolution_mode {
        ProjectResolutionMode::Workspace => {
            println!("mode: workspace");
        }
        ProjectResolutionMode::Package => {
            println!("mode: package");
            println!("note: 当前目录存在 `legion.von`，但未注册到 workspace members，已回退到 package 模式");
        }
        ProjectResolutionMode::Script => {
            println!("mode: script");
            println!("note: 未发现 `legions.von`，已按单文件脚本项目模式解析当前目录");
        }
    }
    println!("project: {}", plan.project.name);
    println!("target: {}", plan.project.build_target.target);
    println!("output: {}", plan.output_dir.display());
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
    materialize_node_bootstrap_aliases(&plan.output_dir, &plan.project.build_target.target)?;

    let manifest_source = fs::read_to_string(&plan.project.manifest_path)
        .into_diagnostic()
        .wrap_err_with(|| format!("读取项目清单失败：{}", plan.project.manifest_path.display()))?;
    let project_manifest = ProjectManifest::parse(&manifest_source)?;
    if let Some(plugin) = &project_manifest.build_plugin {
        unity_export::export_unity_project(&plan, &report, plugin)?;
        println!("unity export: {}", plugin.kind);
    }

    Ok(ExitCode::SUCCESS)
}

fn run_workspace_members_in_parallel(args: &BuildArgs) -> Result<ExitCode> {
    let workspace = LegionWorkspace::discover(&args.project_dir)?;
    let requested_target = args.target.clone();
    let members = workspace
        .member_manifest_dirs()
        .into_iter()
        .filter(|project_dir| {
            let request =
                BuildRequest { project_dir: project_dir.clone(), target: requested_target.clone(), output_dir: args.output_dir.clone() };
            match workspace.build_plan(&request) {
                Ok(_) => true,
                Err(crate::planner::PlannerError::MissingBuildTarget { .. }) => {
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
    let ir_hash = compute_artifact_hash(
        &plan.project.source_files,
        &canonical_triple,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        plan.project.build_target.runtime_async,
    )
    .map_err(|error| miette!("{error}"))?;

    if let Some(report) = try_restore_cached_build(&cache, &plan.project.name, &canonical_triple, &ir_hash, &plan.output_dir) {
        if verbose {
            println!("cache: hit (artifact-set)");
        }
        // Ensure execution manifest exists for restore paths that predate bundling it.
        if !report.run_contracts.is_empty() {
            let _ = write_execution_manifest(plan, &report.run_contracts);
        }
        return Ok(report);
    }

    // WASI 与 Node 同为 wasm32，但宿主不同：Node 走 `env.*`，WASI 走 guest/host_contract。
    // 模板 `<% match arch %>` 因此对 WASI 使用伪架构键 `"wasi"`，避免误选 wasm32/Node 分支。
    let arch = if plan.project.build_target.target.specification == CanonicalSpecification::Wasi {
        "wasi"
    }
    else {
        plan.project.build_target.target.arch.as_str()
    };
    eprintln!("[seed-debug] frontend-compile-start target={} files={} arch={}", canonical_triple, plan.project.source_files.len(), arch);
    let frontend = if plan.project.semantic_source_groups.len() <= 1 {
        compile_frontend_with_cache(&cache, &plan.project.source_files, &canonical_triple, arch, preprocess_templates)?.build_output
    }
    else {
        compile_semantic_source_groups(&plan.project.semantic_source_groups, arch, preprocess_templates)?
    };
    eprintln!("[seed-debug] frontend-compile-done target={} hir_functions={}", canonical_triple, frontend.hir_function_count());
    if verbose {
        println!("frontend: semantic source groups={}", plan.project.semantic_source_groups.len());
    }
    eprintln!("[seed-debug] frontend-ready target={} semantic_groups={}", canonical_triple, plan.project.semantic_source_groups.len());
    let build_output = frontend;
    let target_profile = plan.project.build_target.target.to_profile(None);
    let projection_policy = projection_policy_for_target_profile(&target_profile)?;
    let backend_registry = bundled_backend_registry(&build_output.neutral_plan().semantic_fragments, &target_profile, &projection_policy);
    let clr_suspend_strategy = ClrSuspendStrategy::from_runtime_async_flag(plan.project.build_target.runtime_async);
    let artifact_plan = plan_artifacts_from_build_output(
        &build_output,
        plan.project.build_target.target.clone(),
        projection_policy,
        backend_registry,
        clr_suspend_strategy,
    )
    .map_err(|error| miette!(format!("前端分区规划失败: {error:?}")))?;
    eprintln!("[seed-debug] artifact-plan-ready partitions={}", artifact_plan.partitions.len());
    let driver_bundle = LegionFrontendBuildAdapter::new(build_output, artifact_plan);
    eprintln!("[seed-debug] driver-bundle-ready");

    if verbose {
        println!("hir functions: {}", driver_bundle.build_output.hir_function_count());
        println!("partitions: {}", driver_bundle.artifact_plan.partitions.len());
    }

    fs::create_dir_all(&plan.output_dir).into_diagnostic().wrap_err_with(|| format!("创建输出目录失败 {}", plan.output_dir.display()))?;

    eprintln!("[seed-debug] bundled-backend-compile-start");
    let report = compile_frontend_bundle_with_bundled_backends(
        &driver_bundle,
        &plan.output_dir,
        &plan.project.name,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        target_profile.artifact_policy.generate_runtime_config,
    )?;

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

struct LegionFrontendBuildAdapter {
    build_output: FrontendBuildOutput,
    artifact_plan: ArtifactPartitionPlan,
}

impl LegionFrontendBuildAdapter {
    fn new(build_output: FrontendBuildOutput, artifact_plan: ArtifactPartitionPlan) -> Self {
        Self { build_output, artifact_plan }
    }
}

impl FrontendBuildBundle for LegionFrontendBuildAdapter {
    fn planned_partitions(&self) -> &dyn PlannedArtifactPartitionsView {
        self
    }

    fn submit_backend_input_for_partition(
        &self,
        partition_index: usize,
        backend_family: TargetBackendFamily,
        host_boundary: HostProjectionBoundary,
        output_dir: &Path,
        _lane: TargetLane,
    ) -> Result<LoweredBackendInput> {
        eprintln!("[seed-debug] partition-input-start index={partition_index} backend={backend_family:?}");
        let fragment = assemble_fragment_submission(&self.build_output, &self.artifact_plan, partition_index)?;
        eprintln!("[seed-debug] partition-input-fragment-ready index={partition_index}");
        let host_flavor = self.artifact_plan.target.to_profile(None).host_flavor;
        let result = LoweredBackendInput::from_fragment_submission(
            &fragment,
            backend_family,
            host_boundary,
            output_dir,
            self.artifact_plan.partitions.get(partition_index).map(|partition| partition.lane).unwrap_or(TargetLane::Clr),
            self.artifact_plan.partitions.get(partition_index).map(|partition| partition.clr_suspend_strategy).unwrap_or_default(),
            VmSuspendStrategy::default(),
            &host_flavor,
        );
        eprintln!("[seed-debug] partition-input-lowered index={partition_index} ok={}", result.is_ok());
        result
    }
}

impl PlannedArtifactPartitionsView for LegionFrontendBuildAdapter {
    fn primary_partition_name(&self) -> Option<String> {
        self.artifact_plan
            .partitions
            .iter()
            .find(|partition| partition.entry_operation.is_some())
            .map(|partition| partition.name.clone())
            .or_else(|| {
                self.artifact_plan
                    .partitions
                    .iter()
                    .find(|partition| partition.name.ends_with("::functions"))
                    .map(|partition| partition.name.clone())
            })
            .or_else(|| self.artifact_plan.partitions.first().map(|partition| partition.name.clone()))
    }

    fn partition_count(&self) -> usize {
        self.artifact_plan.partitions.len()
    }

    fn partition(&self, partition_index: usize) -> Option<&emitter::ArtifactPartition> {
        self.artifact_plan.partitions.get(partition_index)
    }

    fn backend_requirement(&self, partition_index: usize) -> Option<nyar_language::nyar::PartitionBackendRequirement> {
        self.artifact_plan.backend_requirement(partition_index)
    }
}

pub(super) fn load_combined_source(source_files: &[PathBuf]) -> Result<String> {
    let mut combined_source = String::new();
    let mut debug_map = String::new();
    let mut offset = 0usize;
    for source_path in source_files {
        let content = fs::read_to_string(source_path)
            .into_diagnostic()
            .map_err(|error| error.wrap_err(format!("读取源码失败 {}", source_path.display())))?;
        // 去除 UTF-8 BOM（U+FEFF），避免解析器在合并源码时遇到非法字符。
        let trimmed = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
        let end = offset + trimmed.len();
        debug_map.push_str(&format!("{offset}-{end}: {}\n", source_path.display()));
        combined_source.push_str(trimmed);
        combined_source.push('\n');
        offset = combined_source.len();
    }
    // 写入调试映射文件，用于定位 parser 错误的字节偏移。
    let _ = fs::write("target/source-offsets.txt", &debug_map);
    Ok(combined_source)
}

pub(super) fn print_artifacts(output_dir: &Path, artifacts: &ArtifactSet) {
    for artifact in &artifacts.artifacts {
        let candidates: &[&str] = match artifact.kind {
            ArtifactKind::Executable => &["exe", "wasm"],
            ArtifactKind::DynamicLibrary => &["dll"],
            ArtifactKind::Object => &["obj"],
            ArtifactKind::AssemblyListing => &["mjs", "msil", "wit"],
        };

        for extension in candidates {
            let artifact_path = output_dir.join(format!("{}.{}", artifact.name, extension));
            if artifact_path.exists() {
                println!("artifact: {}", artifact_path.display());
                break;
            }
        }
    }
}

/// 为 Node 自举写出规范入口别名（`legion.mjs` / `legion.wasm`）。
///
/// 多 partition 时物理名可能是 `legion__main_legion.*`；bootstrap / npm 契约要求规范名。
fn materialize_node_bootstrap_aliases(output_dir: &Path, target: &nyar_language::CanonicalTarget) -> Result<()> {
    let profile = target.to_profile(None);
    if !matches!(profile.host_kind, nyar_language::TargetHostKind::JavaScript) {
        return Ok(());
    }

    let entries = fs::read_dir(output_dir).into_diagnostic().wrap_err_with(|| format!("读取产物目录失败：{}", output_dir.display()))?;
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        let Some(file_name) = path.file_name().and_then(|name| name.to_str())
        else {
            continue;
        };
        for alias in crate::bootstrap_entry_aliases(file_name) {
            let dest = output_dir.join(alias);
            if file_name.ends_with(".mjs") {
                let content = fs::read_to_string(&path).into_diagnostic().wrap_err_with(|| format!("读取启动壳失败：{}", path.display()))?;
                let stem = file_name.trim_end_matches(".mjs");
                let alias_stem = alias.trim_end_matches(".mjs");
                let rewritten = content.replace(&format!("./{stem}.wasm"), &format!("./{alias_stem}.wasm"));
                fs::write(&dest, rewritten).into_diagnostic().wrap_err_with(|| format!("写入入口别名失败：{}", dest.display()))?;
            }
            else {
                fs::copy(&path, &dest)
                    .into_diagnostic()
                    .wrap_err_with(|| format!("复制入口别名失败：{} -> {}", path.display(), dest.display()))?;
            }
            println!("alias: {file_name} -> {alias}");
        }
    }
    Ok(())
}

fn write_execution_manifest(plan: &crate::planner::BuildPlan, specs: &[DriverRunContract]) -> Result<()> {
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

fn attach_source_to_report(error: impl Into<Report>, source: &str) -> Report {
    error.into().with_source_code(NamedSource::new("combined-source.v", source.to_string()))
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
struct HostSelectionEntry {
    contract: String,
    provider: String,
    source_file: String,
    line: usize,
}

fn write_host_selection_spec(output_dir: &Path, providers: &[crate::planner::PlannedHostProvider]) -> Result<()> {
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

/// 预处理源码中的模板指令，根据目标架构选择正确的分支。
///
/// 处理 `<% match arch %>` … `<% end %>` 块：用 T-Grammar 解析后按目标架构
/// 选择 `<% case "xxx" %>` 或 `<% else %>` 分支，递归展开嵌套块。
pub(crate) fn preprocess_templates(source: &str, arch: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut pos = 0;

    while pos < source.len() {
        let Some(rel) = source[pos..].find("<% match ")
        else {
            result.push_str(&source[pos..]);
            break;
        };
        let abs = pos + rel;
        result.push_str(&source[pos..abs]);

        match parse_tgrammar_fragment(&source[abs..]) {
            Ok((nodes, consumed)) if nodes.len() == 1 => {
                let fragment = &source[abs..abs + consumed];
                if let TgNode::Match(match_node) = &nodes[0]
                    && match_node.scrutinee.trim() == "arch"
                {
                    let selected = select_arch_match_body(match_node, arch);
                    result.push_str(&preprocess_templates(&tg_root_to_source(selected, fragment), arch));
                }
                else {
                    result.push_str(fragment);
                }
                pos = abs + consumed;
            }
            _ => {
                result.push_str(&source[abs..abs + "<%".len()]);
                pos = abs + "<%".len();
            }
        }
    }

    result
}

fn select_arch_match_body<'a>(match_node: &'a TgMatch, arch: &str) -> &'a [TgNode] {
    for arm in &match_node.arms {
        if arm.pattern.as_deref().map(normalize_case_pattern).as_deref() == Some(arch) {
            return &arm.body;
        }
    }
    match_node.arms.iter().find(|arm| arm.pattern.is_none()).map(|arm| arm.body.as_slice()).unwrap_or(&[])
}

fn normalize_case_pattern(pattern: &str) -> String {
    let pattern = pattern.trim();
    if (pattern.starts_with('"') && pattern.ends_with('"')) || (pattern.starts_with('\'') && pattern.ends_with('\'')) {
        pattern[1..pattern.len().saturating_sub(1)].to_string()
    }
    else {
        pattern.to_string()
    }
}

fn tg_root_to_source(nodes: &[TgNode], fragment: &str) -> String {
    nodes.iter().map(|node| node_to_source(node, fragment)).collect()
}

fn node_to_source(node: &TgNode, fragment: &str) -> String {
    let span = match node {
        TgNode::Text { span, .. } | TgNode::Stmt { span, .. } | TgNode::Comment { span, .. } => span.clone(),
        TgNode::If(TgIf { span, .. }) | TgNode::Loop(TgLoop { span, .. }) | TgNode::Match(TgMatch { span, .. }) => span.clone(),
    };
    fragment[span].to_string()
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
    fn preprocess_templates_selects_arch_branch_with_unified_end() {
        let source = r#"
<% match arch %>
<% case "clr" %>
clr_body()
<% case "jvm" %>
jvm_body()
<% else %>
default_body()
<% end %>
"#;
        assert!(preprocess_templates(source, "clr").contains("clr_body()"));
        assert!(!preprocess_templates(source, "clr").contains("jvm_body()"));
        assert!(preprocess_templates(source, "wasm").contains("default_body()"));
    }

    #[test]
    fn preprocess_templates_accepts_end_match_label() {
        let source = r#"
<% match arch %>
<% case "clr" %>
<% case "wasm" %>
wasm_body()
<% else %>
fallback()
<% end match %>
"#;
        let expanded = preprocess_templates(source, "wasm");
        assert!(expanded.contains("wasm_body()"));
        assert!(!expanded.contains("<%"));
        assert!(!expanded.contains("fallback()"));
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

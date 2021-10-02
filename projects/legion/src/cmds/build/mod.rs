// `legion build` 命令。
//
// 这里只做前端编译与运行契约落盘，
// 目标路线选择与编码由 `nyar-driver` 负责。

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::Args;
use miette::{miette, IntoDiagnostic, NamedSource, Report, Result, WrapErr};
use nyar_driver::{compile_with_bundled_backends, DriverCompileRequest, DriverRunContract};
use serde::Serialize;
use valkyrie_compiler::{
    lir::LirTargetLane,
    nyar::{ArtifactPartition, TargetLane},
    ArtifactKind, ArtifactPartitionPlan, ArtifactSet, CanonicalTarget, CompilationOptions, HostProjectionBoundary, LirLowerer, RunnerFamily,
    TargetBackendFamily, ValkyrieCompiler,
};
use valkyrie_parser::AstParser;

use crate::{
    cmds::run::RunContract,
    planner::{BuildRequest, LegionWorkspace},
    write_von_pretty,
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
}

/// 执行 `legion build`。
pub fn run(args: &BuildArgs) -> Result<ExitCode> {
    let workspace = LegionWorkspace::discover(&args.project_dir)?;
    let request = BuildRequest { project_dir: args.project_dir.clone(), target: args.target.clone(), output_dir: args.output_dir.clone() };
    let (plan, fallback_to_package) =
        if args.workspace { (workspace.build_plan(&request)?, false) } else { workspace.build_plan_with_local_fallback(&request)? };

    println!("workspace: {}", plan.workspace_root.display());
    if fallback_to_package {
        println!("mode: package");
        println!("note: 当前目录存在 `legion.von`，但未注册到 workspace members，已回退到 package 模式");
    }
    else {
        println!("mode: workspace");
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

    write_host_selection_spec(&plan.output_dir, &plan.project.selected_host_providers)?;

    let combined_source = load_combined_source(&plan.project.source_files)?;
    let arch = plan.project.build_target.target.arch.as_str();
    let combined_source = preprocess_templates(&combined_source, arch);
    // 写入预处理后的源码，用于定位 parser 错误的字节偏移。
    let _ = fs::write("target/preprocessed-source.v", &combined_source);
    let compiler = ValkyrieCompiler::default();
    let parser_root = AstParser::parse_root(&combined_source).map_err(|error| attach_source_to_report(error, &combined_source))?;
    let hir_module = compiler.lower_root(&parser_root).map_err(|error| attach_source_to_report(error, &combined_source))?;
    let target_profile = plan.project.build_target.target.to_profile(None);
    let lir_module = LirLowerer::lower_module_for_lane(&hir_module, lir_lane_for_backend(target_profile.backend_family));
    let artifact_plan = valkyrie_compiler::hir_module_to_artifact_plan(&hir_module, plan.project.build_target.target.clone());

    println!("lir functions: {}", lir_module.functions.len());
    println!("partitions: {}", artifact_plan.partitions.len());
    // 调试：打印名为 length 的函数信息
    for func in &lir_module.functions {
        if func.symbol == "length" {
            eprintln!("[DEBUG LIR] found function 'length': return_type={:?}, param_types={:?}", func.return_type, func.param_types);
        }
    }
    let report = compile_partitions(
        &artifact_plan,
        &hir_module,
        &lir_module,
        &plan.output_dir,
        &plan.project.name,
        plan.project.build_target.msil,
        target_profile.runner_family(),
        target_profile.artifact_policy.generate_runtime_config,
    )?;

    println!("backend status: compiled");
    println!("host kind: {:?}", target_profile.host_kind);
    println!("publish format: {}", target_profile.artifact_policy.default_publish_format);
    if let Some(entry_symbol) = &report.entry_symbol {
        println!("entry: {}", entry_symbol);
    }
    if let Some(run_contract) = &report.run_contract {
        write_run_contract_spec(&plan.output_dir, run_contract)?;
    }
    print_artifacts(&plan.output_dir, &report.artifacts);

    Ok(ExitCode::SUCCESS)
}

fn lir_lane_for_backend(backend_family: TargetBackendFamily) -> LirTargetLane {
    match backend_family {
        TargetBackendFamily::Clr => LirTargetLane::Clr,
        TargetBackendFamily::Jvm => LirTargetLane::Jvm,
        TargetBackendFamily::Wasm => LirTargetLane::Wasm,
        TargetBackendFamily::Native => LirTargetLane::Native,
        _ => LirTargetLane::Clr,
    }
}

fn compile_partitions(
    artifact_plan: &ArtifactPartitionPlan,
    hir_module: &valkyrie_compiler::HirModule,
    lir_module: &valkyrie_compiler::LirModule,
    output_dir: &Path,
    project_name: &str,
    emit_msil_sidecar: bool,
    default_runner_family: RunnerFamily,
    generate_runtime_config: bool,
) -> Result<nyar_driver::DriverCompileReport> {
    let primary_partition_name = select_primary_partition_name(artifact_plan);
    let partition_count = artifact_plan.partitions.len();
    let mut reports = Vec::new();

    for (partition_index, partition) in artifact_plan.partitions.iter().enumerate() {
        let artifact_name = partition_artifact_name(project_name, partition, partition_count);
        let options = CompilationOptions {
            target: partition.binary_target.clone(),
            artifact_name: artifact_name.clone(),
            emit_debug_symbols: false,
            optimize: false,
        };
        let driver_input = valkyrie_compiler::lower_to_driver_input_for_partition(
            hir_module,
            lir_module.clone(),
            backend_family_for_partition(partition),
            partition.host_boundary,
            output_dir.to_path_buf(),
            &partition.exported_operations,
        )?;

        if emit_msil_sidecar {
            let _ = valkyrie_compiler::write_clr_msil_sidecar(output_dir, &artifact_name, &driver_input);
        }

        let requirement =
            artifact_plan.backend_requirement(partition_index).ok_or_else(|| miette!("分区 `{}` 缺少后端需求", partition.name))?;
        let report = compile_with_bundled_backends(DriverCompileRequest {
            artifact_name: &artifact_name,
            requirement,
            input: driver_input,
            runner_family: runner_family_for_partition(partition, default_runner_family),
            generate_runtime_config,
            options: &options,
        })?;
        reports.push((partition.name.clone(), report));
    }

    Ok(merge_partition_reports(reports, primary_partition_name.as_deref()))
}

fn backend_family_for_partition(partition: &ArtifactPartition) -> TargetBackendFamily {
    match partition.lane {
        TargetLane::Clr => TargetBackendFamily::Clr,
        TargetLane::Jvm => TargetBackendFamily::Jvm,
        TargetLane::Wasm => TargetBackendFamily::Wasm,
        TargetLane::Native => TargetBackendFamily::Native,
        TargetLane::Vm => TargetBackendFamily::NyarVm,
    }
}

fn runner_family_for_partition(partition: &ArtifactPartition, fallback: RunnerFamily) -> RunnerFamily {
    match partition.host_boundary {
        HostProjectionBoundary::WasmJsGlue => RunnerFamily::Node,
        HostProjectionBoundary::WasiComponent => RunnerFamily::Wasi,
        _ => fallback,
    }
}

fn select_primary_partition_name(plan: &ArtifactPartitionPlan) -> Option<String> {
    plan.partitions
        .iter()
        .find(|partition| partition.name.ends_with("::functions"))
        .map(|partition| partition.name.clone())
        .or_else(|| plan.partitions.first().map(|partition| partition.name.clone()))
}

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
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                ch
            }
            else {
                '_'
            }
        })
        .collect::<String>();
    format!("{base_name}__{suffix}")
}

fn merge_partition_reports(
    reports: Vec<(String, nyar_driver::DriverCompileReport)>,
    primary_partition_name: Option<&str>,
) -> nyar_driver::DriverCompileReport {
    let mut merged = nyar_driver::DriverCompileReport::default();
    for (partition_name, report) in reports {
        merged.artifacts.artifacts.extend(report.artifacts.artifacts);
        if merged.entry_symbol.is_none() {
            merged.entry_symbol = report.entry_symbol.clone();
        }
        if primary_partition_name == Some(partition_name.as_str()) && report.run_contract.is_some() {
            merged.run_contract = report.run_contract.clone();
        }
        else if merged.run_contract.is_none() {
            merged.run_contract = report.run_contract.clone();
        }
    }
    merged
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

fn write_run_contract_spec(output_dir: &Path, spec: &DriverRunContract) -> Result<()> {
    let contract_path = output_dir.join("run-contract.txt");
    let content = write_von_pretty(&RunContract {
        logical_entry: spec.logical_entry.clone(),
        physical_entry: spec.physical_entry.clone(),
        invocation: spec.invocation.clone(),
        validate: spec.validate.clone(),
    })
    .wrap_err_with(|| format!("序列化运行契约失败 {}", contract_path.display()))?;
    fs::write(&contract_path, content)
        .into_diagnostic()
        .map_err(|error| error.wrap_err(format!("写入运行契约失败 {}", contract_path.display())))
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
    let content = write_von_pretty(&entries).wrap_err_with(|| format!("序列化 host 选择结果失败 {}", output_path.display()))?;
    fs::create_dir_all(output_dir).into_diagnostic().map_err(|error| error.wrap_err(format!("创建输出目录失败 {}", output_dir.display())))?;
    fs::write(&output_path, content)
        .into_diagnostic()
        .map_err(|error| error.wrap_err(format!("写入 host 选择结果失败 {}", output_path.display())))
}

/// 预处理源码中的模板指令，根据目标架构选择正确的分支。
///
/// 处理 `<% match arch %>` ... `<% end match %>` 块，根据目标架构
/// 选择匹配的 `<% case "xxx" %>` 分支或 `<% else %>` 分支，
/// 移除其他分支和指令标记，使后续词法分析只看到选中分支的代码。
fn preprocess_templates(source: &str, arch: &str) -> String {
    let mut result = String::with_capacity(source.len());
    let mut pos = 0;

    while pos < source.len() {
        let match_pos = match source[pos..].find("<% match ") {
            Some(offset) => pos + offset,
            None => {
                result.push_str(&source[pos..]);
                break;
            }
        };

        let directive_end = match source[match_pos..].find("%>") {
            Some(offset) => match_pos + offset + 2,
            None => {
                result.push_str(&source[pos..]);
                break;
            }
        };

        let block_end = match find_matching_end(source, directive_end) {
            Some(end) => end + "<% end match %>".len(),
            None => {
                result.push_str(&source[pos..]);
                break;
            }
        };

        let block_content = &source[directive_end..block_end - "<% end match %>".len()];
        let selected = select_branch(block_content, arch);

        result.push_str(&source[pos..match_pos]);
        result.push_str(&selected);

        pos = block_end;
    }

    result
}

/// 查找匹配的 `<% end match %>`，正确处理嵌套的 `<% match %>` 块。
fn find_matching_end(source: &str, start: usize) -> Option<usize> {
    let mut depth = 1;
    let mut pos = start;

    while pos < source.len() {
        let remaining = &source[pos..];

        let match_offset = remaining.find("<% match ");
        let end_offset = remaining.find("<% end match %>");

        match (match_offset, end_offset) {
            (Some(m), Some(e)) if m < e => {
                depth += 1;
                pos += m + "<% match ".len();
            }
            (Some(_), Some(e)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(pos + e);
                }
                pos += e + "<% end match %>".len();
            }
            (Some(m), None) => {
                depth += 1;
                pos += m + "<% match ".len();
            }
            (None, Some(e)) => {
                depth -= 1;
                if depth == 0 {
                    return Some(pos + e);
                }
                pos += e + "<% end match %>".len();
            }
            (None, None) => break,
        }
    }

    None
}

/// 从模板块内容中选择匹配目标架构的分支。
///
/// 分支结构为 `<% case "clr" %>` ... `<% case "jvm" %>` ... `<% else %>` ...，
/// 返回匹配 `arch` 的分支代码；若无匹配则返回 `else` 分支代码。
fn select_branch(content: &str, arch: &str) -> String {
    let case_marker = "<% case \"";
    let else_marker = "<% else %>";

    let mut directives: Vec<(Option<String>, usize, usize)> = Vec::new();
    let mut search_pos = 0;

    while search_pos < content.len() {
        let case_pos = content[search_pos..].find(case_marker).map(|p| search_pos + p);
        let else_pos = content[search_pos..].find(else_marker).map(|p| search_pos + p);

        let next_pos = match (case_pos, else_pos) {
            (Some(c), Some(e)) => c.min(e),
            (Some(c), None) => c,
            (None, Some(e)) => e,
            (None, None) => break,
        };

        if content[next_pos..].starts_with(case_marker) {
            let name_start = next_pos + case_marker.len();
            if let Some(close) = content[name_start..].find("\">%") {
                let name = content[name_start..name_start + close].to_string();
                let directive_end = name_start + close + 3;
                directives.push((Some(name), next_pos, directive_end));
                search_pos = directive_end;
            }
            else {
                break;
            }
        }
        else if content[next_pos..].starts_with(else_marker) {
            let directive_end = next_pos + else_marker.len();
            directives.push((None, next_pos, directive_end));
            search_pos = directive_end;
        }
        else {
            break;
        }
    }

    let selected_idx = directives
        .iter()
        .position(|(name, _, _)| name.as_deref() == Some(arch))
        .or_else(|| directives.iter().position(|(name, _, _)| name.is_none()));

    if let Some(idx) = selected_idx {
        let (_, _, code_start) = directives[idx];
        let code_end = if idx + 1 < directives.len() { directives[idx + 1].1 } else { content.len() };
        return content[code_start..code_end].trim().to_string();
    }

    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use nyar_driver::DriverCompileReport;
    use valkyrie_compiler::nyar::{BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily};

    fn demo_partition(name: &str) -> ArtifactPartition {
        ArtifactPartition {
            name: name.to_string(),
            exported_operations: Vec::new(),
            lane: TargetLane::Wasm,
            binary_target: BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native),
            input_kind: None,
            host_boundary: HostProjectionBoundary::WasmJsGlue,
            reference_management: valkyrie_compiler::ReferenceManagement::HostGc,
            capabilities: Vec::new(),
            runtime_requirements: Vec::new(),
        }
    }

    #[test]
    fn partition_artifact_name_adds_dimension_suffix_for_multi_partition_plan() {
        let partition = demo_partition("demo::host-interop");
        assert_eq!(partition_artifact_name("demo", &partition, 3), "demo__host-interop");
        assert_eq!(partition_artifact_name("demo", &partition, 1), "demo");
    }

    #[test]
    fn merge_partition_reports_prefers_primary_partition_run_contract() {
        let mut primary = DriverCompileReport::default();
        primary.run_contract = Some(DriverRunContract {
            logical_entry: "main".to_string(),
            physical_entry: "demo__functions.mjs".to_string(),
            invocation: "node".to_string(),
            validate: "node demo__functions.mjs".to_string(),
        });

        let mut secondary = DriverCompileReport::default();
        secondary.run_contract = Some(DriverRunContract {
            logical_entry: "_start".to_string(),
            physical_entry: "demo__host-interop.wasm".to_string(),
            invocation: "wasmtime".to_string(),
            validate: "wasmtime demo__host-interop.wasm".to_string(),
        });

        let merged = merge_partition_reports(
            vec![("demo::host-interop".to_string(), secondary), ("demo::functions".to_string(), primary)],
            Some("demo::functions"),
        );

        assert_eq!(merged.run_contract.expect("run contract").physical_entry, "demo__functions.mjs");
    }
}

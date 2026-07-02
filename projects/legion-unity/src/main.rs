//! `legion-unity` — Unity 目标可选伴随工具。
//!
//! 大多数用户不需要安装；Unity 日常操作在 Editor 插件完成。
//! 已安装时也可通过 `legion unity …` 由 `legion` 自动转发到本工具。

use std::{
    fs,
    path::{Path, PathBuf},
    process::ExitCode,
};

use clap::{Args, Parser, Subcommand};
use legion::{
    BuildPluginSpec, BuildRequest, LegionWorkspace, ProjectManifest,
    cmds::build::{BuildArgs, run as run_build},
    planner::BuildPlan,
    unity_export,
};
use miette::{IntoDiagnostic, Result, WrapErr, miette};
use nyar_language::CanonicalTarget;

#[derive(Debug, Parser)]
#[command(name = "legion-unity", about = "Valkyrie Unity 构建器（MSIL 导出与工程同步）")]
struct UnityCli {
    #[command(subcommand)]
    command: UnityCommand,
}

#[derive(Debug, Subcommand)]
enum UnityCommand {
    /// 编译 V 项目并导出 MSIL 到 `build/unity/msil`。
    Build(UnityBuildArgs),
    /// 仅从 `dist/` 重新导出 MSIL（不重新编译）。
    Export(UnityProjectArgs),
    /// 将 MSIL 同步到 Unity 工程 `Assets/Valkyrie/Plugins`。
    Sync(UnitySyncArgs),
    /// 显示 Unity 导出状态。
    Status(UnityProjectArgs),
}

#[derive(Debug, Clone, Args)]
struct UnityProjectArgs {
    #[arg(value_name = "project-dir", default_value = ".")]
    project_dir: PathBuf,
}

#[derive(Debug, Clone, Args)]
struct UnityBuildArgs {
    #[command(flatten)]
    project: UnityProjectArgs,
    #[arg(long)]
    target: Option<CanonicalTarget>,
    #[arg(short = 'o', long = "output")]
    output_dir: Option<PathBuf>,
    #[arg(long, default_value_t = false)]
    workspace: bool,
}

#[derive(Debug, Clone, Args)]
struct UnitySyncArgs {
    #[command(flatten)]
    project: UnityProjectArgs,
    #[arg(long = "unity-project")]
    unity_project: Option<PathBuf>,
}

fn main() -> Result<ExitCode> {
    run(UnityCli::parse())
}

fn run(cli: UnityCli) -> Result<ExitCode> {
    match cli.command {
        UnityCommand::Build(args) => run_build_cmd(&args),
        UnityCommand::Export(args) => run_export(&args),
        UnityCommand::Sync(args) => run_sync(&args),
        UnityCommand::Status(args) => run_status(&args),
    }
}

fn run_build_cmd(args: &UnityBuildArgs) -> Result<ExitCode> {
    let manifest = load_project_manifest(&args.project.project_dir)?;
    require_unity_plugin(&manifest)?;
    let target = args.target.clone().unwrap_or_else(|| resolve_unity_build_target(&manifest));

    let build_args = BuildArgs {
        project_dir: args.project.project_dir.clone(),
        target,
        output_dir: args.output_dir.clone(),
        workspace: args.workspace,
        debug_artifacts: false,
    };

    println!("unity: build (target={})", build_args.target);
    run_build(&build_args)
}

fn run_export(args: &UnityProjectArgs) -> Result<ExitCode> {
    let manifest = load_project_manifest(&args.project_dir)?;
    let plugin = require_unity_plugin(&manifest)?;
    let build_args = BuildArgs {
        project_dir: args.project_dir.clone(),
        target: resolve_unity_build_target(&manifest),
        output_dir: None,
        workspace: false,
        debug_artifacts: false,
    };
    let plan = resolve_build_plan(&args.project_dir, &build_args)?;
    let entry_method = read_export_entry_method(&unity_msil_dir(&plan, plugin)).unwrap_or_else(|| "main".to_string());
    let report = nyar_emitter::DriverCompileReport { entry_symbol: Some(entry_method), ..Default::default() };
    unity_export::export_unity_project(&plan, &report, plugin)?;
    println!("unity: exported to {}", unity_msil_dir(&plan, plugin).display());
    Ok(ExitCode::SUCCESS)
}

fn run_sync(args: &UnitySyncArgs) -> Result<ExitCode> {
    let manifest = load_project_manifest(&args.project.project_dir)?;
    let plugin = require_unity_plugin(&manifest)?;
    let build_args = BuildArgs {
        project_dir: args.project.project_dir.clone(),
        target: resolve_unity_build_target(&manifest),
        output_dir: None,
        workspace: false,
        debug_artifacts: false,
    };
    let plan = resolve_build_plan(&args.project.project_dir, &build_args)?;
    let msil_dir = unity_msil_dir(&plan, plugin);
    let unity_project = args.unity_project.clone().unwrap_or_else(|| args.project.project_dir.join("unity"));
    if plugin.export_routes.is_empty() {
        let plugins_dir = unity_project.join("Assets").join("Valkyrie").join("Plugins");
        unity_export::sync_msil_to_unity_plugins(&msil_dir, &plugins_dir)?;
        println!("unity: synced MSIL -> {}", plugins_dir.display());
    }
    else {
        unity_export::sync_msil_with_routes(&msil_dir, &unity_project, &plugin.export_routes)?;
        println!("unity: synced partitioned MSIL using export_routes");
    }
    Ok(ExitCode::SUCCESS)
}

fn run_status(args: &UnityProjectArgs) -> Result<ExitCode> {
    let manifest = load_project_manifest(&args.project_dir)?;
    let plugin = require_unity_plugin(&manifest)?;
    let build_args = BuildArgs {
        project_dir: args.project_dir.clone(),
        target: resolve_unity_build_target(&manifest),
        output_dir: None,
        workspace: false,
        debug_artifacts: false,
    };
    let plan = resolve_build_plan(&args.project_dir, &build_args)?;
    let msil_dir = unity_msil_dir(&plan, plugin);
    let export_path = msil_dir.join("valkyrie-export.json");

    println!("project: {}", plan.project.name);
    println!("target: {}", plan.project.build_target.target);
    println!("msil: {}", msil_dir.display());
    println!("export: {}", export_path.display());
    if export_path.is_file() {
        let manifest = unity_export::read_export_manifest_for_status(&export_path)?;
        println!("--- valkyrie-export.json ---");
        println!("version: {}", manifest.version);
        if let Some(entry) = &manifest.entry {
            println!("entry: {} -> {}", entry.assembly, manifest.entry_method.as_deref().unwrap_or("main"));
        }
        else if let Some(entry_assembly) = &manifest.entry_assembly {
            println!("entry_assembly: {}", entry_assembly);
        }
        if !manifest.artifacts.is_empty() {
            println!("artifacts:");
            for artifact in &manifest.artifacts {
                println!("  - {} partition={} role={}", artifact.assembly, artifact.partition.as_deref().unwrap_or("default"), artifact.role);
            }
        }
        let text = fs::read_to_string(&export_path).into_diagnostic()?;
        println!("{text}");
    }
    else {
        println!("status: export manifest missing (run `legion-unity build` or `legion-unity export`)");
    }

    Ok(ExitCode::SUCCESS)
}

fn load_project_manifest(project_dir: &Path) -> Result<ProjectManifest> {
    let manifest_path = project_dir.join("legion.von");
    let source =
        fs::read_to_string(&manifest_path).into_diagnostic().wrap_err_with(|| format!("读取项目清单失败：{}", manifest_path.display()))?;
    ProjectManifest::parse(&source).map_err(|error| miette!("{error}"))
}

fn require_unity_plugin(manifest: &ProjectManifest) -> Result<&BuildPluginSpec> {
    manifest
        .build_plugin
        .as_ref()
        .filter(|plugin| plugin.kind == "unity-project-export")
        .ok_or_else(|| miette!("项目 `{}` 未配置 `build_plugin: {{ kind: \"unity-project-export\", ... }}`", manifest.name))
}

fn resolve_unity_build_target(manifest: &ProjectManifest) -> CanonicalTarget {
    manifest
        .build
        .iter()
        .find(|item| item.publish.iter().any(|publish| publish == "unity-player"))
        .map(|item| item.target.clone())
        .unwrap_or_else(|| CanonicalTarget::parse("clr-microsoft-unknown-managed").expect("default clr target"))
}

fn resolve_build_plan(project_dir: &Path, build_args: &BuildArgs) -> Result<BuildPlan> {
    let workspace = LegionWorkspace::discover_for_project(project_dir)?;
    let request =
        BuildRequest { project_dir: project_dir.to_path_buf(), target: build_args.target.clone(), output_dir: build_args.output_dir.clone() };
    let (plan, _) = workspace.build_plan_with_local_fallback(&request)?;
    Ok(plan)
}

fn unity_msil_dir(plan: &BuildPlan, plugin: &BuildPluginSpec) -> PathBuf {
    plugin
        .input_directory
        .as_deref()
        .map(|path| plan.project.manifest_dir.join(path))
        .unwrap_or_else(|| plan.project.manifest_dir.join("build").join("unity").join("msil"))
}

fn read_export_entry_method(msil_dir: &Path) -> Option<String> {
    let export_path = msil_dir.join("valkyrie-export.json");
    let text = fs::read_to_string(&export_path).ok()?;
    let marker = "\"entry_method\"";
    let index = text.find(marker)?;
    let colon = text[index..].find(':')? + index;
    let start = text[colon..].find('"')? + colon + 1;
    let end = text[start..].find('"')? + start;
    Some(text[start..end].to_string())
}

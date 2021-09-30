#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

mod families;

use std::path::Path;

use miette::Result;
use nyar::{backends::CompilationOptions, packaging::ArtifactSet, RunnerFamily, TargetBackendFamily};
use valkyrie_compiler::{hir::HirModule, lir::LirModule};
use valkyrie_parser::ValkyrieRoot;

/// 驱动层编译请求。
#[derive(Debug)]
pub struct DriverCompileRequest<'a> {
    /// 前端语法根。
    pub parser_root: &'a ValkyrieRoot,
    /// 前端 `HIR`。
    pub hir_module: &'a HirModule,
    /// 已选择 lane 的 `LIR`。
    pub lir_module: LirModule,
    /// 输出目录。
    pub output_dir: &'a Path,
    /// 逻辑产物名。
    pub artifact_name: &'a str,
    /// 目标后端家族。
    pub backend_family: TargetBackendFamily,
    /// 运行家族。
    pub runner_family: RunnerFamily,
    /// 是否输出 `MSIL` sidecar。
    pub emit_msil: bool,
    /// 是否生成 runtime config。
    pub generate_runtime_config: bool,
    /// 通用编译选项。
    pub options: &'a CompilationOptions,
}

/// 驱动层运行契约。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverRunContract {
    /// 逻辑入口名。
    pub logical_entry: String,
    /// 物理入口文件。
    pub physical_entry: String,
    /// 调用命令。
    pub invocation: String,
    /// 校验命令。
    pub validate: String,
}

/// 驱动层编译结果。
#[derive(Debug, Default)]
pub struct DriverCompileReport {
    /// 产物集合。
    pub artifacts: ArtifactSet,
    /// 入口符号。
    pub entry_symbol: Option<String>,
    /// 可选运行契约。
    pub run_contract: Option<DriverRunContract>,
}

/// 使用 bundled backend 执行目标编译。
pub fn compile_with_bundled_backends(request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
    families::compile(request)
}

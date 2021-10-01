#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

mod families;

use clr_backend::ClrBinaryBackendInput;
use jvm_backend::JvmBinaryBackendInput;
use miette::Result;
use native_backend::NativeBinaryBackendInput;
use nyar::{
    backends::CompilationOptions, packaging::ArtifactSet, BackendInputKind, BinaryTarget, PartitionBackendRequirement, RunnerFamily, TargetLane,
};
use wasi_backend::WasmBinaryBackendInput;

/// 驱动层接收的目标专用输入。
#[derive(Debug, Clone)]
pub enum DriverBackendInput {
    /// `CLR` 二进制输入。
    Clr(ClrBinaryBackendInput),
    /// `JVM` 二进制输入。
    Jvm(JvmBinaryBackendInput),
    /// `WASM/WASI` 二进制输入。
    Wasm(WasmBinaryBackendInput),
    /// `native` 二进制输入。
    Native(NativeBinaryBackendInput),
}

impl DriverBackendInput {
    /// 根据当前输入与目标，生成驱动层使用的后端需求。
    pub fn requirement(&self, target: BinaryTarget) -> PartitionBackendRequirement {
        match self {
            Self::Clr(_) => PartitionBackendRequirement { lane: TargetLane::Clr, input_kind: BackendInputKind::MsilText, target },
            Self::Jvm(_) => PartitionBackendRequirement { lane: TargetLane::Jvm, input_kind: BackendInputKind::JvmClassFile, target },
            Self::Wasm(_) => PartitionBackendRequirement { lane: TargetLane::Wasm, input_kind: BackendInputKind::WasmModule, target },
            Self::Native(_) => PartitionBackendRequirement { lane: TargetLane::Native, input_kind: BackendInputKind::CoffObject, target },
        }
    }
}

/// 驱动层编译请求。
#[derive(Debug)]
pub struct DriverCompileRequest<'a> {
    /// 逻辑产物名。
    pub artifact_name: &'a str,
    /// 已经完成规划的后端需求。
    pub requirement: PartitionBackendRequirement,
    /// 目标专用输入。
    pub input: DriverBackendInput,
    /// 运行家族。
    pub runner_family: RunnerFamily,
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

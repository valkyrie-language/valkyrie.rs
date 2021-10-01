use miette::Result;
use nyar::{backends::TargetCodeGenBackend, BackendInputKind, PartitionBackendRequirement, TargetFamily, TargetLane};
use wasi_backend::{WasmBinaryBackend, WasmHostSkeleton};

use super::BundledFamilyCompiler;
use crate::{DriverBackendInput, DriverCompileReport, DriverCompileRequest, DriverRunContract};

pub(super) struct WasmFamilyCompiler;

pub(super) fn supports_requirement(requirement: &PartitionBackendRequirement) -> bool {
    requirement.lane == TargetLane::Wasm
        && requirement.input_kind == BackendInputKind::WasmModule
        && requirement.target.family == TargetFamily::Wasm
}

impl BundledFamilyCompiler for WasmFamilyCompiler {
    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let DriverBackendInput::Wasm(input) = request.input
        else {
            return Err(miette::miette!("`WASM` 家族请求必须携带 `WasmBinaryBackendInput`"));
        };
        let host = input.host;
        let backend = WasmBinaryBackend::new();
        backend.validate(&input)?;
        let artifacts = backend.compile(input, request.options)?;
        Ok(DriverCompileReport { artifacts, entry_symbol: None, run_contract: Some(wasm_run_contract(request.artifact_name, host)) })
    }
}

fn wasm_run_contract(artifact_name: &str, host: WasmHostSkeleton) -> DriverRunContract {
    match host {
        WasmHostSkeleton::Node => DriverRunContract {
            logical_entry: "main".to_string(),
            physical_entry: format!("{}.mjs", artifact_name),
            invocation: "node".to_string(),
            validate: format!("node {}.mjs", artifact_name),
        },
        WasmHostSkeleton::Wasi => DriverRunContract {
            logical_entry: "_start".to_string(),
            physical_entry: format!("{}.wasm", artifact_name),
            invocation: "wasmtime".to_string(),
            validate: format!("wasmtime {}.wasm", artifact_name),
        },
    }
}

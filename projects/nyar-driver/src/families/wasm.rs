use miette::Result;
use nyar::{
    backends::TargetCodeGenBackend, BackendInputKind, HostProjectionBoundary, PartitionBackendRequirement, ReferenceManagement, TargetFamily,
    TargetLane,
};
use wasi_backend::WasmBinaryBackend;

use super::BundledFamilyCompiler;
use crate::{DriverBackendInput, DriverCompileReport, DriverCompileRequest, DriverRunContract};

pub(super) struct WasmFamilyCompiler;

pub(super) fn supports_requirement(requirement: &PartitionBackendRequirement) -> bool {
    requirement.lane == TargetLane::Wasm
        && requirement.input_kind == BackendInputKind::WasmModule
        && requirement.target.family == TargetFamily::Wasm
        && matches!(requirement.host_boundary, HostProjectionBoundary::WasmJsGlue | HostProjectionBoundary::WasiComponent)
        && requirement.reference_management == ReferenceManagement::HostGc
}

impl BundledFamilyCompiler for WasmFamilyCompiler {
    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let DriverBackendInput::Wasm(input) = request.input
        else {
            return Err(miette::miette!("`WASM` 家族请求必须携带 `WasmBinaryBackendInput`"));
        };
        let expected_boundary = input.host_boundary;
        if request.requirement.host_boundary != expected_boundary {
            return Err(miette::miette!(
                "`WASM` 后端输入宿主与规划需求不一致：input={:?}, requirement={:?}",
                expected_boundary,
                request.requirement.host_boundary
            ));
        }
        let backend = WasmBinaryBackend::new();
        backend.validate(&input)?;
        let artifacts = backend.compile(input, request.options)?;
        Ok(DriverCompileReport {
            artifacts,
            entry_symbol: None,
            run_contract: Some(wasm_run_contract(request.artifact_name, request.requirement.host_boundary)),
        })
    }
}

fn wasm_run_contract(artifact_name: &str, boundary: HostProjectionBoundary) -> DriverRunContract {
    match boundary {
        HostProjectionBoundary::WasmJsGlue => DriverRunContract {
            logical_entry: "main".to_string(),
            physical_entry: format!("{}.mjs", artifact_name),
            invocation: "node".to_string(),
            validate: format!("node {}.mjs", artifact_name),
        },
        HostProjectionBoundary::WasiComponent => DriverRunContract {
            logical_entry: "_start".to_string(),
            physical_entry: format!("{}.wasm", artifact_name),
            invocation: "wasmtime".to_string(),
            validate: format!("wasmtime {}.wasm", artifact_name),
        },
        other => unreachable!("unexpected wasm host boundary: {:?}", other),
    }
}

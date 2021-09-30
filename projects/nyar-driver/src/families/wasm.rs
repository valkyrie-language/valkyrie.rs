use miette::Result;
use nyar::TargetBackendFamily;
use wasi_backend::{compile_wasm_bundle, WasmCompileRequest, WasmHostSkeleton};

use super::BundledFamilyCompiler;
use crate::{DriverCompileReport, DriverCompileRequest, DriverRunContract};

pub(super) struct WasmFamilyCompiler;

impl BundledFamilyCompiler for WasmFamilyCompiler {
    fn family(&self) -> TargetBackendFamily {
        TargetBackendFamily::Wasm
    }

    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let report = compile_wasm_bundle(WasmCompileRequest {
            hir_module: request.hir_module,
            lir_module: request.lir_module,
            output_dir: request.output_dir.to_path_buf(),
            artifact_name: request.artifact_name,
            runner_family: request.runner_family,
            options: request.options,
        })?;
        Ok(DriverCompileReport {
            artifacts: report.artifacts,
            entry_symbol: None,
            run_contract: Some(wasm_run_contract(request.artifact_name, report.host)),
        })
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

#[cfg(test)]
mod tests {
    use wasi_backend::WasmHostSkeleton;

    use super::wasm_run_contract;

    #[test]
    fn creates_node_run_contract() {
        let contract = wasm_run_contract("demo", WasmHostSkeleton::Node);

        assert_eq!(contract.logical_entry, "main");
        assert_eq!(contract.physical_entry, "demo.mjs");
        assert_eq!(contract.invocation, "node");
    }

    #[test]
    fn creates_wasi_run_contract() {
        let contract = wasm_run_contract("demo", WasmHostSkeleton::Wasi);

        assert_eq!(contract.logical_entry, "_start");
        assert_eq!(contract.physical_entry, "demo.wasm");
        assert_eq!(contract.validate, "wasmtime demo.wasm");
    }
}

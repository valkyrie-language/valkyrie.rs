use clr_backend::{compile_clr_bundle, ClrCompileRequest};
use miette::Result;
use nyar::TargetBackendFamily;

use super::BundledFamilyCompiler;
use crate::{DriverCompileReport, DriverCompileRequest, DriverRunContract};

pub(super) struct ClrFamilyCompiler;

impl BundledFamilyCompiler for ClrFamilyCompiler {
    fn family(&self) -> TargetBackendFamily {
        TargetBackendFamily::Clr
    }

    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let report = compile_clr_bundle(ClrCompileRequest {
            parser_root: request.parser_root,
            hir_module: request.hir_module,
            lir_module: request.lir_module,
            output_dir: request.output_dir,
            artifact_name: request.artifact_name,
            emit_msil: request.emit_msil,
            generate_runtime_config: request.generate_runtime_config,
            options: request.options,
        })?;

        Ok(DriverCompileReport {
            artifacts: report.artifacts,
            entry_symbol: report.entry_symbol.clone(),
            run_contract: Some(DriverRunContract {
                logical_entry: report.entry_symbol.unwrap_or_default(),
                physical_entry: report.artifact_file_name.clone(),
                invocation: "dotnet".to_string(),
                validate: format!("dotnet exec {}", report.artifact_file_name),
            }),
        })
    }
}

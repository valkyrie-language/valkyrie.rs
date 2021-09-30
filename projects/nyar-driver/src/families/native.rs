use miette::Result;
use native_backend::{compile_native_bundle, NativeCompileRequest};
use nyar::TargetBackendFamily;

use super::BundledFamilyCompiler;
use crate::{DriverCompileReport, DriverCompileRequest};

pub(super) struct NativeFamilyCompiler;

impl BundledFamilyCompiler for NativeFamilyCompiler {
    fn family(&self) -> TargetBackendFamily {
        TargetBackendFamily::Native
    }

    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let report = compile_native_bundle(NativeCompileRequest {
            lir_module: request.lir_module,
            output_dir: request.output_dir.to_path_buf(),
            options: request.options,
        })?;
        Ok(DriverCompileReport { artifacts: report.artifacts, entry_symbol: None, run_contract: None })
    }
}

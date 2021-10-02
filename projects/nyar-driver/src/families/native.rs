use miette::Result;
use native_backend::NativeBinaryBackend;
use nyar::{
    backends::TargetCodeGenBackend, BackendInputKind, HostProjectionBoundary, PartitionBackendRequirement, ReferenceManagement, TargetFamily,
    TargetLane,
};

use super::BundledFamilyCompiler;
use crate::{DriverBackendInput, DriverCompileReport, DriverCompileRequest};

pub(super) struct NativeFamilyCompiler;

pub(super) fn supports_requirement(requirement: &PartitionBackendRequirement) -> bool {
    requirement.lane == TargetLane::Native
        && requirement.input_kind == BackendInputKind::CoffObject
        && requirement.target.family == TargetFamily::Native
        && requirement.host_boundary == HostProjectionBoundary::Native
        && requirement.reference_management == ReferenceManagement::PerceusRc
}

impl BundledFamilyCompiler for NativeFamilyCompiler {
    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let DriverBackendInput::Native(input) = request.input
        else {
            return Err(miette::miette!("`native` 家族请求必须携带 `NativeBinaryBackendInput`"));
        };
        let backend = NativeBinaryBackend::new();
        backend.validate(&input)?;
        let artifacts = backend.compile(input, request.options)?;
        Ok(DriverCompileReport { artifacts, entry_symbol: None, run_contract: None })
    }
}

use clr_backend::{write_dotnet_runtime_config, ClrBinaryBackend};
use miette::Result;
use nyar::{
    backends::{clr::ClrImageKind, TargetCodeGenBackend},
    BackendInputKind, HostProjectionBoundary, PartitionBackendRequirement, ReferenceManagement, TargetFamily, TargetLane,
};

use super::BundledFamilyCompiler;
use crate::{DriverBackendInput, DriverCompileReport, DriverCompileRequest, DriverRunContract};

pub(super) struct ClrFamilyCompiler;

pub(super) fn supports_requirement(requirement: &PartitionBackendRequirement) -> bool {
    requirement.lane == TargetLane::Clr
        && requirement.input_kind == BackendInputKind::MsilText
        && requirement.target.family == TargetFamily::Clr
        && requirement.host_boundary == HostProjectionBoundary::Clr
        && requirement.reference_management == ReferenceManagement::HostGc
}

impl BundledFamilyCompiler for ClrFamilyCompiler {
    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let DriverBackendInput::Clr(input) = request.input
        else {
            return Err(miette::miette!("`CLR` 家族请求必须携带 `ClrBinaryBackendInput`"));
        };
        let has_entry = input.module.global_methods.iter().any(|method| method.is_entry_point)
            || input.module.types.iter().flat_map(|ty| ty.methods.iter()).any(|method| method.is_entry_point);
        let image_kind = input.image_kind.unwrap_or_else(|| ClrImageKind::infer(has_entry));
        let output_dir = input.output_dir.clone();
        let backend = ClrBinaryBackend::new();
        backend.validate(&input)?;
        let artifacts = backend.compile(input, request.options)?;
        if request.generate_runtime_config {
            write_dotnet_runtime_config(&output_dir, request.artifact_name)?;
        }

        Ok(DriverCompileReport {
            artifacts,
            entry_symbol: if image_kind == ClrImageKind::Executable { Some("Main".to_string()) } else { None },
            run_contract: Some(DriverRunContract {
                logical_entry: "Main".to_string(),
                physical_entry: format!("{}.{}", request.artifact_name, image_kind.file_extension()),
                invocation: "dotnet".to_string(),
                validate: format!("dotnet exec {}.{}", request.artifact_name, image_kind.file_extension()),
            }),
        })
    }
}

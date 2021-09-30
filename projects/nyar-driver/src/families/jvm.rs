use jvm_backend::{compile_jvm_bundle, JvmCompileRequest};
use miette::Result;
use nyar::TargetBackendFamily;

use super::BundledFamilyCompiler;
use crate::{DriverCompileReport, DriverCompileRequest, DriverRunContract};

/// `JVM` 后端家族编译器。
pub(super) struct JvmFamilyCompiler;

impl BundledFamilyCompiler for JvmFamilyCompiler {
    fn family(&self) -> TargetBackendFamily {
        TargetBackendFamily::Jvm
    }

    fn compile(&self, request: DriverCompileRequest<'_>) -> Result<DriverCompileReport> {
        let report = compile_jvm_bundle(JvmCompileRequest {
            lir_module: request.lir_module,
            output_dir: request.output_dir.to_path_buf(),
            emit_class_file: true,
            options: request.options,
        })?;
        Ok(DriverCompileReport {
            artifacts: report.artifacts,
            entry_symbol: Some(report.entry_class.clone()),
            run_contract: Some(jvm_run_contract(request.artifact_name, &report.entry_class)),
        })
    }
}

/// 构造 `java -jar` 运行契约。
fn jvm_run_contract(artifact_name: &str, entry_class: &str) -> DriverRunContract {
    DriverRunContract {
        logical_entry: entry_class.to_string(),
        physical_entry: format!("{}.jar", artifact_name),
        invocation: "java".to_string(),
        validate: format!("java -jar {}.jar", artifact_name),
    }
}

#[cfg(test)]
mod tests {
    use super::jvm_run_contract;

    #[test]
    fn creates_jar_run_contract() {
        let contract = jvm_run_contract("demo", "legion.tools.Main");

        assert_eq!(contract.logical_entry, "legion.tools.Main");
        assert_eq!(contract.physical_entry, "demo.jar");
        assert_eq!(contract.invocation, "java");
        assert_eq!(contract.validate, "java -jar demo.jar");
    }
}

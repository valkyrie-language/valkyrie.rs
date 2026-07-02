//! Compile V sources to host bytecode for a target family.

use std::{fs, path::Path};

use emitter::{
    FragmentSubmission, FrontendBuildBundle, LoweredBackendInput, PlannedArtifactPartitionsView, bundled_backend_registry,
    compile_frontend_bundle_with_bundled_backends,
};
use miette::{IntoDiagnostic, Result, WrapErr};
use nyar_language::{
    ArtifactPartitionPlan, CanonicalTarget, FrontendBuildOutput, ValkyrieCompiler, assemble_fragment_submission,
    nyar::{ClrSuspendStrategy, HostProjectionBoundary, TargetBackendFamily, TargetLane, projection_policy_for_target_profile},
    plan_artifacts_from_build_output,
};

use crate::{
    host_backend::HostBackend,
    wasm::{WasmCompileReport, copy_wasm_artifacts_to_dist},
};

/// Host artifact kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HostArtifactKind {
    /// WebAssembly module.
    Wasm,
    /// JVM `.class`.
    JvmClass,
    /// Native executable.
    NativeExecutable,
}

/// Host compile report.
#[derive(Debug, Clone)]
pub struct HostCompileReport {
    /// Primary artifact bytes.
    pub artifact_bytes: Vec<u8>,
    /// Artifact kind.
    pub kind: HostArtifactKind,
    /// WASM-specific report (browser).
    pub wasm: Option<WasmCompileReport>,
}

/// Compile V sources to host bytecode.
pub fn compile_v_bundle(
    combined_v_source: &str,
    output_dir: &Path,
    module_name: &str,
    target: &CanonicalTarget,
    backend: HostBackend,
) -> Result<HostCompileReport> {
    let compiler = ValkyrieCompiler::default();
    let build_output = compiler.compile_source_to_build_output(combined_v_source).map_err(|error| miette::miette!("{error}"))?;
    let target_profile = target.to_profile(None);
    let projection_policy = projection_policy_for_target_profile(&target_profile)?;
    let backend_registry = bundled_backend_registry(&build_output.neutral_plan().semantic_fragments, &target_profile, &projection_policy);
    let artifact_plan =
        plan_artifacts_from_build_output(&build_output, target.clone(), projection_policy, backend_registry, ClrSuspendStrategy::default())
            .map_err(|error| miette::miette!("frontend partition planning failed: {error:?}"))?;
    let driver_bundle = VoaFrontendBuildAdapter::new(build_output, artifact_plan);

    fs::create_dir_all(output_dir).into_diagnostic().wrap_err("failed to create output directory")?;

    let _report = compile_frontend_bundle_with_bundled_backends(
        &driver_bundle,
        output_dir,
        module_name,
        false,
        true,
        target_profile.artifact_policy.generate_runtime_config,
    )?;

    match backend {
        HostBackend::BrowserDom => {
            let safe_name = module_name.replace('.', "-");
            let wasm_report = WasmCompileReport { wasm_filename: format!("{safe_name}.wasm"), glue_filename: format!("{safe_name}.mjs") };
            copy_wasm_artifacts_to_dist(output_dir, &wasm_report)?;
            let wasm_path = output_dir.join(&wasm_report.wasm_filename);
            let bytes = fs::read(&wasm_path).into_diagnostic().wrap_err("failed to read wasm")?;
            Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::Wasm, wasm: Some(wasm_report) })
        }
        HostBackend::AndroidCompose => match find_android_shared_object(output_dir) {
            Some(native_path) => {
                let bytes = fs::read(&native_path).into_diagnostic()?;
                validate_elf_shared_object(&bytes)?;
                Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::NativeExecutable, wasm: None })
            }
            None => android_compose_fallback_or_error(),
        },
        HostBackend::IosSwiftUi => {
            let native_path = find_native_executable(output_dir).ok_or_else(|| miette::miette!("native executable artifact not found"))?;
            let bytes = fs::read(&native_path).into_diagnostic()?;
            Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::NativeExecutable, wasm: None })
        }
        HostBackend::WindowsNative | HostBackend::LinuxNative | HostBackend::MacOsNative => {
            let native_path =
                find_native_executable(output_dir).ok_or_else(|| miette::miette!("desktop native executable artifact not found"))?;
            let bytes = fs::read(&native_path).into_diagnostic()?;
            Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::NativeExecutable, wasm: None })
        }
        HostBackend::WechatMiniProgram => {
            let safe_name = module_name.replace('.', "-");
            let wasm_report = WasmCompileReport { wasm_filename: format!("{safe_name}.wasm"), glue_filename: format!("{safe_name}.mjs") };
            copy_wasm_artifacts_to_dist(output_dir, &wasm_report)?;
            let wasm_path = output_dir.join(&wasm_report.wasm_filename);
            let bytes = fs::read(&wasm_path).into_diagnostic().wrap_err("failed to read miniprogram wasm")?;
            Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::Wasm, wasm: Some(wasm_report) })
        }
        HostBackend::Terminal => {
            let native_path =
                find_native_executable(output_dir).ok_or_else(|| miette::miette!("terminal native executable artifact not found"))?;
            let bytes = fs::read(&native_path).into_diagnostic()?;
            Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::NativeExecutable, wasm: None })
        }
    }
}

fn find_artifact(dir: &Path, ext: &str) -> Option<std::path::PathBuf> {
    let mut stack = vec![dir.to_path_buf()];
    while let Some(current) = stack.pop() {
        for entry in fs::read_dir(&current).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            }
            else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
                return Some(path);
            }
        }
    }
    None
}

fn find_native_executable(dir: &Path) -> Option<std::path::PathBuf> {
    find_artifact(dir, "exe").or_else(|| find_artifact(dir, "")).or_else(|| {
        let mut stack = vec![dir.to_path_buf()];
        while let Some(current) = stack.pop() {
            for entry in fs::read_dir(&current).ok()? {
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_file() && path.extension().is_none() {
                    return Some(path);
                }
                if path.is_dir() {
                    stack.push(path);
                }
            }
        }
        None
    })
}

fn find_android_shared_object(dir: &Path) -> Option<std::path::PathBuf> {
    find_artifact(dir, "so")
}

pub fn validate_elf_shared_object(bytes: &[u8]) -> Result<()> {
    if bytes.len() < 20 || &bytes[0..4] != b"\x7fELF" {
        return Err(miette::miette!("Android host artifact is not ELF (need .so)"));
    }
    let e_type = u16::from_le_bytes([bytes[16], bytes[17]]);
    if e_type != 3 {
        return Err(miette::miette!("Android host must be ET_DYN shared library (.so), e_type={e_type}"));
    }
    let e_machine = u16::from_le_bytes([bytes[18], bytes[19]]);
    if e_machine != 183 {
        return Err(miette::miette!("Android host must be AArch64 (EM_AARCH64=183), e_machine={e_machine}"));
    }
    let haystack = String::from_utf8_lossy(bytes);
    for sym in ["JNI_OnLoad", "asgard_invoke_export", "asgard_patch_native"] {
        if !haystack.contains(sym) {
            return Err(miette::miette!("Android .so missing export `{sym}` (JNI must be linked into ASGARDNT)"));
        }
    }
    Ok(())
}

/// Minimal ET_DYN ELF (pipeline integration test when `cfg(test)` and no NDK).
#[cfg(test)]
fn android_compose_fallback_or_error() -> Result<HostCompileReport> {
    let bytes = minimal_test_elf_shared_object();
    Ok(HostCompileReport { artifact_bytes: bytes, kind: HostArtifactKind::NativeExecutable, wasm: None })
}

#[cfg(not(test))]
fn android_compose_fallback_or_error() -> Result<HostCompileReport> {
    Err(miette::miette!("Android .so (ET_DYN) AOT artifact not found; Android logic must be a shared library"))
}

/// Minimal ET_DYN ELF (tests only).
#[cfg(test)]
fn minimal_test_elf_shared_object() -> Vec<u8> {
    use std_data::binary::{
        aarch64::{emit_jni_glue_module, merge_jni_and_logic, ret_bytes},
        elf::{SharedElfWriter, SharedObjectExport},
    };
    let jni = emit_jni_glue_module().expect("jni");
    let (image, exports) =
        merge_jni_and_logic(jni, ret_bytes().to_vec(), vec![SharedObjectExport { name: "asgard_invoke_export".into(), text_offset: 0 }])
            .expect("merge");
    SharedElfWriter::write_aarch64(&image, &exports).unwrap_or_else(|_| {
        let mut bytes = vec![0u8; 64];
        bytes[0..4].copy_from_slice(b"\x7fELF");
        bytes[4] = 2;
        bytes[5] = 1;
        bytes[16] = 3;
        bytes[18] = 183;
        bytes[19] = 0;
        bytes
    })
}

struct VoaFrontendBuildAdapter {
    build_output: FrontendBuildOutput,
    artifact_plan: ArtifactPartitionPlan,
}

impl VoaFrontendBuildAdapter {
    fn new(build_output: FrontendBuildOutput, artifact_plan: ArtifactPartitionPlan) -> Self {
        Self { build_output, artifact_plan }
    }
}

impl FrontendBuildBundle for VoaFrontendBuildAdapter {
    fn planned_partitions(&self) -> &dyn PlannedArtifactPartitionsView {
        self
    }

    fn submit_backend_input_for_partition(
        &self,
        partition_index: usize,
        backend_family: TargetBackendFamily,
        host_boundary: HostProjectionBoundary,
        output_dir: &Path,
        _lane: TargetLane,
    ) -> Result<LoweredBackendInput> {
        let fragment = assemble_fragment_submission(&self.build_output, &self.artifact_plan, partition_index)
            .map_err(|error| miette::miette!("{error}"))?;
        let host_flavor = self.artifact_plan.target.to_profile(None).host_flavor;
        LoweredBackendInput::from_fragment_submission(
            &fragment,
            backend_family,
            host_boundary,
            output_dir,
            self.artifact_plan.partitions.get(partition_index).map(|partition| partition.lane).unwrap_or(TargetLane::Wasm),
            self.artifact_plan.partitions.get(partition_index).map(|partition| partition.clr_suspend_strategy).unwrap_or_default(),
            nyar::VmSuspendStrategy::default(),
            &host_flavor,
        )
        .map_err(|error| miette::miette!("{error}"))
    }
}

impl PlannedArtifactPartitionsView for VoaFrontendBuildAdapter {
    fn primary_partition_name(&self) -> Option<String> {
        self.artifact_plan
            .partitions
            .iter()
            .find(|partition| partition.name.ends_with("::functions"))
            .map(|partition| partition.name.clone())
            .or_else(|| self.artifact_plan.partitions.first().map(|partition| partition.name.clone()))
    }

    fn partition_count(&self) -> usize {
        self.artifact_plan.partitions.len()
    }

    fn partition(&self, partition_index: usize) -> Option<&emitter::ArtifactPartition> {
        self.artifact_plan.partitions.get(partition_index)
    }

    fn backend_requirement(&self, partition_index: usize) -> Option<nyar_language::nyar::PartitionBackendRequirement> {
        self.artifact_plan.backend_requirement(partition_index)
    }
}

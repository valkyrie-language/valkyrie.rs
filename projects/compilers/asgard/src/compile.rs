//! Compile V sources to host bytecode for a target family.

use std::{fs, path::Path};

use legion_workspace::{compile_source_snapshot, planner::{BuildRequest, WorkspaceResolver}};
use miette::{IntoDiagnostic, Result, WrapErr};
use nyar_language::{compile_source_groups_to_artifacts, CanonicalTarget, CompilerSourceGroup};

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

/// Resolver 形成的完整源码快照。
#[derive(Debug, Clone)]
pub struct ResolvedCompilerSources {
    /// 应用组身份。
    pub project_name: String,
    /// 按依赖顺序排列的源码组。
    pub groups: Vec<CompilerSourceGroup>,
}

/// 从 manifest 与当前源码闭包形成 Compiler 输入。
pub fn resolve_project_source_groups(project_dir: &Path, target: &CanonicalTarget) -> Result<ResolvedCompilerSources> {
    let resolver = WorkspaceResolver::discover_for_project(project_dir)
        .map_err(|error| miette::miette!("解析项目工作区失败: {error}"))?;
    let plan = resolver
        .build_plan(&BuildRequest { project_dir: project_dir.to_path_buf(), target: target.clone(), output_dir: None })
        .map_err(|error| miette::miette!("解析项目构建计划失败: {error}"))?;
    let project_name = plan.project.name.clone();
    let groups = compile_source_snapshot(&plan.project.semantic_source_groups)
        .map_err(|error| miette::miette!("读取项目源码快照失败: {error}"))?;
    Ok(ResolvedCompilerSources { project_name, groups })
}

/// 将编译器生成的显式源码附着到已解析的应用组。
pub fn append_generated_source(sources: &ResolvedCompilerSources, generated_source: &str) -> Result<Vec<CompilerSourceGroup>> {
    let mut groups = sources.groups.clone();
    let application = groups
        .iter_mut()
        .find(|group| group.dependency_key == sources.project_name)
        .ok_or_else(|| miette::miette!("源码快照缺少应用组 `{}`", sources.project_name))?;
    if !generated_source.trim().is_empty() {
        application.source.push('\n');
        application.source.push_str(generated_source);
        application.source.push('\n');
    }
    Ok(groups)
}

/// 将已解析的源码组编译为宿主制品。
pub fn compile_v_bundle(
    source_groups: &[CompilerSourceGroup],
    output_dir: &Path,
    module_name: &str,
    target: &CanonicalTarget,
    backend: HostBackend,
) -> Result<HostCompileReport> {
    let compiler = nyar_language::ValkyrieCompiler::default();
    let target_profile = target.to_profile(None);
    fs::create_dir_all(output_dir).into_diagnostic().wrap_err("failed to create output directory")?;
    let _report = compile_source_groups_to_artifacts(
        &compiler,
        &source_groups,
        target.arch.as_str(),
        target.clone(),
        nyar_language::nyar::ClrSuspendStrategy::default(),
        emitter::nyar_backend_wasi::WasmPackageKind::Binary,
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
    use vcc_data::binary::{
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

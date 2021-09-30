//! `Native/MSVC` 后端。
//!
//! 这里承载原生对象文件骨架输出。

pub mod lowering;

use std::path::PathBuf;

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use nyar::{
    abstractions::{ArtifactFormat, BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    backends::{BackendDescriptor, CompilationOptions, TargetCodeGenBackend},
    packaging::{ArtifactDescriptor, ArtifactSet, TargetLane},
    TargetLoweringLane,
};
use nyar_binary_format::{CoffObject, CoffObjectWriter};
use valkyrie_compiler::lir::LirModule;

pub use lowering::{lower_lir_to_native_assembly, NativeLirLoweringLane};
pub use nyar_binary_format::{CoffHeader, CoffMachine, CoffRelocation, CoffRelocationKind, CoffSection, CoffSymbol};

/// `Native/MSVC` 高层编译请求。
#[derive(Debug)]
pub struct NativeCompileRequest<'a> {
    /// 已选择 lane 的 `LIR`。
    pub lir_module: LirModule,
    /// 输出目录。
    pub output_dir: PathBuf,
    /// 通用编译选项。
    pub options: &'a CompilationOptions,
}

/// `Native/MSVC` 高层编译结果。
#[derive(Debug, Default)]
pub struct NativeCompileReport {
    /// 产物集合。
    pub artifacts: ArtifactSet,
}

/// `Native/MSVC` 后端输入。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeBinaryBackendInput {
    /// `COFF` 目标文件。
    pub object: CoffObject,
    /// 输出目录。
    pub output_dir: PathBuf,
}

/// `Native/MSVC` 二进制后端。
pub struct NativeBinaryBackend {
    descriptor: BackendDescriptor,
}

impl NativeBinaryBackend {
    /// 创建一个新的 `Native/MSVC` 二进制后端。
    pub fn new() -> Self {
        Self {
            descriptor: BackendDescriptor {
                name: "native-msvc".to_string(),
                input_kind: BackendInputKind::CoffObject,
                supported_targets: vec![
                    BinaryTarget::new(TargetFamily::Native, BinaryArch::X64, BinaryFlavor::Native),
                    BinaryTarget::new(TargetFamily::Native, BinaryArch::Arm64, BinaryFlavor::Native),
                    BinaryTarget::new(TargetFamily::Native, BinaryArch::X86, BinaryFlavor::Native),
                ],
            },
        }
    }
}

impl Default for NativeBinaryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetCodeGenBackend for NativeBinaryBackend {
    type Input = NativeBinaryBackendInput;

    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    fn validate(&self, input: &Self::Input) -> Result<()> {
        if input.object.symbols.is_empty() {
            return Err(miette!("Native 后端至少需要一个函数骨架"));
        }

        Ok(())
    }

    fn compile(&self, input: Self::Input, options: &CompilationOptions) -> Result<ArtifactSet> {
        std::fs::create_dir_all(&input.output_dir)
            .into_diagnostic()
            .wrap_err_with(|| format!("创建输出目录失败：{}", input.output_dir.display()))?;

        let object_bytes = CoffObjectWriter::write(&input.object)?;
        let object_path = input.output_dir.join(format!("{}.obj", options.artifact_name));
        std::fs::write(&object_path, object_bytes)
            .into_diagnostic()
            .wrap_err_with(|| format!("写入 `COFF` 目标文件失败：{}", object_path.display()))?;

        let mut artifacts = ArtifactSet::default();
        artifacts.push(ArtifactDescriptor {
            name: options.artifact_name.clone(),
            kind: nyar::ArtifactKind::Object,
            format: ArtifactFormat::Coff,
            target: options.target.clone(),
            lane: TargetLane::Native,
        });
        Ok(artifacts)
    }
}

/// 使用 `Native/MSVC` bundled backend 完成完整编译流程。
pub fn compile_native_bundle(request: NativeCompileRequest<'_>) -> Result<NativeCompileReport> {
    let lane = NativeLirLoweringLane::new();
    let lowered = lane.lower_partition(request.lir_module)?;
    let backend = NativeBinaryBackend::new();
    let input = NativeBinaryBackendInput { object: lowered.input, output_dir: request.output_dir };
    backend.validate(&input)?;
    let artifacts = backend.compile(input, request.options)?;
    Ok(NativeCompileReport { artifacts })
}

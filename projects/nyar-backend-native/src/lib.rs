#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

use std::path::PathBuf;

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use nyar::{
    abstractions::{ArtifactFormat, BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    backends::{BackendDescriptor, CompilationOptions, TargetCodeGenBackend},
    packaging::{ArtifactDescriptor, ArtifactSet, TargetLane},
};
use nyar_binary_format::{CoffObject, CoffObjectWriter};

pub use nyar_binary_format::{CoffHeader, CoffMachine, CoffRelocation, CoffRelocationKind, CoffSection, CoffSymbol};

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

//! `CLR` 二进制后端。
//!
//! 直接写入 `PE/COFF` 二进制，不依赖 `ilasm`，保证跨平台可用。
//! `MSIL` 模型、文本读写与二进制编码统一收口在本 crate。

#![warn(missing_docs)]

pub mod hosting;
pub mod interop;
pub mod lowering;
pub mod metadata;
pub mod msil;
pub mod pe;
pub mod pipeline;

pub use hosting::{write_dotnet_runtime_config, DotNetFramework, DotNetRuntimeConfig, DotNetRuntimeOptions};
pub use interop::{build_clr_method_signature, resolve_clr_import_ref};
pub use lowering::{lower_lir_to_msil, ClrLirLoweringLane};
pub use metadata::{ClrMetadataBuilder, ClrMetadataError};
pub use msil::{
    MethodBodyEncoder, MethodBodyError, MsilAssembly, MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilMethodRef,
    MsilMethodSignature, MsilModule, MsilOpcode, MsilParser, MsilTextMethod, MsilTextWriter, MsilType, MsilTypeDef,
};
pub use nyar_binary_format::{CoffHeader, CoffMachine, CoffObject, CoffRelocation, CoffRelocationKind, CoffSection, CoffSymbol};
pub use pe::{PeWriter, PeWriterError, PeWriterOptions};
pub use pipeline::{compile_clr_bundle, ClrCompileReport, ClrCompileRequest};

use std::path::PathBuf;

use miette::{miette, IntoDiagnostic, Result, WrapErr};
use nyar::{
    abstractions::{ArtifactFormat, BackendInputKind, BinaryTarget},
    backends::{clr::ClrImageKind, BackendDescriptor, CompilationOptions, TargetCodeGenBackend},
    packaging::{ArtifactDescriptor, ArtifactSet, TargetLane},
};
/// `CLR` 二进制后端输入。
#[derive(Debug, Clone)]
pub struct ClrBinaryBackendInput {
    /// `MSIL` 模块。
    pub module: MsilModule,
    /// 输出目录。
    pub output_dir: PathBuf,
    /// 期望生成的镜像口味；为空时自动推断。
    pub image_kind: Option<ClrImageKind>,
}

/// `CLR` 二进制后端。
///
/// 将 `MsilModule` 直接编码为 `PE/COFF` 二进制 `.exe`，
/// 不调用 `ilasm`，保证 `Linux` 上也可运行（生成产物供 `Windows` 运行）。
pub struct ClrBinaryBackend {
    /// 后端描述。
    descriptor: BackendDescriptor,
}

impl ClrBinaryBackend {
    /// 创建一个新的 `CLR` 二进制后端。
    pub fn new() -> Self {
        Self {
            descriptor: BackendDescriptor {
                name: "clr-binary".to_string(),
                input_kind: BackendInputKind::MsilText,
                supported_targets: vec![BinaryTarget::clr()],
            },
        }
    }
}

impl Default for ClrBinaryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetCodeGenBackend for ClrBinaryBackend {
    type Input = ClrBinaryBackendInput;

    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    fn validate(&self, input: &Self::Input) -> Result<()> {
        if input.module.assembly.name.is_empty() {
            return Err(miette!(code = "nyar::clr::backend::empty_assembly", help = "请先为 `MSIL` 模块填充程序集名称", "程序集名称不能为空"));
        }

        let has_entry = input.module.global_methods.iter().any(|m| m.is_entry_point)
            || input.module.types.iter().flat_map(|t| t.methods.iter()).any(|m| m.is_entry_point);

        if matches!(input.image_kind, Some(ClrImageKind::Executable)) && !has_entry {
            return Err(miette!(
                code = "nyar::clr::backend::no_entry_point",
                help = "可执行 `CLR` 镜像必须包含入口点方法",
                "可执行 `CLR` 镜像必须包含入口点"
            ));
        }

        Ok(())
    }

    fn compile(&self, input: Self::Input, options: &CompilationOptions) -> Result<ArtifactSet> {
        let has_entry = input.module.global_methods.iter().any(|m| m.is_entry_point)
            || input.module.types.iter().flat_map(|t| t.methods.iter()).any(|m| m.is_entry_point);
        let image_kind = input.image_kind.unwrap_or_else(|| ClrImageKind::infer(has_entry));
        let module_file_name = format!("{}.{}", options.artifact_name, image_kind.file_extension());
        let pe_bytes = PeWriter::new(PeWriterOptions {
            assembly_name: input.module.assembly.name.clone(),
            module_name: module_file_name.clone(),
            image_kind,
        })
        .write_module(&input.module)
        .wrap_err("PE 写入失败")?;

        std::fs::create_dir_all(&input.output_dir)
            .into_diagnostic()
            .wrap_err_with(|| format!("创建输出目录失败：{}", input.output_dir.display()))?;

        let artifact_path = input.output_dir.join(&module_file_name);
        std::fs::write(&artifact_path, &pe_bytes)
            .into_diagnostic()
            .wrap_err_with(|| format!("写入 PE 文件失败：{}", artifact_path.display()))?;

        let mut artifacts = ArtifactSet::default();
        artifacts.push(ArtifactDescriptor {
            name: options.artifact_name.clone(),
            kind: image_kind.artifact_kind(),
            format: ArtifactFormat::Pe,
            target: options.target.clone(),
            lane: TargetLane::Clr,
        });

        Ok(artifacts)
    }
}

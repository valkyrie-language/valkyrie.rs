//! `WASI` / `WebAssembly` 后端容器入口。
//!
//! 这里按 `wasm / wat / wit` 三个输出边界收口，
//! 不再复用 `CLR` 的 `MSIL / PE / COFF` 结构。

#![warn(missing_docs)]

mod binding_builders;
pub mod wasm;
pub mod wat;
pub mod wit;

use std::path::PathBuf;

use miette::{IntoDiagnostic, Result, WrapErr};
use nyar::{
    abstractions::{ArtifactFormat, BackendInputKind, BinaryTarget},
    backends::{BackendDescriptor, CompilationOptions, TargetCodeGenBackend},
    packaging::{ArtifactDescriptor, ArtifactSet, TargetLane},
    HostProjectionBoundary,
};

use binding_builders::{generate_host_binding_artifacts, BindingGenerationContext};
pub use wasm::{WasmBinaryError, WasmBinaryModule, WasmCustomSection, WasmSection};
pub use wat::{WatDocument, WatError};
pub use wit::{WitError, WitInterface, WitPackage};

/// `WASM/WASI` 二进制后端输入。
#[derive(Debug, Clone)]
pub struct WasmBinaryBackendInput {
    /// `WASM` 模块。
    pub module: WasmBinaryModule,
    /// 输出目录。
    pub output_dir: PathBuf,
    /// 宿主投影边界。
    pub host_boundary: HostProjectionBoundary,
    /// 导入声明列表（`(module, field)` 对），用于生成 `WasmJsGlue` 启动壳的 `import` 实现。
    pub imports: Vec<(String, String)>,
}

/// `WASM/WASI` 二进制后端。
pub struct WasmBinaryBackend {
    descriptor: BackendDescriptor,
}

impl WasmBinaryBackend {
    /// 创建一个新的 `WASM/WASI` 二进制后端。
    pub fn new() -> Self {
        Self {
            descriptor: BackendDescriptor {
                name: "wasm-binary".to_string(),
                input_kind: BackendInputKind::WasmModule,
                supported_targets: vec![BinaryTarget::new(nyar::TargetFamily::Wasm, nyar::BinaryArch::Any, nyar::BinaryFlavor::Native)],
            },
        }
    }
}

impl Default for WasmBinaryBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetCodeGenBackend for WasmBinaryBackend {
    type Input = WasmBinaryBackendInput;

    fn descriptor(&self) -> &BackendDescriptor {
        &self.descriptor
    }

    fn validate(&self, input: &Self::Input) -> Result<()> {
        match input.host_boundary {
            HostProjectionBoundary::WasmJsGlue | HostProjectionBoundary::WasiComponent => Ok(()),
            other => Err(miette::miette!("`WASM` 后端只支持 `WasmJsGlue/WasiComponent`，实际得到 {:?}", other)),
        }
    }

    fn compile(&self, input: Self::Input, options: &CompilationOptions) -> Result<ArtifactSet> {
        std::fs::create_dir_all(&input.output_dir)
            .into_diagnostic()
            .wrap_err_with(|| format!("创建输出目录失败：{}", input.output_dir.display()))?;

        let wasm_path = input.output_dir.join(format!("{}.wasm", options.artifact_name));
        let wasm_bytes = input.module.to_bytes().map_err(|error| miette::miette!("WASM 写入失败：{error}"))?;
        std::fs::write(&wasm_path, wasm_bytes).into_diagnostic().wrap_err_with(|| format!("写入 WASM 文件失败：{}", wasm_path.display()))?;

        let mut artifacts = ArtifactSet::default();
        artifacts.push(ArtifactDescriptor {
            name: options.artifact_name.clone(),
            kind: nyar::ArtifactKind::Executable,
            format: ArtifactFormat::RawBinary,
            target: options.target.clone(),
            lane: TargetLane::Wasm,
        });

        let binding_artifacts = generate_host_binding_artifacts(
            input.host_boundary,
            BindingGenerationContext {
                artifact_name: &options.artifact_name,
                output_dir: &input.output_dir,
                target: &options.target,
                imports: &input.imports,
            },
        )?;
        for artifact in binding_artifacts {
            artifacts.push(artifact);
        }

        Ok(artifacts)
    }
}

//! Frontend-owned bridge from `HIR/LIR` into target-specific `nyar-driver` inputs.

mod clr_lowering;
mod clr_runtime_lowering;
mod jvm_lowering;
mod native_lowering;
mod wasi_interop;
mod wasi_lowering;

use std::{collections::HashSet, path::Path};

use clr_backend::{ClrBinaryBackendInput, MsilTextWriter};
use jvm_backend::JvmBinaryBackendInput;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use native_backend::NativeBinaryBackendInput;
use nyar_driver::DriverBackendInput;
use wasi_backend::{WasmBinaryBackendInput, WasmHostSkeleton};

use crate::{hir::HirModule, lir::LirModule, RunnerFamily, TargetBackendFamily};

pub use clr_lowering::{lower_lir_to_msil, ClrLirLoweringLane};
pub use jvm_lowering::{lower_lir_to_jvm_class, JvmLirLoweringLane};
pub use native_lowering::{lower_lir_to_native_assembly, NativeLirLoweringLane};
pub use wasi_interop::{collect_wasm_imports, resolve_wasm_import, WasmImport};
pub use wasi_lowering::{lower_lir_to_wasm_module, WasmLirLoweringLane};

/// Lowers frontend-owned `HIR/LIR` into target-specific driver input.
pub fn lower_to_driver_input(
    hir_module: &HirModule,
    mut lir_module: LirModule,
    backend_family: TargetBackendFamily,
    runner_family: RunnerFamily,
    output_dir: std::path::PathBuf,
) -> Result<DriverBackendInput> {
    Ok(match backend_family {
        TargetBackendFamily::Clr => {
            DriverBackendInput::Clr(ClrBinaryBackendInput { module: lower_lir_to_msil(&lir_module), output_dir, image_kind: None })
        }
        TargetBackendFamily::Jvm => DriverBackendInput::Jvm(JvmBinaryBackendInput {
            class_file: lower_lir_to_jvm_class(&lir_module)?,
            output_dir,
            emit_class_file: true,
        }),
        TargetBackendFamily::Wasm => {
            let imports = collect_wasm_imports(hir_module);
            let import_symbols: HashSet<&str> = imports.iter().map(|item| item.symbol.as_str()).collect();
            lir_module.functions.retain(|function| !import_symbols.contains(function.symbol.as_str()));
            DriverBackendInput::Wasm(WasmBinaryBackendInput {
                module: lower_lir_to_wasm_module(&lir_module),
                output_dir,
                host: host_skeleton_for_runner(runner_family),
                imports: imports.into_iter().map(|item| (item.module, item.field)).collect(),
            })
        }
        TargetBackendFamily::Native => {
            DriverBackendInput::Native(NativeBinaryBackendInput { object: lower_lir_to_native_assembly(&lir_module)?, output_dir })
        }
        other => return Err(miette!("前端桥接尚未接入 {:?} 后端家族", other)),
    })
}

/// Writes a `CLR` text sidecar when the selected driver input carries `MSIL`.
pub fn write_clr_msil_sidecar(output_dir: &Path, artifact_name: &str, input: &DriverBackendInput) -> Result<bool> {
    let DriverBackendInput::Clr(input) = input
    else {
        return Ok(false);
    };
    std::fs::create_dir_all(output_dir).into_diagnostic().wrap_err_with(|| format!("创建输出目录失败：{}", output_dir.display()))?;
    let sidecar_path = output_dir.join(format!("{}.msil", artifact_name));
    let text = MsilTextWriter::default().write_module(&input.module);
    std::fs::write(&sidecar_path, text).into_diagnostic().wrap_err_with(|| format!("写入 `MSIL` sidecar 失败：{}", sidecar_path.display()))?;
    Ok(true)
}

fn host_skeleton_for_runner(runner: RunnerFamily) -> WasmHostSkeleton {
    match runner {
        RunnerFamily::Node => WasmHostSkeleton::Node,
        RunnerFamily::Wasi => WasmHostSkeleton::Wasi,
        _ => WasmHostSkeleton::Node,
    }
}

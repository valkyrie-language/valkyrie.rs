//! Frontend-owned bridge from `HIR/LIR` into target-specific `nyar-driver` inputs.

mod clr_array_lowering;
mod clr_call_lowering;
mod clr_effect_runtime;
mod clr_lowering;
mod clr_runtime_lowering;
mod clr_terminator_lowering;
mod jvm_host_bridge;
mod jvm_intrinsics;
mod jvm_lowering;
mod jvm_operation_lowering;
mod native_lowering;
mod wasi_interop;
mod wasi_lowering;

use std::{collections::HashSet, path::Path};

use clr_backend::{ClrBinaryBackendInput, MsilTextWriter};
use jvm_backend::JvmBinaryBackendInput;
use miette::{miette, IntoDiagnostic, Result, WrapErr};
use native_backend::NativeBinaryBackendInput;
use nyar::{HostProjectionBoundary, QualifiedName};
use nyar_driver::DriverBackendInput;
use wasi_backend::WasmBinaryBackendInput;

use crate::{hir::HirModule, lir::LirModule, symbols::stable_hir_function_symbol, TargetBackendFamily};

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
    host_boundary: HostProjectionBoundary,
    output_dir: std::path::PathBuf,
) -> Result<DriverBackendInput> {
    lower_to_driver_input_for_partition(hir_module, lir_module, backend_family, host_boundary, output_dir, &[])
}

/// Lowers frontend-owned `HIR/LIR` into target-specific driver input for a partition.
pub fn lower_to_driver_input_for_partition(
    hir_module: &HirModule,
    mut lir_module: LirModule,
    backend_family: TargetBackendFamily,
    host_boundary: HostProjectionBoundary,
    output_dir: std::path::PathBuf,
    exported_operations: &[QualifiedName],
) -> Result<DriverBackendInput> {
    let hir_module =
        if exported_operations.is_empty() { hir_module.clone() } else { filter_hir_module_for_partition(hir_module, exported_operations) };
    if !exported_operations.is_empty() {
        filter_lir_module_for_partition(&mut lir_module, exported_operations);
    }

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
            let imports = collect_wasm_imports(&hir_module);
            let import_symbols: HashSet<&str> = imports.iter().map(|item| item.symbol.as_str()).collect();
            lir_module.functions.retain(|function| !import_symbols.contains(function.symbol.as_str()));
            DriverBackendInput::Wasm(WasmBinaryBackendInput {
                module: lower_lir_to_wasm_module(&lir_module)?,
                output_dir,
                host_boundary,
                imports: imports.into_iter().map(|item| (item.module, item.field)).collect(),
            })
        }
        TargetBackendFamily::Native => {
            DriverBackendInput::Native(NativeBinaryBackendInput { object: lower_lir_to_native_assembly(&lir_module)?, output_dir })
        }
        other => return Err(miette!("前端桥接尚未接入 {:?} 后端家族", other)),
    })
}

fn filter_hir_module_for_partition(hir_module: &HirModule, exported_operations: &[QualifiedName]) -> HirModule {
    let allowed_symbols = exported_operations.iter().map(partition_operation_symbol).collect::<HashSet<_>>();
    let mut filtered = hir_module.clone();
    filtered.functions.retain(|function| allowed_symbols.contains(&stable_hir_function_symbol(&hir_module.name, function)));
    filtered
}

fn filter_lir_module_for_partition(lir_module: &mut LirModule, exported_operations: &[QualifiedName]) {
    let allowed_symbols = exported_operations.iter().map(partition_operation_symbol).collect::<HashSet<_>>();
    lir_module.functions.retain(|function| allowed_symbols.contains(function.symbol.as_str()));
}

fn partition_operation_symbol(operation: &QualifiedName) -> String {
    operation.to_string()
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

//! `WASM/WASI` 路线 lowering。
//!
//! 这里仅把 lane-aware `LIR` 收口成最小可编码的 `WASM` 模块骨架，
//! 不把 `LIR` 扩成统一跨端执行图。

use miette::{miette, Result};
use nyar::{
    abstractions::{BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use wasi_backend::WasmBinaryModule;

use crate::lir::{LirModule, LirTargetLane, LirTerminator};

/// `WASM` lane 的 `LIR -> WasmBinaryModule` 承接器。
pub struct WasmLirLoweringLane {
    descriptor: TargetLoweringLaneDescriptor,
}

impl WasmLirLoweringLane {
    /// 创建一个新的 `WASM` lane lowering。
    pub fn new() -> Self {
        Self {
            descriptor: TargetLoweringLaneDescriptor {
                name: "wasm-binary-lowering".to_string(),
                lane: TargetLane::Wasm,
                input_kind: BackendInputKind::WasmModule,
                target: BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native),
            },
        }
    }
}

impl Default for WasmLirLoweringLane {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetLoweringLane for WasmLirLoweringLane {
    type PartitionInput = LirModule;
    type BackendInput = WasmBinaryModule;

    fn descriptor(&self) -> &TargetLoweringLaneDescriptor {
        &self.descriptor
    }

    fn lower_partition(&self, partition: Self::PartitionInput) -> Result<LaneLoweringResult<Self::BackendInput>> {
        if partition.lane != LirTargetLane::Wasm {
            return Err(miette!(
                code = "nyar::wasm::lowering::lane_mismatch",
                help = "请确认当前 `LIR` 分区已经选择 `WASM` 目标路线",
                "当前 lane 是 {:?}，不能进入 WASM lowering",
                partition.lane
            ));
        }

        let artifact_name = partition.name.clone();
        let input = lower_lir_to_wasm_module(&partition)?;
        Ok(LaneLoweringResult { input, artifact_name })
    }
}

/// 将 `LIR` 模块降低为最小 `WASM` 模块骨架。
pub fn lower_lir_to_wasm_module(lir: &LirModule) -> Result<WasmBinaryModule> {
    ensure_no_effect_runtime_terminators(lir, "WASM")?;
    let mut module = WasmBinaryModule::new();
    module.push_custom_section("nyar.module", lir.name.as_bytes().to_vec());
    module.push_custom_section("nyar.lane", b"wasm".to_vec());
    module.push_custom_section(
        "nyar.functions",
        lir.functions.iter().flat_map(|function| function.symbol.bytes().chain(std::iter::once(b'\n'))).collect(),
    );
    Ok(module)
}

fn ensure_no_effect_runtime_terminators(lir: &LirModule, backend_name: &str) -> Result<()> {
    for function in &lir.functions {
        for block in &function.blocks {
            if let LirTerminator::PerformEffect { effect, .. } = &block.terminator {
                return Err(miette!(
                    "{backend_name} backend 暂未支持 `{:?}` effect/runtime lowering（函数 `{}`，块 `{}`）",
                    effect,
                    function.symbol,
                    block.label
                ));
            }
        }
    }
    Ok(())
}

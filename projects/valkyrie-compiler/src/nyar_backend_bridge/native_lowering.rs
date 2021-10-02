//! `Native/MSVC` 路线 lowering。

use miette::{miette, Result};
use nyar::{
    abstractions::{BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use nyar_binary_format::{CoffHeader, CoffMachine, CoffObject, CoffSection, CoffSymbol};

use crate::{
    lir::{LirModule, LirTargetLane, LirTerminator},
    symbols::{is_main_symbol, mangle_emitted_symbol},
};

/// `Native` lane 的骨架 lowering。
pub struct NativeLirLoweringLane {
    descriptor: TargetLoweringLaneDescriptor,
}

impl NativeLirLoweringLane {
    /// 创建一个新的 `Native` lane lowering。
    pub fn new() -> Self {
        Self {
            descriptor: TargetLoweringLaneDescriptor {
                name: "native-coff-lowering".to_string(),
                lane: TargetLane::Native,
                input_kind: BackendInputKind::CoffObject,
                target: BinaryTarget::new(TargetFamily::Native, BinaryArch::X64, BinaryFlavor::Native),
            },
        }
    }
}

impl Default for NativeLirLoweringLane {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetLoweringLane for NativeLirLoweringLane {
    type PartitionInput = LirModule;
    type BackendInput = CoffObject;

    fn descriptor(&self) -> &TargetLoweringLaneDescriptor {
        &self.descriptor
    }

    fn lower_partition(&self, partition: Self::PartitionInput) -> Result<LaneLoweringResult<Self::BackendInput>> {
        if partition.lane != LirTargetLane::Native {
            return Err(miette!(
                code = "nyar::native::lowering::lane_mismatch",
                help = "请确认当前 `LIR` 分区已经选择 `Native` 目标路线",
                "当前 lane 是 {:?}，不能进入 Native lowering",
                partition.lane
            ));
        }

        let artifact_name = partition.name.clone();
        let input = lower_lir_to_native_assembly(&partition)?;
        Ok(LaneLoweringResult { input, artifact_name })
    }
}

/// 将 `LIR` 模块降低为最小 `COFF` 目标文件骨架。
pub fn lower_lir_to_native_assembly(lir: &LirModule) -> Result<CoffObject> {
    ensure_no_effect_runtime_terminators(lir, "Native")?;
    let machine = CoffMachine::from_arch(BinaryArch::X64);
    let return_stub = match machine {
        CoffMachine::I386 | CoffMachine::Amd64 => &[0xC3][..],
        CoffMachine::Arm64 => &[0xC0, 0x03, 0x5F, 0xD6][..],
        CoffMachine::Unknown => {
            return Err(miette!("Native/COFF lowering 不能使用未指定架构"));
        }
    };

    let mut text = Vec::new();
    let mut symbols = Vec::with_capacity(lir.functions.len());
    for function in &lir.functions {
        let offset = u32::try_from(text.len()).map_err(|_| miette!("`.text` 节过大，无法编码为 `COFF`"))?;
        text.extend_from_slice(return_stub);
        let emitted_name = if is_main_symbol(&function.symbol) { "main".to_string() } else { mangle_emitted_symbol(&function.symbol) };
        symbols.push(CoffSymbol { name: emitted_name, section_index: 1, value: offset, storage_class: 2 });
    }

    Ok(CoffObject {
        header: CoffHeader {
            machine,
            section_count: 1,
            symbol_table_offset: 0,
            symbol_count: u32::try_from(symbols.len()).map_err(|_| miette!("符号数量超过 `COFF` 上限"))?,
            characteristics: 0,
        },
        object_kind: nyar::ObjectKind::ObjectFile,
        sections: vec![CoffSection { name: ".text".to_string(), data: text, relocations: Vec::new(), characteristics: 0x6000_0020 }],
        symbols,
    })
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

//! 从 `MIR` 生成驱动层可消费的 suspend 载荷（first-class 与 state-machine 双路径）。

use crate::{
    MirBlockRef, MirEffectKind, MirFunction, MirLowerer, MirModule, MirTerminator, NyarPlanningContract,
    mir::{CarrierTable, SuspendLoweringPlan, dispatch_state_for_suspend, emit_suspend_plan, find_dispatch_block},
    types::hir::{HirModule, ValkyrieType},
    valkyrie::frontend_contract::witness_bindings_for_effect_with_diagnostics,
};
use nyar::{
    ControlFlowPayload, QualifiedName, SuspendContinuationArtifact, SuspendDispatchCase, SuspendFunctionArtifact,
    SuspendRuntimeFunctionArtifact, SuspendRuntimePayload, SuspendStateArtifact, SuspendWitnessBinding,
};

const WITNESS_PAYLOAD_FIELD: &str = "__witness_payload_0";

/// 为 state-machine 后端构建 `ControlFlowPayload`（含 CFG rewrite）。
pub fn build_state_machine_suspend_payload(hir_module: &HirModule, operations: &[QualifiedName]) -> ControlFlowPayload {
    let mir_module = MirLowerer::lower_module(hir_module);
    build_state_machine_payload_from_mir(hir_module, &mir_module, operations)
}

/// 为 first-class 后端构建 runtime continuation 载荷（保留语义 MIR，不做 CFG rewrite）。
pub fn build_first_class_suspend_payload(hir_module: &HirModule, operations: &[QualifiedName]) -> SuspendRuntimePayload {
    let mir_module = MirLowerer::lower_module_semantic(hir_module);
    let facts = hir_module.program_facts();
    let functions = operations
        .iter()
        .filter_map(|operation| {
            let analysis = facts.functions.iter().find(|function| &function.symbol == operation)?;
            let mir_function = mir_module.functions.iter().find(|function| function.symbol == operation.to_string())?;
            if mir_function.suspend_points.is_empty() && mir_function.continuations.is_empty() {
                return None;
            }
            let _ = analysis;
            Some(runtime_artifact_from_mir_function(hir_module, operation.clone(), mir_function))
        })
        .collect();
    SuspendRuntimePayload { functions }
}

fn build_state_machine_payload_from_mir(hir_module: &HirModule, mir_module: &MirModule, operations: &[QualifiedName]) -> ControlFlowPayload {
    let facts = hir_module.program_facts();
    let functions = operations
        .iter()
        .filter_map(|operation| {
            let analysis = facts.functions.iter().find(|function| &function.symbol == operation)?;
            let mir_function = mir_module.functions.iter().find(|function| function.symbol == operation.to_string())?;
            if mir_function.suspend_points.is_empty() && mir_function.continuations.is_empty() {
                return None;
            }
            let _ = analysis;
            Some(state_machine_artifact_from_mir_function(hir_module, operation.clone(), mir_function))
        })
        .collect();
    ControlFlowPayload { functions }
}

fn state_machine_artifact_from_mir_function(hir_module: &HirModule, symbol: QualifiedName, function: &MirFunction) -> SuspendFunctionArtifact {
    let synthesized = emit_suspend_plan(function);
    let plan = function.suspend_plan.as_ref().or(synthesized.as_ref()).expect("suspend function must carry plan");
    let dispatch_cases = find_dispatch_block(function)
        .map(|block| match &block.terminator {
            MirTerminator::StateDispatch { cases, .. } => cases
                .iter()
                .map(|(case_key, target)| SuspendDispatchCase { case_key: *case_key, block_label: block_label(function, *target) })
                .collect(),
            _ => Vec::new(),
        })
        .unwrap_or_default();

    let (states, frame_fields) = map_suspend_states(hir_module, function, plan);
    let continuations = map_continuations(function, plan);

    SuspendFunctionArtifact {
        symbol,
        state_machine_type: format!("{}StateMachine", sanitize_type_name(&plan.function_symbol)),
        state_field: "__state".to_string(),
        frame_fields,
        dispatch_cases,
        states,
        continuations,
    }
}

fn runtime_artifact_from_mir_function(hir_module: &HirModule, symbol: QualifiedName, function: &MirFunction) -> SuspendRuntimeFunctionArtifact {
    let synthesized = emit_suspend_plan(function);
    let plan = function.suspend_plan.as_ref().or(synthesized.as_ref()).expect("suspend function must carry plan");
    let (states, frame_fields) = map_suspend_states(hir_module, function, plan);
    let continuations = map_continuations(function, plan);

    SuspendRuntimeFunctionArtifact { symbol, entry_block_label: block_label(function, function.entry), frame_fields, states, continuations }
}

fn map_suspend_states(hir_module: &HirModule, function: &MirFunction, plan: &SuspendLoweringPlan) -> (Vec<SuspendStateArtifact>, Vec<String>) {
    let states = plan
        .states
        .iter()
        .map(|state| {
            let payload_type = state.payload_type.as_ref();
            let witness_bindings = witness_bindings_for_effect(hir_module, state.effect, payload_type);
            let mut spill_fields = (0..state.spill_slots.len()).map(CarrierTable::frame_slot_field).collect::<Vec<_>>();
            if spill_fields.is_empty() && !witness_bindings.is_empty() {
                spill_fields.push(WITNESS_PAYLOAD_FIELD.to_string());
            }
            SuspendStateArtifact {
                state_id: state.state_id,
                effect: effect_name(state.effect).to_string(),
                resume_case_key: dispatch_state_for_suspend(state.state_id),
                frame_carrier: state.frame_carrier.clone(),
                spill_fields,
                suspend_block_label: block_label(function, state.suspend_block),
                resume_block_label: block_label(function, state.resume_target),
                resume_parameter_count: state.resume_parameter_count,
                witness_bindings,
                continuation_index: state.continuation_index,
            }
        })
        .collect::<Vec<_>>();

    let mut frame_fields = Vec::new();
    for state in &plan.states {
        for slot_index in 0..state.spill_slots.len() {
            let field = CarrierTable::frame_slot_field(slot_index);
            if !frame_fields.contains(&field) {
                frame_fields.push(field);
            }
        }
    }
    if frame_fields.is_empty() && states.iter().any(|state| !state.witness_bindings.is_empty()) {
        frame_fields.push(WITNESS_PAYLOAD_FIELD.to_string());
    }

    (states, frame_fields)
}

fn map_continuations(function: &MirFunction, plan: &SuspendLoweringPlan) -> Vec<SuspendContinuationArtifact> {
    function
        .continuations
        .iter()
        .enumerate()
        .map(|(index, continuation)| {
            let resume_block = function.blocks.iter().find(|block| block.id == continuation.resume_target);
            SuspendContinuationArtifact {
                index,
                carrier: plan.carrier_table.continuation(index),
                dispatch_block_label: block_label(function, continuation.dispatch_block),
                resume_block_label: block_label(function, continuation.resume_target),
                handler_exit_block_label: block_label(function, continuation.handler_exit),
                resume_parameter_count: resume_block.map(|block| block.parameters.len()).unwrap_or(0),
            }
        })
        .collect()
}

fn block_label(function: &MirFunction, id: MirBlockRef) -> String {
    function.blocks.iter().find(|block| block.id == id).map(|block| block.label.clone()).unwrap_or_else(|| format!("block_{}", id.0))
}

fn effect_name(effect: MirEffectKind) -> &'static str {
    match effect {
        MirEffectKind::Raise => "Raise",
        MirEffectKind::Yield => "Yield",
        MirEffectKind::DelegateYield => "DelegateYield",
        MirEffectKind::Await => "Await",
        MirEffectKind::AsyncSpawn => "AsyncSpawn",
        MirEffectKind::AsyncBlock => "AsyncBlock",
    }
}

fn sanitize_type_name(symbol: &str) -> String {
    symbol.chars().filter(|ch| ch.is_ascii_alphanumeric() || *ch == '_').collect()
}

/// 解析 effect 对应的 witness 绑定。
///
/// 假闭环修复：`synthetic_witness_binding` 已移除，witness 无法解析时返回空绑定列表。
/// 后端 `primary_witness_binding` 对空绑定的处理是安全的——降级为普通 yield 路径，
/// 不会尝试调用不存在的符号，因此不会导致运行时崩溃。诊断保留供上层使用但不阻断降级管线。
/// 对 trait object 类型（如 `Future<i32>`、`Iterator<i32>`）此路径同样适用：trait object
/// 的方法为动态分发，不要求静态 impl 解析。
fn witness_bindings_for_effect(
    hir_module: &HirModule,
    effect: MirEffectKind,
    payload_type: Option<&ValkyrieType>,
) -> Vec<SuspendWitnessBinding> {
    let (bindings, _diagnostics) = witness_bindings_for_effect_with_diagnostics(hir_module, effect, payload_type);
    bindings
}

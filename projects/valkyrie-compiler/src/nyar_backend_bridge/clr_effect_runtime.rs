use std::collections::{BTreeMap, BTreeSet};

use crate::{
    lir::{LirContinuation, LirEffectKind, LirFunction, LirOperand, LirRuntimeContinuation, LirRuntimeFrame, LirSuspendPoint},
    mir::{MirBlockRef, MirConstant, MirValueRef},
};
use clr_backend::{MsilInstruction, MsilInstructionOperand, MsilMethodRef, MsilMethodSignature, MsilOpcode, MsilType};
use valkyrie_types::hir::ValkyrieType as HirType;

#[derive(Clone, Copy)]
pub struct RuntimeFrameBinding<'a> {
    pub suspend_point: &'a LirSuspendPoint,
    pub runtime_frame: &'a LirRuntimeFrame,
}

#[derive(Clone, Copy)]
pub struct RuntimeContinuationBinding<'a> {
    pub continuation: &'a LirContinuation,
    pub runtime_continuation: &'a LirRuntimeContinuation,
}

#[derive(Clone, Copy)]
pub struct RuntimeResumeLoaderBinding<'a> {
    pub continuation: &'a LirContinuation,
    pub runtime_continuation: &'a LirRuntimeContinuation,
    pub runtime_frame: Option<&'a LirRuntimeFrame>,
}

#[derive(Clone, Copy)]
pub struct RuntimeFrameResumeLoaderBinding<'a> {
    pub runtime_frame: &'a LirRuntimeFrame,
}

#[derive(Clone, Copy)]
pub struct RuntimeSchedulerSlots {
    pub active_frame_slot: usize,
    pub active_continuation_slot: usize,
}

pub fn allocate_runtime_scheduler_slots(local_types: &mut Vec<MsilType>) -> RuntimeSchedulerSlots {
    let active_frame_slot = local_types.len();
    local_types.push(MsilType::Object);
    let active_continuation_slot = local_types.len();
    local_types.push(MsilType::Object);
    RuntimeSchedulerSlots { active_frame_slot, active_continuation_slot }
}

pub fn collect_runtime_frame_bindings(function: &LirFunction) -> BTreeMap<MirBlockRef, RuntimeFrameBinding<'_>> {
    let mut bindings = BTreeMap::new();
    for suspend_point in &function.suspend_points {
        if let Some(runtime_frame) = function.runtime_frames.iter().find(|frame| frame.state_id == suspend_point.state_id) {
            bindings.insert(suspend_point.suspend_block, RuntimeFrameBinding { suspend_point, runtime_frame });
        }
    }
    bindings
}

pub fn collect_runtime_continuation_bindings(function: &LirFunction) -> BTreeMap<MirBlockRef, RuntimeContinuationBinding<'_>> {
    let mut bindings = BTreeMap::new();
    for continuation in &function.continuations {
        if let Some(runtime_continuation) =
            function.runtime_continuations.iter().find(|runtime| runtime.dispatch_block == continuation.dispatch_block)
        {
            bindings.insert(continuation.dispatch_block, RuntimeContinuationBinding { continuation, runtime_continuation });
        }
    }
    bindings
}

pub fn collect_runtime_resume_loader_bindings(function: &LirFunction) -> BTreeMap<MirBlockRef, RuntimeResumeLoaderBinding<'_>> {
    let mut bindings = BTreeMap::new();
    for continuation in &function.continuations {
        let Some(runtime_continuation) =
            function.runtime_continuations.iter().find(|runtime| runtime.dispatch_block == continuation.dispatch_block)
        else {
            continue;
        };
        let runtime_frame =
            runtime_continuation.frame_state_id.and_then(|state_id| function.runtime_frames.iter().find(|frame| frame.state_id == state_id));
        bindings.insert(continuation.resume_target, RuntimeResumeLoaderBinding { continuation, runtime_continuation, runtime_frame });
    }
    bindings
}

pub fn collect_runtime_frame_resume_loader_bindings(function: &LirFunction) -> BTreeMap<MirBlockRef, RuntimeFrameResumeLoaderBinding<'_>> {
    let mut bindings = BTreeMap::new();
    for runtime_frame in &function.runtime_frames {
        let covered_by_continuation = function
            .runtime_continuations
            .iter()
            .any(|runtime_continuation| runtime_continuation.frame_state_id == Some(runtime_frame.state_id));
        if covered_by_continuation {
            continue;
        }
        bindings.insert(runtime_frame.resume_target, RuntimeFrameResumeLoaderBinding { runtime_frame });
    }
    bindings
}

pub fn collect_runtime_handler_exit_targets(function: &LirFunction) -> BTreeSet<MirBlockRef> {
    function.continuations.iter().map(|continuation| continuation.handler_exit).collect()
}

pub fn lower_runtime_continuation_prelude(
    binding: RuntimeContinuationBinding<'_>,
    scheduler_slots: RuntimeSchedulerSlots,
    runtime_namespace: &str,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &BTreeMap<MirValueRef, HirType>,
    label: Option<String>,
) {
    spill_eval_stack_for_runtime(eval_stack, local_slots, local_types, instructions, value_types);
    let owner = format!("{runtime_namespace}.{}", binding.runtime_continuation.carrier);
    let carrier_slot = allocate_runtime_carrier_local(&owner, instructions, max_stack, local_types, label);
    emit_runtime_int_field_store(&owner, carrier_slot, "dispatch_block", binding.continuation.dispatch_block.0 as i64, instructions, max_stack);
    emit_runtime_int_field_store(&owner, carrier_slot, "resume_target", binding.continuation.resume_target.0 as i64, instructions, max_stack);
    emit_runtime_int_field_store(
        &owner,
        carrier_slot,
        "resume_parameter_ref",
        binding.continuation.resume_parameter.0 as i64,
        instructions,
        max_stack,
    );
    emit_runtime_int_field_store(&owner, carrier_slot, "handler_exit", binding.continuation.handler_exit.0 as i64, instructions, max_stack);
    emit_runtime_default_field_store(
        &owner,
        carrier_slot,
        &binding.runtime_continuation.resume_parameter_field,
        binding.runtime_continuation.resume_parameter_type.as_ref(),
        instructions,
        max_stack,
    );
    emit_runtime_int_field_store(
        &owner,
        carrier_slot,
        "frame_state_id",
        binding.runtime_continuation.frame_state_id.map(i64::from).unwrap_or(-1),
        instructions,
        max_stack,
    );
    emit_ldloc(carrier_slot, instructions, max_stack);
    emit_stloc(scheduler_slots.active_continuation_slot, instructions, None);
    let _ = parameter_slots;
}

pub fn lower_perform_effect_frame(
    runtime_frame_binding: Option<RuntimeFrameBinding<'_>>,
    scheduler_slots: RuntimeSchedulerSlots,
    runtime_continuation_bindings: &BTreeMap<MirBlockRef, RuntimeContinuationBinding<'_>>,
    runtime_namespace: &str,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    _eval_stack: &mut Vec<MirValueRef>,
    _value_types: &BTreeMap<MirValueRef, HirType>,
) {
    if let Some(binding) = runtime_frame_binding {
        let owner = format!("{runtime_namespace}.{}", binding.runtime_frame.carrier);
        let carrier_slot = allocate_runtime_carrier_local(&owner, instructions, max_stack, local_types, None);
        emit_runtime_int_field_store(&owner, carrier_slot, "state_id", binding.suspend_point.state_id as i64, instructions, max_stack);
        emit_runtime_int_field_store(
            &owner,
            carrier_slot,
            "resume_target",
            binding.runtime_frame.resume_target.0 as i64,
            instructions,
            max_stack,
        );
        if let Some(continuation_index) = binding.runtime_frame.continuation_index {
            emit_runtime_int_field_store(&owner, carrier_slot, "continuation_slot", continuation_index as i64, instructions, max_stack);
            emit_runtime_int_field_store(&owner, carrier_slot, "continuation_index", continuation_index as i64, instructions, max_stack);
            if let Some(continuation_binding) = runtime_continuation_bindings
                .values()
                .find(|continuation_binding| continuation_binding.runtime_continuation.frame_state_id == Some(binding.runtime_frame.state_id))
            {
                let continuation_owner = format!("{runtime_namespace}.{}", continuation_binding.runtime_continuation.carrier);
                emit_ldloc(scheduler_slots.active_continuation_slot, instructions, max_stack);
                lower_constant(&MirConstant::Int(binding.runtime_frame.state_id as i64), instructions, max_stack, None);
                instructions.push(MsilInstruction {
                    label: None,
                    opcode: MsilOpcode::Stfld,
                    operand: Some(MsilInstructionOperand::Field(continuation_owner, "frame_state_id".to_string())),
                });
                *max_stack = (*max_stack).max(2);
            }
        }
        for slot in &binding.runtime_frame.slots {
            emit_runtime_value_field_store(
                &owner,
                carrier_slot,
                &slot.field_name,
                slot.value,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
            );
        }
        emit_ldloc(carrier_slot, instructions, max_stack);
        emit_stloc(scheduler_slots.active_frame_slot, instructions, None);
    }
}

pub fn lower_runtime_resume_loader_prelude(
    binding: RuntimeResumeLoaderBinding<'_>,
    scheduler_slots: RuntimeSchedulerSlots,
    runtime_namespace: &str,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    local_slots: &BTreeMap<MirValueRef, usize>,
    label: Option<String>,
) {
    let skip_label = format!("CLR_RUNTIME_RESUME_SKIP_{}", binding.continuation.resume_target.0);
    emit_ldloc(scheduler_slots.active_continuation_slot, instructions, max_stack);
    instructions.push(MsilInstruction {
        label,
        opcode: MsilOpcode::Brfalse,
        operand: Some(MsilInstructionOperand::BranchTarget(skip_label.clone())),
    });

    if let Some(runtime_frame) = binding.runtime_frame {
        let frame_owner = format!("{runtime_namespace}.{}", runtime_frame.carrier);
        emit_ldloc(scheduler_slots.active_frame_slot, instructions, max_stack);
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Brfalse,
            operand: Some(MsilInstructionOperand::BranchTarget(skip_label.clone())),
        });
        for slot in &runtime_frame.slots {
            let Some(local_slot) = local_slots.get(&slot.value).copied()
            else {
                continue;
            };
            emit_ldloc(scheduler_slots.active_frame_slot, instructions, max_stack);
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Ldfld,
                operand: Some(MsilInstructionOperand::Field(frame_owner.clone(), slot.field_name.clone())),
            });
            *max_stack = (*max_stack).max(1);
            emit_stloc(local_slot, instructions, None);
        }
    }

    if let Some(parameter_slot) = local_slots.get(&binding.continuation.resume_parameter).copied() {
        let continuation_owner = format!("{runtime_namespace}.{}", binding.runtime_continuation.carrier);
        emit_ldloc(scheduler_slots.active_continuation_slot, instructions, max_stack);
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Ldfld,
            operand: Some(MsilInstructionOperand::Field(continuation_owner, binding.runtime_continuation.resume_parameter_field.clone())),
        });
        *max_stack = (*max_stack).max(1);
        emit_stloc(parameter_slot, instructions, None);
    }

    lower_constant(&MirConstant::Unit, instructions, max_stack, None);
    emit_stloc(scheduler_slots.active_frame_slot, instructions, None);
    lower_constant(&MirConstant::Unit, instructions, max_stack, None);
    emit_stloc(scheduler_slots.active_continuation_slot, instructions, Some(skip_label));
}

pub fn lower_runtime_frame_resume_loader_prelude(
    binding: RuntimeFrameResumeLoaderBinding<'_>,
    scheduler_slots: RuntimeSchedulerSlots,
    runtime_namespace: &str,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    local_slots: &BTreeMap<MirValueRef, usize>,
    label: Option<String>,
) {
    let skip_label = format!("CLR_RUNTIME_FRAME_RESUME_SKIP_{}", binding.runtime_frame.resume_target.0);
    let frame_owner = format!("{runtime_namespace}.{}", binding.runtime_frame.carrier);
    emit_ldloc(scheduler_slots.active_frame_slot, instructions, max_stack);
    instructions.push(MsilInstruction {
        label,
        opcode: MsilOpcode::Brfalse,
        operand: Some(MsilInstructionOperand::BranchTarget(skip_label.clone())),
    });

    for slot in &binding.runtime_frame.slots {
        let Some(local_slot) = local_slots.get(&slot.value).copied()
        else {
            continue;
        };
        emit_ldloc(scheduler_slots.active_frame_slot, instructions, max_stack);
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Ldfld,
            operand: Some(MsilInstructionOperand::Field(frame_owner.clone(), slot.field_name.clone())),
        });
        *max_stack = (*max_stack).max(1);
        emit_stloc(local_slot, instructions, None);
    }

    lower_constant(&MirConstant::Unit, instructions, max_stack, None);
    emit_stloc(scheduler_slots.active_frame_slot, instructions, Some(skip_label));
}

pub fn lower_runtime_resume_jump(
    binding: RuntimeResumeLoaderBinding<'_>,
    scheduler_slots: RuntimeSchedulerSlots,
    runtime_namespace: &str,
    argument: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
    eval_stack: &mut Vec<MirValueRef>,
) {
    let continuation_owner = format!("{runtime_namespace}.{}", binding.runtime_continuation.carrier);
    emit_ldloc(scheduler_slots.active_continuation_slot, instructions, max_stack);
    lower_operand_for_runtime(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Stfld,
        operand: Some(MsilInstructionOperand::Field(continuation_owner, binding.runtime_continuation.resume_parameter_field.clone())),
    });
    *max_stack = (*max_stack).max(2);
    eval_stack.clear();
}

pub fn lower_runtime_handler_exit_cleanup(
    scheduler_slots: RuntimeSchedulerSlots,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    label: Option<String>,
) {
    lower_constant(&MirConstant::Unit, instructions, max_stack, label);
    emit_stloc(scheduler_slots.active_frame_slot, instructions, None);
    lower_constant(&MirConstant::Unit, instructions, max_stack, None);
    emit_stloc(scheduler_slots.active_continuation_slot, instructions, None);
}

pub fn lower_effect_resume_argument(
    effect: LirEffectKind,
    payload: Option<&LirOperand>,
    target_parameter: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &BTreeMap<MirValueRef, HirType>,
) {
    match effect {
        LirEffectKind::Yield | LirEffectKind::DelegateYield => {
            lower_constant(&MirConstant::Unit, instructions, max_stack, None);
        }
        LirEffectKind::Raise => {
            if let Some(payload) = payload {
                lower_operand_for_runtime(payload, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            }
            else {
                let ty = target_parameter.and_then(|parameter| value_types.get(&parameter));
                lower_default_hir_value(ty, instructions, max_stack);
            }
        }
        LirEffectKind::Await | LirEffectKind::AsyncBlock | LirEffectKind::AsyncSpawn => {
            let ty = target_parameter.and_then(|parameter| value_types.get(&parameter));
            lower_default_hir_value(ty, instructions, max_stack);
        }
    }
}

pub fn emit_stloc(slot: usize, instructions: &mut Vec<MsilInstruction>, label: Option<String>) {
    let (opcode, operand) = match slot {
        0 => (MsilOpcode::Stloc0, None),
        1 => (MsilOpcode::Stloc1, None),
        2 => (MsilOpcode::Stloc2, None),
        3 => (MsilOpcode::Stloc3, None),
        _ => (MsilOpcode::Stloc, Some(MsilInstructionOperand::Integer(slot as i64))),
    };
    instructions.push(MsilInstruction { label, opcode, operand });
}

pub fn emit_ldloc(slot: usize, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16) {
    let (opcode, operand) = match slot {
        0 => (MsilOpcode::Ldloc0, None),
        1 => (MsilOpcode::Ldloc1, None),
        2 => (MsilOpcode::Ldloc2, None),
        3 => (MsilOpcode::Ldloc3, None),
        _ => (MsilOpcode::Ldloc, Some(MsilInstructionOperand::Integer(slot as i64))),
    };
    instructions.push(MsilInstruction { label: None, opcode, operand });
    *max_stack = (*max_stack).max(1);
}

fn allocate_runtime_carrier_local(
    owner: &str,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    local_types: &mut Vec<MsilType>,
    label: Option<String>,
) -> usize {
    let slot = local_types.len();
    local_types.push(MsilType::Named(owner.to_string()));
    instructions.push(MsilInstruction {
        label,
        opcode: MsilOpcode::Newobj,
        operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
            owner: Some(owner.to_string()),
            name: ".ctor".to_string(),
            signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
        })),
    });
    *max_stack = (*max_stack).max(1);
    emit_stloc(slot, instructions, None);
    slot
}

fn emit_runtime_int_field_store(
    owner: &str,
    carrier_slot: usize,
    field_name: &str,
    value: i64,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
) {
    emit_ldloc(carrier_slot, instructions, max_stack);
    lower_constant(&MirConstant::Int(value), instructions, max_stack, None);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Stfld,
        operand: Some(MsilInstructionOperand::Field(owner.to_string(), field_name.to_string())),
    });
    *max_stack = (*max_stack).max(2);
}

fn emit_runtime_default_field_store(
    owner: &str,
    carrier_slot: usize,
    field_name: &str,
    ty: Option<&HirType>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
) {
    emit_ldloc(carrier_slot, instructions, max_stack);
    lower_default_hir_value(ty, instructions, max_stack);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Stfld,
        operand: Some(MsilInstructionOperand::Field(owner.to_string(), field_name.to_string())),
    });
    *max_stack = (*max_stack).max(2);
}

fn emit_runtime_value_field_store(
    owner: &str,
    carrier_slot: usize,
    field_name: &str,
    value_ref: MirValueRef,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
) {
    emit_ldloc(carrier_slot, instructions, max_stack);
    lower_value_ref_for_runtime(&value_ref, instructions, max_stack, parameter_slots, local_slots);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Stfld,
        operand: Some(MsilInstructionOperand::Field(owner.to_string(), field_name.to_string())),
    });
    *max_stack = (*max_stack).max(2);
}

fn lower_default_hir_value(ty: Option<&HirType>, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16) {
    match ty {
        Some(HirType::Boolean) => lower_constant(&MirConstant::Bool(false), instructions, max_stack, None),
        Some(HirType::Integer8 { .. })
        | Some(HirType::Integer16 { .. })
        | Some(HirType::Integer32 { .. })
        | Some(HirType::Integer64 { .. })
        | Some(HirType::Character) => lower_constant(&MirConstant::Int(0), instructions, max_stack, None),
        Some(HirType::Float32) | Some(HirType::Float64) => {
            lower_constant(&MirConstant::Float64(ordered_float::OrderedFloat(0.0)), instructions, max_stack, None)
        }
        Some(HirType::Utf8 | HirType::Utf16 | HirType::Named(_) | HirType::Array(_)) | Some(HirType::Unit | HirType::Void) | None => {
            lower_constant(&MirConstant::Unit, instructions, max_stack, None)
        }
        _ => lower_constant(&MirConstant::Unit, instructions, max_stack, None),
    }
}

fn lower_constant(constant: &MirConstant, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16, label: Option<String>) {
    match constant {
        MirConstant::Int(value) => {
            let opcode = match *value {
                0 => MsilOpcode::LdcI4_0,
                1 => MsilOpcode::LdcI4_1,
                2 => MsilOpcode::LdcI4_2,
                3 => MsilOpcode::LdcI4_3,
                4 => MsilOpcode::LdcI4_4,
                5 => MsilOpcode::LdcI4_5,
                6 => MsilOpcode::LdcI4_6,
                7 => MsilOpcode::LdcI4_7,
                8 => MsilOpcode::LdcI4_8,
                -1 => MsilOpcode::LdcI4M1,
                _ if *value >= i8::MIN as i64 && *value <= i8::MAX as i64 => MsilOpcode::LdcI4S,
                _ if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => MsilOpcode::LdcI4,
                _ => MsilOpcode::LdcI8,
            };
            let operand = match opcode {
                MsilOpcode::LdcI4 | MsilOpcode::LdcI4S | MsilOpcode::LdcI8 => Some(MsilInstructionOperand::Integer(*value)),
                _ => None,
            };
            instructions.push(MsilInstruction { label, opcode, operand });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::Bool(value) => {
            instructions.push(MsilInstruction { label, opcode: if *value { MsilOpcode::LdcI4_1 } else { MsilOpcode::LdcI4_0 }, operand: None });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::Float64(value) => {
            instructions.push(MsilInstruction {
                label,
                opcode: MsilOpcode::LdcR8,
                operand: Some(MsilInstructionOperand::Float(value.0.to_string())),
            });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::String(value) => {
            instructions.push(MsilInstruction {
                label,
                opcode: MsilOpcode::Ldstr,
                operand: Some(MsilInstructionOperand::StringLiteral(value.clone())),
            });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::Unit => {
            instructions.push(MsilInstruction { label, opcode: MsilOpcode::Ldnull, operand: None });
            *max_stack = (*max_stack).max(1);
        }
    }
}

fn lower_value_ref_for_runtime(
    value_ref: &MirValueRef,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
) {
    if let Some(parameter_index) = parameter_slots.get(value_ref).copied() {
        let (opcode, operand) = match parameter_index {
            0 => (MsilOpcode::Ldarg0, None),
            1 => (MsilOpcode::Ldarg1, None),
            2 => (MsilOpcode::Ldarg2, None),
            3 => (MsilOpcode::Ldarg3, None),
            _ => (MsilOpcode::Ldarg, Some(MsilInstructionOperand::Integer(parameter_index as i64))),
        };
        instructions.push(MsilInstruction { label: None, opcode, operand });
        *max_stack = (*max_stack).max(1);
        return;
    }

    if let Some(slot) = local_slots.get(value_ref).copied() {
        emit_ldloc(slot, instructions, max_stack);
        return;
    }

    instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Nop, operand: None });
}

fn lower_operand_for_runtime(
    operand: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
    eval_stack: &mut Vec<MirValueRef>,
) {
    match operand {
        LirOperand::Value(value_ref) => {
            if eval_stack.last() == Some(value_ref) {
                return;
            }
            lower_value_ref_for_runtime(value_ref, instructions, max_stack, parameter_slots, local_slots);
            eval_stack.push(*value_ref);
        }
        LirOperand::Constant(constant) => {
            lower_constant(constant, instructions, max_stack, None);
        }
        LirOperand::Symbol(path) => {
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Ldsfld,
                operand: Some(MsilInstructionOperand::Symbol(path.to_string())),
            });
            *max_stack = (*max_stack).max(1);
        }
    }
}

fn spill_eval_stack_for_runtime(
    eval_stack: &mut Vec<MirValueRef>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    instructions: &mut Vec<MsilInstruction>,
    value_types: &BTreeMap<MirValueRef, HirType>,
) {
    while let Some(value) = eval_stack.pop() {
        let slot = local_types.len();
        let spilled_type = value_types.get(&value).map(lower_hir_type_for_runtime).unwrap_or(MsilType::Object);
        local_types.push(spilled_type);
        emit_stloc(slot, instructions, None);
        local_slots.insert(value, slot);
    }
}

fn lower_hir_type_for_runtime(ty: &HirType) -> MsilType {
    match ty {
        HirType::Void | HirType::Unit => MsilType::Void,
        HirType::Boolean => MsilType::Bool,
        HirType::Character => MsilType::Char,
        HirType::Integer8 { signed } => {
            if *signed {
                MsilType::Int8
            }
            else {
                MsilType::UInt8
            }
        }
        HirType::Integer16 { signed } => {
            if *signed {
                MsilType::Int16
            }
            else {
                MsilType::UInt16
            }
        }
        HirType::Integer32 { signed } => {
            if *signed {
                MsilType::Int32
            }
            else {
                MsilType::UInt32
            }
        }
        HirType::Integer64 { signed } => {
            if *signed {
                MsilType::Int64
            }
            else {
                MsilType::UInt64
            }
        }
        HirType::Float32 => MsilType::Float32,
        HirType::Float64 => MsilType::Float64,
        HirType::Utf8 | HirType::Utf16 => MsilType::String,
        HirType::Named(name) => MsilType::Named(name.to_string()),
        HirType::Array(item) => MsilType::sz_array(lower_hir_type_for_runtime(item)),
        _ => MsilType::Object,
    }
}

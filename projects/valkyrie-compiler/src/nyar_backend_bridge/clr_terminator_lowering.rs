use std::collections::{BTreeMap, BTreeSet};

use super::{
    clr_effect_runtime::{
        emit_stloc, lower_effect_resume_argument, lower_perform_effect_frame, lower_runtime_resume_jump, RuntimeContinuationBinding,
        RuntimeFrameBinding, RuntimeResumeLoaderBinding, RuntimeSchedulerSlots,
    },
    clr_lowering::{lower_operand, spill_eval_stack},
};
use crate::{
    lir::{LirOperand, LirTerminator},
    mir::{MirBlockRef, MirValueRef},
};
use clr_backend::{MsilInstruction, MsilInstructionOperand, MsilOpcode, MsilType};
use valkyrie_types::hir::ValkyrieType as HirType;

pub(super) fn lower_terminator(
    terminator: &LirTerminator,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    return_type: &HirType,
    value_types: &BTreeMap<MirValueRef, HirType>,
    block_labels: &BTreeMap<MirBlockRef, String>,
    block_params: &BTreeMap<MirBlockRef, Vec<MirValueRef>>,
    runtime_scheduler_slots: RuntimeSchedulerSlots,
    runtime_namespace: &str,
    runtime_frame_binding: Option<RuntimeFrameBinding<'_>>,
    runtime_continuation_bindings: &BTreeMap<MirBlockRef, RuntimeContinuationBinding<'_>>,
    runtime_resume_loader_bindings: &BTreeMap<MirBlockRef, RuntimeResumeLoaderBinding<'_>>,
    runtime_handler_exit_targets: &BTreeSet<MirBlockRef>,
) {
    match terminator {
        LirTerminator::Return { value } => {
            let returns_void = matches!(return_type, HirType::Unit | HirType::Void);
            if !returns_void {
                if let Some(value_operand) = value {
                    let on_top = match value_operand {
                        LirOperand::Value(v) => eval_stack.last() == Some(v),
                        _ => false,
                    };
                    if !on_top {
                        spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
                        lower_operand(value_operand, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                    }
                }
            }
            else {
                eval_stack.clear();
            }
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
        }
        LirTerminator::Jump { target, arguments } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            if let Some(binding) = runtime_resume_loader_bindings.get(target).copied() {
                if let Some(argument) = arguments.first() {
                    lower_runtime_resume_jump(
                        binding,
                        runtime_scheduler_slots,
                        runtime_namespace,
                        argument,
                        instructions,
                        max_stack,
                        parameter_slots,
                        local_slots,
                        eval_stack,
                    );
                }
            }
            else {
                let target_params = block_params.get(target).cloned().unwrap_or_default();
                for (index, arg) in arguments.iter().enumerate() {
                    lower_operand(arg, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                    if let Some(param_ref) = target_params.get(index) {
                        let slot = *local_slots.get(param_ref).expect("缺少目标块参数的本地槽位");
                        emit_stloc(slot, instructions, None);
                    }
                    eval_stack.clear();
                }
            }

            let target_label = block_labels.get(target).cloned().unwrap_or_else(|| format!("BB{}", target.0));
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Br,
                operand: Some(MsilInstructionOperand::BranchTarget(target_label)),
            });
        }
        LirTerminator::Branch { condition, then_target, else_target, .. } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            lower_operand(condition, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            let then_label = block_labels.get(then_target).cloned().unwrap_or_else(|| format!("BB{}", then_target.0));
            let else_label = block_labels.get(else_target).cloned().unwrap_or_else(|| format!("BB{}", else_target.0));
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Brtrue,
                operand: Some(MsilInstructionOperand::BranchTarget(then_label)),
            });
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Br,
                operand: Some(MsilInstructionOperand::BranchTarget(else_label)),
            });
            eval_stack.clear();
        }
        LirTerminator::PerformEffect { effect, payload, resume_target } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            lower_perform_effect_frame(
                runtime_frame_binding,
                runtime_scheduler_slots,
                runtime_continuation_bindings,
                runtime_namespace,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                value_types,
            );
            let target_params = block_params.get(resume_target).cloned().unwrap_or_default();
            if let Some(parameter) = target_params.first().copied() {
                lower_effect_resume_argument(
                    *effect,
                    payload.as_ref(),
                    Some(parameter),
                    instructions,
                    max_stack,
                    parameter_slots,
                    local_slots,
                    eval_stack,
                    value_types,
                );
                let slot = *local_slots.get(&parameter).expect("缺少 effect 恢复块参数槽位");
                emit_stloc(slot, instructions, None);
                eval_stack.clear();
            }
            let target_label = block_labels.get(resume_target).cloned().unwrap_or_else(|| format!("BB{}", resume_target.0));
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Br,
                operand: Some(MsilInstructionOperand::BranchTarget(target_label)),
            });
        }
        LirTerminator::Unreachable => {
            let _ = runtime_handler_exit_targets;
            let _ = runtime_scheduler_slots;
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
        }
    }
}

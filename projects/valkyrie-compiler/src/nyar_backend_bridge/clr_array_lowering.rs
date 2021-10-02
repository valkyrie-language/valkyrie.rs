use std::collections::BTreeMap;

use crate::{
    lir::LirOperand,
    mir::{MirConstant, MirValueRef},
};
use clr_backend::{MsilInstruction, MsilInstructionOperand, MsilOpcode, MsilType};
use valkyrie_types::hir::ValkyrieType as HirType;

use super::{
    clr_effect_runtime::emit_stloc,
    clr_lowering::{infer_operand_type, lower_constant, lower_hir_type, lower_local_slot, lower_operand, spill_eval_stack},
};

pub(super) fn lower_array_subscript_access(
    object: &LirOperand,
    index: &LirOperand,
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    // 数组下标访问需要按 `array, index` 的顺序入栈。
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    match index {
        LirOperand::Value(value_ref) => {
            super::clr_lowering::lower_value_ref(value_ref, instructions, max_stack, parameter_slots, local_slots);
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

    let opcode =
        infer_subscript_output_type(object, value_types).map(|ty| array_load_opcode(&lower_hir_type(&ty))).unwrap_or(MsilOpcode::LdelemRef);
    instructions.push(MsilInstruction { label: None, opcode, operand: None });
    *max_stack = (*max_stack).max(2);

    if let LirOperand::Value(_) = object {
        eval_stack.pop();
    }

    if let Some(output) = output {
        if let Some(inferred_type) = infer_subscript_output_type(object, value_types) {
            value_types.insert(output, inferred_type);
        }
        eval_stack.push(output);
    }
}

pub(super) fn lower_array_subscript_store(
    object: &LirOperand,
    index: &LirOperand,
    value: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    lower_operand(index, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    let opcode =
        infer_subscript_output_type(object, value_types).map(|ty| array_store_opcode(&lower_hir_type(&ty))).unwrap_or(MsilOpcode::StelemRef);
    instructions.push(MsilInstruction { label: None, opcode, operand: None });
    *max_stack = (*max_stack).max(3);
    eval_stack.clear();
}

pub(super) fn lower_array_literal(
    element_type: &HirType,
    items: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_constant(&MirConstant::Int(items.len() as i64), instructions, max_stack, None);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Newarr,
        operand: Some(MsilInstructionOperand::Type(lower_hir_type(element_type).to_string())),
    });
    *max_stack = (*max_stack).max(1);

    let array_slot = local_types.len();
    local_types.push(MsilType::sz_array(lower_hir_type(element_type)));
    emit_stloc(array_slot, instructions, None);

    let store_opcode = array_store_opcode(&lower_hir_type(element_type));
    for (index, item) in items.iter().enumerate() {
        lower_local_slot(array_slot, instructions, max_stack);
        lower_constant(&MirConstant::Int(index as i64), instructions, max_stack, None);
        lower_operand(item, instructions, max_stack, parameter_slots, local_slots, eval_stack);
        instructions.push(MsilInstruction { label: None, opcode: store_opcode, operand: None });
        *max_stack = (*max_stack).max(3);
        eval_stack.clear();
    }

    if let Some(output) = output {
        value_types.insert(output, HirType::Array(Box::new(element_type.clone())));
        local_slots.insert(output, array_slot);
        super::clr_lowering::lower_value_ref(&output, instructions, max_stack, parameter_slots, local_slots);
        eval_stack.push(output);
    }
}

fn infer_subscript_output_type(object: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> Option<HirType> {
    match infer_operand_type(object, value_types)? {
        HirType::Array(item) => Some(*item),
        _ => None,
    }
}

fn array_load_opcode(ty: &MsilType) -> MsilOpcode {
    match ty {
        MsilType::Bool | MsilType::Int8 | MsilType::UInt8 => MsilOpcode::LdelemU1,
        MsilType::Int32 | MsilType::UInt32 => MsilOpcode::LdelemI4,
        _ => MsilOpcode::LdelemRef,
    }
}

fn array_store_opcode(ty: &MsilType) -> MsilOpcode {
    match ty {
        MsilType::Bool | MsilType::Int8 | MsilType::UInt8 => MsilOpcode::StelemI1,
        MsilType::Int32 | MsilType::UInt32 => MsilOpcode::StelemI4,
        _ => MsilOpcode::StelemRef,
    }
}

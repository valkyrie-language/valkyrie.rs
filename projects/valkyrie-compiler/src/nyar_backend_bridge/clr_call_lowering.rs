use std::collections::BTreeMap;

use crate::{
    lir::{LirOperand, LirOperation, LirOperationKind},
    mir::{MirBuiltinBinaryOp, MirBuiltinCall, MirBuiltinCompareOp, MirValueRef},
};
use clr_backend::{MsilInstruction, MsilInstructionOperand, MsilMethodRef, MsilMethodSignature, MsilOpcode, MsilType};
use valkyrie_types::{hir::ValkyrieType as HirType, Identifier};

use super::{
    clr_array_lowering::{lower_array_literal, lower_array_subscript_access, lower_array_subscript_store},
    clr_lowering::{infer_operand_type, lower_hir_type, lower_operand, spill_eval_stack, LocalStructLayout},
};

/// 运算符方法的内建降级策略。
///
/// `Simple`：单个 opcode 即可完成计算。
/// `NegatedComparison`：先做比较，再与 0 比较，用于 `!=`/`<=`/`>=`。
/// `LogicalNot`：直接对布尔结果执行 `ldc.i4.0 + ceq`。
enum IntrinsicCallLowering {
    /// 单条 opcode 即可完成计算。
    Simple(MsilOpcode),
    /// 先执行比较 opcode，再 `ldc.i4.0 + ceq` 取反。
    NegatedComparison(MsilOpcode),
    /// 对单个布尔参数执行逻辑取反。
    LogicalNot,
    /// 数组长度：`ldlen` + `conv.i4`。
    ArrayLength,
    /// 透明包装类型构造：直接保留参数值。
    Identity,
}

pub(super) fn lower_special_operation(
    operation: &LirOperation,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    return_types: &BTreeMap<String, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) -> bool {
    match &operation.kind {
        LirOperationKind::Call { callee, arguments, builtin, .. } => {
            lower_call_operation(
                operation.output,
                callee,
                arguments,
                *builtin,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                return_types,
                value_types,
            );
            true
        }
        LirOperationKind::ArrayNew { element_type, length } => {
            lower_array_new_operation(
                operation.output,
                element_type,
                length,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                value_types,
            );
            true
        }
        LirOperationKind::ArrayLiteral { element_type, items } => {
            lower_array_literal(
                element_type,
                items,
                operation.output,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                value_types,
            );
            true
        }
        LirOperationKind::StructNew { type_name, fields } => {
            lower_struct_new_operation(
                operation.output,
                type_name,
                fields,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                local_structs,
                value_types,
            );
            true
        }
        LirOperationKind::FieldGet { object, field } => {
            lower_field_get_operation(
                operation.output,
                object,
                field,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                local_structs,
                value_types,
            );
            true
        }
        LirOperationKind::FieldSet { object, field, value } => {
            lower_field_set_operation(
                object,
                field,
                value,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                local_structs,
                value_types,
            );
            true
        }
        _ => false,
    }
}

fn lower_call_operation(
    output: Option<MirValueRef>,
    callee: &LirOperand,
    arguments: &[LirOperand],
    builtin: Option<MirBuiltinCall>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    return_types: &BTreeMap<String, HirType>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    if matches!(builtin, Some(MirBuiltinCall::ArrayGet)) {
        lower_array_subscript_access(
            &arguments[0],
            &arguments[1],
            output,
            instructions,
            max_stack,
            parameter_slots,
            local_slots,
            local_types,
            eval_stack,
            value_types,
        );
    }
    else if matches!(builtin, Some(MirBuiltinCall::ArraySet)) {
        lower_array_subscript_store(
            &arguments[0],
            &arguments[1],
            &arguments[2],
            instructions,
            max_stack,
            parameter_slots,
            local_slots,
            local_types,
            eval_stack,
            value_types,
        );
    }
    else if let Some(op_kind) = builtin.and_then(intrinsic_from_builtin) {
        if let Some(output) = output {
            if let Some(inferred_type) = infer_intrinsic_output_type(&op_kind, arguments, value_types) {
                value_types.insert(output, inferred_type);
            }
        }
        lower_intrinsic_call(
            op_kind,
            arguments,
            output,
            instructions,
            max_stack,
            parameter_slots,
            local_slots,
            local_types,
            eval_stack,
            value_types,
            None,
        );
    }
    else {
        // 常规 call：先溢出 eval_stack，再逐个压入参数。
        spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
        for argument in arguments {
            lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
        }
        let callee_operand = match callee {
            LirOperand::Symbol(path) => Some(MsilInstructionOperand::Method(MsilMethodRef {
                owner: None,
                name: path.to_string(),
                signature: MsilMethodSignature::new(
                    infer_call_return_type(callee, return_types).map(|ty| lower_hir_type(&ty)).unwrap_or(MsilType::Object),
                    arguments
                        .iter()
                        .map(|argument| infer_operand_type(argument, value_types).map(|ty| lower_hir_type(&ty)).unwrap_or(MsilType::Object))
                        .collect(),
                ),
            })),
            LirOperand::Value(_) | LirOperand::Constant(_) => {
                lower_operand(callee, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                None
            }
        };
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Call, operand: callee_operand });
        *max_stack = (*max_stack).max((arguments.len() + 1) as u16);
        // call 消费所有参数，产出 1 个返回值。清空 eval_stack 并压入结果。
        // 但若 callee 返回 void，则不应将结果推入 eval_stack（CLR 的 call 指令对 void 方法不在栈上留下返回值）。
        eval_stack.clear();
        if let Some(output) = output {
            let returns_void = callee_returns_void(callee, return_types);
            if !returns_void {
                if let Some(inferred_type) = infer_call_return_type(callee, return_types) {
                    value_types.insert(output, inferred_type);
                }
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
        }
    }
}

fn lower_array_new_operation(
    output: Option<MirValueRef>,
    element_type: &HirType,
    length: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(length, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Newarr,
        operand: Some(MsilInstructionOperand::Type(lower_hir_type(element_type).to_string())),
    });
    *max_stack = (*max_stack).max(1);
    eval_stack.clear();
    if let Some(output) = output {
        value_types.insert(output, HirType::Array(Box::new(element_type.clone())));
        eval_stack.push(output);
        *max_stack = (*max_stack).max(eval_stack.len() as u16);
    }
}

fn lower_struct_new_operation(
    output: Option<MirValueRef>,
    type_name: &str,
    fields: &[(String, LirOperand)],
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    let ordered_fields = order_struct_fields(type_name, fields, local_structs);
    let parameter_types = ordered_fields
        .iter()
        .map(|(field_name, value)| {
            local_structs
                .get(type_name)
                .and_then(|layout| layout.field_types.get(field_name))
                .map(lower_hir_type)
                .or_else(|| infer_operand_type(value, value_types).map(|ty| lower_hir_type(&ty)))
                .unwrap_or(MsilType::Object)
        })
        .collect::<Vec<_>>();
    for (_, value) in &ordered_fields {
        lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    }
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Newobj,
        operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
            owner: Some(normalize_local_type_name(type_name, local_structs).unwrap_or_else(|| type_name.to_string())),
            name: ".ctor".to_string(),
            signature: MsilMethodSignature::new(MsilType::Void, parameter_types),
        })),
    });
    *max_stack = (*max_stack).max(ordered_fields.len() as u16);
    eval_stack.clear();
    if let Some(output) = output {
        value_types.insert(output, HirType::Named(Identifier::new(extract_simple_type_name(type_name))));
        eval_stack.push(output);
        *max_stack = (*max_stack).max(eval_stack.len() as u16);
    }
}

fn lower_field_get_operation(
    output: Option<MirValueRef>,
    object: &LirOperand,
    field: &str,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    if field == "length" {
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ldlen, operand: None });
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::ConvI4, operand: None });
        *max_stack = (*max_stack).max(1);
        eval_stack.clear();
        if let Some(output) = output {
            value_types.insert(output, HirType::Integer32 { signed: true });
            eval_stack.push(output);
            *max_stack = (*max_stack).max(eval_stack.len() as u16);
        }
    }
    else {
        let field_owner = infer_field_owner_name(object, field, value_types, local_structs).unwrap_or_default();
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Ldfld,
            operand: Some(MsilInstructionOperand::Field(field_owner, field.to_string())),
        });
        *max_stack = (*max_stack).max(1);
        eval_stack.clear();
        if let Some(output) = output {
            if let Some(inferred_type) = infer_field_output_type(object, field, value_types, local_structs) {
                value_types.insert(output, inferred_type);
            }
            eval_stack.push(output);
            *max_stack = (*max_stack).max(eval_stack.len() as u16);
        }
    }
}

fn lower_field_set_operation(
    object: &LirOperand,
    field: &str,
    value: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
    value_types: &BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    let field_owner = infer_field_owner_name(object, field, value_types, local_structs).unwrap_or_default();
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Stfld,
        operand: Some(MsilInstructionOperand::Field(field_owner, field.to_string())),
    });
    *max_stack = (*max_stack).max(2);
    eval_stack.clear();
}

fn intrinsic_from_builtin(builtin: MirBuiltinCall) -> Option<IntrinsicCallLowering> {
    Some(match builtin {
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Add) => IntrinsicCallLowering::Simple(MsilOpcode::Add),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Sub) => IntrinsicCallLowering::Simple(MsilOpcode::Sub),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Mul) => IntrinsicCallLowering::Simple(MsilOpcode::Mul),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Div) => IntrinsicCallLowering::Simple(MsilOpcode::Div),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Rem) => IntrinsicCallLowering::Simple(MsilOpcode::Rem),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq) => IntrinsicCallLowering::Simple(MsilOpcode::Ceq),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Lt) => IntrinsicCallLowering::Simple(MsilOpcode::Clt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Gt) => IntrinsicCallLowering::Simple(MsilOpcode::Cgt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ne) => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Ceq),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Le) => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Cgt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ge) => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Clt),
        MirBuiltinCall::NumericNeg => IntrinsicCallLowering::Simple(MsilOpcode::Neg),
        MirBuiltinCall::LogicalAnd => IntrinsicCallLowering::Simple(MsilOpcode::And),
        MirBuiltinCall::LogicalOr => IntrinsicCallLowering::Simple(MsilOpcode::Or),
        MirBuiltinCall::LogicalNot => IntrinsicCallLowering::LogicalNot,
        MirBuiltinCall::ArrayLength => IntrinsicCallLowering::ArrayLength,
        MirBuiltinCall::Identity => IntrinsicCallLowering::Identity,
        MirBuiltinCall::ArrayGet | MirBuiltinCall::ArraySet => return None,
    })
}

fn infer_intrinsic_output_type(
    op_kind: &IntrinsicCallLowering,
    arguments: &[LirOperand],
    value_types: &BTreeMap<MirValueRef, HirType>,
) -> Option<HirType> {
    match op_kind {
        IntrinsicCallLowering::Simple(opcode) => match opcode {
            MsilOpcode::Ceq | MsilOpcode::Clt | MsilOpcode::Cgt => Some(HirType::Boolean),
            _ => arguments.first().and_then(|argument| infer_operand_type(argument, value_types)),
        },
        IntrinsicCallLowering::NegatedComparison(_) | IntrinsicCallLowering::LogicalNot => Some(HirType::Boolean),
        IntrinsicCallLowering::ArrayLength => Some(HirType::Integer32 { signed: true }),
        IntrinsicCallLowering::Identity => arguments.first().and_then(|argument| infer_operand_type(argument, value_types)),
    }
}

fn callee_returns_void(callee: &LirOperand, return_types: &BTreeMap<String, HirType>) -> bool {
    let LirOperand::Symbol(path) = callee
    else {
        return false;
    };
    let symbol_name = path.to_string();
    let return_type = match return_types.get(&symbol_name) {
        Some(ty) => ty,
        None => return false,
    };
    matches!(return_type, HirType::Unit | HirType::Void)
}

fn infer_call_return_type(callee: &LirOperand, return_types: &BTreeMap<String, HirType>) -> Option<HirType> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    return_types.get(&path.to_string()).cloned()
}

fn infer_field_owner_name(
    object: &LirOperand,
    field: &str,
    value_types: &BTreeMap<MirValueRef, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> Option<String> {
    if let Some(local_type_name) = infer_named_type_name(object, value_types) {
        return local_structs.get(&local_type_name).map(|layout| layout.qualified_name.clone());
    }

    let matches = local_structs
        .values()
        .filter(|layout| layout.field_types.contains_key(field))
        .map(|layout| layout.qualified_name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if matches.len() == 1 {
        return matches.into_iter().next();
    }
    None
}

fn infer_field_output_type(
    object: &LirOperand,
    field: &str,
    value_types: &BTreeMap<MirValueRef, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> Option<HirType> {
    let local_type_name = infer_named_type_name(object, value_types)?;
    local_structs.get(&local_type_name)?.field_types.get(field).cloned()
}

fn infer_named_type_name(object: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> Option<String> {
    match infer_operand_type(object, value_types)? {
        HirType::Named(name) => Some(name.to_string()),
        _ => None,
    }
}

fn normalize_local_type_name(type_name: &str, local_structs: &BTreeMap<String, LocalStructLayout>) -> Option<String> {
    let layout = local_structs.get(type_name)?;
    if type_name == layout.name {
        Some(layout.qualified_name.clone())
    }
    else {
        Some(type_name.to_string())
    }
}

fn extract_simple_type_name(type_name: &str) -> &str {
    type_name.rsplit('.').next().unwrap_or(type_name)
}

fn order_struct_fields(
    type_name: &str,
    fields: &[(String, LirOperand)],
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> Vec<(String, LirOperand)> {
    let Some(layout) = local_structs.get(type_name)
    else {
        return fields.to_vec();
    };

    let supplied_fields = fields.iter().cloned().collect::<BTreeMap<_, _>>();
    let mut ordered = Vec::with_capacity(fields.len());
    for field_name in &layout.field_order {
        if let Some(value) = supplied_fields.get(field_name) {
            ordered.push((field_name.clone(), value.clone()));
        }
    }

    if ordered.len() == fields.len() {
        ordered
    }
    else {
        fields.to_vec()
    }
}

fn lower_intrinsic_call(
    op_kind: IntrinsicCallLowering,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &BTreeMap<MirValueRef, HirType>,
    label: Option<String>,
) {
    let arg_count = arguments.len();
    let all_values = arguments.iter().all(|arg| matches!(arg, LirOperand::Value(_)));
    if all_values && eval_stack.len() >= arg_count && arg_count >= 1 {
        let matches = (0..arg_count).all(|i| {
            let arg_val = match &arguments[i] {
                LirOperand::Value(v) => *v,
                _ => unreachable!(),
            };
            let stack_val = eval_stack[eval_stack.len() - arg_count + i];
            arg_val == stack_val
        });
        if matches {
            emit_intrinsic_call(op_kind, label, instructions, max_stack, arg_count, eval_stack, output);
            return;
        }
    }

    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    for argument in arguments {
        lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    }

    emit_intrinsic_call(op_kind, label, instructions, max_stack, arg_count, eval_stack, output);
}

fn emit_intrinsic_call(
    op_kind: IntrinsicCallLowering,
    label: Option<String>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    arg_count: usize,
    eval_stack: &mut Vec<MirValueRef>,
    output: Option<MirValueRef>,
) {
    match op_kind {
        IntrinsicCallLowering::Simple(opcode) => {
            instructions.push(MsilInstruction { label, opcode, operand: None });
            *max_stack = (*max_stack).max(arg_count as u16);
        }
        IntrinsicCallLowering::NegatedComparison(opcode) => {
            instructions.push(MsilInstruction { label, opcode, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ceq, operand: None });
            *max_stack = (*max_stack).max(2);
        }
        IntrinsicCallLowering::LogicalNot => {
            instructions.push(MsilInstruction { label, opcode: MsilOpcode::LdcI4_0, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ceq, operand: None });
            *max_stack = (*max_stack).max(2);
        }
        IntrinsicCallLowering::ArrayLength => {
            instructions.push(MsilInstruction { label, opcode: MsilOpcode::Ldlen, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::ConvI4, operand: None });
            *max_stack = (*max_stack).max(1);
        }
        IntrinsicCallLowering::Identity => {
            *max_stack = (*max_stack).max(arg_count as u16);
        }
    }

    for _ in 0..arg_count {
        eval_stack.pop();
    }

    if let Some(output) = output {
        eval_stack.push(output);
    }
}

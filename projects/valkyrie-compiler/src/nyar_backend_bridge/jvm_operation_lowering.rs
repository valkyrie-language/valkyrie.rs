use std::collections::BTreeMap;

use crate::{
    lir::{LirDispatchKind, LirOperand, LirOperation, LirOperationKind},
    mir::{MirBuiltinCall, MirConstant, MirValueRef},
};
use jvm_backend::{JvmInstruction, JvmMethodDescriptor, JvmMethodRef, JvmTypeDescriptor};
use miette::{miette, Result};
use valkyrie_types::NamePath;

use super::{
    jvm_host_bridge::try_lower_host_bridge_call,
    jvm_intrinsics::{
        intrinsic_output_type, jvm_intrinsic_from_builtin, load_receiver_operand, lower_intrinsic_call, try_intrinsic_call,
        try_method_intrinsic, JvmIntrinsicCallLowering,
    },
    jvm_lowering::{
        array_load_instruction, array_new_instruction, array_store_instruction, constant_type, emitted_function_name, jvm_type_descriptor,
        load_instruction_for_type, logical_symbol_name, operand_type, push_default_descriptor_value, store_instruction_for_type,
        FunctionLoweringContext,
    },
};

pub(super) fn lower_operation(
    operation: &LirOperation,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    match &operation.kind {
        LirOperationKind::LoadConstant { constant, ty } => {
            let inferred_output_ty = operation.output.and_then(|output| context.value_types.get(&output).cloned());
            let explicit_ty = ty.as_ref().and_then(|ty| jvm_type_descriptor(ty).ok());
            let jvm_ty = match (explicit_ty, inferred_output_ty) {
                (
                    Some(JvmTypeDescriptor::Int),
                    Some(inferred @ (JvmTypeDescriptor::Long | JvmTypeDescriptor::Float | JvmTypeDescriptor::Double)),
                ) => Some(inferred),
                (Some(JvmTypeDescriptor::Object(name)), Some(inferred)) if name == "java/lang/Object" => Some(inferred),
                (Some(explicit), _) => Some(explicit),
                (None, inferred) => inferred,
            };
            lower_constant(constant, jvm_ty.as_ref(), instructions)?;
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::LoadSymbol { path } => {
            if let Some(var_name) = local_symbol_name(path) {
                if let Some(slot) = context.try_slot_for_var(&var_name) {
                    let ty = context.type_for_var(&var_name);
                    if let Some(output) = operation.output {
                        context.value_types.insert(output, ty.clone());
                    }
                    instructions.push(load_instruction_for_type(&ty, slot));
                    store_output(operation.output, context, instructions);
                    return Ok(());
                }
            }
            // 模块路径等非局部符号暂用 null 占位，待后续支持完整的外部符号解析。
            instructions.push(JvmInstruction::AConstNull);
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::Move { source } => {
            lower_operand(source, context, instructions)?;
            // 根据 source 类型动态选择存储指令，避免预推断类型不完整导致的类型不匹配。
            if let Some(output) = operation.output {
                let slot = context.slot_for_output(output);
                let ty = operand_type(source, context);
                instructions.push(store_instruction_for_type(&ty, slot));
                context.value_types.insert(output, ty);
            }
        }
        LirOperationKind::StoreVar { name, value, ty: _ } => {
            lower_operand(value, context, instructions)?;
            let slot = context.slot_for_var(name);
            // 优先使用变量已记录的类型，否则再从 value 推断。
            let ty = context.var_types.get(name).cloned().unwrap_or_else(|| operand_type(value, context));
            context.var_types.insert(name.clone(), ty.clone());
            instructions.push(store_instruction_for_type(&ty, slot));
            if let Some(output) = operation.output {
                context.value_slots.insert(output, slot);
                context.value_types.insert(output, ty);
            }
        }
        LirOperationKind::Call { dispatch, callee, arguments, builtin, witness, effect } => {
            if witness.is_some() || effect.is_some() {
                return Err(miette!("JVM 最小 lowering 暂不支持 witness / effect 调用"));
            }
            if *dispatch != LirDispatchKind::Static {
                return Err(miette!("JVM 最小 lowering 暂只支持静态调用"));
            }
            if matches!(builtin, Some(MirBuiltinCall::ArrayGet)) {
                let array_ty = operand_type(&arguments[0], context);
                let element_ty = match array_ty {
                    JvmTypeDescriptor::Array(item) => *item,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                };
                lower_operand(&arguments[0], context, instructions)?;
                lower_operand(&arguments[1], context, instructions)?;
                instructions.push(array_load_instruction(&element_ty));
                store_output(operation.output, context, instructions);
            }
            else if matches!(builtin, Some(MirBuiltinCall::ArraySet)) {
                let array_ty = operand_type(&arguments[0], context);
                let element_ty = match array_ty {
                    JvmTypeDescriptor::Array(item) => *item,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                };
                lower_operand(&arguments[0], context, instructions)?;
                lower_operand(&arguments[1], context, instructions)?;
                lower_operand(&arguments[2], context, instructions)?;
                instructions.push(array_store_instruction(&element_ty));
            }
            else if matches!(builtin, Some(MirBuiltinCall::Identity)) {
                let argument = arguments.first().ok_or_else(|| miette!("Identity 内建调用缺少参数"))?;
                lower_operand(argument, context, instructions)?;
                if let Some(output) = operation.output {
                    context.value_types.insert(output, operand_type(argument, context));
                }
                store_output(operation.output, context, instructions);
            }
            else if let Some(intrinsic) = builtin.and_then(jvm_intrinsic_from_builtin) {
                lower_intrinsic_call(intrinsic, arguments, operation.output, context, instructions)?;
            }
            else if let Some((intrinsic, receiver_path)) = try_method_intrinsic(callee) {
                let receiver_operand = load_receiver_operand(&receiver_path, context, instructions)?;
                let mut combined_args = vec![receiver_operand];
                combined_args.extend_from_slice(arguments);
                lower_intrinsic_call(intrinsic, &combined_args, operation.output, context, instructions)?;
            }
            else {
                lower_static_call(callee, arguments, operation.output, context, instructions)?;
            }
        }
        LirOperationKind::ArrayNew { element_type, length } => {
            lower_operand(length, context, instructions)?;
            let element_descriptor = jvm_type_descriptor(element_type)?;
            instructions.push(array_new_instruction(&element_descriptor));
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::ArrayLiteral { element_type, items } => {
            let element_descriptor = jvm_type_descriptor(element_type)?;
            let array_descriptor = JvmTypeDescriptor::Array(Box::new(element_descriptor.clone()));
            let slot = if let Some(output) = operation.output {
                context.value_types.insert(output, array_descriptor.clone());
                context.slot_for_output(output)
            }
            else {
                context.allocate_slot()
            };

            instructions.push(JvmInstruction::IConst(items.len() as i32));
            instructions.push(array_new_instruction(&element_descriptor));
            instructions.push(store_instruction_for_type(&array_descriptor, slot));

            for (index, item) in items.iter().enumerate() {
                instructions.push(load_instruction_for_type(&array_descriptor, slot));
                instructions.push(JvmInstruction::IConst(index as i32));
                lower_operand(item, context, instructions)?;
                instructions.push(array_store_instruction(&element_descriptor));
            }
        }
        LirOperationKind::StructNew { type_name: _, fields: _ } => {
            instructions.push(JvmInstruction::AConstNull);
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::FieldGet { object, field } => {
            let field_ty = context.infer_field_get_type(object, field);
            if let Some(output) = operation.output {
                context.value_types.insert(output, field_ty.clone());
            }
            if field == "length" {
                lower_operand(object, context, instructions)?;
                instructions.push(JvmInstruction::ArrayLength);
            }
            else {
                push_default_descriptor_value(&field_ty, instructions);
            }
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::FieldSet { object: _, field: _, value: _ } => {}
        LirOperationKind::PatternMatch { .. } => {
            instructions.push(JvmInstruction::IConst(0));
            if let Some(output) = operation.output {
                context.value_types.insert(output, JvmTypeDescriptor::Boolean);
            }
            store_output(operation.output, context, instructions);
        }
    }
    Ok(())
}

pub(super) fn infer_operation_output_type(operation: &LirOperation, signatures: &BTreeMap<String, JvmMethodDescriptor>) -> JvmTypeDescriptor {
    match &operation.kind {
        LirOperationKind::LoadConstant { constant, ty } => {
            ty.as_ref().and_then(|hir_ty| jvm_type_descriptor(hir_ty).ok()).unwrap_or_else(|| constant_type(constant))
        }
        LirOperationKind::LoadSymbol { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        LirOperationKind::Move { source } => super::jvm_lowering::infer_operand_type(source, signatures),
        LirOperationKind::StoreVar { name, ty, .. } => {
            if let Some(hir_ty) = ty {
                if let Ok(jvm_ty) = jvm_type_descriptor(hir_ty) {
                    return jvm_ty;
                }
            }
            let _ = name;
            JvmTypeDescriptor::Int
        }
        LirOperationKind::Call { callee, builtin, arguments, .. } => {
            if matches!(builtin, Some(MirBuiltinCall::Identity)) {
                return arguments
                    .first()
                    .map(|argument| super::jvm_lowering::infer_operand_type(argument, signatures))
                    .unwrap_or(JvmTypeDescriptor::Object("java/lang/Object".to_string()));
            }
            if matches!(builtin, Some(MirBuiltinCall::BinaryNumeric(_) | MirBuiltinCall::NumericNeg)) {
                return arguments
                    .first()
                    .map(|argument| super::jvm_lowering::infer_operand_type(argument, signatures))
                    .unwrap_or(JvmTypeDescriptor::Int);
            }
            if let Some(intrinsic) = builtin.and_then(jvm_intrinsic_from_builtin) {
                return intrinsic_output_type(intrinsic);
            }
            match try_intrinsic_call(callee) {
                Some(JvmIntrinsicCallLowering::ArrayLiteral) => {
                    JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string())))
                }
                Some(JvmIntrinsicCallLowering::StringSplit) => {
                    JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string())))
                }
                Some(
                    JvmIntrinsicCallLowering::StringTrim
                    | JvmIntrinsicCallLowering::StringToLower
                    | JvmIntrinsicCallLowering::StringToUpper
                    | JvmIntrinsicCallLowering::StringSlice
                    | JvmIntrinsicCallLowering::StringReplace,
                ) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                Some(_) => JvmTypeDescriptor::Int,
                None => {
                    if let LirOperand::Symbol(path) = callee {
                        if let Some(symbol) = logical_symbol_name(path) {
                            if let Some(resolved_symbol) = resolve_signature_symbol(&symbol, signatures) {
                                if let Some(descriptor) = signatures.get(resolved_symbol) {
                                    return descriptor.return_type.clone();
                                }
                            }
                        }
                    }
                    JvmTypeDescriptor::Object("java/lang/Object".to_string())
                }
            }
        }
        LirOperationKind::StructNew { type_name, .. } => JvmTypeDescriptor::Object(type_name.clone()),
        LirOperationKind::FieldGet { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        LirOperationKind::FieldSet { .. } => JvmTypeDescriptor::Void,
        LirOperationKind::PatternMatch { .. } => JvmTypeDescriptor::Boolean,
        LirOperationKind::ArrayNew { element_type, .. } => match jvm_type_descriptor(element_type) {
            Ok(jvm_ty) => JvmTypeDescriptor::Array(Box::new(jvm_ty)),
            Err(_) => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
        },
        LirOperationKind::ArrayLiteral { element_type, .. } => match jvm_type_descriptor(element_type) {
            Ok(jvm_ty) => JvmTypeDescriptor::Array(Box::new(jvm_ty)),
            Err(_) => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
        },
    }
}

pub(super) fn lower_operand(
    operand: &LirOperand,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    lower_operand_with_hint(operand, None, context, instructions)
}

pub(super) fn lower_operand_with_hint(
    operand: &LirOperand,
    expected_ty: Option<&JvmTypeDescriptor>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    match operand {
        LirOperand::Value(value) => {
            let slot = context.slot_for_value(*value)?;
            let ty = context.type_for_value(*value);
            instructions.push(load_instruction_for_type(&ty, slot));
        }
        LirOperand::Constant(constant) => {
            let constant_ty = constant_type(constant);
            let hint = expected_ty.unwrap_or(&constant_ty);
            lower_constant(constant, Some(hint), instructions)?;
        }
        LirOperand::Symbol(path) => {
            if let Some(var_name) = local_symbol_name(path) {
                if let Some(slot) = context.try_slot_for_var(&var_name) {
                    let ty = context.type_for_var(&var_name);
                    instructions.push(load_instruction_for_type(&ty, slot));
                    return Ok(());
                }
            }
            instructions.push(JvmInstruction::AConstNull);
        }
    }
    Ok(())
}

pub(super) fn store_required_output(
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    let output = output.ok_or_else(|| miette!("当前 JVM 最小 lowering 要求值产生操作必须带输出槽位"))?;
    let slot = context.slot_for_output(output);
    let ty = context.type_for_value(output);
    instructions.push(store_instruction_for_type(&ty, slot));
    Ok(())
}

fn store_output(output: Option<MirValueRef>, context: &mut FunctionLoweringContext<'_>, instructions: &mut Vec<JvmInstruction>) {
    if let Some(output) = output {
        let slot = context.slot_for_output(output);
        let ty = context.type_for_value(output);
        instructions.push(store_instruction_for_type(&ty, slot));
    }
}

fn lower_constant(constant: &MirConstant, ty: Option<&JvmTypeDescriptor>, instructions: &mut Vec<JvmInstruction>) -> Result<()> {
    match constant {
        MirConstant::Int(value) => {
            let prefer_long = matches!(ty, Some(JvmTypeDescriptor::Long));
            if prefer_long || *value < i32::MIN as i64 || *value > i32::MAX as i64 {
                match *value {
                    0 => instructions.push(JvmInstruction::LConst0),
                    1 => instructions.push(JvmInstruction::LConst1),
                    value => instructions.push(JvmInstruction::LdcLong(value)),
                }
            }
            else {
                let value = i32::try_from(*value).map_err(|_| miette!("JVM 最小 lowering 暂只支持 `i32` 常量"))?;
                instructions.push(JvmInstruction::IConst(value));
            }
        }
        MirConstant::Float64(value) => match value.0 {
            0.0 => {
                instructions.push(JvmInstruction::DConst0);
            }
            1.0 => {
                instructions.push(JvmInstruction::DConst1);
            }
            value => {
                instructions.push(JvmInstruction::LdcDouble(value.to_bits()));
            }
        },
        MirConstant::Bool(value) => {
            instructions.push(JvmInstruction::IConst(i32::from(*value)));
        }
        MirConstant::Unit => {
            instructions.push(JvmInstruction::IConst(0));
        }
        MirConstant::String(value) => {
            instructions.push(JvmInstruction::LdcString(value.clone()));
        }
    }
    Ok(())
}

fn lower_static_call(
    callee: &LirOperand,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    let symbol = symbol_name(callee)?;
    let resolved_symbol = resolve_signature_symbol(&symbol, context.signatures).unwrap_or(symbol.as_str());
    if try_lower_host_bridge_call(&symbol, arguments, output, context, instructions)? {
        return Ok(());
    }
    let descriptor = context.signatures.get(resolved_symbol).cloned().unwrap_or_else(|| {
        let parameter_types = arguments.iter().map(|arg| operand_type(arg, context)).collect();
        JvmMethodDescriptor::new(parameter_types, JvmTypeDescriptor::Object("java/lang/Object".to_string()))
    });
    for (index, argument) in arguments.iter().enumerate() {
        lower_operand_with_hint(argument, descriptor.parameter_types.get(index), context, instructions)?;
        if let Some(expected_ty) = descriptor.parameter_types.get(index) {
            let actual_ty = operand_type(argument, context);
            if needs_checkcast(&actual_ty, expected_ty) {
                if let JvmTypeDescriptor::Object(class_name) = expected_ty {
                    instructions.push(JvmInstruction::CheckCast(class_name.clone()));
                }
            }
        }
    }
    instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
        owner: context.owner.to_string(),
        name: emitted_function_name(resolved_symbol),
        descriptor: descriptor.clone(),
    }));

    if descriptor.return_type != JvmTypeDescriptor::Void {
        if let Some(output) = output {
            context.value_types.insert(output, descriptor.return_type.clone());
        }
        store_required_output(output, context, instructions)?;
    }
    Ok(())
}

fn symbol_name(callee: &LirOperand) -> Result<String> {
    let LirOperand::Symbol(path) = callee
    else {
        return Err(miette!("JVM 最小 lowering 暂只支持符号静态调用"));
    };
    logical_symbol_name(path).ok_or_else(|| miette!("无法解析被调符号 `{}`", path))
}

pub(super) fn resolve_signature_symbol<'a>(symbol: &'a str, signatures: &'a BTreeMap<String, JvmMethodDescriptor>) -> Option<&'a str> {
    if signatures.contains_key(symbol) {
        return Some(symbol);
    }

    let mut matches = signatures.keys().filter(|key| key.rsplit("::").next() == Some(symbol));
    let first = matches.next()?;
    if matches.next().is_some() {
        None
    }
    else {
        Some(first.as_str())
    }
}

fn needs_checkcast(actual: &JvmTypeDescriptor, expected: &JvmTypeDescriptor) -> bool {
    match (actual, expected) {
        (JvmTypeDescriptor::Object(actual_name), JvmTypeDescriptor::Object(expected_name)) => {
            actual_name == "java/lang/Object" && expected_name != "java/lang/Object"
        }
        _ => false,
    }
}

fn local_symbol_name(path: &NamePath) -> Option<String> {
    if path.parts().len() == 1 {
        path.parts().last().map(|segment| segment.to_string())
    }
    else {
        None
    }
}

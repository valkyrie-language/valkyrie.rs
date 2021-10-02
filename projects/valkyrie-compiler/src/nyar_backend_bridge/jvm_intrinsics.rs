use jvm_backend::{JvmInstruction, JvmMethodDescriptor, JvmMethodRef, JvmTypeDescriptor};
use miette::{miette, Result};
use valkyrie_types::NamePath;

use crate::{
    lir::LirOperand,
    mir::{MirBuiltinBinaryOp, MirBuiltinCall, MirBuiltinCompareOp, MirConstant, MirValueRef},
};

use super::{
    jvm_lowering::{
        array_new_instruction, array_store_instruction, emit_compare_branch, infer_operand_type, is_reference_descriptor,
        load_instruction_for_type, numeric_binary_instruction, numeric_neg_instruction, operand_is_string, operand_type,
        store_instruction_for_type, string_value_of_descriptor, FunctionLoweringContext,
    },
    jvm_operation_lowering::{lower_operand, lower_operand_with_hint, store_required_output},
};

#[derive(Clone, Copy)]
pub(super) enum JvmIntrinsicCallLowering {
    BinaryNumeric(JvmBinaryNumericOp),
    Compare(JvmIntComparison),
    NumericNeg,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    /// 数组/字符串长度，对应 `arraylength` 指令，返回 int。
    ArrayLength,
    /// 数组字面量构造，对应 `newarray` / `anewarray` 指令序列。
    ArrayLiteral,
    /// 字符串去除首尾空白，对应 `invokevirtual java/lang/String.trim()`。
    StringTrim,
    /// 字符串转小写，对应 `invokevirtual java/lang/String.toLowerCase()`。
    StringToLower,
    /// 字符串转大写，对应 `invokevirtual java/lang/String.toUpperCase()`。
    StringToUpper,
    /// 字符串前缀匹配，对应 `invokevirtual java/lang/String.startsWith(Ljava/lang/String;)Z`。
    StringStartsWith,
    /// 字符串后缀匹配，对应 `invokevirtual java/lang/String.endsWith(Ljava/lang/String;)Z`。
    StringEndsWith,
    /// 字符串包含判断，对应 `invokevirtual java/lang/String.contains(Ljava/lang/CharSequence;)Z`。
    StringContains,
    /// 字符串切片，对应 `invokevirtual java/lang/String.substring(II)Ljava/lang/String;`。
    StringSlice,
    /// 字符串替换，对应 `invokevirtual java/lang/String.replace(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;`。
    StringReplace,
    /// 字符串分割，对应 `invokevirtual java/lang/String.split(Ljava/lang/String;)[Ljava/lang/String;`。
    StringSplit,
}

#[derive(Clone, Copy)]
pub(super) enum JvmBinaryNumericOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Clone, Copy)]
pub(super) enum JvmIntComparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

pub(super) fn try_intrinsic_call(callee: &LirOperand) -> Option<JvmIntrinsicCallLowering> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    if path.parts().len() != 1 {
        return None;
    }
    Some(match path.parts()[0].as_str() {
        "array" => JvmIntrinsicCallLowering::ArrayLiteral,
        "trim" => JvmIntrinsicCallLowering::StringTrim,
        "to_lower" | "to_lowercase" | "lowercase" => JvmIntrinsicCallLowering::StringToLower,
        "to_upper" | "to_uppercase" | "uppercase" => JvmIntrinsicCallLowering::StringToUpper,
        "starts_with" | "startsWith" => JvmIntrinsicCallLowering::StringStartsWith,
        "ends_with" | "endsWith" => JvmIntrinsicCallLowering::StringEndsWith,
        "contains" => JvmIntrinsicCallLowering::StringContains,
        "slice" | "substring" => JvmIntrinsicCallLowering::StringSlice,
        "replace" => JvmIntrinsicCallLowering::StringReplace,
        "split" => JvmIntrinsicCallLowering::StringSplit,
        _ => return None,
    })
}

pub(super) fn jvm_intrinsic_from_builtin(builtin: MirBuiltinCall) -> Option<JvmIntrinsicCallLowering> {
    Some(match builtin {
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Add) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Add),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Sub) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Sub),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Mul) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Mul),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Div) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Div),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Rem) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Rem),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Eq),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ne) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Ne),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Lt) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Lt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Le) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Le),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Gt) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Gt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ge) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Ge),
        MirBuiltinCall::NumericNeg => JvmIntrinsicCallLowering::NumericNeg,
        MirBuiltinCall::LogicalAnd => JvmIntrinsicCallLowering::LogicalAnd,
        MirBuiltinCall::LogicalOr => JvmIntrinsicCallLowering::LogicalOr,
        MirBuiltinCall::LogicalNot => JvmIntrinsicCallLowering::LogicalNot,
        MirBuiltinCall::ArrayLength => JvmIntrinsicCallLowering::ArrayLength,
        MirBuiltinCall::ArrayGet | MirBuiltinCall::ArraySet | MirBuiltinCall::Identity => return None,
    })
}

pub(super) fn intrinsic_output_type(intrinsic: JvmIntrinsicCallLowering) -> JvmTypeDescriptor {
    match intrinsic {
        JvmIntrinsicCallLowering::BinaryNumeric(_) | JvmIntrinsicCallLowering::NumericNeg => JvmTypeDescriptor::Int,
        JvmIntrinsicCallLowering::Compare(_)
        | JvmIntrinsicCallLowering::LogicalAnd
        | JvmIntrinsicCallLowering::LogicalOr
        | JvmIntrinsicCallLowering::LogicalNot => JvmTypeDescriptor::Boolean,
        JvmIntrinsicCallLowering::ArrayLength => JvmTypeDescriptor::Int,
        JvmIntrinsicCallLowering::ArrayLiteral => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
        JvmIntrinsicCallLowering::StringTrim
        | JvmIntrinsicCallLowering::StringToLower
        | JvmIntrinsicCallLowering::StringToUpper
        | JvmIntrinsicCallLowering::StringSlice
        | JvmIntrinsicCallLowering::StringReplace => JvmTypeDescriptor::Object("java/lang/String".to_string()),
        JvmIntrinsicCallLowering::StringStartsWith | JvmIntrinsicCallLowering::StringEndsWith | JvmIntrinsicCallLowering::StringContains => {
            JvmTypeDescriptor::Boolean
        }
        JvmIntrinsicCallLowering::StringSplit => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
    }
}

/// 尝试将方法调用风格的内建函数（如 `x.length()`）解析为内建调用。
///
/// 当 callee 是恰好两段路径且最后一段是内建函数名时，返回内建类型和接收者路径。
/// 例如 `NamePath(["targets", "length"])` 返回 `(ArrayLength, NamePath(["targets"]))`。
/// 多段路径（如 `request.project.length`）暂不支持，需后续添加字段访问支持。
pub(super) fn try_method_intrinsic(callee: &LirOperand) -> Option<(JvmIntrinsicCallLowering, NamePath)> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    if path.parts().len() != 2 {
        return None;
    }
    let last_segment = path.parts().last()?;
    let intrinsic = match last_segment.as_str() {
        "len" | "length" => JvmIntrinsicCallLowering::ArrayLength,
        "trim" => JvmIntrinsicCallLowering::StringTrim,
        "to_lower" | "to_lowercase" | "lowercase" => JvmIntrinsicCallLowering::StringToLower,
        "to_upper" | "to_uppercase" | "uppercase" => JvmIntrinsicCallLowering::StringToUpper,
        "starts_with" | "startsWith" => JvmIntrinsicCallLowering::StringStartsWith,
        "ends_with" | "endsWith" => JvmIntrinsicCallLowering::StringEndsWith,
        "contains" => JvmIntrinsicCallLowering::StringContains,
        "slice" | "substring" => JvmIntrinsicCallLowering::StringSlice,
        "replace" => JvmIntrinsicCallLowering::StringReplace,
        "split" => JvmIntrinsicCallLowering::StringSplit,
        _ => return None,
    };
    let receiver_segments = path.parts()[..path.parts().len() - 1].to_vec();
    Some((intrinsic, NamePath::new(receiver_segments)))
}

/// 加载接收者操作数到栈上，并返回对应的 `LirOperand::Value`。
///
/// 接收者路径为单段时，从命名变量槽位加载。
pub(super) fn load_receiver_operand(
    receiver_path: &NamePath,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<LirOperand> {
    if receiver_path.parts().len() != 1 {
        return Err(miette!("方法调用内建函数的接收者路径仅支持单段，收到 {:?}", receiver_path));
    }
    let var_name = receiver_path.parts()[0].as_str();
    let slot = context.try_slot_for_var(var_name).ok_or_else(|| miette!("方法调用内建函数的接收者变量 `{}` 未找到槽位", var_name))?;
    let ty = context.type_for_var(var_name);
    instructions.push(load_instruction_for_type(&ty, slot));
    // 分配临时槽位存储接收者值，创建合成 MirValueRef 供内建 lowering 使用。
    let temp_slot = context.allocate_slot();
    let temp_ref = MirValueRef(u32::MAX / 2 + temp_slot as u32);
    instructions.push(store_instruction_for_type(&ty, temp_slot));
    context.value_slots.insert(temp_ref, temp_slot);
    context.value_types.insert(temp_ref, ty);
    Ok(LirOperand::Value(temp_ref))
}

pub(super) fn lower_intrinsic_call(
    intrinsic: JvmIntrinsicCallLowering,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    match intrinsic {
        JvmIntrinsicCallLowering::BinaryNumeric(op) => {
            if arguments.len() != 2 {
                return Err(miette!("二元数值内建调用参数数量错误"));
            }
            if matches!(op, JvmBinaryNumericOp::Add) {
                let lhs_ty = operand_type(&arguments[0], context);
                let rhs_ty = operand_type(&arguments[1], context);
                let lhs_is_string = operand_is_string(&arguments[0], context);
                let rhs_is_string = operand_is_string(&arguments[1], context);
                if lhs_is_string || rhs_is_string || is_reference_descriptor(&lhs_ty) || is_reference_descriptor(&rhs_ty) {
                    lower_operand(&arguments[0], context, instructions)?;
                    if !lhs_is_string {
                        instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                            owner: "java/lang/String".to_string(),
                            name: "valueOf".to_string(),
                            descriptor: string_value_of_descriptor(&lhs_ty),
                        }));
                    }
                    lower_operand(&arguments[1], context, instructions)?;
                    if !rhs_is_string {
                        instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                            owner: "java/lang/String".to_string(),
                            name: "valueOf".to_string(),
                            descriptor: string_value_of_descriptor(&rhs_ty),
                        }));
                    }
                    instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                        owner: "java/lang/String".to_string(),
                        name: "concat".to_string(),
                        descriptor: JvmMethodDescriptor::new(
                            vec![JvmTypeDescriptor::Object("java/lang/String".to_string())],
                            JvmTypeDescriptor::Object("java/lang/String".to_string()),
                        ),
                    }));
                    if let Some(output) = output {
                        context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
                    }
                    store_required_output(output, context, instructions)?;
                    return Ok(());
                }
            }
            let lhs_ty = operand_type(&arguments[0], context);
            let rhs_ty = operand_type(&arguments[1], context);
            if is_reference_descriptor(&lhs_ty) || is_reference_descriptor(&rhs_ty) {
                if let Some(output) = output {
                    context.value_types.insert(output, JvmTypeDescriptor::Int);
                }
                instructions.push(JvmInstruction::IConst(0));
                store_required_output(output, context, instructions)?;
                return Ok(());
            }
            let result_ty = lhs_ty;
            lower_operand_with_hint(&arguments[0], Some(&result_ty), context, instructions)?;
            lower_operand_with_hint(&arguments[1], Some(&result_ty), context, instructions)?;
            instructions.push(numeric_binary_instruction(&result_ty, op)?);
            if let Some(output) = output {
                context.value_types.insert(output, result_ty);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::Compare(compare) => {
            if arguments.len() != 2 {
                return Err(miette!("数值比较内建调用参数数量错误"));
            }
            let compare_ty = operand_type(&arguments[0], context);
            lower_operand_with_hint(&arguments[0], Some(&compare_ty), context, instructions)?;
            lower_operand_with_hint(&arguments[1], Some(&compare_ty), context, instructions)?;
            let true_label = context.fresh_label("cmp_true");
            let end_label = context.fresh_label("cmp_end");
            emit_compare_branch(&compare_ty, compare, &true_label, instructions)?;
            instructions.push(JvmInstruction::IConst(0));
            instructions.push(JvmInstruction::Goto(end_label.clone()));
            instructions.push(JvmInstruction::Label(true_label));
            instructions.push(JvmInstruction::IConst(1));
            instructions.push(JvmInstruction::Label(end_label));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::NumericNeg => {
            if arguments.len() != 1 {
                return Err(miette!("数值取负内建调用参数数量错误"));
            }
            let value_ty = operand_type(&arguments[0], context);
            lower_operand_with_hint(&arguments[0], Some(&value_ty), context, instructions)?;
            instructions.push(numeric_neg_instruction(&value_ty)?);
            if let Some(output) = output {
                context.value_types.insert(output, value_ty);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::LogicalAnd => {
            if arguments.len() != 2 {
                return Err(miette!("逻辑与内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::IAnd);
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Boolean);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::LogicalOr => {
            if arguments.len() != 2 {
                return Err(miette!("逻辑或内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::IOr);
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Boolean);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::LogicalNot => {
            if arguments.len() != 1 {
                return Err(miette!("逻辑取反内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            let true_label = context.fresh_label("not_true");
            let end_label = context.fresh_label("not_end");
            instructions.push(JvmInstruction::IfEq(true_label.clone()));
            instructions.push(JvmInstruction::IConst(0));
            instructions.push(JvmInstruction::Goto(end_label.clone()));
            instructions.push(JvmInstruction::Label(true_label));
            instructions.push(JvmInstruction::IConst(1));
            instructions.push(JvmInstruction::Label(end_label));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::ArrayLength => {
            if arguments.len() != 1 {
                return Err(miette!("数组长度内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            let arg_ty = infer_operand_type(&arguments[0], context.signatures);
            let arg_ty = match &arguments[0] {
                LirOperand::Value(v) => context.type_for_value(*v),
                _ => arg_ty,
            };
            if matches!(arg_ty, JvmTypeDescriptor::Object(ref name) if name == "java/lang/String") {
                instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                    owner: "java/lang/String".to_string(),
                    name: "length".to_string(),
                    descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Int),
                }));
            }
            else {
                instructions.push(JvmInstruction::ArrayLength);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::ArrayLiteral => {
            let element_type = if let Some(output) = output {
                let array_ty = context.type_for_value(output);
                match array_ty {
                    JvmTypeDescriptor::Array(item) => *item,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                }
            }
            else if !arguments.is_empty() {
                match &arguments[0] {
                    LirOperand::Value(v) => context.type_for_value(*v),
                    LirOperand::Constant(MirConstant::String(_)) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                    LirOperand::Constant(MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit) => JvmTypeDescriptor::Int,
                    LirOperand::Constant(MirConstant::Float64(_)) => JvmTypeDescriptor::Double,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                }
            }
            else {
                JvmTypeDescriptor::Object("java/lang/Object".to_string())
            };
            instructions.push(JvmInstruction::IConst(arguments.len() as i32));
            instructions.push(array_new_instruction(&element_type));
            for (index, argument) in arguments.iter().enumerate() {
                instructions.push(JvmInstruction::Dup);
                instructions.push(JvmInstruction::IConst(index as i32));
                lower_operand(argument, context, instructions)?;
                instructions.push(array_store_instruction(&element_type));
            }
            if let Some(output) = output {
                let array_ty = JvmTypeDescriptor::Array(Box::new(element_type));
                context.value_types.insert(output, array_ty);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringTrim => {
            if arguments.len() != 1 {
                return Err(miette!("字符串 trim 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "trim".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Object("java/lang/String".to_string())),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringToLower => {
            if arguments.len() != 1 {
                return Err(miette!("字符串 to_lower 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "toLowerCase".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Object("java/lang/String".to_string())),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringToUpper => {
            if arguments.len() != 1 {
                return Err(miette!("字符串 to_upper 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "toUpperCase".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Object("java/lang/String".to_string())),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringStartsWith => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 starts_with 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "startsWith".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".to_string())], JvmTypeDescriptor::Int),
            }));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringEndsWith => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 ends_with 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "endsWith".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".to_string())], JvmTypeDescriptor::Int),
            }));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringContains => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 contains 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "contains".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/lang/CharSequence".to_string())],
                    JvmTypeDescriptor::Int,
                ),
            }));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringSlice => {
            if arguments.len() != 3 {
                return Err(miette!("字符串 slice 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            lower_operand(&arguments[2], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "substring".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Int, JvmTypeDescriptor::Int],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringReplace => {
            if arguments.len() != 3 {
                return Err(miette!("字符串 replace 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            lower_operand(&arguments[2], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "replace".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/lang/CharSequence".to_string()),
                        JvmTypeDescriptor::Object("java/lang/CharSequence".to_string()),
                    ],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringSplit => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 split 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "split".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/lang/String".to_string())],
                    JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
                ),
            }));
            if let Some(output) = output {
                let array_ty = JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string())));
                context.value_types.insert(output, array_ty);
            }
            store_required_output(output, context, instructions)?;
        }
    }
    Ok(())
}

use std::collections::BTreeMap;

use valkyrie_types::NamePath;

use super::{infer_builder_operand_type, MirBuiltinBinaryOp, MirBuiltinCall, MirBuiltinCompareOp, MirOperand, MirValueRef, ValkyrieType};

pub(super) fn plain_type_pattern_matches(actual_type: &ValkyrieType, pattern_name: &NamePath) -> bool {
    let Some(expected_name) = pattern_name.parts().last().map(|identifier| identifier.as_str())
    else {
        return false;
    };

    match actual_type {
        ValkyrieType::Void => expected_name == "void",
        ValkyrieType::Unit => expected_name == "unit",
        ValkyrieType::Boolean => expected_name == "bool",
        ValkyrieType::Integer8 { signed } => expected_name == if *signed { "i8" } else { "u8" },
        ValkyrieType::Integer16 { signed } => expected_name == if *signed { "i16" } else { "u16" },
        ValkyrieType::Integer32 { signed } => expected_name == if *signed { "i32" } else { "u32" },
        ValkyrieType::Integer64 { signed } => expected_name == if *signed { "i64" } else { "u64" },
        ValkyrieType::Integer128 { signed } => expected_name == if *signed { "i128" } else { "u128" },
        ValkyrieType::Float32 => expected_name == "f32",
        ValkyrieType::Float64 => expected_name == "f64",
        ValkyrieType::Character => expected_name == "char",
        ValkyrieType::Utf8 => expected_name == "utf8",
        ValkyrieType::Utf16 => expected_name == "utf16",
        ValkyrieType::Named(name) => name.as_str() == expected_name,
        ValkyrieType::Apply(base, _) => plain_type_pattern_matches(base, pattern_name),
        ValkyrieType::Array(_) => expected_name == "array",
        ValkyrieType::Tuple(_) => expected_name == "tuple",
        ValkyrieType::Row(_) => expected_name == "row",
        ValkyrieType::Function(_) => expected_name == "function",
        ValkyrieType::TraitObject(_) => expected_name == "trait_object",
        ValkyrieType::Associated(_) => expected_name == "associated",
        ValkyrieType::AutoType => expected_name == "auto",
        ValkyrieType::SelfType => expected_name == "Self",
        ValkyrieType::Generic(generic) => generic.name.as_str() == expected_name,
        ValkyrieType::TypeLambda(_) => expected_name == "type_lambda",
    }
}

pub(super) fn resolve_builtin_call(
    callee: &MirOperand,
    arguments: &[MirOperand],
    value_types: &BTreeMap<MirValueRef, ValkyrieType>,
) -> Option<MirBuiltinCall> {
    let MirOperand::Symbol(path) = callee
    else {
        return None;
    };
    let first = arguments.first().and_then(|arg| infer_builder_operand_type(arg, value_types));
    let second = arguments.get(1).and_then(|arg| infer_builder_operand_type(arg, value_types));
    match path.to_string().as_str() {
        "suffix []" if matches!(first, Some(ValkyrieType::Array(_))) && is_builder_integer_type(second.as_ref()) => {
            Some(MirBuiltinCall::ArrayGet)
        }
        "suffix []=" if matches!(first, Some(ValkyrieType::Array(_))) && is_builder_integer_type(second.as_ref()) => {
            Some(MirBuiltinCall::ArraySet)
        }
        "len" | "length" if matches!(first, Some(ValkyrieType::Array(_))) => Some(MirBuiltinCall::ArrayLength),
        "prefix -" if is_builder_numeric_type(first.as_ref()) => Some(MirBuiltinCall::NumericNeg),
        "infix &&" if matches!(first, Some(ValkyrieType::Boolean)) && matches!(second, Some(ValkyrieType::Boolean)) => {
            Some(MirBuiltinCall::LogicalAnd)
        }
        "infix ||" if matches!(first, Some(ValkyrieType::Boolean)) && matches!(second, Some(ValkyrieType::Boolean)) => {
            Some(MirBuiltinCall::LogicalOr)
        }
        "prefix !" if matches!(first, Some(ValkyrieType::Boolean)) => Some(MirBuiltinCall::LogicalNot),
        "ExitCode" if is_builder_integer_type(first.as_ref()) => Some(MirBuiltinCall::Identity),
        "infix +" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Add),
        "infix -" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Sub),
        "infix *" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Mul),
        "infix /" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Div),
        "infix %" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Rem),
        "infix ==" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Eq),
        "infix !=" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Ne),
        "infix <" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Lt),
        "infix <=" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Le),
        "infix >" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Gt),
        "infix >=" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Ge),
        _ => None,
    }
}

pub(super) fn builtin_call_output_type(
    builtin: MirBuiltinCall,
    arguments: &[MirOperand],
    value_types: &BTreeMap<MirValueRef, ValkyrieType>,
) -> Option<ValkyrieType> {
    match builtin {
        MirBuiltinCall::BinaryNumeric(_) | MirBuiltinCall::NumericNeg | MirBuiltinCall::Identity => {
            arguments.first().and_then(|argument| infer_builder_operand_type(argument, value_types))
        }
        MirBuiltinCall::Compare(_) | MirBuiltinCall::LogicalAnd | MirBuiltinCall::LogicalOr | MirBuiltinCall::LogicalNot => {
            Some(ValkyrieType::Boolean)
        }
        MirBuiltinCall::ArrayGet => match arguments.first().and_then(|argument| infer_builder_operand_type(argument, value_types))? {
            ValkyrieType::Array(item) => Some(*item),
            _ => None,
        },
        MirBuiltinCall::ArraySet => Some(ValkyrieType::Unit),
        MirBuiltinCall::ArrayLength => Some(ValkyrieType::Integer32 { signed: true }),
    }
}

fn resolve_binary_numeric_builtin(lhs: Option<&ValkyrieType>, rhs: Option<&ValkyrieType>, op: MirBuiltinBinaryOp) -> Option<MirBuiltinCall> {
    if lhs == rhs && is_builder_numeric_type(lhs) {
        Some(MirBuiltinCall::BinaryNumeric(op))
    }
    else {
        None
    }
}

fn resolve_compare_builtin(lhs: Option<&ValkyrieType>, rhs: Option<&ValkyrieType>, op: MirBuiltinCompareOp) -> Option<MirBuiltinCall> {
    if lhs == rhs && (is_builder_numeric_type(lhs) || matches!(lhs, Some(ValkyrieType::Boolean))) {
        Some(MirBuiltinCall::Compare(op))
    }
    else {
        None
    }
}

fn is_builder_numeric_type(ty: Option<&ValkyrieType>) -> bool {
    matches!(
        ty,
        Some(
            ValkyrieType::Integer8 { .. }
                | ValkyrieType::Integer16 { .. }
                | ValkyrieType::Integer32 { .. }
                | ValkyrieType::Integer64 { .. }
                | ValkyrieType::Integer128 { .. }
                | ValkyrieType::Float32
                | ValkyrieType::Float64
        )
    )
}

fn is_builder_integer_type(ty: Option<&ValkyrieType>) -> bool {
    matches!(
        ty,
        Some(
            ValkyrieType::Integer8 { .. }
                | ValkyrieType::Integer16 { .. }
                | ValkyrieType::Integer32 { .. }
                | ValkyrieType::Integer64 { .. }
                | ValkyrieType::Integer128 { .. }
        )
    )
}

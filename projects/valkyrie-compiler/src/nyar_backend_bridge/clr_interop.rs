//! `CLR` 前端互操作约定。
//!
//! 收口从前端 `AST/HIR` 到 `MSIL` 所需的 `CLR` 专属规则，
//! 避免命令层自行拼接或拆解 `CLR` 约定字符串。

use clr_backend::{MsilMethodRef, MsilMethodSignature, MsilType};
use miette::Result;
use crate::hir::{HirAttribute, HirExpr, HirExprKind, HirFunction, HirLiteral, HirStringSegment, ValkyrieType};

fn panic_on_unresolved_named_primitive(name: &str) -> ! {
    panic!("CLR interop 收到未归一化的 HIR 原始类型 `{name}`；这说明 AST -> HIR 类型解析仍然泄漏了历史名称，不能在 emit 阶段继续兼容");
}

fn ensure_named_type_is_not_unresolved_primitive(name: &str) {
    if matches!(
        name,
        "i8" | "sbyte"
            | "i16"
            | "short"
            | "i32"
            | "int"
            | "i64"
            | "long"
            | "u8"
            | "byte"
            | "u16"
            | "ushort"
            | "u32"
            | "uint"
            | "u64"
            | "ulong"
            | "f32"
            | "float"
            | "f64"
            | "double"
            | "bool"
            | "boolean"
            | "char"
            | "utf8"
            | "utf16"
            | "unit"
            | "void"
            | "string"
            | "str"
            | "String"
    ) {
        panic_on_unresolved_named_primitive(name);
    }
}

/// 基于 `HIR` 生成 `CLR` 方法签名。
pub fn build_clr_method_signature(function: &HirFunction, is_entry_point: bool) -> Result<MsilMethodSignature> {
    let return_type =
        if is_entry_point { map_entry_return_type_to_msil(&function.return_type)? } else { map_hir_type_to_msil(&function.return_type)? };
    let parameters = function.params.iter().map(|param| map_hir_type_to_msil(&param.ty)).collect::<Result<Vec<_>>>()?;
    Ok(MsilMethodSignature::new(return_type, parameters))
}

/// 基于函数属性解析 `CLR` 外部方法引用。
pub fn resolve_clr_import_ref(hir_function: &HirFunction) -> Result<Option<MsilMethodRef>> {
    let Some(attribute) = hir_function.annotations.iter().find(|attribute| attribute.name.to_string() == "clr")
    else {
        return Ok(None);
    };
    let arguments = extract_attribute_string_arguments(attribute);
    if arguments.len() < 3 {
        return Ok(None);
    }

    Ok(Some(MsilMethodRef {
        owner: Some(format!("[{}]{}", arguments[0], arguments[1])),
        name: arguments[2].clone(),
        signature: build_clr_method_signature(hir_function, false)?,
    }))
}

fn map_entry_return_type_to_msil(ty: &ValkyrieType) -> Result<MsilType> {
    match ty {
        ValkyrieType::Void | ValkyrieType::Integer32 { signed: true } => map_hir_type_to_msil(ty),
        ValkyrieType::Named(name) if name.as_str() == "ExitCode" => map_hir_type_to_msil(ty),
        _ => Ok(MsilType::Int32),
    }
}

/// 将 `HIR` 类型映射为 `MSIL` 类型。
///
/// `utf8` 和 `utf16` 在 `CLR` 中都映射为 `System.String`，
/// 因为 `CLR` 的 `System.String` 始终是 `UTF-16` 编码。
pub fn map_hir_type_to_msil(ty: &ValkyrieType) -> Result<MsilType> {
    match ty {
        ValkyrieType::Integer8 { signed: true } => Ok(MsilType::Int8),
        ValkyrieType::Integer8 { signed: false } => Ok(MsilType::UInt8),
        ValkyrieType::Integer16 { signed: true } => Ok(MsilType::Int16),
        ValkyrieType::Integer16 { signed: false } => Ok(MsilType::UInt16),
        ValkyrieType::Integer32 { signed: true } => Ok(MsilType::Int32),
        ValkyrieType::Integer32 { signed: false } => Ok(MsilType::UInt32),
        ValkyrieType::Integer64 { signed: true } => Ok(MsilType::Int64),
        ValkyrieType::Integer64 { signed: false } => Ok(MsilType::UInt64),
        ValkyrieType::Float32 => Ok(MsilType::Float32),
        ValkyrieType::Float64 => Ok(MsilType::Float64),
        ValkyrieType::Boolean => Ok(MsilType::Bool),
        ValkyrieType::Character => Ok(MsilType::Char),
        ValkyrieType::Utf8 => Ok(MsilType::String),
        ValkyrieType::Utf16 => Ok(MsilType::String),
        // `Unit` 表示无返回值或单位值，在 `CLR` 方法签名中映射为 `void`。
        ValkyrieType::Unit => Ok(MsilType::Void),
        ValkyrieType::Void => Ok(MsilType::Void),
        ValkyrieType::Array(inner) => {
            let inner = map_hir_type_to_msil(inner)?;
            Ok(MsilType::sz_array(inner))
        }
        ValkyrieType::Named(name) => {
            ensure_named_type_is_not_unresolved_primitive(name.as_str());
            match name.as_str() {
                "ExitCode" => Ok(MsilType::Int32),
                _ => Ok(MsilType::Object),
            }
        }
        _ => Ok(MsilType::Object),
    }
}

fn extract_attribute_string_arguments(attribute: &HirAttribute) -> Vec<String> {
    attribute.arguments.iter().filter_map(|argument| extract_string_literal(argument.value.as_ref())).collect()
}

fn extract_string_literal(expr: &HirExpr) -> Option<String> {
    let HirExprKind::Literal(HirLiteral::String(literal)) = &expr.kind
    else {
        return None;
    };

    let mut text = String::new();
    for segment in &literal.segments {
        match segment {
            HirStringSegment::Text(value) => text.push_str(value),
            HirStringSegment::Interpolation { .. } => return None,
        }
    }
    Some(text)
}

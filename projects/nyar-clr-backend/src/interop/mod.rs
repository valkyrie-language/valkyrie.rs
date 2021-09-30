//! `CLR` 前端互操作约定。
//!
//! 收口从前端 `AST/HIR` 到 `MSIL` 所需的 `CLR` 专属规则，
//! 避免命令层自行拼接或拆解 `CLR` 约定字符串。

use crate::msil::{MsilMethodRef, MsilMethodSignature, MsilType};
use miette::{miette, Result};
use valkyrie_compiler::hir::{HirExprKind, HirFunction, HirLiteral, HirStringSegment, HirType};

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

fn map_entry_return_type_to_msil(ty: &HirType) -> Result<MsilType> {
    match ty {
        HirType::Void | HirType::Integer32 => map_hir_type_to_msil(ty),
        HirType::Named(name) if name.as_str() == "ExitCode" => map_hir_type_to_msil(ty),
        _ => Ok(MsilType::Int32),
    }
}

/// 将 `HIR` 类型映射为 `MSIL` 类型。
///
/// `utf8` 和 `utf16` 在 `CLR` 中都映射为 `System.String`，
/// 因为 `CLR` 的 `System.String` 始终是 `UTF-16` 编码。
pub fn map_hir_type_to_msil(ty: &HirType) -> Result<MsilType> {
    match ty {
        HirType::Integer32 => Ok(MsilType::Int32),
        HirType::Integer64 => Ok(MsilType::Int64),
        HirType::Float32 => Ok(MsilType::Float32),
        HirType::Float64 => Ok(MsilType::Float64),
        HirType::Boolean => Ok(MsilType::Bool),
        HirType::Utf8 => Ok(MsilType::String),
        HirType::Utf16 => Ok(MsilType::String),
        // `Unit` 表示无返回值或单位值，在 `CLR` 方法签名中映射为 `void`。
        HirType::Unit => Ok(MsilType::Void),
        HirType::Void => Ok(MsilType::Void),
        HirType::Array(inner) => {
            let inner = map_hir_type_to_msil(inner)?;
            Ok(MsilType::sz_array(inner))
        }
        HirType::Named(name) => match name.as_str() {
            "i32" | "int" => Ok(MsilType::Int32),
            "i64" | "long" => Ok(MsilType::Int64),
            "f32" | "float" => Ok(MsilType::Float32),
            "f64" | "double" => Ok(MsilType::Float64),
            "bool" | "boolean" => Ok(MsilType::Bool),
            "u8" | "byte" => Ok(MsilType::UInt8),
            "u16" | "ushort" => Ok(MsilType::UInt16),
            "u32" | "uint" => Ok(MsilType::UInt32),
            "u64" | "ulong" => Ok(MsilType::UInt64),
            "i8" | "sbyte" => Ok(MsilType::Int8),
            "i16" | "short" => Ok(MsilType::Int16),
            "char" => Ok(MsilType::Char),
            "utf8" => Ok(MsilType::String),
            "utf16" => Ok(MsilType::String),
            "string" | "str" | "String" => Err(miette!(
                code = "nyar::clr::interop::legacy_string_type",
                help = "语言内部已移除 `string` / `str`，请在源代码里显式写 `utf8` 或 `utf16`",
                "不再接受旧文本类型名 `{}`",
                name.as_str()
            )),
            "unit" => Ok(MsilType::Void),
            "void" => Ok(MsilType::Void),
            "ExitCode" => Ok(MsilType::Int32),
            _ => Ok(MsilType::Object),
        },
        _ => Ok(MsilType::Object),
    }
}

fn extract_attribute_string_arguments(attribute: &valkyrie_compiler::hir::HirAttribute) -> Vec<String> {
    attribute.arguments.iter().filter_map(|argument| extract_string_literal(argument.value.as_ref())).collect()
}

fn extract_string_literal(expr: &valkyrie_compiler::hir::HirExpr) -> Option<String> {
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

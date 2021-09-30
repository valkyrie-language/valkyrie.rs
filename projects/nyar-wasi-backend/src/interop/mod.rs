//! `WASM` 前端互操作约定。
//!
//! 收口从前端 `HIR` 到 `WASM Import` 段所需的专属规则，
//! 解析 `[wasm_import("module", "field")]` 属性并生成 `LirImport` 声明。

use valkyrie_compiler::{
    hir::{HirExpr, HirExprKind, HirFunction, HirLiteral, HirStringSegment},
    lir::LirImport,
};

/// 从 `HIR` 函数解析 `wasm_import` 属性，生成 `LirImport` 声明。
///
/// 属性格式：`[wasm_import("module_name", "field_name")]`
/// - 第一个参数：导入模块名（如 `"env"`）
/// - 第二个参数：导入字段名（如 `"read_file"`）
///
/// 函数签名（参数类型和返回类型）来自 `HIR` 函数声明本身。
pub fn resolve_wasm_import(hir_function: &HirFunction) -> Option<LirImport> {
    let attribute = hir_function.annotations.iter().find(|attr| attr.name.to_string() == "wasm_import")?;
    let arguments = extract_attribute_string_arguments(attribute);
    if arguments.len() < 2 {
        return None;
    }

    Some(LirImport {
        module: arguments[0].clone(),
        field: arguments[1].clone(),
        symbol: hir_function.name.to_string(),
        param_types: hir_function.params.iter().map(|p| p.ty.clone()).collect(),
        return_type: hir_function.return_type.clone(),
    })
}

/// 从 `HIR` 模块收集所有 `wasm_import` 声明。
pub fn collect_wasm_imports(hir_module: &valkyrie_compiler::hir::HirModule) -> Vec<LirImport> {
    hir_module.functions.iter().filter_map(resolve_wasm_import).collect()
}

/// 从属性参数中提取纯字符串字面量列表。
fn extract_attribute_string_arguments(attribute: &valkyrie_compiler::hir::HirAttribute) -> Vec<String> {
    attribute.arguments.iter().filter_map(|argument| extract_string_literal(argument.value.as_ref())).collect()
}

/// 从 `HIR` 表达式中提取字符串字面量。
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

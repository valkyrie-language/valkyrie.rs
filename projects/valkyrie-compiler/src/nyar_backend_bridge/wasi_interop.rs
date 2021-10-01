//! `WASM` 前端互操作约定。
//!
//! 收口从前端 `HIR` 到 `WASM Import` 段所需的专属规则，
//! 解析 `[wasm_import("module", "field")]` 属性并生成后端自用的导入描述。

use crate::hir::{HirAttribute, HirExpr, HirExprKind, HirFunction, HirLiteral, HirModule, HirStringSegment};

/// `WASM` 导入描述。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WasmImport {
    /// 导入模块名。
    pub module: String,
    /// 导入字段名。
    pub field: String,
    /// 对应的函数符号名。
    pub symbol: String,
}

/// 从 `HIR` 函数解析 `wasm_import` 属性，生成导入描述。
///
/// 属性格式：`[wasm_import("module_name", "field_name")]`
/// - 第一个参数：导入模块名（如 `"env"`）
/// - 第二个参数：导入字段名（如 `"read_file"`）
pub fn resolve_wasm_import(hir_function: &HirFunction) -> Option<WasmImport> {
    let attribute = hir_function.annotations.iter().find(|attr| attr.name.to_string() == "wasm_import")?;
    let arguments = extract_attribute_string_arguments(attribute);
    if arguments.len() < 2 {
        return None;
    }

    Some(WasmImport { module: arguments[0].clone(), field: arguments[1].clone(), symbol: hir_function.name.to_string() })
}

/// 从 `HIR` 模块收集所有 `wasm_import` 声明。
pub fn collect_wasm_imports(hir_module: &HirModule) -> Vec<WasmImport> {
    hir_module.functions.iter().filter_map(resolve_wasm_import).collect()
}

/// 从属性参数中提取纯字符串字面量列表。
fn extract_attribute_string_arguments(attribute: &HirAttribute) -> Vec<String> {
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

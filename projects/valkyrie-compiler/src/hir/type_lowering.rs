//! `AST` 类型表达式到 `HIR` 类型的 lowering 与预检查。

use std::{cell::RefCell, collections::BTreeSet};

use valkyrie_parser::{ast::TypePath as AstTypePath, ParseError, RootStatement, TypeExpression, ValkyrieRoot};
use valkyrie_types::{
    hir::{FunctionType, RowMethodType, RowType, ValkyrieType},
    Identifier,
};

thread_local! {
    static SHADOWED_BUILTIN_TYPE_ALIASES: RefCell<Vec<BTreeSet<String>>> = RefCell::new(Vec::new());
}

/// 管理当前 lowering 过程中的内建类型别名遮蔽作用域。
#[derive(Debug)]
pub(crate) struct BuiltinTypeAliasScope;

impl BuiltinTypeAliasScope {
    /// 进入一次新的根级 lowering 作用域。
    pub(crate) fn enter(root: &ValkyrieRoot) -> Self {
        let aliases = collect_shadowed_builtin_type_aliases(root);
        SHADOWED_BUILTIN_TYPE_ALIASES.with(|stack| {
            stack.borrow_mut().push(aliases);
        });
        Self
    }
}

impl Drop for BuiltinTypeAliasScope {
    fn drop(&mut self) {
        SHADOWED_BUILTIN_TYPE_ALIASES.with(|stack| {
            let _ = stack.borrow_mut().pop();
        });
    }
}

/// 校验前端 `AST` 类型表达式是否满足当前 `HIR` lowering 前提。
pub(crate) fn validate_type_expression(ty: &TypeExpression) -> Result<(), ParseError> {
    match ty {
        TypeExpression::Path(path) => {
            if let Some(name) = path.name.parts.last() {
                if is_legacy_text_type_name(name) {
                    return Err(ParseError::invalid(format!("legacy text type `{name}` has been removed; use `utf8` or `utf16` explicitly")));
                }
                if let Some(canonical_name) = legacy_builtin_type_alias(name) {
                    return Err(ParseError::invalid(format!(
                        "legacy builtin type alias `{name}` has been removed; use `{canonical_name}` explicitly"
                    )));
                }
            }
            for argument in &path.arguments {
                validate_type_expression(argument)?;
            }
        }
        TypeExpression::Array { item, .. } => validate_type_expression(item)?,
        TypeExpression::Tuple { items, .. } => {
            for item in items {
                validate_type_expression(item)?;
            }
        }
        TypeExpression::Pointer { item, .. } => validate_type_expression(item)?,
        TypeExpression::Row { methods, .. } => {
            for method in methods {
                for param in &method.params {
                    validate_type_expression(param)?;
                }
                validate_type_expression(&method.return_type)?;
            }
        }
        TypeExpression::Associated { .. } | TypeExpression::Nullable { .. } | TypeExpression::Function { .. } => {}
    }
    Ok(())
}

/// 将 `AST` 类型表达式降到最小 `HIR` 类型表示。
pub(crate) fn lower_type_expression(ty: &TypeExpression) -> ValkyrieType {
    match ty {
        TypeExpression::Path(path) => lower_type_path(path),
        TypeExpression::Array { item, .. } => ValkyrieType::Array(Box::new(lower_type_expression(item))),
        TypeExpression::Tuple { items, .. } => {
            if items.is_empty() {
                ValkyrieType::Unit
            }
            else {
                ValkyrieType::Tuple(items.iter().map(lower_type_expression).collect())
            }
        }
        TypeExpression::Pointer { item, .. } => lower_type_expression(item),
        TypeExpression::Row { methods, .. } => ValkyrieType::Row(RowType {
            methods: methods
                .iter()
                .map(|method| RowMethodType {
                    name: method.name.name.clone(),
                    params: method.params.iter().map(lower_type_expression).collect(),
                    return_type: lower_type_expression(&method.return_type),
                })
                .collect(),
        }),
        TypeExpression::Associated { ty, .. } => lower_type_expression(ty),
        TypeExpression::Nullable { item, .. } => lower_type_expression(item),
        TypeExpression::Function { params, return_type, .. } => ValkyrieType::Function(Box::new(FunctionType {
            params: params.iter().map(lower_type_expression).collect(),
            return_type: lower_type_expression(return_type),
        })),
    }
}

/// 渲染类型表达式，供错误消息与回退路径使用。
pub(crate) fn render_type_expression(ty: &TypeExpression) -> String {
    match ty {
        TypeExpression::Path(path) => {
            let base = path.name.parts.join("::");
            if path.arguments.is_empty() {
                base
            }
            else {
                let args = path.arguments.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
                format!("{base}<{args}>")
            }
        }
        TypeExpression::Array { item, .. } => format!("[{}]", render_type_expression(item)),
        TypeExpression::Tuple { items, .. } => {
            if items.is_empty() {
                return "()".to_string();
            }
            let inner = items.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
            format!("({inner})")
        }
        TypeExpression::Pointer { kind, item, .. } => {
            let prefix = match kind {
                valkyrie_parser::ast::PointerKind::ReadOnly => "◇",
                valkyrie_parser::ast::PointerKind::Mutable => "◆",
            };
            format!("{prefix}{}", render_type_expression(item))
        }
        TypeExpression::Row { methods, .. } => {
            let inner = methods
                .iter()
                .map(|method| {
                    let params = method.params.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
                    format!("{}({params}) -> {}", method.name, render_type_expression(&method.return_type))
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!("{{ {inner} }}")
        }
        TypeExpression::Associated { name, ty, .. } => format!("{name}={}", render_type_expression(ty)),
        TypeExpression::Nullable { item, .. } => format!("{}?", render_type_expression(item)),
        TypeExpression::Function { params, return_type, .. } => {
            let params_str = params.iter().map(render_type_expression).collect::<Vec<_>>().join(", ");
            format!("micro({params_str}) -> {}", render_type_expression(return_type))
        }
    }
}

fn is_legacy_text_type_name(name: &str) -> bool {
    matches!(name, "string" | "str" | "String")
}

fn legacy_builtin_type_alias(name: &str) -> Option<&'static str> {
    match name {
        "sbyte" => Some("i8"),
        "short" => Some("i16"),
        "int" => Some("i32"),
        "long" => Some("i64"),
        "byte" => Some("u8"),
        "ushort" => Some("u16"),
        "uint" => Some("u32"),
        "ulong" => Some("u64"),
        "float" => Some("f32"),
        "double" => Some("f64"),
        "boolean" => Some("bool"),
        _ => None,
    }
}

fn canonical_builtin_type(name: &str) -> Option<ValkyrieType> {
    match name {
        "i8" => Some(ValkyrieType::Integer8 { signed: true }),
        "i16" => Some(ValkyrieType::Integer16 { signed: true }),
        "i32" => Some(ValkyrieType::Integer32 { signed: true }),
        "i64" => Some(ValkyrieType::Integer64 { signed: true }),
        "u8" => Some(ValkyrieType::Integer8 { signed: false }),
        "u16" => Some(ValkyrieType::Integer16 { signed: false }),
        "u32" => Some(ValkyrieType::Integer32 { signed: false }),
        "u64" => Some(ValkyrieType::Integer64 { signed: false }),
        "f32" => Some(ValkyrieType::Float32),
        "f64" => Some(ValkyrieType::Float64),
        "bool" => Some(ValkyrieType::Boolean),
        "char" => Some(ValkyrieType::Character),
        "utf8" => Some(ValkyrieType::Utf8),
        "utf16" => Some(ValkyrieType::Utf16),
        "unit" => Some(ValkyrieType::Unit),
        "void" => Some(ValkyrieType::Void),
        _ => None,
    }
}

fn collect_shadowed_builtin_type_aliases(root: &ValkyrieRoot) -> BTreeSet<String> {
    root.statements
        .iter()
        .filter_map(|statement| match statement {
            RootStatement::TypeAlias(alias) if canonical_builtin_type(alias.name.name.as_str()).is_some() => {
                Some(alias.name.name.as_str().to_string())
            }
            _ => None,
        })
        .collect()
}

fn is_shadowed_builtin_type(name: &str) -> bool {
    SHADOWED_BUILTIN_TYPE_ALIASES.with(|stack| stack.borrow().last().is_some_and(|aliases| aliases.contains(name)))
}

fn lower_type_path(path: &AstTypePath) -> ValkyrieType {
    let last = path.name.parts.last().cloned().unwrap_or_default();
    let base = if !is_shadowed_builtin_type(last.as_str())
        && (path.name.parts.len() == 1 || is_known_builtin_type_namespace(&path.name.parts, last.as_str()))
    {
        canonical_builtin_type(last.as_str()).unwrap_or_else(|| ValkyrieType::Named(Identifier::new(&last)))
    }
    else {
        ValkyrieType::Named(Identifier::new(&last))
    };
    if path.arguments.is_empty() {
        base
    }
    else {
        ValkyrieType::Apply(Box::new(base), path.arguments.iter().map(lower_type_expression).collect())
    }
}

fn is_known_builtin_type_namespace(parts: &[String], last: &str) -> bool {
    let prefix = &parts[..parts.len().saturating_sub(1)];
    match prefix {
        [core, primitive] if core == "core" && primitive == "primitive" => canonical_builtin_type(last).is_some(),
        [core, text] if core == "core" && text == "text" => matches!(last, "char" | "utf8" | "utf16"),
        [std, text] if std == "std" && text == "text" => matches!(last, "utf8" | "utf16"),
        _ => false,
    }
}

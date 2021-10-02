use std::collections::BTreeMap;

use valkyrie_types::{
    hir::{HirExpr, HirExprKind},
    Identifier, NamePath,
};

use super::{MirBuilder, MirConstant, MirOperand, MirValueRef, ValkyrieType};

impl MirBuilder {
    pub(super) fn resolve_static_expr(&self, expr: &HirExpr) -> Option<HirExpr> {
        match &expr.kind {
            HirExprKind::Variable(identifier) => self.static_bindings.get(identifier.name.as_str()).cloned().or_else(|| Some(expr.clone())),
            _ => Some(expr.clone()),
        }
    }

    pub(super) fn resolve_static_iterable_items(&self, expr: &HirExpr) -> Option<Vec<HirExpr>> {
        let resolved = self.resolve_static_expr(expr)?;
        match resolved.kind {
            HirExprKind::ArrayLiteral { items } => Some(items),
            HirExprKind::Call { callee, args, .. } if callee_name_matches(&callee.kind, "array") => Some(args),
            HirExprKind::Call { callee, args, resolved: call_resolved } if callee_name_matches(&callee.kind, "tuple") => {
                Some(vec![HirExpr { kind: HirExprKind::Call { callee, args, resolved: call_resolved }, span: resolved.span }])
            }
            _ => None,
        }
    }

    /// 检测 callee 是否为实例方法调用，返回 `(接收者操作数, 方法名)`。
    ///
    /// 处理两种 callee 形式：
    /// 1. `FieldAccess { object, field }`：当 `object` 的根变量在 `bindings` 中时，
    ///    将 `object` 作为接收者，`field` 作为方法名。
    /// 2. `Path([first, second, ...])`：当 `first` 在 `bindings` 中时，
    ///    将 `first` 对应的绑定值作为接收者，`second` 作为方法名。
    ///    仅处理 2 段路径；多段路径（如 `obj.field.method`）需要字段访问支持，暂不处理。
    pub(super) fn extract_method_call(&mut self, callee: &HirExpr) -> Option<(MirOperand, Identifier)> {
        match &callee.kind {
            HirExprKind::FieldAccess { object, field } => {
                let root_name = root_variable_name(object)?;
                if !self.bindings.contains_key(root_name) {
                    return None;
                }
                let receiver_operand = self.lower_expr_to_operand(object);
                Some((receiver_operand, field.clone()))
            }
            HirExprKind::Path(path) if path.parts().len() == 2 => {
                let receiver_name = &path.parts()[0];
                if !self.bindings.contains_key(receiver_name.as_str()) {
                    return None;
                }
                let receiver_operand = self.bindings.get(receiver_name.as_str()).cloned()?;
                let method_name = path.parts()[1].clone();
                Some((receiver_operand, method_name))
            }
            _ => None,
        }
    }
}

pub(super) fn infer_builder_operand_type(operand: &MirOperand, value_types: &BTreeMap<MirValueRef, ValkyrieType>) -> Option<ValkyrieType> {
    match operand {
        MirOperand::Value(value_ref) => value_types.get(value_ref).cloned(),
        MirOperand::Constant(constant) => infer_builder_constant_type(constant),
        MirOperand::Symbol(_) => None,
    }
}

pub(super) fn named_type_name(ty: &ValkyrieType) -> Option<&str> {
    match ty {
        ValkyrieType::Named(name) => Some(name.as_str()),
        ValkyrieType::Apply(base, _) => named_type_name(base),
        _ => None,
    }
}

pub(super) fn future_resume_type(ty: &ValkyrieType) -> Option<ValkyrieType> {
    match ty {
        ValkyrieType::Apply(base, arguments) if arguments.len() == 1 && matches!(named_type_name(base), Some("Future" | "Promise")) => {
            arguments.first().cloned()
        }
        _ => None,
    }
}

/// 判断 callee 名称是否匹配预期。
///
/// 同时支持 `HirExprKind::Path`（多部分路径）和 `HirExprKind::Variable`（单部分名称），
/// 因为 `lower_name_expression` 会将单部分名称降级为 `Variable`。
pub(super) fn callee_name_matches(kind: &HirExprKind, expected: &str) -> bool {
    match kind {
        HirExprKind::Path(path) if path.to_string() == expected => true,
        HirExprKind::Variable(ident) if ident.name.as_str() == expected => true,
        _ => false,
    }
}

pub(super) fn lower_callee_operand(
    expr: &HirExpr,
    resolved: Option<&valkyrie_types::hir::HirResolvedCall>,
    builder: &mut MirBuilder,
) -> MirOperand {
    if let Some(resolved) = resolved {
        return MirOperand::Symbol(resolved.symbol.clone());
    }
    match &expr.kind {
        HirExprKind::Path(path) => MirOperand::Symbol(path.clone()),
        HirExprKind::Variable(identifier) => builder
            .bindings
            .get(identifier.name.as_str())
            .cloned()
            .unwrap_or_else(|| MirOperand::Symbol(NamePath::new(vec![identifier.name.clone()]))),
        HirExprKind::GenericApply { callee, .. } => lower_callee_operand(callee, None, builder),
        HirExprKind::FieldAccess { .. } => match try_resolve_as_path(expr) {
            Some(path) => MirOperand::Symbol(path),
            None => builder.lower_expr_to_operand(expr),
        },
        _ => builder.lower_expr_to_operand(expr),
    }
}

/// 尝试将表达式解析为 `NamePath`。
///
/// 递归处理 `Variable` 和 `FieldAccess` 链，将其组合为完整路径。
/// 例如 `std.iterator.collect_array` 会被解析为 `NamePath(["std", "iterator", "collect_array"])`。
/// 若表达式不是纯路径形式（如对象是复杂表达式），返回 `None`。
pub(super) fn try_resolve_as_path(expr: &HirExpr) -> Option<NamePath> {
    match &expr.kind {
        HirExprKind::Variable(identifier) => Some(NamePath::new(vec![identifier.name.clone()])),
        HirExprKind::Path(path) => Some(path.clone()),
        HirExprKind::FieldAccess { object, field } => {
            let mut path = try_resolve_as_path(object)?;
            path.append(field.clone());
            Some(path)
        }
        _ => None,
    }
}

/// 递归提取 `FieldAccess` 链的根变量名。
///
/// 例如 `request.target.length` 的根变量是 `request`。
/// 若表达式不是 `Variable` 或 `FieldAccess` 链，返回 `None`。
/// 用于区分实例方法调用（`obj.method()`）和静态路径调用（`module.function()`）。
pub(super) fn root_variable_name(expr: &HirExpr) -> Option<&str> {
    match &expr.kind {
        HirExprKind::Variable(identifier) => Some(identifier.name.as_str()),
        HirExprKind::FieldAccess { object, .. } => root_variable_name(object),
        _ => None,
    }
}

fn infer_builder_constant_type(constant: &MirConstant) -> Option<ValkyrieType> {
    Some(match constant {
        MirConstant::Int(value) if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => ValkyrieType::Integer32 { signed: true },
        MirConstant::Int(_) => ValkyrieType::Integer64 { signed: true },
        MirConstant::Float64(_) => ValkyrieType::Float64,
        MirConstant::Bool(_) => ValkyrieType::Boolean,
        MirConstant::String(_) => ValkyrieType::Utf8,
        MirConstant::Unit => ValkyrieType::Unit,
    })
}

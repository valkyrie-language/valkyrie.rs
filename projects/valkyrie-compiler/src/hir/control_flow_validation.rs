use std::collections::{BTreeMap, BTreeSet};

use valkyrie_parser::ParseError;
use valkyrie_types::hir::{
    HirBlock, HirExpr, HirExprKind, HirFunction, HirLiteral, HirMatchArm, HirModule, HirPattern, HirStatement, HirStatementKind, ValkyrieType,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct HirValidationState {
    catch_depth: usize,
    in_catch_arm_body: bool,
    in_case_arm_body: bool,
    has_case_fallthrough_target: bool,
    in_guard: bool,
    allow_blocking: bool,
    allow_yield: bool,
    loop_stack: Vec<LoopValidationContext>,
    return_type_stack: Vec<ValkyrieType>,
    local_scopes: Vec<BTreeMap<String, ValkyrieType>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopValidationContext {
    label: Option<String>,
    accepts_break_value: bool,
    break_value_type: Option<ValkyrieType>,
}

pub fn validate_control_flow_module(module: &HirModule) -> Result<(), ParseError> {
    for function in &module.functions {
        let mut state = HirValidationState::default();
        validate_hir_function(function, &mut state)?;
    }
    Ok(())
}

fn validate_hir_function(function: &HirFunction, state: &mut HirValidationState) -> Result<(), ParseError> {
    let saved_allow_blocking = state.allow_blocking;
    let saved_allow_yield = state.allow_yield;
    state.allow_blocking = true;
    state.allow_yield = true;
    push_return_type(state, function.return_type.clone());
    push_local_scope(state);
    for param in &function.params {
        bind_local_type(state, param.name.name.as_str(), param.ty.clone());
    }
    for statement in &function.body.statements {
        validate_hir_statement(statement, state)?;
    }
    if let Some(expr) = &function.body.expr {
        validate_hir_expr(expr, state, true)?;
    }
    pop_local_scope(state);
    pop_return_type(state);
    state.allow_blocking = saved_allow_blocking;
    state.allow_yield = saved_allow_yield;
    Ok(())
}

fn validate_hir_statement(statement: &HirStatement, state: &mut HirValidationState) -> Result<(), ParseError> {
    match &statement.kind {
        HirStatementKind::Let { pattern, initializer, ty, .. } => {
            if let Some(initializer) = initializer {
                validate_hir_expr(initializer, state, true)?;
            }
            let inferred_type = ty.clone().or_else(|| initializer.as_ref().and_then(|value| infer_static_expr_type(value, state)));
            if let Some(inferred_type) = inferred_type {
                bind_pattern_type(state, pattern, &inferred_type);
            }
        }
        HirStatementKind::Expr(expr) => validate_hir_expr(expr, state, false)?,
    }
    Ok(())
}

fn validate_hir_block(block: &HirBlock, state: &mut HirValidationState, value_context: bool) -> Result<(), ParseError> {
    push_local_scope(state);
    for statement in &block.statements {
        validate_hir_statement(statement, state)?;
    }
    if let Some(expr) = &block.expr {
        validate_hir_expr(expr, state, value_context)?;
    }
    pop_local_scope(state);
    Ok(())
}

fn validate_hir_expr(expr: &HirExpr, state: &mut HirValidationState, value_context: bool) -> Result<(), ParseError> {
    match &expr.kind {
        HirExprKind::Call { callee, args, .. } => {
            validate_hir_expr(callee, state, true)?;
            for arg in args {
                validate_hir_expr(arg, state, true)?;
            }
        }
        HirExprKind::FieldInit { value, .. }
        | HirExprKind::Await(value)
        | HirExprKind::Awake(value)
        | HirExprKind::BlockOn(value)
        | HirExprKind::YieldFrom(value)
        | HirExprKind::Raise(value)
        | HirExprKind::Resume(value) => {
            if matches!(expr.kind, HirExprKind::Resume(_)) && (state.catch_depth == 0 || !state.in_catch_arm_body) {
                return Err(ParseError::invalid("控制流调度校验失败：检测到未位于 `catch arm body` 的 `resume`"));
            }
            if state.in_guard {
                let control_flow_name = match &expr.kind {
                    HirExprKind::Await(_) => "`await`",
                    HirExprKind::Awake(_) => "`awake`",
                    HirExprKind::BlockOn(_) => "`block`",
                    HirExprKind::YieldFrom(_) => "`yield from`",
                    HirExprKind::Resume(_) => "`resume`",
                    _ => "",
                };
                if !control_flow_name.is_empty() {
                    return Err(ParseError::invalid(format!("控制流调度校验失败：guard 中不允许出现会打断控制流连续性的 {control_flow_name}")));
                }
            }
            if matches!(expr.kind, HirExprKind::BlockOn(_)) {
                validate_blocking_context(expr, state)?;
            }
            if matches!(expr.kind, HirExprKind::YieldFrom(_)) {
                validate_yield_context(expr, state)?;
            }
            validate_hir_expr(value, state, true)?;
            if matches!(expr.kind, HirExprKind::Await(_) | HirExprKind::Awake(_) | HirExprKind::BlockOn(_)) {
                validate_future_control_operand(expr, value, state)?;
            }
        }
        HirExprKind::Construct { args, .. } | HirExprKind::ArrayLiteral { items: args } => {
            for arg in args {
                validate_hir_expr(arg, state, true)?;
            }
        }
        HirExprKind::Fallthrough => {
            if state.in_case_arm_body && state.has_case_fallthrough_target && !state.in_catch_arm_body {
                return Ok(());
            }
            if state.in_case_arm_body && !state.has_case_fallthrough_target {
                return Err(ParseError::invalid("控制流调度校验失败：最后一个 `case` arm 不允许使用 `fallthrough`"));
            }
            return Err(ParseError::invalid("控制流调度校验失败：`fallthrough` 仅允许出现在 `case` statement 体系中"));
        }
        HirExprKind::With { base, updates } => {
            validate_hir_expr(base, state, true)?;
            for (_, value) in updates {
                validate_hir_expr(value, state, true)?;
            }
        }
        HirExprKind::SuperCall { args, .. } => {
            for arg in args {
                validate_hir_expr(arg, state, true)?;
            }
        }
        HirExprKind::ArrayNew { length, .. } => validate_hir_expr(length, state, true)?,
        HirExprKind::FieldAccess { object, .. } => validate_hir_expr(object, state, true)?,
        HirExprKind::StoreField { object, value, .. } => {
            validate_hir_expr(object, state, true)?;
            validate_hir_expr(value, state, true)?;
        }
        HirExprKind::GenericApply { callee, .. } => validate_hir_expr(callee, state, true)?,
        HirExprKind::Block(block) => validate_hir_block(block, state, value_context)?,
        HirExprKind::Lambda { params, body, .. } => {
            let mut lambda_state = HirValidationState::default();
            if let HirExprKind::Lambda { return_type, .. } = &expr.kind {
                push_return_type(&mut lambda_state, return_type.clone());
            }
            push_local_scope(&mut lambda_state);
            for param in params {
                bind_local_type(&mut lambda_state, param.name.name.as_str(), param.ty.clone());
            }
            validate_hir_block(body, &mut lambda_state, true)?;
            pop_local_scope(&mut lambda_state);
            pop_return_type(&mut lambda_state);
        }
        HirExprKind::AnonymousClass { fields, methods, .. } => {
            for (_, value) in fields {
                validate_hir_expr(value, state, true)?;
            }
            for method in methods {
                validate_hir_function(method, &mut HirValidationState::default())?;
            }
        }
        HirExprKind::If { condition, then_branch, else_branch } => {
            validate_hir_expr(condition, state, true)?;
            validate_hir_block(then_branch, state, value_context)?;
            if let Some(else_branch) = else_branch {
                validate_hir_block(else_branch, state, value_context)?;
            }
        }
        HirExprKind::Match { scrutinee, arms } => {
            validate_hir_expr(scrutinee, state, true)?;
            let scrutinee_type = infer_static_expr_type(scrutinee, state);
            for arm in arms {
                push_local_scope(state);
                if let Some(scrutinee_type) = &scrutinee_type {
                    bind_pattern_type(state, &arm.pattern, scrutinee_type);
                }
                let saved_in_guard = state.in_guard;
                state.in_guard = true;
                if let Some(guard) = &arm.guard {
                    validate_hir_expr(guard, state, true)?;
                }
                state.in_guard = saved_in_guard;
                validate_hir_expr(&arm.body, state, value_context)?;
                pop_local_scope(state);
            }
        }
        HirExprKind::Case { scrutinee, arms } => {
            validate_hir_expr(scrutinee, state, true)?;
            let scrutinee_type = infer_static_expr_type(scrutinee, state);
            let mut previous_fallthrough_bindings = BTreeSet::new();
            for (index, arm) in arms.iter().enumerate() {
                if let Some(leaked_name) = detect_case_fallthrough_binding_leak(arm, &previous_fallthrough_bindings, state) {
                    return Err(ParseError::invalid(format!(
                        "控制流调度校验失败：`fallthrough` 不继承上一 `case` arm 的 pattern 绑定，当前 arm 非法引用了 `{leaked_name}`"
                    )));
                }
                push_local_scope(state);
                if let Some(scrutinee_type) = &scrutinee_type {
                    bind_pattern_type(state, &arm.pattern, scrutinee_type);
                }
                let saved_in_case_arm_body = state.in_case_arm_body;
                let saved_has_case_fallthrough_target = state.has_case_fallthrough_target;
                let saved_in_guard = state.in_guard;
                state.in_guard = true;
                if let Some(guard) = &arm.guard {
                    validate_hir_expr(guard, state, true)?;
                }
                state.in_guard = saved_in_guard;
                state.in_case_arm_body = true;
                state.has_case_fallthrough_target = index + 1 < arms.len();
                validate_hir_expr(&arm.body, state, false)?;
                state.in_case_arm_body = saved_in_case_arm_body;
                state.has_case_fallthrough_target = saved_has_case_fallthrough_target;
                pop_local_scope(state);
                previous_fallthrough_bindings =
                    if expr_contains_fallthrough(&arm.body) { collect_pattern_bound_names(&arm.pattern) } else { BTreeSet::new() };
            }
        }
        HirExprKind::Loop { iterator, condition, body, label, .. } => {
            if let Some(iterator) = iterator {
                validate_hir_expr(iterator, state, true)?;
            }
            if let Some(condition) = condition {
                validate_hir_expr(condition, state, true)?;
            }
            push_loop_context(state, label.as_ref().map(|it| it.as_str()), value_context);
            validate_hir_block(body, state, false)?;
            pop_loop_context(state);
        }
        HirExprKind::Return(value) | HirExprKind::Yield(value) => {
            if matches!(expr.kind, HirExprKind::Yield(_)) && state.in_guard {
                return Err(ParseError::invalid("控制流调度校验失败：guard 中不允许出现会打断控制流连续性的 `yield`"));
            }
            if matches!(expr.kind, HirExprKind::Yield(_)) {
                validate_yield_context(expr, state)?;
            }
            if let Some(value) = value {
                validate_hir_expr(value, state, true)?;
            }
            if matches!(expr.kind, HirExprKind::Return(_)) {
                validate_return_value(expr, value.as_deref(), state)?;
            }
        }
        HirExprKind::Assign { value, .. } => validate_hir_expr(value, state, true)?,
        HirExprKind::Break { label, expr } => {
            if let Some(expr) = expr {
                validate_hir_expr(expr, state, true)?;
                let break_type = infer_static_expr_type(expr, state);
                let target = resolve_break_target_mut(label.as_ref().map(|it| it.as_str()), state);
                if let Some(loop_context) = target {
                    if !loop_context.accepts_break_value {
                        return Err(ParseError::invalid("控制流调度校验失败：`break expr` 的目标 loop 当前不接受值"));
                    }
                    if let Some(break_type) = break_type {
                        if let Some(expected_type) = &loop_context.break_value_type {
                            if !control_flow_value_type_compatible(expected_type, &break_type) {
                                return Err(ParseError::invalid(format!(
                                    "控制流调度校验失败：`break expr` 的值类型 `{}` 与目标 loop 已收敛的结果类型 `{}` 不兼容",
                                    display_type(&break_type),
                                    display_type(expected_type)
                                )));
                            }
                        }
                        else {
                            loop_context.break_value_type = Some(break_type);
                        }
                    }
                }
                else {
                    return Err(ParseError::invalid("控制流调度校验失败：`break expr` 未找到可接受值的目标 loop"));
                }
            }
        }
        HirExprKind::Catch { expr, arms } => {
            validate_hir_expr(expr, state, true)?;
            state.catch_depth += 1;
            for arm in arms {
                let saved_in_catch_arm_body = state.in_catch_arm_body;
                let saved_in_guard = state.in_guard;
                state.in_catch_arm_body = false;
                state.in_guard = true;
                if let Some(guard) = &arm.guard {
                    validate_hir_expr(guard, state, true)?;
                }
                state.in_guard = false;
                state.in_catch_arm_body = true;
                validate_hir_expr(&arm.body, state, value_context)?;
                state.in_catch_arm_body = saved_in_catch_arm_body;
                state.in_guard = saved_in_guard;
            }
            state.catch_depth = state.catch_depth.saturating_sub(1);
        }
        HirExprKind::Literal(_) | HirExprKind::Variable(_) | HirExprKind::Path(_) | HirExprKind::Continue { .. } => {}
    }
    Ok(())
}

fn push_loop_context(state: &mut HirValidationState, label: Option<&str>, value_context: bool) {
    state.loop_stack.push(LoopValidationContext {
        label: label.map(|it| it.to_string()),
        accepts_break_value: value_context,
        break_value_type: None,
    });
}

fn pop_loop_context(state: &mut HirValidationState) {
    let _ = state.loop_stack.pop();
}

fn resolve_break_target_mut<'a>(label: Option<&str>, state: &'a mut HirValidationState) -> Option<&'a mut LoopValidationContext> {
    if let Some(label) = label {
        for loop_context in state.loop_stack.iter_mut().rev() {
            if loop_context.label.as_deref() == Some(label) {
                return Some(loop_context);
            }
        }
        None
    }
    else {
        state.loop_stack.iter_mut().next_back()
    }
}

fn infer_static_expr_type(expr: &HirExpr, state: &HirValidationState) -> Option<ValkyrieType> {
    match &expr.kind {
        HirExprKind::Literal(literal) => infer_literal_type(literal),
        HirExprKind::Variable(identifier) => lookup_local_type(state, identifier.name.as_str()),
        HirExprKind::Block(block) => infer_static_block_type(block, state),
        HirExprKind::If { then_branch, else_branch, .. } => {
            let then_type = infer_static_block_type(then_branch, state)?;
            let else_type = else_branch.as_ref().and_then(|branch| infer_static_block_type(branch, state))?;
            if control_flow_value_type_compatible(&then_type, &else_type) {
                Some(then_type)
            }
            else {
                None
            }
        }
        HirExprKind::Match { scrutinee, arms } => infer_static_match_type(scrutinee, arms, state),
        HirExprKind::Case { .. } => None,
        HirExprKind::Awake(_) | HirExprKind::Yield(_) | HirExprKind::YieldFrom(_) => Some(ValkyrieType::Unit),
        HirExprKind::Await(value) | HirExprKind::BlockOn(value) => {
            let value_type = infer_static_expr_type(value, state)?;
            future_resume_type(&value_type)
        }
        _ => None,
    }
}

fn infer_static_block_type(block: &HirBlock, state: &HirValidationState) -> Option<ValkyrieType> {
    if let Some(expr) = &block.expr {
        infer_static_expr_type(expr, state)
    }
    else {
        Some(ValkyrieType::Unit)
    }
}

fn infer_static_match_type(scrutinee: &HirExpr, arms: &[HirMatchArm], state: &HirValidationState) -> Option<ValkyrieType> {
    let mut inferred_type: Option<ValkyrieType> = None;
    let scrutinee_type = infer_static_expr_type(scrutinee, state);
    for arm in arms {
        let mut arm_state = state.clone();
        push_local_scope(&mut arm_state);
        if let Some(scrutinee_type) = &scrutinee_type {
            bind_pattern_type(&mut arm_state, &arm.pattern, scrutinee_type);
        }
        let arm_type = infer_static_expr_type(&arm.body, &arm_state)?;
        if let Some(current_type) = &inferred_type {
            if !control_flow_value_type_compatible(current_type, &arm_type) {
                return None;
            }
        }
        else {
            inferred_type = Some(arm_type);
        }
    }
    inferred_type
}

fn infer_literal_type(literal: &HirLiteral) -> Option<ValkyrieType> {
    match literal {
        HirLiteral::Integer64(_) => Some(ValkyrieType::Integer64 { signed: true }),
        HirLiteral::Float64(_) => Some(ValkyrieType::Float64),
        HirLiteral::Bool(_) => Some(ValkyrieType::Boolean),
        HirLiteral::Unit => Some(ValkyrieType::Unit),
        HirLiteral::String(_) => None,
    }
}

fn validate_future_control_operand(expr: &HirExpr, value: &HirExpr, state: &HirValidationState) -> Result<(), ParseError> {
    let Some(value_type) = infer_static_expr_type(value, state)
    else {
        return Ok(());
    };
    if future_resume_type(&value_type).is_some() {
        return Ok(());
    }
    let control_flow_name = match expr.kind {
        HirExprKind::Await(_) => "`await`",
        HirExprKind::Awake(_) => "`awake`",
        HirExprKind::BlockOn(_) => "`block`",
        _ => return Ok(()),
    };
    Err(ParseError::invalid(format!(
        "控制流调度校验失败：{control_flow_name} 需要 `Future<T>` 或 `Promise<T>` 类型的操作数，当前得到 `{}`",
        display_type(&value_type)
    )))
}

fn validate_return_value(expr: &HirExpr, value: Option<&HirExpr>, state: &HirValidationState) -> Result<(), ParseError> {
    let Some(expected_type) = current_return_type(state)
    else {
        return Ok(());
    };

    let Some(value) = value
    else {
        if return_without_value_compatible(expected_type) {
            return Ok(());
        }
        return Err(ParseError::invalid(format!("控制流调度校验失败：函数返回类型 `{}` 不接受无值 `return`", display_type(expected_type))));
    };

    let Some(actual_type) = infer_static_expr_type(value, state)
    else {
        return Ok(());
    };
    if control_flow_value_type_compatible(expected_type, &actual_type) {
        return Ok(());
    }

    let _ = expr;
    Err(ParseError::invalid(format!(
        "控制流调度校验失败：`return expr` 的值类型 `{}` 与函数返回类型 `{}` 不兼容",
        display_type(&actual_type),
        display_type(expected_type)
    )))
}

fn validate_blocking_context(expr: &HirExpr, state: &HirValidationState) -> Result<(), ParseError> {
    if !matches!(expr.kind, HirExprKind::BlockOn(_)) || state.allow_blocking {
        return Ok(());
    }
    Err(ParseError::invalid("控制流调度校验失败：`block` 当前不位于允许阻塞的上下文"))
}

fn validate_yield_context(expr: &HirExpr, state: &HirValidationState) -> Result<(), ParseError> {
    let control_flow_name = match expr.kind {
        HirExprKind::Yield(_) => "`yield`",
        HirExprKind::YieldFrom(_) => "`yield from`",
        _ => {
            return Ok(());
        }
    };
    if state.allow_yield {
        return Ok(());
    }
    Err(ParseError::invalid(format!("控制流调度校验失败：{control_flow_name} 只允许出现在生成器函数或等价 handler 上下文")))
}

fn push_local_scope(state: &mut HirValidationState) {
    state.local_scopes.push(BTreeMap::new());
}

fn pop_local_scope(state: &mut HirValidationState) {
    let _ = state.local_scopes.pop();
}

fn push_return_type(state: &mut HirValidationState, ty: ValkyrieType) {
    state.return_type_stack.push(ty);
}

fn pop_return_type(state: &mut HirValidationState) {
    let _ = state.return_type_stack.pop();
}

fn current_return_type(state: &HirValidationState) -> Option<&ValkyrieType> {
    state.return_type_stack.last()
}

fn bind_local_type(state: &mut HirValidationState, name: &str, ty: ValkyrieType) {
    if let Some(scope) = state.local_scopes.last_mut() {
        scope.insert(name.to_string(), ty);
    }
}

fn bind_pattern_type(state: &mut HirValidationState, pattern: &HirPattern, ty: &ValkyrieType) {
    bind_pattern_type_with_hint(state, pattern, Some(ty));
}

fn bind_pattern_type_with_hint(state: &mut HirValidationState, pattern: &HirPattern, ty: Option<&ValkyrieType>) {
    match pattern {
        HirPattern::Variable(identifier) => {
            bind_local_type(state, identifier.name.as_str(), ty.cloned().unwrap_or(ValkyrieType::AutoType));
        }
        HirPattern::TypedBind { identifier, ty: pattern_type } => {
            let explicit = pattern_type.parts().last().cloned().map(ValkyrieType::Named);
            bind_local_type(state, identifier.name.as_str(), explicit.or_else(|| ty.cloned()).unwrap_or(ValkyrieType::AutoType));
        }
        HirPattern::Tuple(items) => {
            if let Some(ValkyrieType::Tuple(types)) = ty {
                for (item, item_type) in items.iter().zip(types.iter()) {
                    bind_pattern_type_with_hint(state, item, Some(item_type));
                }
            }
            for item in items.iter().skip(match ty {
                Some(ValkyrieType::Tuple(types)) => types.len(),
                _ => 0,
            }) {
                bind_pattern_type_with_hint(state, item, None);
            }
        }
        HirPattern::Extractor(extractor) => match extractor {
            valkyrie_types::hir::HirExtractorPattern::Array { prefix, rest, suffix, .. } => {
                let item_hint = match ty {
                    Some(ValkyrieType::Array(item)) => Some(item.as_ref()),
                    _ => None,
                };
                for item in prefix {
                    bind_pattern_type_with_hint(state, item, item_hint);
                }
                if let Some(rest) = rest {
                    bind_local_type(
                        state,
                        rest.name.as_str(),
                        item_hint.cloned().map(|item| ValkyrieType::Array(Box::new(item))).unwrap_or(ValkyrieType::AutoType),
                    );
                }
                for item in suffix {
                    bind_pattern_type_with_hint(state, item, item_hint);
                }
            }
            valkyrie_types::hir::HirExtractorPattern::Constructor { fields, .. } => {
                for field in fields {
                    bind_pattern_type_with_hint(state, field, None);
                }
            }
        },
        HirPattern::Object { fields, rest, .. } => {
            for (_, field_pattern) in fields {
                bind_pattern_type_with_hint(state, field_pattern, None);
            }
            if let Some(rest) = rest {
                bind_local_type(state, rest.name.as_str(), ty.cloned().unwrap_or(ValkyrieType::AutoType));
            }
        }
        HirPattern::Or(patterns) => {
            for pattern in patterns {
                bind_pattern_type_with_hint(state, pattern, ty);
            }
        }
        _ => {}
    }
}

fn collect_pattern_bound_names(pattern: &HirPattern) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_pattern_bound_names_into(pattern, &mut names);
    names
}

fn collect_pattern_bound_names_into(pattern: &HirPattern, names: &mut BTreeSet<String>) {
    match pattern {
        HirPattern::Variable(identifier) => {
            names.insert(identifier.name.to_string());
        }
        HirPattern::TypedBind { identifier, .. } => {
            names.insert(identifier.name.to_string());
        }
        HirPattern::Tuple(items) => {
            for item in items {
                collect_pattern_bound_names_into(item, names);
            }
        }
        HirPattern::Extractor(extractor) => match extractor {
            valkyrie_types::hir::HirExtractorPattern::Array { prefix, rest, suffix, .. } => {
                for item in prefix {
                    collect_pattern_bound_names_into(item, names);
                }
                if let Some(rest) = rest {
                    names.insert(rest.name.to_string());
                }
                for item in suffix {
                    collect_pattern_bound_names_into(item, names);
                }
            }
            valkyrie_types::hir::HirExtractorPattern::Constructor { fields, .. } => {
                for field in fields {
                    collect_pattern_bound_names_into(field, names);
                }
            }
        },
        HirPattern::Object { fields, rest, .. } => {
            for (_, field_pattern) in fields {
                collect_pattern_bound_names_into(field_pattern, names);
            }
            if let Some(rest) = rest {
                names.insert(rest.name.to_string());
            }
        }
        HirPattern::Or(patterns) => {
            for pattern in patterns {
                collect_pattern_bound_names_into(pattern, names);
            }
        }
        _ => {}
    }
}

fn detect_case_fallthrough_binding_leak(
    arm: &HirMatchArm,
    previous_fallthrough_bindings: &BTreeSet<String>,
    state: &HirValidationState,
) -> Option<String> {
    if previous_fallthrough_bindings.is_empty() {
        return None;
    }

    let current_bindings = collect_pattern_bound_names(&arm.pattern);
    let mut referenced = BTreeSet::new();
    if let Some(guard) = &arm.guard {
        collect_expr_variable_names(guard, &mut referenced);
    }
    collect_expr_variable_names(&arm.body, &mut referenced);

    referenced.into_iter().find(|name| {
        previous_fallthrough_bindings.contains(name) && !current_bindings.contains(name) && lookup_local_type(state, name).is_none()
    })
}

fn collect_expr_variable_names(expr: &HirExpr, names: &mut BTreeSet<String>) {
    match &expr.kind {
        HirExprKind::Variable(identifier) => {
            names.insert(identifier.name.to_string());
        }
        HirExprKind::Call { callee, args, .. } => {
            collect_expr_variable_names(callee, names);
            for arg in args {
                collect_expr_variable_names(arg, names);
            }
        }
        HirExprKind::Construct { args, .. } | HirExprKind::ArrayLiteral { items: args } => {
            for arg in args {
                collect_expr_variable_names(arg, names);
            }
        }
        HirExprKind::ArrayNew { length, .. }
        | HirExprKind::FieldInit { value: length, .. }
        | HirExprKind::FieldAccess { object: length, .. }
        | HirExprKind::Await(length)
        | HirExprKind::Awake(length)
        | HirExprKind::BlockOn(length)
        | HirExprKind::Yield(Some(length))
        | HirExprKind::YieldFrom(length)
        | HirExprKind::Raise(length)
        | HirExprKind::Resume(length)
        | HirExprKind::Assign { value: length, .. }
        | HirExprKind::GenericApply { callee: length, .. } => collect_expr_variable_names(length, names),
        HirExprKind::Yield(None) | HirExprKind::Fallthrough | HirExprKind::Literal(_) | HirExprKind::Path(_) | HirExprKind::Continue { .. } => {
        }
        HirExprKind::With { base, updates } => {
            collect_expr_variable_names(base, names);
            for (_, value) in updates {
                collect_expr_variable_names(value, names);
            }
        }
        HirExprKind::SuperCall { args, .. } => {
            for arg in args {
                collect_expr_variable_names(arg, names);
            }
        }
        HirExprKind::StoreField { object, value, .. } => {
            collect_expr_variable_names(object, names);
            collect_expr_variable_names(value, names);
        }
        HirExprKind::Block(block) => {
            for statement in &block.statements {
                collect_statement_variable_names(statement, names);
            }
            if let Some(expr) = &block.expr {
                collect_expr_variable_names(expr, names);
            }
        }
        HirExprKind::Lambda { body, .. } => {
            for statement in &body.statements {
                collect_statement_variable_names(statement, names);
            }
            if let Some(expr) = &body.expr {
                collect_expr_variable_names(expr, names);
            }
        }
        HirExprKind::AnonymousClass { fields, methods, .. } => {
            for (_, value) in fields {
                collect_expr_variable_names(value, names);
            }
            for method in methods {
                for statement in &method.body.statements {
                    collect_statement_variable_names(statement, names);
                }
                if let Some(expr) = &method.body.expr {
                    collect_expr_variable_names(expr, names);
                }
            }
        }
        HirExprKind::If { condition, then_branch, else_branch } => {
            collect_expr_variable_names(condition, names);
            for statement in &then_branch.statements {
                collect_statement_variable_names(statement, names);
            }
            if let Some(expr) = &then_branch.expr {
                collect_expr_variable_names(expr, names);
            }
            if let Some(else_branch) = else_branch {
                for statement in &else_branch.statements {
                    collect_statement_variable_names(statement, names);
                }
                if let Some(expr) = &else_branch.expr {
                    collect_expr_variable_names(expr, names);
                }
            }
        }
        HirExprKind::Match { scrutinee, arms } | HirExprKind::Case { scrutinee, arms } => {
            collect_expr_variable_names(scrutinee, names);
            for arm in arms {
                if let Some(guard) = &arm.guard {
                    collect_expr_variable_names(guard, names);
                }
                collect_expr_variable_names(&arm.body, names);
            }
        }
        HirExprKind::Loop { iterator, condition, body, .. } => {
            if let Some(iterator) = iterator {
                collect_expr_variable_names(iterator, names);
            }
            if let Some(condition) = condition {
                collect_expr_variable_names(condition, names);
            }
            for statement in &body.statements {
                collect_statement_variable_names(statement, names);
            }
            if let Some(expr) = &body.expr {
                collect_expr_variable_names(expr, names);
            }
        }
        HirExprKind::Return(Some(value)) | HirExprKind::Break { expr: Some(value), .. } => collect_expr_variable_names(value, names),
        HirExprKind::Return(None) | HirExprKind::Break { expr: None, .. } | HirExprKind::Catch { .. } => {}
    }
}

fn collect_statement_variable_names(statement: &HirStatement, names: &mut BTreeSet<String>) {
    match &statement.kind {
        HirStatementKind::Let { initializer, .. } => {
            if let Some(initializer) = initializer {
                collect_expr_variable_names(initializer, names);
            }
        }
        HirStatementKind::Expr(expr) => collect_expr_variable_names(expr, names),
    }
}

fn expr_contains_fallthrough(expr: &HirExpr) -> bool {
    match &expr.kind {
        HirExprKind::Fallthrough => true,
        HirExprKind::Call { callee, args, .. } => expr_contains_fallthrough(callee) || args.iter().any(expr_contains_fallthrough),
        HirExprKind::Construct { args, .. } | HirExprKind::ArrayLiteral { items: args } => args.iter().any(expr_contains_fallthrough),
        HirExprKind::ArrayNew { length, .. }
        | HirExprKind::FieldInit { value: length, .. }
        | HirExprKind::FieldAccess { object: length, .. }
        | HirExprKind::Await(length)
        | HirExprKind::Awake(length)
        | HirExprKind::BlockOn(length)
        | HirExprKind::Yield(Some(length))
        | HirExprKind::YieldFrom(length)
        | HirExprKind::Raise(length)
        | HirExprKind::Resume(length)
        | HirExprKind::Assign { value: length, .. }
        | HirExprKind::GenericApply { callee: length, .. } => expr_contains_fallthrough(length),
        HirExprKind::Yield(None) | HirExprKind::Literal(_) | HirExprKind::Variable(_) | HirExprKind::Path(_) | HirExprKind::Continue { .. } => {
            false
        }
        HirExprKind::With { base, updates } => {
            expr_contains_fallthrough(base) || updates.iter().any(|(_, value)| expr_contains_fallthrough(value))
        }
        HirExprKind::SuperCall { args, .. } => args.iter().any(expr_contains_fallthrough),
        HirExprKind::StoreField { object, value, .. } => expr_contains_fallthrough(object) || expr_contains_fallthrough(value),
        HirExprKind::Block(block) => {
            block.statements.iter().any(statement_contains_fallthrough)
                || block.expr.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
        }
        HirExprKind::Lambda { body, .. } => {
            body.statements.iter().any(statement_contains_fallthrough) || body.expr.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
        }
        HirExprKind::AnonymousClass { fields, methods, .. } => {
            fields.iter().any(|(_, value)| expr_contains_fallthrough(value))
                || methods.iter().any(|method| {
                    method.body.statements.iter().any(statement_contains_fallthrough)
                        || method.body.expr.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
                })
        }
        HirExprKind::If { condition, then_branch, else_branch } => {
            expr_contains_fallthrough(condition)
                || then_branch.statements.iter().any(statement_contains_fallthrough)
                || then_branch.expr.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
                || else_branch.as_ref().is_some_and(|branch| {
                    branch.statements.iter().any(statement_contains_fallthrough)
                        || branch.expr.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
                })
        }
        HirExprKind::Match { scrutinee, arms } | HirExprKind::Case { scrutinee, arms } => {
            expr_contains_fallthrough(scrutinee)
                || arms
                    .iter()
                    .any(|arm| arm.guard.as_ref().is_some_and(|guard| expr_contains_fallthrough(guard)) || expr_contains_fallthrough(&arm.body))
        }
        HirExprKind::Loop { iterator, condition, body, .. } => {
            iterator.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
                || condition.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
                || body.statements.iter().any(statement_contains_fallthrough)
                || body.expr.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr))
        }
        HirExprKind::Return(Some(value)) | HirExprKind::Break { expr: Some(value), .. } => expr_contains_fallthrough(value),
        HirExprKind::Return(None) | HirExprKind::Break { expr: None, .. } | HirExprKind::Catch { .. } => false,
    }
}

fn statement_contains_fallthrough(statement: &HirStatement) -> bool {
    match &statement.kind {
        HirStatementKind::Let { initializer, .. } => initializer.as_ref().is_some_and(|expr| expr_contains_fallthrough(expr)),
        HirStatementKind::Expr(expr) => expr_contains_fallthrough(expr),
    }
}

fn lookup_local_type(state: &HirValidationState, name: &str) -> Option<ValkyrieType> {
    state.local_scopes.iter().rev().find_map(|scope| scope.get(name).cloned())
}

fn control_flow_value_type_compatible(expected: &ValkyrieType, actual: &ValkyrieType) -> bool {
    expected == actual || matches!(expected, ValkyrieType::AutoType) || matches!(actual, ValkyrieType::AutoType)
}

fn return_without_value_compatible(expected: &ValkyrieType) -> bool {
    matches!(expected, ValkyrieType::Unit | ValkyrieType::Void | ValkyrieType::AutoType)
}

fn future_resume_type(ty: &ValkyrieType) -> Option<ValkyrieType> {
    match ty {
        ValkyrieType::Apply(base, arguments) if arguments.len() == 1 && matches!(named_type_name(base), Some("Future" | "Promise")) => {
            arguments.first().cloned()
        }
        _ => None,
    }
}

fn named_type_name(ty: &ValkyrieType) -> Option<&str> {
    match ty {
        ValkyrieType::Named(name) => Some(name.as_str()),
        ValkyrieType::Apply(base, _) => named_type_name(base),
        _ => None,
    }
}

fn display_type(ty: &ValkyrieType) -> String {
    match ty {
        ValkyrieType::Void => "void".to_string(),
        ValkyrieType::Unit => "unit".to_string(),
        ValkyrieType::Boolean => "bool".to_string(),
        ValkyrieType::Integer8 { signed } => integer_type_name(*signed, 8),
        ValkyrieType::Integer16 { signed } => integer_type_name(*signed, 16),
        ValkyrieType::Integer32 { signed } => integer_type_name(*signed, 32),
        ValkyrieType::Integer64 { signed } => integer_type_name(*signed, 64),
        ValkyrieType::Integer128 { signed } => integer_type_name(*signed, 128),
        ValkyrieType::Float32 => "f32".to_string(),
        ValkyrieType::Float64 => "f64".to_string(),
        ValkyrieType::Character => "char".to_string(),
        ValkyrieType::Utf8 => "utf8".to_string(),
        ValkyrieType::Utf16 => "utf16".to_string(),
        ValkyrieType::Named(name) => name.to_string(),
        ValkyrieType::Apply(base, arguments) => {
            format!("{}<{}>", display_type(base), arguments.iter().map(display_type).collect::<Vec<_>>().join(", "))
        }
        ValkyrieType::Generic(generic) => generic.name.to_string(),
        ValkyrieType::Function(function) => format!(
            "micro({}) -> {}",
            function.params.iter().map(display_type).collect::<Vec<_>>().join(", "),
            display_type(&function.return_type)
        ),
        ValkyrieType::Tuple(items) => format!("({})", items.iter().map(display_type).collect::<Vec<_>>().join(", ")),
        ValkyrieType::Array(item) => format!("[{}]", display_type(item)),
        ValkyrieType::TypeLambda(lambda) => format!(
            "type lambda({}) -> {}",
            lambda.params.iter().map(|item| item.name.to_string()).collect::<Vec<_>>().join(", "),
            display_type(&lambda.body)
        ),
        ValkyrieType::TraitObject(object) => {
            format!("{}<{}>", object.trait_path, object.type_arguments.iter().map(display_type).collect::<Vec<_>>().join(", "))
        }
        ValkyrieType::Associated(associated) => format!("{}::{}", display_type(&associated.base), associated.name),
        ValkyrieType::AutoType => "auto".to_string(),
        ValkyrieType::SelfType => "Self".to_string(),
    }
}

fn integer_type_name(signed: bool, bits: u16) -> String {
    if signed {
        format!("i{bits}")
    }
    else {
        format!("u{bits}")
    }
}

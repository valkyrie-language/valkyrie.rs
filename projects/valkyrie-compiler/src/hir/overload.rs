//! Minimal overload ranking helpers.
//!
//! These helpers model only the settled precedence:
//! nominal exact > nominal subtype > trait > row.

#![allow(missing_docs)]

use std::collections::BTreeMap;

use crate::hir::type_relation::{ParameterMatchResult, TypeRelationContext};
use valkyrie_types::{
    hir::{
        HirBlock, HirCallableDomain, HirExpr, HirExprKind, HirExtractorPattern, HirFunction, HirIdentifier, HirMatchArm, HirModule, HirPattern,
        HirResolvedCall, HirStatement, HirStatementKind, HirStruct, HirVariant, ValkyrieType,
    },
    Identifier, NamePath,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverloadDomain {
    Function,
    Constructor,
    Operator,
    Extractor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadSignature {
    pub params: Vec<ValkyrieType>,
    pub return_type: ValkyrieType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverloadCandidate {
    pub symbol: NamePath,
    pub owner: Option<Identifier>,
    pub domain: OverloadDomain,
    pub signature: OverloadSignature,
    pub match_kind: OverloadMatchKind,
}

impl OverloadCandidate {
    pub fn new(
        symbol: NamePath,
        domain: OverloadDomain,
        params: Vec<ValkyrieType>,
        return_type: ValkyrieType,
        match_kind: OverloadMatchKind,
    ) -> Self {
        Self { symbol, owner: None, domain, signature: OverloadSignature { params, return_type }, match_kind }
    }

    pub fn new_method(
        owner: Identifier,
        symbol: NamePath,
        domain: OverloadDomain,
        params: Vec<ValkyrieType>,
        return_type: ValkyrieType,
        match_kind: OverloadMatchKind,
    ) -> Self {
        Self { symbol, owner: Some(owner), domain, signature: OverloadSignature { params, return_type }, match_kind }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOverload {
    pub symbol: NamePath,
    pub domain: OverloadDomain,
    pub signature: OverloadSignature,
    pub match_kind: OverloadMatchKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverloadMatchKind {
    NominalExact,
    NominalSubtype { distance: usize },
    Trait,
    Row,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OverloadResolutionError {
    NoMatch,
    Ambiguous { candidates: Vec<NamePath> },
}

pub fn resolve_overload(candidates: &[OverloadCandidate]) -> Result<ResolvedOverload, OverloadResolutionError> {
    let mut ranked = candidates.iter().map(|candidate| (rank(&candidate.match_kind), candidate)).collect::<Vec<_>>();

    ranked.sort_by_key(|(rank, _)| *rank);

    let Some((best_rank, best_candidate)) = ranked.first()
    else {
        return Err(OverloadResolutionError::NoMatch);
    };

    let tied = ranked.iter().filter(|(rank, _)| rank == best_rank).map(|(_, candidate)| candidate.symbol.clone()).collect::<Vec<_>>();

    if tied.len() > 1 {
        Err(OverloadResolutionError::Ambiguous { candidates: tied })
    }
    else {
        Ok(ResolvedOverload {
            symbol: best_candidate.symbol.clone(),
            domain: best_candidate.domain.clone(),
            signature: best_candidate.signature.clone(),
            match_kind: best_candidate.match_kind.clone(),
        })
    }
}

fn rank(match_kind: &OverloadMatchKind) -> (u8, usize) {
    match match_kind {
        OverloadMatchKind::NominalExact => (0, 0),
        OverloadMatchKind::NominalSubtype { distance } => (1, *distance),
        OverloadMatchKind::Trait => (2, 0),
        OverloadMatchKind::Row => (3, 0),
    }
}

pub fn resolve_hir_calls(module: &mut HirModule) {
    let candidates = collect_module_candidates(module);
    let type_relations = TypeRelationContext::from_module(module);

    for function in &mut module.functions {
        resolve_function_calls(function, &candidates, &type_relations);
    }
    for item in &mut module.structs {
        for method in &mut item.methods {
            resolve_function_calls(method, &candidates, &type_relations);
        }
    }
    for item in &mut module.traits {
        for method in &mut item.methods {
            resolve_function_calls(method, &candidates, &type_relations);
        }
        for method in &mut item.default_methods {
            resolve_function_calls(method, &candidates, &type_relations);
        }
    }
    for item in &mut module.impls {
        for method in &mut item.methods {
            resolve_function_calls(method, &candidates, &type_relations);
        }
    }
}

fn collect_module_candidates(module: &HirModule) -> Vec<OverloadCandidate> {
    let mut candidates = Vec::new();
    candidates.extend(module.functions.iter().map(|function| build_function_candidate(function)));
    for item in &module.structs {
        candidates.push(build_struct_constructor_candidate(item));
    }
    for item in &module.structs {
        candidates.extend(item.methods.iter().map(|method| build_method_candidate(method, Some(item.name.clone()))));
    }
    for item in &module.traits {
        candidates.extend(item.methods.iter().map(|method| build_method_candidate(method, None)));
        candidates.extend(item.default_methods.iter().map(|method| build_method_candidate(method, None)));
    }
    for item in &module.impls {
        let owner = match &item.target {
            ValkyrieType::Named(name) => Some(name.clone()),
            _ => None,
        };
        candidates.extend(item.methods.iter().map(|method| build_method_candidate(method, owner.clone())));
    }
    for item in &module.enums {
        candidates.extend(item.variants.iter().map(|variant| build_variant_constructor_candidate(variant)));
    }
    candidates
}

fn build_function_candidate(function: &HirFunction) -> OverloadCandidate {
    OverloadCandidate::new(
        NamePath::new(vec![function.name.clone()]),
        classify_callable_domain(&function.name),
        function.params.iter().map(|param| param.ty.clone()).collect(),
        function.return_type.clone(),
        OverloadMatchKind::Row,
    )
}

fn build_struct_constructor_candidate(item: &HirStruct) -> OverloadCandidate {
    OverloadCandidate::new(
        NamePath::new(vec![item.name.clone()]),
        OverloadDomain::Constructor,
        item.fields.iter().map(|field| field.ty.clone()).collect(),
        ValkyrieType::Named(item.name.clone()),
        OverloadMatchKind::Row,
    )
}

fn build_variant_constructor_candidate(variant: &HirVariant) -> OverloadCandidate {
    OverloadCandidate::new(
        NamePath::new(vec![variant.name.clone()]),
        OverloadDomain::Constructor,
        variant.tuple_types.clone(),
        variant.result_type.clone().unwrap_or_else(|| ValkyrieType::Named(variant.name.clone())),
        OverloadMatchKind::Row,
    )
}

fn build_method_candidate(function: &HirFunction, owner: Option<Identifier>) -> OverloadCandidate {
    if let Some(owner) = owner {
        return OverloadCandidate::new_method(
            owner,
            NamePath::new(vec![function.name.clone()]),
            classify_callable_domain(&function.name),
            function.params.iter().map(|param| param.ty.clone()).collect(),
            function.return_type.clone(),
            OverloadMatchKind::Row,
        );
    }
    OverloadCandidate::new(
        NamePath::new(vec![function.name.clone()]),
        classify_callable_domain(&function.name),
        function.params.iter().map(|param| param.ty.clone()).collect(),
        function.return_type.clone(),
        OverloadMatchKind::Row,
    )
}

fn classify_callable_domain(name: &Identifier) -> OverloadDomain {
    let text = name.as_str();
    if text.starts_with("prefix ") || text.starts_with("infix ") || text.starts_with("suffix ") || text.starts_with("postfix ") {
        OverloadDomain::Operator
    }
    else {
        OverloadDomain::Function
    }
}

fn resolve_function_calls(function: &mut HirFunction, candidates: &[OverloadCandidate], type_relations: &TypeRelationContext) {
    let mut locals = function.params.iter().map(|param| (param.name.name.to_string(), param.ty.clone())).collect::<BTreeMap<_, _>>();
    resolve_block_calls(&mut function.body, candidates, type_relations, &mut locals);
}

fn resolve_block_calls(
    block: &mut HirBlock,
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    locals: &mut BTreeMap<String, ValkyrieType>,
) {
    for statement in &mut block.statements {
        resolve_statement_calls(statement, candidates, type_relations, locals);
    }
    if let Some(expr) = &mut block.expr {
        resolve_expr_calls(expr, candidates, type_relations, locals);
    }
}

fn resolve_statement_calls(
    statement: &mut HirStatement,
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    locals: &mut BTreeMap<String, ValkyrieType>,
) {
    match &mut statement.kind {
        HirStatementKind::Let { pattern, initializer, ty, .. } => {
            if let Some(initializer) = initializer {
                resolve_expr_calls(initializer, candidates, type_relations, locals);
            }
            let binding_type = ty.clone().or_else(|| initializer.as_ref().and_then(|expr| infer_expr_type(expr, locals)));
            if let Some(binding_type) = binding_type {
                bind_pattern_type(pattern, &binding_type, locals);
            }
        }
        HirStatementKind::Expr(expr) => resolve_expr_calls(expr, candidates, type_relations, locals),
    }
}

fn resolve_expr_calls(
    expr: &mut HirExpr,
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    locals: &mut BTreeMap<String, ValkyrieType>,
) {
    match &mut expr.kind {
        HirExprKind::Call { callee, args, resolved } => {
            resolve_expr_calls(callee, candidates, type_relations, locals);
            for arg in args.iter_mut() {
                resolve_expr_calls(arg, candidates, type_relations, locals);
            }
            *resolved = try_resolve_call(callee, args, candidates, type_relations, locals);
        }
        HirExprKind::GenericApply { callee, .. }
        | HirExprKind::FieldInit { value: callee, .. }
        | HirExprKind::Await(callee)
        | HirExprKind::Awake(callee)
        | HirExprKind::BlockOn(callee)
        | HirExprKind::YieldFrom(callee)
        | HirExprKind::Raise(callee)
        | HirExprKind::Resume(callee)
        | HirExprKind::Assign { value: callee, .. }
        | HirExprKind::FieldAccess { object: callee, .. } => resolve_expr_calls(callee, candidates, type_relations, locals),
        HirExprKind::StoreField { object, value, .. } => {
            resolve_expr_calls(object, candidates, type_relations, locals);
            resolve_expr_calls(value, candidates, type_relations, locals);
        }
        HirExprKind::ArrayNew { length, .. } => resolve_expr_calls(length, candidates, type_relations, locals),
        HirExprKind::ArrayLiteral { items: args } => {
            for arg in args {
                resolve_expr_calls(arg, candidates, type_relations, locals);
            }
        }
        HirExprKind::Construct { name, args, resolved } => {
            for arg in args.iter_mut() {
                resolve_expr_calls(arg, candidates, type_relations, locals);
            }
            *resolved = try_resolve_constructor(name, args, candidates, type_relations, locals);
        }
        HirExprKind::Block(block) => resolve_block_calls(block, candidates, type_relations, locals),
        HirExprKind::Lambda { params, body, .. } => {
            let mut lambda_locals = locals.clone();
            for param in params {
                lambda_locals.insert(param.name.name.to_string(), param.ty.clone());
            }
            resolve_block_calls(body, candidates, type_relations, &mut lambda_locals);
        }
        HirExprKind::AnonymousClass { fields, methods, .. } => {
            for (_, value) in fields {
                resolve_expr_calls(value, candidates, type_relations, locals);
            }
            for method in methods {
                resolve_function_calls(method, candidates, type_relations);
            }
        }
        HirExprKind::If { condition, then_branch, else_branch } => {
            resolve_expr_calls(condition, candidates, type_relations, locals);
            let mut then_locals = locals.clone();
            resolve_block_calls(then_branch, candidates, type_relations, &mut then_locals);
            if let Some(else_branch) = else_branch {
                let mut else_locals = locals.clone();
                resolve_block_calls(else_branch, candidates, type_relations, &mut else_locals);
            }
        }
        HirExprKind::Match { scrutinee, arms } | HirExprKind::Case { scrutinee, arms } => {
            resolve_expr_calls(scrutinee, candidates, type_relations, locals);
            let scrutinee_type = infer_expr_type(scrutinee, locals);
            for arm in arms {
                resolve_arm_calls(arm, candidates, type_relations, locals, scrutinee_type.as_ref());
            }
        }
        HirExprKind::Loop { iterator, condition, body, .. } => {
            if let Some(iterator) = iterator {
                resolve_expr_calls(iterator, candidates, type_relations, locals);
            }
            if let Some(condition) = condition {
                resolve_expr_calls(condition, candidates, type_relations, locals);
            }
            let mut body_locals = locals.clone();
            resolve_block_calls(body, candidates, type_relations, &mut body_locals);
        }
        HirExprKind::Return(Some(value)) | HirExprKind::Break { expr: Some(value), .. } | HirExprKind::Yield(Some(value)) => {
            resolve_expr_calls(value, candidates, type_relations, locals);
        }
        HirExprKind::Catch { expr, arms } => {
            resolve_expr_calls(expr, candidates, type_relations, locals);
            for arm in arms {
                resolve_arm_calls(arm, candidates, type_relations, locals, None);
            }
        }
        HirExprKind::Literal(_)
        | HirExprKind::Variable(_)
        | HirExprKind::Path(_)
        | HirExprKind::Return(None)
        | HirExprKind::Break { expr: None, .. }
        | HirExprKind::Continue { .. }
        | HirExprKind::Yield(None)
        | HirExprKind::Fallthrough
        | HirExprKind::SuperCall { .. }
        | HirExprKind::With { .. } => {}
    }
}

fn resolve_arm_calls(
    arm: &mut HirMatchArm,
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    locals: &BTreeMap<String, ValkyrieType>,
    scrutinee_type: Option<&ValkyrieType>,
) {
    let mut arm_locals = locals.clone();
    resolve_pattern_calls(&mut arm.pattern, candidates, type_relations, scrutinee_type);
    if let Some(scrutinee_type) = scrutinee_type {
        bind_pattern_type(&arm.pattern, scrutinee_type, &mut arm_locals);
    }
    if let Some(guard) = &mut arm.guard {
        resolve_expr_calls(guard, candidates, type_relations, &mut arm_locals);
    }
    resolve_expr_calls(&mut arm.body, candidates, type_relations, &mut arm_locals);
}

fn resolve_pattern_calls(
    pattern: &mut HirPattern,
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    scrutinee_type: Option<&ValkyrieType>,
) {
    match pattern {
        HirPattern::Name(name) => {
            if name.parts().len() == 1 {
                let first_part = name.parts().first().expect("single-segment name should have first part");
                if first_part.as_str().chars().next().is_some_and(|c| c.is_lowercase()) {
                    unreachable!(
                        "单段小写名字模式 `{}` 应由解析器处理为 Variable，不应出现在 Name 中",
                        name.parts().iter().map(|p| p.as_str()).collect::<Vec<_>>().join("::")
                    );
                }
            }
            if let Some(actual_type) = scrutinee_type {
                if should_resolve_name_pattern_as_type(name, actual_type) {
                    *pattern = HirPattern::Type(name.clone());
                }
            }
        }
        HirPattern::Extractor(HirExtractorPattern::Constructor { fields, canonical_callee, resolved, .. }) => {
            for field in fields.iter_mut() {
                resolve_pattern_calls(field, candidates, type_relations, None);
            }
            if let Some(actual_type) = scrutinee_type {
                *resolved = try_resolve_pattern_extractor(canonical_callee, actual_type, candidates, type_relations);
            }
        }
        HirPattern::Extractor(HirExtractorPattern::Array { prefix, suffix, canonical_callee, resolved, .. }) => {
            for item in prefix.iter_mut() {
                resolve_pattern_calls(item, candidates, type_relations, None);
            }
            for item in suffix.iter_mut() {
                resolve_pattern_calls(item, candidates, type_relations, None);
            }
            if let Some(actual_type) = scrutinee_type {
                *resolved = try_resolve_pattern_extractor(canonical_callee, actual_type, candidates, type_relations);
            }
        }
        HirPattern::Tuple(items) => {
            for item in items.iter_mut() {
                resolve_pattern_calls(item, candidates, type_relations, None);
            }
        }
        HirPattern::Or(items) => {
            for item in items.iter_mut() {
                resolve_pattern_calls(item, candidates, type_relations, scrutinee_type);
            }
        }
        HirPattern::Object { fields, .. } => {
            for (_, item) in fields.iter_mut() {
                resolve_pattern_calls(item, candidates, type_relations, None);
            }
        }
        _ => {}
    }
}

fn try_resolve_call(
    callee: &HirExpr,
    args: &[HirExpr],
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    locals: &BTreeMap<String, ValkyrieType>,
) -> Option<HirResolvedCall> {
    let callee_name = extract_callable_name(callee)?;
    let actual_types = args.iter().map(|arg| infer_expr_type(arg, locals)).collect::<Option<Vec<_>>>()?;
    let filtered = candidates
        .iter()
        .filter(|candidate| candidate.symbol.parts().last().is_some_and(|name| name == &callee_name))
        .filter_map(|candidate| {
            let match_kind = compute_call_match_kind(type_relations, &actual_types, &candidate.signature.params)?;
            Some(OverloadCandidate::new(
                candidate.symbol.clone(),
                candidate.domain.clone(),
                candidate.signature.params.clone(),
                candidate.signature.return_type.clone(),
                match_kind,
            ))
        })
        .collect::<Vec<_>>();
    let resolved = resolve_overload(&filtered).ok()?;
    Some(HirResolvedCall {
        symbol: resolved.symbol,
        domain: match resolved.domain {
            OverloadDomain::Function => HirCallableDomain::Function,
            OverloadDomain::Constructor => HirCallableDomain::Constructor,
            OverloadDomain::Operator => HirCallableDomain::Operator,
            OverloadDomain::Extractor => HirCallableDomain::Extractor,
        },
        return_type: resolved.signature.return_type,
    })
}

fn try_resolve_constructor(
    name: &Identifier,
    args: &[HirExpr],
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
    locals: &BTreeMap<String, ValkyrieType>,
) -> Option<HirResolvedCall> {
    let actual_types = args.iter().map(|arg| infer_expr_type(arg, locals)).collect::<Option<Vec<_>>>()?;
    let filtered = candidates
        .iter()
        .filter(|candidate| candidate.domain == OverloadDomain::Constructor)
        .filter(|candidate| candidate.symbol.parts().last().is_some_and(|candidate_name| candidate_name == name))
        .filter_map(|candidate| {
            let match_kind = compute_call_match_kind(type_relations, &actual_types, &candidate.signature.params)?;
            Some(OverloadCandidate::new(
                candidate.symbol.clone(),
                candidate.domain.clone(),
                candidate.signature.params.clone(),
                candidate.signature.return_type.clone(),
                match_kind,
            ))
        })
        .collect::<Vec<_>>();
    let resolved = resolve_overload(&filtered).ok()?;
    Some(HirResolvedCall { symbol: resolved.symbol, domain: HirCallableDomain::Constructor, return_type: resolved.signature.return_type })
}

fn try_resolve_pattern_extractor(
    canonical_callee: &NamePath,
    actual_type: &ValkyrieType,
    candidates: &[OverloadCandidate],
    type_relations: &TypeRelationContext,
) -> Option<HirResolvedCall> {
    let filtered = candidates
        .iter()
        .filter(|candidate| candidate.domain == OverloadDomain::Function)
        .filter(|candidate| candidate.owner.as_ref().is_some_and(|owner| canonical_callee.parts().first() == Some(owner)))
        .filter(|candidate| matches!(candidate.signature.params.first(), Some(ValkyrieType::r#SelfType)))
        .filter(|candidate| candidate.symbol.parts().last() == canonical_callee.parts().last())
        .filter_map(|candidate| {
            let param_type = candidate.signature.params.first()?;
            let match_kind = compute_call_match_kind(type_relations, std::slice::from_ref(actual_type), std::slice::from_ref(param_type))?;
            Some(OverloadCandidate::new(
                candidate.symbol.clone(),
                OverloadDomain::Extractor,
                candidate.signature.params.clone(),
                candidate.signature.return_type.clone(),
                match_kind,
            ))
        })
        .collect::<Vec<_>>();
    let resolved = resolve_overload(&filtered).ok()?;
    Some(HirResolvedCall { symbol: resolved.symbol, domain: HirCallableDomain::Extractor, return_type: resolved.signature.return_type })
}

fn extract_callable_name(callee: &HirExpr) -> Option<Identifier> {
    match &callee.kind {
        HirExprKind::Variable(identifier) => Some(identifier.name.clone()),
        HirExprKind::Path(path) => path.parts().last().cloned(),
        HirExprKind::GenericApply { callee, .. } => extract_callable_name(callee),
        _ => None,
    }
}

fn compute_call_match_kind(
    type_relations: &TypeRelationContext,
    actual_types: &[ValkyrieType],
    expected_types: &[ValkyrieType],
) -> Option<OverloadMatchKind> {
    if actual_types.len() != expected_types.len() {
        return None;
    }

    let mut saw_trait = false;
    let mut saw_row = false;
    let mut subtype_distance = 0usize;

    for (actual, expected) in actual_types.iter().zip(expected_types) {
        match type_relations.match_parameter(actual, expected) {
            ParameterMatchResult::NominalExact => {}
            ParameterMatchResult::NominalSubtype { distance } => subtype_distance += distance,
            ParameterMatchResult::Trait { .. } => saw_trait = true,
            ParameterMatchResult::Row => saw_row = true,
            ParameterMatchResult::NoMatch { .. } => {
                return None;
            }
        }
    }

    if saw_row {
        Some(OverloadMatchKind::Row)
    }
    else if saw_trait {
        Some(OverloadMatchKind::Trait)
    }
    else if subtype_distance > 0 {
        Some(OverloadMatchKind::NominalSubtype { distance: subtype_distance })
    }
    else {
        Some(OverloadMatchKind::NominalExact)
    }
}

fn should_resolve_name_pattern_as_type(name: &NamePath, actual_type: &ValkyrieType) -> bool {
    if name.parts().len() != 1 {
        return false;
    }

    match actual_type {
        ValkyrieType::Named(actual_name) => name.parts().last().is_some_and(|expected| expected == actual_name),
        ValkyrieType::Apply(base, _) => should_resolve_name_pattern_as_type(name, base),
        _ => false,
    }
}

fn infer_expr_type(expr: &HirExpr, locals: &BTreeMap<String, ValkyrieType>) -> Option<ValkyrieType> {
    match &expr.kind {
        HirExprKind::Literal(literal) => Some(match literal {
            valkyrie_types::hir::HirLiteral::Integer64(_) => ValkyrieType::Integer64 { signed: true },
            valkyrie_types::hir::HirLiteral::Float64(_) => ValkyrieType::Float64,
            valkyrie_types::hir::HirLiteral::String(_) => ValkyrieType::Named(Identifier::new("utf8")),
            valkyrie_types::hir::HirLiteral::Bool(_) => ValkyrieType::Boolean,
            valkyrie_types::hir::HirLiteral::Unit => ValkyrieType::Unit,
        }),
        HirExprKind::Variable(HirIdentifier { name, .. }) => locals.get(name.as_str()).cloned(),
        HirExprKind::Call { resolved, .. } => resolved.as_ref().map(|call| call.return_type.clone()),
        HirExprKind::ArrayLiteral { items } => {
            let item_type = items.first().and_then(|item| infer_expr_type(item, locals))?;
            Some(ValkyrieType::Array(Box::new(item_type)))
        }
        HirExprKind::Construct { name, .. } => Some(ValkyrieType::Named(name.clone())),
        HirExprKind::If { then_branch, else_branch, .. } => {
            let then_type = then_branch.expr.as_ref().and_then(|expr| infer_expr_type(expr, locals))?;
            let else_type = else_branch.as_ref().and_then(|branch| branch.expr.as_ref()).and_then(|expr| infer_expr_type(expr, locals))?;
            if then_type == else_type {
                Some(then_type)
            }
            else {
                None
            }
        }
        HirExprKind::Block(block) => block.expr.as_ref().and_then(|expr| infer_expr_type(expr, locals)),
        _ => None,
    }
}

fn bind_pattern_type(pattern: &HirPattern, ty: &ValkyrieType, locals: &mut BTreeMap<String, ValkyrieType>) {
    match pattern {
        HirPattern::Variable(identifier) => {
            locals.insert(identifier.name.to_string(), ty.clone());
        }
        HirPattern::TypedBind { identifier, ty: bound_ty } => {
            locals.insert(identifier.name.to_string(), ValkyrieType::Named(bound_ty.name().clone()));
        }
        HirPattern::Tuple(items) if matches!(ty, ValkyrieType::Tuple(_)) => {
            if let ValkyrieType::Tuple(types) = ty {
                for (pattern, item_ty) in items.iter().zip(types) {
                    bind_pattern_type(pattern, item_ty, locals);
                }
            }
        }
        _ => {}
    }
}

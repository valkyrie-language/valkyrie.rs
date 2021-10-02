use valkyrie_types::{
    hir::{HirExpr, HirExprKind, HirLiteral, ValkyrieType},
    Identifier, NamePath,
};

use super::{
    builtin_call_output_type, infer_builder_operand_type, lower_callee_operand, resolve_builtin_call, MirBuilder, MirConstant, MirDispatchKind,
    MirInstruction, MirInstructionKind, MirOperand, MirTerminator, MirValueOrigin,
};

impl MirBuilder {
    pub(super) fn lower_expr_to_operand(&mut self, expr: &HirExpr) -> MirOperand {
        self.lower_expr_to_operand_with_hint(expr, None)
    }

    pub(super) fn lower_expr_to_operand_with_hint(&mut self, expr: &HirExpr, expected_type: Option<&ValkyrieType>) -> MirOperand {
        match &expr.kind {
            HirExprKind::Literal(literal) => {
                let (constant, ty) = lower_literal(literal, expected_type);
                let value = self.next_value(MirValueOrigin::Literal);
                self.instructions
                    .push(MirInstruction { output: Some(value), kind: MirInstructionKind::LoadConstant { constant, ty: ty.clone() } });
                if let Some(ty) = ty {
                    self.value_types.insert(value, ty);
                }
                MirOperand::Value(value)
            }
            HirExprKind::Variable(identifier) => self
                .bindings
                .get(identifier.name.as_str())
                .cloned()
                .unwrap_or_else(|| MirOperand::Symbol(NamePath::new(vec![identifier.name.clone()]))),
            HirExprKind::Path(path) => {
                let value = self.next_value(MirValueOrigin::Path);
                self.instructions.push(MirInstruction { output: Some(value), kind: MirInstructionKind::LoadSymbol { path: path.clone() } });
                MirOperand::Value(value)
            }
            HirExprKind::Call { callee, args, resolved } => {
                if let Some((receiver_operand, method_name)) = self.extract_method_call(callee) {
                    let mut arguments = args.iter().map(|arg| self.lower_expr_to_operand(arg)).collect::<Vec<_>>();
                    arguments.insert(0, receiver_operand);
                    let callee = MirOperand::Symbol(NamePath::new(vec![method_name.clone()]));
                    let value = self.next_value(MirValueOrigin::CallResult);
                    self.instructions.push(MirInstruction {
                        output: Some(value),
                        kind: MirInstructionKind::Call {
                            dispatch: MirDispatchKind::Static,
                            callee,
                            arguments,
                            builtin: None,
                            witness: None,
                            effect: None,
                        },
                    });
                    if let Some(return_type) = self.return_types.get(method_name.as_str()).cloned() {
                        self.value_types.insert(value, return_type);
                    }
                    return MirOperand::Value(value);
                }
                let callee = lower_callee_operand(callee, resolved.as_ref(), self);
                let arguments = args.iter().map(|arg| self.lower_expr_to_operand(arg)).collect::<Vec<_>>();
                let builtin = resolve_builtin_call(&callee, &arguments, &self.value_types);
                let value = self.next_value(MirValueOrigin::CallResult);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::Call {
                        dispatch: MirDispatchKind::Static,
                        callee: callee.clone(),
                        arguments: arguments.clone(),
                        builtin,
                        witness: None,
                        effect: None,
                    },
                });
                if let Some(ty) = builtin
                    .and_then(|builtin| builtin_call_output_type(builtin, &arguments, &self.value_types))
                    .or_else(|| resolved.as_ref().map(|call| call.return_type.clone()))
                    .or_else(|| match &callee {
                        MirOperand::Symbol(path) => self.return_types.get(&path.to_string()).cloned(),
                        _ => None,
                    })
                {
                    self.value_types.insert(value, ty);
                }
                MirOperand::Value(value)
            }
            HirExprKind::ArrayNew { element_type, length } => {
                let length_operand = self.lower_expr_to_operand(length);
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::ArrayNew { element_type: element_type.clone(), length: length_operand },
                });
                self.value_types.insert(value, ValkyrieType::Array(Box::new(element_type.clone())));
                MirOperand::Value(value)
            }
            HirExprKind::ArrayLiteral { items } => {
                let array_item_type = match expected_type {
                    Some(ValkyrieType::Array(item)) => Some(item.as_ref()),
                    _ => None,
                };
                let element_type = infer_array_literal_element_type(items, array_item_type);
                let item_operands =
                    items.iter().map(|item| self.lower_expr_to_operand_with_hint(item, Some(&element_type))).collect::<Vec<_>>();
                let array_value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(array_value),
                    kind: MirInstructionKind::ArrayLiteral { element_type: element_type.clone(), items: item_operands },
                });
                self.value_types.insert(array_value, ValkyrieType::Array(Box::new(element_type)));
                MirOperand::Value(array_value)
            }
            HirExprKind::Construct { name, args, resolved } => {
                let mut fields = Vec::with_capacity(args.len());
                for arg in args {
                    if let HirExprKind::FieldInit { name, value } = &arg.kind {
                        let value_operand = self.lower_expr_to_operand(value);
                        fields.push((name.to_string(), value_operand));
                    }
                }
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions
                    .push(MirInstruction { output: Some(value), kind: MirInstructionKind::StructNew { type_name: name.to_string(), fields } });
                self.value_types
                    .insert(value, resolved.as_ref().map(|call| call.return_type.clone()).unwrap_or_else(|| ValkyrieType::Named(name.clone())));
                MirOperand::Value(value)
            }
            HirExprKind::FieldAccess { object, field } => {
                let object_operand = self.lower_expr_to_operand(object);
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::FieldGet { object: object_operand, field: field.to_string() },
                });
                MirOperand::Value(value)
            }
            HirExprKind::StoreField { object, field, value } => {
                let object_operand = self.lower_expr_to_operand(object);
                let value_operand = self.lower_expr_to_operand(value);
                self.instructions.push(MirInstruction {
                    output: None,
                    kind: MirInstructionKind::FieldSet { object: object_operand, field: field.to_string(), value: value_operand },
                });
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Return(value) => {
                let terminand = value.as_deref().map(|e| self.lower_expr_to_operand(e));
                self.terminate(MirTerminator::Return { value: terminand });
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Assign { target, value } => {
                let operand = self.lower_expr_to_operand(value);
                let name = target.as_str().to_string();
                let new_value = self.next_value(MirValueOrigin::LetBinding { name: name.clone() });
                self.instructions.push(MirInstruction {
                    output: Some(new_value),
                    kind: MirInstructionKind::StoreVar { name: name.clone(), value: operand.clone(), ty: None },
                });
                if let Some(ty) = infer_builder_operand_type(&operand, &self.value_types) {
                    self.value_types.insert(new_value, ty);
                }
                self.bindings.insert(name, MirOperand::Value(new_value));
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::If { condition, then_branch, else_branch } => self.lower_if_expr(condition, then_branch, else_branch),
            HirExprKind::Block(body) => self.lower_block_expr(body),
            HirExprKind::Loop { label, pattern, iterator, condition, body, .. } => {
                self.lower_loop_expr(label, pattern, iterator, condition, body)
            }
            HirExprKind::Break { label, expr } => self.lower_break_expr(label, expr),
            HirExprKind::Continue { label } => self.lower_continue_expr(label),
            HirExprKind::Match { scrutinee, arms } => self.lower_match_expr(scrutinee, arms),
            HirExprKind::Case { scrutinee, arms } => self.lower_case_expr(scrutinee, arms),
            HirExprKind::Yield(value) => self.lower_yield_expr(value.as_deref()),
            HirExprKind::YieldFrom(value) => self.lower_yield_from_expr(value),
            HirExprKind::Await(value) => self.lower_await_expr(value),
            HirExprKind::Awake(value) => self.lower_awake_expr(value),
            HirExprKind::BlockOn(value) => self.lower_block_on_expr(value),
            HirExprKind::Raise(value) => self.lower_raise_expr(value),
            HirExprKind::Resume(value) => self.lower_resume_expr(value),
            HirExprKind::Catch { expr, arms } => self.lower_catch_expr(expr, arms),
            HirExprKind::Fallthrough => self.lower_fallthrough_expr(),
            _ => {
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::LoadSymbol { path: NamePath::new(vec![Identifier::new("unsupported_hir_expr")]) },
                });
                MirOperand::Value(value)
            }
        }
    }
}

pub(super) fn infer_array_literal_element_type(items: &[HirExpr], hint: Option<&ValkyrieType>) -> ValkyrieType {
    if let Some(hint) = hint {
        return hint.clone();
    }
    match items.first().map(|item| &item.kind) {
        Some(HirExprKind::Literal(HirLiteral::Bool(_))) => ValkyrieType::Boolean,
        Some(HirExprKind::Literal(HirLiteral::String(_))) => ValkyrieType::Utf8,
        Some(HirExprKind::Literal(HirLiteral::Float64(_))) => ValkyrieType::Float64,
        Some(HirExprKind::Literal(HirLiteral::Integer64(_))) => ValkyrieType::Integer32 { signed: true },
        _ => ValkyrieType::Integer32 { signed: true },
    }
}

pub(super) fn lower_literal(literal: &HirLiteral, expected_type: Option<&ValkyrieType>) -> (MirConstant, Option<ValkyrieType>) {
    match literal {
        HirLiteral::Integer64(value) => (
            MirConstant::Int(*value),
            Some(match expected_type {
                Some(ValkyrieType::Integer32 { signed }) => ValkyrieType::Integer32 { signed: *signed },
                Some(ValkyrieType::Integer64 { signed }) => ValkyrieType::Integer64 { signed: *signed },
                _ if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => ValkyrieType::Integer32 { signed: true },
                _ => ValkyrieType::Integer64 { signed: true },
            }),
        ),
        HirLiteral::Float64(value) => (MirConstant::Float64(*value), Some(ValkyrieType::Float64)),
        HirLiteral::Bool(value) => (MirConstant::Bool(*value), Some(ValkyrieType::Boolean)),
        HirLiteral::String(value) => (
            MirConstant::String(
                value
                    .segments
                    .iter()
                    .map(|segment| match segment {
                        valkyrie_types::hir::HirStringSegment::Text(text) => text.clone(),
                        valkyrie_types::hir::HirStringSegment::Interpolation { expr, .. } => {
                            format!("${{{}}}", render_interpolation_expr(expr))
                        }
                    })
                    .collect::<String>(),
            ),
            Some(ValkyrieType::Utf8),
        ),
        HirLiteral::Unit => (MirConstant::Unit, Some(ValkyrieType::Unit)),
    }
}

fn render_interpolation_expr(expr: &HirExpr) -> String {
    match &expr.kind {
        HirExprKind::Variable(identifier) => identifier.name.to_string(),
        HirExprKind::Path(path) => path.to_string(),
        HirExprKind::Literal(HirLiteral::Integer64(value)) => value.to_string(),
        HirExprKind::Literal(HirLiteral::Bool(value)) => value.to_string(),
        HirExprKind::Literal(HirLiteral::String(value)) => value
            .segments
            .iter()
            .map(|segment| match segment {
                valkyrie_types::hir::HirStringSegment::Text(text) => text.clone(),
                valkyrie_types::hir::HirStringSegment::Interpolation { .. } => "${...}".to_string(),
            })
            .collect(),
        HirExprKind::Call { callee, .. } => format!("{}(...)", render_interpolation_expr(callee)),
        _ => "...".to_string(),
    }
}

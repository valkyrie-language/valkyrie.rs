use std::collections::BTreeSet;

use valkyrie_types::{
    hir::{HirExpr, HirExprKind, HirExtractorPattern, HirLiteral, HirPattern, HirResolvedCall, ValkyrieType},
    Identifier, NamePath,
};

use super::{
    callee_name_matches, infer_builder_operand_type, lower_literal, named_type_name, plain_type_pattern_matches, MirBuilder, MirBuiltinCall,
    MirBuiltinCompareOp, MirConstant, MirDispatchKind, MirInstruction, MirInstructionKind, MirOperand, MirValueOrigin,
};

impl MirBuilder {
    pub(super) fn record_static_binding(&mut self, pattern: &HirPattern, initializer: Option<&HirExpr>) {
        match pattern {
            HirPattern::Variable(identifier) => {
                if let Some(expr) = initializer {
                    self.static_bindings.insert(identifier.name.to_string(), expr.clone());
                }
                else {
                    self.static_bindings.remove(identifier.name.as_str());
                }
            }
            HirPattern::Tuple(items) => {
                for item in items {
                    self.record_static_binding(item, None);
                }
            }
            HirPattern::Extractor(extractor) => match extractor {
                HirExtractorPattern::Array { prefix, suffix, .. } => {
                    for item in prefix {
                        self.record_static_binding(item, None);
                    }
                    for item in suffix {
                        self.record_static_binding(item, None);
                    }
                }
                HirExtractorPattern::Constructor { fields, .. } => {
                    for field in fields {
                        self.record_static_binding(field, None);
                    }
                }
            },
            _ => {}
        }
    }

    pub(super) fn bind_pattern_from_expr(&mut self, pattern: &HirPattern, expr: &HirExpr, ty: Option<ValkyrieType>) {
        match pattern {
            HirPattern::Tuple(items) => {
                if let Some(tuple_items) = tuple_literal_items(expr) {
                    for (item_pattern, item_expr) in items.iter().zip(tuple_items.iter()) {
                        self.bind_pattern_from_expr(item_pattern, item_expr, None);
                    }
                    return;
                }
            }
            HirPattern::Wildcard => {
                let _ = self.lower_expr_to_operand(expr);
                return;
            }
            _ => {}
        }

        let operand = self.lower_expr_to_operand_with_hint(expr, ty.as_ref());
        self.bind_pattern_from_operand(pattern, operand, ty);
    }

    pub(super) fn bind_pattern_from_operand(&mut self, pattern: &HirPattern, operand: MirOperand, ty: Option<ValkyrieType>) {
        self.bind_pattern_from_operand_with_payload(pattern, operand, ty, None);
    }

    pub(super) fn bind_pattern_from_operand_with_payload(
        &mut self,
        pattern: &HirPattern,
        operand: MirOperand,
        ty: Option<ValkyrieType>,
        extractor_payload: Option<MirOperand>,
    ) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Variable(identifier) => {
                let name = identifier.name.to_string();
                let value = self.next_value(MirValueOrigin::LetBinding { name: name.clone() });
                let inferred_type = ty.clone().or_else(|| infer_builder_operand_type(&operand, &self.value_types));
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::StoreVar { name: name.clone(), value: operand, ty },
                });
                if let Some(inferred_type) = inferred_type {
                    self.value_types.insert(value, inferred_type);
                }
                self.bindings.insert(name, MirOperand::Value(value));
            }
            HirPattern::TypedBind { identifier, ty: pattern_type } => {
                let typed_hint = ty.or_else(|| pattern_type.parts().last().cloned().map(ValkyrieType::Named));
                self.bind_pattern_from_operand_with_payload(&HirPattern::Variable(identifier.clone()), operand, typed_hint, None);
            }
            HirPattern::Tuple(items) => {
                for (index, item_pattern) in items.iter().enumerate() {
                    let extracted = self.lower_static_call(&format!("tuple_get_{index}"), vec![operand.clone()], MirValueOrigin::Temporary);
                    self.bind_pattern_from_operand_with_payload(item_pattern, MirOperand::Value(extracted), None, None);
                }
            }
            HirPattern::Extractor(extractor) => {
                let payload = match extractor_payload {
                    Some(payload) => payload,
                    None => match extractor {
                        HirExtractorPattern::Constructor { resolved: Some(resolved), .. }
                        | HirExtractorPattern::Array { resolved: Some(resolved), .. } => {
                            self.lower_extractor_call_operand(resolved, operand.clone())
                        }
                        _ => {
                            self.bind_unknown_pattern_bindings(&HirPattern::Extractor(extractor.clone()));
                            return;
                        }
                    },
                };
                self.bind_extractor_pattern_from_payload(extractor, payload);
            }
            HirPattern::Object { fields, rest, .. } => {
                self.bind_object_pattern_from_operand(fields, rest.as_ref(), operand);
            }
            _ => {
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::LoadSymbol { path: NamePath::new(vec![Identifier::new("unsupported_pattern")]) },
                });
            }
        }
    }

    pub(super) fn lower_pattern_match_operand(&mut self, pattern: &HirPattern, value: MirOperand) -> MirOperand {
        match pattern {
            HirPattern::Wildcard | HirPattern::Variable(_) | HirPattern::Else => MirOperand::Constant(MirConstant::Bool(true)),
            HirPattern::Literal(literal) => self.lower_literal_pattern_match_operand(literal, value),
            HirPattern::Range { start, end, inclusive_end } => {
                self.lower_range_pattern_match_operand(start.as_ref(), end.as_ref(), *inclusive_end, value)
            }
            HirPattern::Tuple(items) => self.lower_tuple_pattern_match_operand(items, value),
            HirPattern::Extractor(extractor) => self.lower_extractor_pattern_match_operand(extractor, value),
            HirPattern::Name(name) => self.lower_fallback_pattern_match(HirPattern::Name(name.clone()), value),
            HirPattern::Type(name) => self.lower_type_pattern_match_operand(name, value),
            HirPattern::TypedBind { ty, .. } => self.lower_type_pattern_match_operand(ty, value),
            HirPattern::Object { name, fields, .. } => self.lower_object_pattern_match_operand(name, fields, value),
            HirPattern::Or(patterns) => self.lower_or_pattern_match_operand(patterns, value),
        }
    }

    pub(super) fn lower_pattern_match_probe(&mut self, pattern: &HirPattern, value: MirOperand) -> (MirOperand, Option<MirOperand>) {
        match pattern {
            HirPattern::Extractor(extractor) => {
                let Some(resolved) = extractor_resolved_call(extractor)
                else {
                    return (self.lower_fallback_pattern_match(HirPattern::Extractor(extractor.clone()), value), None);
                };
                let payload = self.lower_extractor_call_operand(resolved, value.clone());
                let matched = self.lower_extractor_matched_operand(payload.clone());
                let condition = self.lower_extractor_nested_pattern_match_operand(extractor, payload.clone(), matched, resolved);
                (condition, Some(payload))
            }
            _ => (self.lower_pattern_match_operand(pattern, value), None),
        }
    }

    pub(super) fn lower_type_pattern_match_operand(&mut self, name: &NamePath, value: MirOperand) -> MirOperand {
        let Some(actual_type) = infer_builder_operand_type(&value, &self.value_types)
        else {
            return self.lower_fallback_pattern_match(HirPattern::Type(name.clone()), value);
        };

        MirOperand::Constant(MirConstant::Bool(self.type_pattern_matches(&actual_type, name)))
    }

    pub(super) fn lower_tuple_pattern_match_operand(&mut self, items: &[HirPattern], value: MirOperand) -> MirOperand {
        let Some(ValkyrieType::Tuple(types)) = infer_builder_operand_type(&value, &self.value_types)
        else {
            return self.lower_fallback_pattern_match(HirPattern::Tuple(items.to_vec()), value);
        };
        if items.len() != types.len() {
            return MirOperand::Constant(MirConstant::Bool(false));
        }

        let mut accumulated: Option<MirOperand> = None;
        for (index, item_pattern) in items.iter().enumerate() {
            let extracted = self.lower_static_call(&format!("tuple_get_{index}"), vec![value.clone()], MirValueOrigin::Temporary);
            self.value_types.insert(extracted, types[index].clone());
            let item_match = self.lower_pattern_match_operand(item_pattern, MirOperand::Value(extracted));
            accumulated = Some(self.merge_pattern_match_operands(accumulated, item_match));
        }

        accumulated.unwrap_or(MirOperand::Constant(MirConstant::Bool(true)))
    }

    /// extractor 模式会转发成显式 extractor 方法调用，并约定 tuple payload：
    /// 第 0 槽是 match flag，后续槽位是绑定载荷。
    pub(super) fn lower_extractor_pattern_match_operand(&mut self, extractor: &HirExtractorPattern, value: MirOperand) -> MirOperand {
        let Some(resolved) = extractor_resolved_call(extractor)
        else {
            return self.lower_fallback_pattern_match(HirPattern::Extractor(extractor.clone()), value);
        };
        let payload = self.lower_extractor_call_operand(resolved, value.clone());
        let matched = self.lower_extractor_matched_operand(payload.clone());
        self.lower_extractor_nested_pattern_match_operand(extractor, payload, matched, resolved)
    }

    pub(super) fn lower_literal_pattern_match_operand(&mut self, literal: &HirLiteral, value: MirOperand) -> MirOperand {
        let (constant, expected_type) = lower_literal(literal, None);
        match literal {
            HirLiteral::Bool(_) | HirLiteral::Integer64(_) | HirLiteral::Float64(_) => {
                let Some(expected_type) = expected_type
                else {
                    return self.lower_fallback_pattern_match(HirPattern::Literal(literal.clone()), value);
                };
                let Some(actual_type) = infer_builder_operand_type(&value, &self.value_types)
                else {
                    return self.lower_fallback_pattern_match(HirPattern::Literal(literal.clone()), value);
                };
                if actual_type != expected_type {
                    return MirOperand::Constant(MirConstant::Bool(false));
                }

                let output = self.next_value(MirValueOrigin::Temporary);
                let rhs = MirOperand::Constant(constant);
                self.instructions.push(MirInstruction {
                    output: Some(output),
                    kind: MirInstructionKind::Call {
                        dispatch: MirDispatchKind::Static,
                        callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new("infix ==")])),
                        arguments: vec![value, rhs],
                        builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)),
                        witness: None,
                        effect: None,
                    },
                });
                self.value_types.insert(output, ValkyrieType::Boolean);
                MirOperand::Value(output)
            }
            HirLiteral::Unit => {
                if matches!(infer_builder_operand_type(&value, &self.value_types), Some(ValkyrieType::Unit)) {
                    MirOperand::Constant(MirConstant::Bool(true))
                }
                else {
                    MirOperand::Constant(MirConstant::Bool(false))
                }
            }
            HirLiteral::String(_) => {
                let Some(expected_type) = expected_type
                else {
                    return self.lower_fallback_pattern_match(HirPattern::Literal(literal.clone()), value);
                };
                let Some(actual_type) = infer_builder_operand_type(&value, &self.value_types)
                else {
                    return self.lower_fallback_pattern_match(HirPattern::Literal(literal.clone()), value);
                };
                if actual_type != expected_type {
                    return MirOperand::Constant(MirConstant::Bool(false));
                }

                match (value, constant) {
                    (MirOperand::Constant(MirConstant::String(actual)), MirConstant::String(expected)) => {
                        MirOperand::Constant(MirConstant::Bool(actual == expected))
                    }
                    (value, _) => self.lower_fallback_pattern_match(HirPattern::Literal(literal.clone()), value),
                }
            }
        }
    }

    pub(super) fn lower_range_pattern_match_operand(
        &mut self,
        start: Option<&HirLiteral>,
        end: Option<&HirLiteral>,
        inclusive_end: bool,
        value: MirOperand,
    ) -> MirOperand {
        let Some(actual_type) = infer_builder_operand_type(&value, &self.value_types)
        else {
            return self.lower_fallback_pattern_match(HirPattern::Range { start: start.cloned(), end: end.cloned(), inclusive_end }, value);
        };

        if !matches!(
            actual_type,
            ValkyrieType::Integer8 { .. }
                | ValkyrieType::Integer16 { .. }
                | ValkyrieType::Integer32 { .. }
                | ValkyrieType::Integer64 { .. }
                | ValkyrieType::Integer128 { .. }
                | ValkyrieType::Float32
                | ValkyrieType::Float64
        ) {
            return self.lower_fallback_pattern_match(HirPattern::Range { start: start.cloned(), end: end.cloned(), inclusive_end }, value);
        }

        let mut accumulated: Option<MirOperand> = None;
        if let Some(start_literal) = start {
            let (start_constant, _) = lower_literal(start_literal, Some(&actual_type));
            let lower_bound = self.lower_compare_constant_operand(value.clone(), start_constant, MirBuiltinCompareOp::Ge);
            accumulated = Some(self.merge_pattern_match_operands(accumulated, lower_bound));
        }
        if let Some(end_literal) = end {
            let (end_constant, _) = lower_literal(end_literal, Some(&actual_type));
            let upper_bound = self.lower_compare_constant_operand(
                value,
                end_constant,
                if inclusive_end { MirBuiltinCompareOp::Le } else { MirBuiltinCompareOp::Lt },
            );
            accumulated = Some(self.merge_pattern_match_operands(accumulated, upper_bound));
        }
        accumulated.unwrap_or(MirOperand::Constant(MirConstant::Bool(true)))
    }

    pub(super) fn lower_object_pattern_match_operand(
        &mut self,
        name: &Option<NamePath>,
        fields: &[(Identifier, HirPattern)],
        value: MirOperand,
    ) -> MirOperand {
        let Some(actual_type) = infer_builder_operand_type(&value, &self.value_types)
        else {
            return self.lower_fallback_pattern_match(HirPattern::Object { name: name.clone(), fields: fields.to_vec(), rest: None }, value);
        };

        if let Some(name) = name {
            if !self.type_pattern_matches(&actual_type, name) {
                return MirOperand::Constant(MirConstant::Bool(false));
            }
        }

        if fields.is_empty() {
            return MirOperand::Constant(MirConstant::Bool(true));
        }

        let Some(struct_name) = named_type_name(&actual_type)
        else {
            return MirOperand::Constant(MirConstant::Bool(false));
        };

        if !self.has_known_struct_layout(struct_name) {
            return self.lower_fallback_pattern_match(HirPattern::Object { name: name.clone(), fields: fields.to_vec(), rest: None }, value);
        }

        let mut accumulated: Option<MirOperand> = None;
        for (field_name, field_pattern) in fields {
            if !self.struct_has_field(struct_name, field_name.as_str()) {
                return MirOperand::Constant(MirConstant::Bool(false));
            }
            let field_operand = self.lower_object_field_operand(value.clone(), struct_name, field_name);
            let field_match = self.lower_pattern_match_operand(field_pattern, field_operand);
            accumulated = Some(self.merge_pattern_match_operands(accumulated, field_match));
        }

        accumulated.unwrap_or(MirOperand::Constant(MirConstant::Bool(true)))
    }

    pub(super) fn lower_or_pattern_match_operand(&mut self, patterns: &[HirPattern], value: MirOperand) -> MirOperand {
        let mut accumulated: Option<MirOperand> = None;
        for pattern in patterns {
            let candidate = self.lower_pattern_match_operand(pattern, value.clone());
            accumulated = Some(self.merge_pattern_match_any_operands(accumulated, candidate));
        }
        accumulated.unwrap_or(MirOperand::Constant(MirConstant::Bool(false)))
    }

    pub(super) fn has_known_struct_layout(&self, struct_name: &str) -> bool {
        self.struct_field_layouts.contains_key(struct_name) || self.struct_parent_index.contains_key(struct_name)
    }

    fn lower_extractor_nested_pattern_match_operand(
        &mut self,
        extractor: &HirExtractorPattern,
        payload: MirOperand,
        matched: MirOperand,
        resolved: &HirResolvedCall,
    ) -> MirOperand {
        let mut accumulated = Some(matched);
        match extractor {
            HirExtractorPattern::Constructor { fields, .. } => {
                for (slot, field_pattern) in fields.iter().enumerate() {
                    let extracted =
                        self.lower_extractor_payload_slot_operand(payload.clone(), slot, self.infer_extractor_slot_type(resolved, slot));
                    let item_match = self.lower_pattern_match_operand(field_pattern, extracted);
                    accumulated = Some(self.merge_pattern_match_operands(accumulated, item_match));
                }
            }
            HirExtractorPattern::Array { prefix, suffix, .. } => {
                for (slot, field_pattern) in prefix.iter().enumerate() {
                    let extracted =
                        self.lower_extractor_payload_slot_operand(payload.clone(), slot, self.infer_extractor_slot_type(resolved, slot));
                    let item_match = self.lower_pattern_match_operand(field_pattern, extracted);
                    accumulated = Some(self.merge_pattern_match_operands(accumulated, item_match));
                }
                let suffix_base = prefix.len()
                    + match extractor {
                        HirExtractorPattern::Array { rest, .. } if rest.is_some() => 1,
                        _ => 0,
                    };
                for (index, field_pattern) in suffix.iter().enumerate() {
                    let slot = suffix_base + index;
                    let extracted =
                        self.lower_extractor_payload_slot_operand(payload.clone(), slot, self.infer_extractor_slot_type(resolved, slot));
                    let item_match = self.lower_pattern_match_operand(field_pattern, extracted);
                    accumulated = Some(self.merge_pattern_match_operands(accumulated, item_match));
                }
            }
        }
        accumulated.unwrap_or(MirOperand::Constant(MirConstant::Bool(true)))
    }

    pub(super) fn lower_extractor_call_operand(&mut self, resolved: &HirResolvedCall, value: MirOperand) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(resolved.symbol.clone()),
                arguments: vec![value.clone()],
                builtin: None,
                witness: None,
                effect: None,
            },
        });
        self.value_types.insert(output, resolved.return_type.clone());
        MirOperand::Value(output)
    }

    fn lower_extractor_matched_operand(&mut self, payload: MirOperand) -> MirOperand {
        self.lower_extractor_payload_slot_operand(payload, usize::MAX, Some(ValkyrieType::Boolean))
    }

    fn lower_extractor_payload_slot_operand(&mut self, payload: MirOperand, slot: usize, expected_type: Option<ValkyrieType>) -> MirOperand {
        let tuple_index = if slot == usize::MAX { 0 } else { slot + 1 };
        let extracted = self.lower_static_call(&format!("tuple_get_{tuple_index}"), vec![payload], MirValueOrigin::Temporary);
        if let Some(expected_type) = expected_type {
            self.value_types.insert(extracted, expected_type);
        }
        MirOperand::Value(extracted)
    }

    fn infer_extractor_slot_type(&self, resolved: &HirResolvedCall, slot: usize) -> Option<ValkyrieType> {
        match &resolved.return_type {
            ValkyrieType::Tuple(items) => items.get(slot + 1).cloned(),
            _ => None,
        }
    }

    pub(super) fn lower_object_field_operand(&mut self, value: MirOperand, struct_name: &str, field_name: &Identifier) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions
            .push(MirInstruction { output: Some(output), kind: MirInstructionKind::FieldGet { object: value, field: field_name.to_string() } });
        if let Some(field_type) = self.lookup_struct_field_type(struct_name, field_name.as_str()) {
            self.value_types.insert(output, field_type);
        }
        MirOperand::Value(output)
    }

    pub(super) fn lower_compare_constant_operand(&mut self, lhs: MirOperand, rhs: MirConstant, op: MirBuiltinCompareOp) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new(compare_builtin_name(op))])),
                arguments: vec![lhs, MirOperand::Constant(rhs)],
                builtin: Some(MirBuiltinCall::Compare(op)),
                witness: None,
                effect: None,
            },
        });
        self.value_types.insert(output, ValkyrieType::Boolean);
        MirOperand::Value(output)
    }

    #[allow(dead_code)]
    pub(super) fn lower_array_length_operand(&mut self, value: MirOperand) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new("length")])),
                arguments: vec![value],
                builtin: Some(MirBuiltinCall::ArrayLength),
                witness: None,
                effect: None,
            },
        });
        self.value_types.insert(output, ValkyrieType::Integer32 { signed: true });
        MirOperand::Value(output)
    }

    #[allow(dead_code)]
    pub(super) fn lower_array_get_operand(&mut self, array: MirOperand, index: MirOperand, item_type: Option<&ValkyrieType>) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new("suffix []")])),
                arguments: vec![array, index],
                builtin: Some(MirBuiltinCall::ArrayGet),
                witness: None,
                effect: None,
            },
        });
        if let Some(item_type) = item_type {
            self.value_types.insert(output, item_type.clone());
        }
        MirOperand::Value(output)
    }

    #[allow(dead_code)]
    pub(super) fn lower_int_sub_operand(&mut self, lhs: MirOperand, rhs: MirOperand) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new("infix -")])),
                arguments: vec![lhs, rhs],
                builtin: Some(MirBuiltinCall::BinaryNumeric(super::MirBuiltinBinaryOp::Sub)),
                witness: None,
                effect: None,
            },
        });
        self.value_types.insert(output, ValkyrieType::Integer32 { signed: true });
        MirOperand::Value(output)
    }

    pub(super) fn lower_logical_and_operand(&mut self, lhs: MirOperand, rhs: MirOperand) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new("infix &&")])),
                arguments: vec![lhs, rhs],
                builtin: Some(MirBuiltinCall::LogicalAnd),
                witness: None,
                effect: None,
            },
        });
        self.value_types.insert(output, ValkyrieType::Boolean);
        MirOperand::Value(output)
    }

    pub(super) fn lower_logical_or_operand(&mut self, lhs: MirOperand, rhs: MirOperand) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction {
            output: Some(output),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new("infix ||")])),
                arguments: vec![lhs, rhs],
                builtin: Some(MirBuiltinCall::LogicalOr),
                witness: None,
                effect: None,
            },
        });
        self.value_types.insert(output, ValkyrieType::Boolean);
        MirOperand::Value(output)
    }

    pub(super) fn merge_pattern_match_operands(&mut self, lhs: Option<MirOperand>, rhs: MirOperand) -> MirOperand {
        match (lhs, rhs) {
            (None, rhs) => rhs,
            (Some(MirOperand::Constant(MirConstant::Bool(false))), _) => MirOperand::Constant(MirConstant::Bool(false)),
            (Some(_), MirOperand::Constant(MirConstant::Bool(false))) => MirOperand::Constant(MirConstant::Bool(false)),
            (Some(MirOperand::Constant(MirConstant::Bool(true))), rhs) => rhs,
            (Some(lhs), MirOperand::Constant(MirConstant::Bool(true))) => lhs,
            (Some(lhs), rhs) => self.lower_logical_and_operand(lhs, rhs),
        }
    }

    pub(super) fn merge_pattern_match_any_operands(&mut self, lhs: Option<MirOperand>, rhs: MirOperand) -> MirOperand {
        match (lhs, rhs) {
            (None, rhs) => rhs,
            (Some(MirOperand::Constant(MirConstant::Bool(true))), _) => MirOperand::Constant(MirConstant::Bool(true)),
            (Some(_), MirOperand::Constant(MirConstant::Bool(true))) => MirOperand::Constant(MirConstant::Bool(true)),
            (Some(MirOperand::Constant(MirConstant::Bool(false))), rhs) => rhs,
            (Some(lhs), MirOperand::Constant(MirConstant::Bool(false))) => lhs,
            (Some(lhs), rhs) => self.lower_logical_or_operand(lhs, rhs),
        }
    }

    pub(super) fn lookup_struct_field_type(&self, struct_name: &str, field_name: &str) -> Option<ValkyrieType> {
        self.lookup_struct_field_type_recursive(struct_name, field_name, &mut BTreeSet::new())
    }

    pub(super) fn lookup_struct_field_type_recursive(
        &self,
        struct_name: &str,
        field_name: &str,
        visiting: &mut BTreeSet<String>,
    ) -> Option<ValkyrieType> {
        if !visiting.insert(struct_name.to_string()) {
            return None;
        }

        if let Some(field_type) = self
            .struct_field_layouts
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(name, _)| name == field_name))
            .map(|(_, ty)| ty.clone())
        {
            visiting.remove(struct_name);
            return Some(field_type);
        }

        let result = self
            .struct_parent_index
            .get(struct_name)
            .and_then(|parents| parents.iter().find_map(|parent| self.lookup_struct_field_type_recursive(parent, field_name, visiting)));
        visiting.remove(struct_name);
        result
    }

    pub(super) fn struct_has_field(&self, struct_name: &str, field_name: &str) -> bool {
        self.lookup_struct_field_type(struct_name, field_name).is_some()
    }

    pub(super) fn type_pattern_matches(&self, actual_type: &ValkyrieType, pattern_name: &NamePath) -> bool {
        let Some(expected_name) = pattern_name.parts().last().map(|identifier| identifier.as_str())
        else {
            return false;
        };

        match actual_type {
            ValkyrieType::Named(name) => self.named_or_parent_matches(name.as_str(), expected_name),
            ValkyrieType::Apply(base, _) => self.type_pattern_matches(base, pattern_name),
            _ => plain_type_pattern_matches(actual_type, pattern_name),
        }
    }

    pub(super) fn named_or_parent_matches(&self, actual_name: &str, expected_name: &str) -> bool {
        if actual_name == expected_name {
            return true;
        }

        self.struct_inherits_from(actual_name, expected_name, &mut BTreeSet::new())
    }

    pub(super) fn struct_inherits_from(&self, actual_name: &str, expected_name: &str, visiting: &mut BTreeSet<String>) -> bool {
        if !visiting.insert(actual_name.to_string()) {
            return false;
        }

        let found = self.struct_parent_index.get(actual_name).is_some_and(|parents| {
            parents.iter().any(|parent| parent == expected_name || self.struct_inherits_from(parent, expected_name, visiting))
        });
        visiting.remove(actual_name);
        found
    }

    /// extractor 绑定读取 extractor payload，而不是再把语法糖直接压成结构指令。
    pub(super) fn bind_extractor_pattern_from_payload(&mut self, extractor: &HirExtractorPattern, payload: MirOperand) {
        match extractor {
            HirExtractorPattern::Constructor { fields, .. } => {
                for (slot, field_pattern) in fields.iter().enumerate() {
                    let extracted = self.lower_extractor_payload_slot_operand(payload.clone(), slot, None);
                    self.bind_pattern_from_operand(field_pattern, extracted, None);
                }
            }
            HirExtractorPattern::Array { prefix, rest, suffix, .. } => {
                let item_hint = infer_builder_operand_type(&payload, &self.value_types).and_then(|ty| match ty {
                    ValkyrieType::Tuple(items) => items.get(1).cloned(),
                    _ => None,
                });
                for (slot, field_pattern) in prefix.iter().enumerate() {
                    let extracted = self.lower_extractor_payload_slot_operand(payload.clone(), slot, item_hint.clone());
                    self.bind_pattern_from_operand(field_pattern, extracted, item_hint.clone());
                }
                let mut next_slot = prefix.len();
                if let Some(rest) = rest {
                    let rest_type = item_hint.clone().map(|item| ValkyrieType::Array(Box::new(item)));
                    let extracted = self.lower_extractor_payload_slot_operand(payload.clone(), next_slot, rest_type.clone());
                    self.bind_pattern_from_operand(&HirPattern::Variable(rest.clone()), extracted, rest_type);
                    next_slot += 1;
                }
                for (index, field_pattern) in suffix.iter().enumerate() {
                    let extracted = self.lower_extractor_payload_slot_operand(payload.clone(), next_slot + index, item_hint.clone());
                    self.bind_pattern_from_operand(field_pattern, extracted, item_hint.clone());
                }
            }
        }
    }

    pub(super) fn bind_object_pattern_from_operand(
        &mut self,
        fields: &[(Identifier, HirPattern)],
        rest: Option<&valkyrie_types::hir::HirIdentifier>,
        operand: MirOperand,
    ) {
        let Some(actual_type) = infer_builder_operand_type(&operand, &self.value_types)
        else {
            for (_, field_pattern) in fields {
                self.bind_unknown_pattern_bindings(field_pattern);
            }
            if let Some(rest) = rest {
                self.bind_unknown_pattern_bindings(&HirPattern::Variable(rest.clone()));
            }
            return;
        };

        let Some(struct_name) = named_type_name(&actual_type)
        else {
            for (_, field_pattern) in fields {
                self.bind_unknown_pattern_bindings(field_pattern);
            }
            if let Some(rest) = rest {
                self.bind_unknown_pattern_bindings(&HirPattern::Variable(rest.clone()));
            }
            return;
        };

        if !self.has_known_struct_layout(struct_name) {
            for (_, field_pattern) in fields {
                self.bind_unknown_pattern_bindings(field_pattern);
            }
            if let Some(rest) = rest {
                self.bind_unknown_pattern_bindings(&HirPattern::Variable(rest.clone()));
            }
            return;
        }

        for (field_name, field_pattern) in fields {
            if !self.struct_has_field(struct_name, field_name.as_str()) {
                self.bind_unknown_pattern_bindings(field_pattern);
                continue;
            }

            let extracted = self.lower_object_field_operand(operand.clone(), struct_name, field_name);
            self.bind_pattern_from_operand(field_pattern, extracted, None);
        }
        if let Some(rest) = rest {
            self.bind_pattern_from_operand(&HirPattern::Variable(rest.clone()), operand, Some(actual_type));
        }
    }

    pub(super) fn bind_unknown_pattern_bindings(&mut self, pattern: &HirPattern) {
        match pattern {
            HirPattern::Wildcard | HirPattern::Else => {}
            HirPattern::Variable(identifier) => {
                self.bindings
                    .insert(identifier.name.to_string(), MirOperand::Symbol(NamePath::new(vec![Identifier::new("unsupported_pattern")])));
            }
            HirPattern::Tuple(items) => {
                for item in items {
                    self.bind_unknown_pattern_bindings(item);
                }
            }
            HirPattern::Extractor(extractor) => match extractor {
                HirExtractorPattern::Array { prefix, rest, suffix, .. } => {
                    for item in prefix {
                        self.bind_unknown_pattern_bindings(item);
                    }
                    if let Some(rest) = rest {
                        self.bind_unknown_pattern_bindings(&HirPattern::Variable(rest.clone()));
                    }
                    for item in suffix {
                        self.bind_unknown_pattern_bindings(item);
                    }
                }
                HirExtractorPattern::Constructor { fields, .. } => {
                    for field in fields {
                        self.bind_unknown_pattern_bindings(field);
                    }
                }
            },
            HirPattern::Object { fields, rest, .. } => {
                for (_, field_pattern) in fields {
                    self.bind_unknown_pattern_bindings(field_pattern);
                }
                if let Some(rest) = rest {
                    self.bind_unknown_pattern_bindings(&HirPattern::Variable(rest.clone()));
                }
            }
            HirPattern::TypedBind { identifier, .. } => {
                self.bind_unknown_pattern_bindings(&HirPattern::Variable(identifier.clone()));
            }
            _ => {}
        }
    }

    pub(super) fn lower_fallback_pattern_match(&mut self, pattern: HirPattern, value: MirOperand) -> MirOperand {
        let output = self.next_value(MirValueOrigin::Temporary);
        self.instructions.push(MirInstruction { output: Some(output), kind: MirInstructionKind::PatternMatch { value, pattern } });
        self.value_types.insert(output, ValkyrieType::Boolean);
        MirOperand::Value(output)
    }
}

fn extractor_resolved_call(extractor: &HirExtractorPattern) -> Option<&HirResolvedCall> {
    match extractor {
        HirExtractorPattern::Constructor { resolved, .. } | HirExtractorPattern::Array { resolved, .. } => resolved.as_ref(),
    }
}

fn tuple_literal_items(expr: &HirExpr) -> Option<&[HirExpr]> {
    match &expr.kind {
        HirExprKind::Call { callee, args, .. } if callee_name_matches(&callee.kind, "tuple") => Some(args.as_slice()),
        _ => None,
    }
}

fn compare_builtin_name(op: MirBuiltinCompareOp) -> &'static str {
    match op {
        MirBuiltinCompareOp::Eq => "infix ==",
        MirBuiltinCompareOp::Ne => "infix !=",
        MirBuiltinCompareOp::Lt => "infix <",
        MirBuiltinCompareOp::Le => "infix <=",
        MirBuiltinCompareOp::Gt => "infix >",
        MirBuiltinCompareOp::Ge => "infix >=",
    }
}

use valkyrie_types::{
    hir::{HirExpr, HirExprKind, HirLiteral},
    SourceID, SourceSpan,
};

use super::{infer_builder_operand_type, LoopContext, MirBuilder, MirConstant, MirOperand, MirTerminator};

impl MirBuilder {
    pub(super) fn lower_if_expr(
        &mut self,
        condition: &HirExpr,
        then_branch: &crate::hir::HirBlock,
        else_branch: &Option<Box<crate::hir::HirBlock>>,
    ) -> MirOperand {
        let cond_val = self.lower_expr_to_operand(condition);
        let pre_if_bindings = self.bindings.clone();
        let cond_block_id = self.current_block;

        let then_block = self.new_block("then");
        let else_block = self.new_block("else");
        let merge_block = self.new_block("merge");

        self.current_block = cond_block_id;
        self.terminate(MirTerminator::Branch { condition: cond_val, then_target: then_block, else_target: else_block });
        self.flush_block("cond");

        self.current_block = then_block;
        self.bindings = pre_if_bindings.clone();
        for statement in &then_branch.statements {
            self.lower_statement(statement);
            if self.terminator.is_some() {
                break;
            }
        }
        if let Some(tail) = &then_branch.expr {
            let _ = self.lower_expr_to_operand(tail);
        }
        let then_returns = self.terminator.is_some();
        if self.terminator.is_none() {
            self.terminate(MirTerminator::Jump { target: merge_block, arguments: Vec::new() });
        }
        self.flush_block("then");

        self.current_block = else_block;
        self.bindings = pre_if_bindings.clone();
        let else_returns;
        if let Some(else_body) = else_branch {
            for statement in &else_body.statements {
                self.lower_statement(statement);
                if self.terminator.is_some() {
                    break;
                }
            }
            if let Some(tail) = &else_body.expr {
                let _ = self.lower_expr_to_operand(tail);
            }
            else_returns = self.terminator.is_some();
        }
        else {
            else_returns = false;
        }
        if self.terminator.is_none() {
            self.terminate(MirTerminator::Jump { target: merge_block, arguments: Vec::new() });
        }
        self.flush_block("else");

        self.current_block = merge_block;
        self.bindings = pre_if_bindings;
        if then_returns && else_returns {
            self.terminate(MirTerminator::Unreachable);
        }

        MirOperand::Constant(MirConstant::Unit)
    }

    pub(super) fn lower_block_expr(&mut self, body: &crate::hir::HirBlock) -> MirOperand {
        let saved_bindings = self.bindings.clone();
        let saved_static_bindings = self.static_bindings.clone();

        for statement in &body.statements {
            self.lower_statement(statement);
            if self.terminator.is_some() {
                break;
            }
        }

        let result = if self.terminator.is_none() {
            body.expr.as_ref().map(|expr| self.lower_expr_to_operand(expr)).unwrap_or(MirOperand::Constant(MirConstant::Unit))
        }
        else {
            MirOperand::Constant(MirConstant::Unit)
        };

        self.bindings = saved_bindings;
        self.static_bindings = saved_static_bindings;
        result
    }

    pub(super) fn lower_loop_expr(
        &mut self,
        label: &Option<valkyrie_types::Identifier>,
        pattern: &Option<crate::hir::HirPattern>,
        iterator: &Option<Box<HirExpr>>,
        condition: &Option<Box<HirExpr>>,
        body: &crate::hir::HirBlock,
    ) -> MirOperand {
        if let (Some(loop_pattern), Some(iterator_expr)) = (pattern.as_ref(), iterator.as_deref()) {
            if let Some(iteration_items) = self.resolve_static_iterable_items(iterator_expr) {
                for item in iteration_items {
                    self.bind_pattern_from_expr(loop_pattern, &item, None);
                    for statement in &body.statements {
                        self.lower_statement(statement);
                        if self.terminator.is_some() {
                            break;
                        }
                    }
                    if self.terminator.is_some() {
                        break;
                    }
                    if let Some(tail) = &body.expr {
                        let _ = self.lower_expr_to_operand(tail);
                    }
                    if self.terminator.is_some() {
                        break;
                    }
                }
                return MirOperand::Constant(MirConstant::Unit);
            }
        }

        let pre_loop_bindings = self.bindings.clone();
        let outer_block_id = self.current_block;
        let loop_header_id = super::MirBlockRef(self.blocks.len() as u32);

        let outer_label = self.current_label.clone();
        self.terminate(MirTerminator::Jump { target: loop_header_id, arguments: Vec::new() });
        self.flush_block(&outer_label);
        self.terminator = None;

        self.new_block("loop_header");
        let loop_body_id = self.new_block("loop_body");
        let loop_exit_id = self.new_block("loop_exit");

        self.loop_stack.push(LoopContext {
            label: label.as_ref().map(|value| value.to_string()),
            header: loop_header_id,
            exit: loop_exit_id,
            exit_value: None,
            carried_values: Vec::new(),
            carried_value_refs: std::collections::BTreeMap::new(),
        });
        self.current_block = loop_body_id;
        self.current_label = "loop_body".to_string();
        self.instructions.clear();
        self.terminator = None;
        self.bindings = pre_loop_bindings.clone();

        for statement in &body.statements {
            self.lower_statement(statement);
            if self.terminator.is_some() {
                break;
            }
        }
        if let Some(tail) = &body.expr {
            let _ = self.lower_expr_to_operand(tail);
        }

        if self.terminator.is_none() {
            if self.current_label.starts_with("after_") {
                self.terminate(MirTerminator::Unreachable);
            }
            else {
                self.terminate(MirTerminator::Jump { target: loop_header_id, arguments: Vec::new() });
            }
        }
        let first_pass_body_label = self.current_label.clone();
        self.flush_block(&first_pass_body_label);
        self.terminator = None;

        let carried_bindings_after = self.bindings.clone();
        let carried_names: Vec<String> =
            pre_loop_bindings.keys().filter(|name| carried_bindings_after.get(*name) != pre_loop_bindings.get(*name)).cloned().collect();

        let _temp_context = self.loop_stack.pop().expect("temp loop context");

        let mut carried_value_refs: std::collections::BTreeMap<String, super::MirValueRef> = std::collections::BTreeMap::new();
        for name in &carried_names {
            let ty = pre_loop_bindings.get(name).and_then(|op| {
                if let MirOperand::Value(v) = op {
                    self.value_types.get(v).cloned()
                }
                else {
                    None
                }
            });
            let value = self.next_value(super::MirValueOrigin::BlockParameter { block: loop_header_id, name: name.to_string() });
            if let Some(ty) = ty {
                self.value_types.insert(value, ty);
            }
            self.blocks[loop_header_id.0 as usize].parameters.push(value);
            carried_value_refs.insert(name.to_string(), value);
        }

        let carried_args: Vec<MirOperand> = carried_names.iter().filter_map(|name| pre_loop_bindings.get(name).cloned()).collect();
        if let Some(outer_block) = self.blocks.get(outer_block_id.0 as usize) {
            if matches!(outer_block.terminator, MirTerminator::Jump { target, .. } if target == loop_header_id) {
                self.blocks[outer_block_id.0 as usize].terminator = MirTerminator::Jump { target: loop_header_id, arguments: carried_args };
            }
        }

        self.current_block = loop_header_id;
        self.current_label = "loop_header".to_string();
        self.instructions.clear();
        self.terminator = None;
        self.bindings = pre_loop_bindings.clone();
        for name in &carried_names {
            if let Some(&param) = carried_value_refs.get(name) {
                self.bindings.insert(name.clone(), MirOperand::Value(param));
            }
        }

        let cond_val = self.lower_expr_to_operand(condition.as_ref().unwrap_or(&Box::new(HirExpr {
            kind: HirExprKind::Literal(HirLiteral::Bool(true)),
            span: SourceSpan::new(SourceID::default(), 0, 0),
        })));
        self.terminate(MirTerminator::Branch { condition: cond_val, then_target: loop_body_id, else_target: loop_exit_id });
        self.flush_block("loop_header");
        self.terminator = None;

        self.loop_stack.push(LoopContext {
            label: label.as_ref().map(|value| value.to_string()),
            header: loop_header_id,
            exit: loop_exit_id,
            exit_value: None,
            carried_values: carried_names.clone(),
            carried_value_refs: carried_value_refs.clone(),
        });
        self.current_block = loop_body_id;
        self.current_label = "loop_body".to_string();
        self.instructions.clear();
        self.terminator = None;
        self.bindings = pre_loop_bindings.clone();
        for name in &carried_names {
            if let Some(&param) = carried_value_refs.get(name) {
                self.bindings.insert(name.clone(), MirOperand::Value(param));
            }
        }

        for statement in &body.statements {
            self.lower_statement(statement);
        }
        if let Some(tail) = &body.expr {
            let _ = self.lower_expr_to_operand(tail);
        }

        let loop_context = self.loop_stack.pop().expect("loop context should exist");

        if self.terminator.is_none() {
            if self.current_label.starts_with("after_") {
                self.terminate(MirTerminator::Unreachable);
            }
            else {
                let continue_args: Vec<MirOperand> = carried_names.iter().filter_map(|name| self.bindings.get(name).cloned()).collect();
                self.terminate(MirTerminator::Jump { target: loop_header_id, arguments: continue_args });
            }
        }
        let second_pass_body_label = self.current_label.clone();
        self.flush_block(&second_pass_body_label);
        self.terminator = None;

        self.current_block = loop_exit_id;
        self.current_label = "loop_exit".to_string();
        self.instructions.clear();
        self.terminator = None;
        self.bindings = pre_loop_bindings;

        loop_context.exit_value.map(MirOperand::Value).unwrap_or(MirOperand::Constant(MirConstant::Unit))
    }

    pub(super) fn lower_break_expr(&mut self, label: &Option<valkyrie_types::Identifier>, expr: &Option<Box<HirExpr>>) -> MirOperand {
        if let Some(loop_index) = self.resolve_loop_index(label.as_ref()) {
            let exit = self.loop_stack[loop_index].exit;
            let arguments = if let Some(expr) = expr.as_deref() {
                let operand = self.lower_expr_to_operand(expr);
                let ty = infer_builder_operand_type(&operand, &self.value_types);
                let _ = self.ensure_loop_exit_parameter(loop_index, ty);
                vec![operand]
            }
            else if self.loop_stack[loop_index].exit_value.is_some() {
                vec![MirOperand::Constant(MirConstant::Unit)]
            }
            else {
                Vec::new()
            };
            self.terminate(MirTerminator::Jump { target: exit, arguments });
        }
        else {
            self.terminate(MirTerminator::Unreachable);
        }
        MirOperand::Constant(MirConstant::Unit)
    }

    pub(super) fn lower_continue_expr(&mut self, label: &Option<valkyrie_types::Identifier>) -> MirOperand {
        if let Some(loop_index) = self.resolve_loop_index(label.as_ref()) {
            let loop_context = self.loop_stack[loop_index].clone();
            let LoopContext { header, carried_values, .. } = loop_context;
            let arguments: Vec<MirOperand> = carried_values.iter().filter_map(|name| self.bindings.get(name).cloned()).collect();
            self.terminate(MirTerminator::Jump { target: header, arguments });
        }
        else {
            self.terminate(MirTerminator::Unreachable);
        }
        MirOperand::Constant(MirConstant::Unit)
    }
}

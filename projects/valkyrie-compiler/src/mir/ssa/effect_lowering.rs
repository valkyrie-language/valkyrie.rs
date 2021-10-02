use super::*;

impl MirBuilder {
    fn record_suspend_point(
        &mut self,
        effect: MirEffectKind,
        suspend_block: MirBlockRef,
        resume_target: MirBlockRef,
        payload: Option<&MirOperand>,
        resume_parameter_count: usize,
        continuation_index: Option<usize>,
    ) {
        let payload_type = payload.and_then(|operand| infer_builder_operand_type(operand, &self.value_types));
        let spill_candidates: Vec<MirValueRef> = self
            .bindings
            .values()
            .filter_map(|operand| match operand {
                MirOperand::Value(value) => Some(*value),
                _ => None,
            })
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        let state_id = self.next_state_id();
        self.suspend_points.push(MirSuspendPoint {
            state_id,
            effect,
            suspend_block,
            resume_target,
            resume_parameter_count,
            payload_type,
            spill_candidates,
            continuation_index,
        });
    }

    fn lower_perform_effect(
        &mut self,
        label: &str,
        effect: MirEffectKind,
        payload: Option<MirOperand>,
        resume_parameter_type: Option<ValkyrieType>,
    ) -> MirOperand {
        let current_block = self.current_block;
        let current_label = self.current_label.clone();
        let resume_block = self.new_block(label);
        let resume_value = self.ensure_block_parameter(resume_block, label, resume_parameter_type);
        self.current_block = current_block;
        self.current_label = current_label.clone();
        let continuation_index = self.resume_stack.last().map(|ctx| ctx.continuation);
        self.record_suspend_point(effect, current_block, resume_block, payload.as_ref(), 1, continuation_index);
        self.terminate(MirTerminator::PerformEffect { effect, payload, resume_target: resume_block });
        self.flush_block(&current_label);
        self.current_block = resume_block;
        self.current_label = label.to_string();
        self.instructions.clear();
        self.terminator = None;
        MirOperand::Value(resume_value)
    }

    fn lower_perform_effect_without_resume_parameter(&mut self, label: &str, effect: MirEffectKind, payload: Option<MirOperand>) {
        let current_block = self.current_block;
        let current_label = self.current_label.clone();
        let resume_block = self.new_block(label);
        self.current_block = current_block;
        self.current_label = current_label.clone();
        let continuation_index = self.resume_stack.last().map(|ctx| ctx.continuation);
        self.record_suspend_point(effect, current_block, resume_block, payload.as_ref(), 0, continuation_index);
        self.terminate(MirTerminator::PerformEffect { effect, payload, resume_target: resume_block });
        self.flush_block(&current_label);
        self.current_block = resume_block;
        self.current_label = label.to_string();
        self.instructions.clear();
        self.terminator = None;
    }

    pub(super) fn lower_uncaught_raise(&mut self, payload: Option<MirOperand>) -> MirOperand {
        self.lower_perform_effect("raise_resume", MirEffectKind::Raise, payload, None)
    }

    pub(super) fn lower_handler_raise(&mut self, handler_index: usize, payload: Option<MirOperand>) -> MirOperand {
        let payload = payload.unwrap_or(MirOperand::Constant(MirConstant::Unit));
        let payload_type = infer_builder_operand_type(&payload, &self.value_types);
        let current_block = self.current_block;
        let current_label = self.current_label.clone();
        let dispatch_block = self.new_block("catch_dispatch");
        let resume_block = self.new_block("catch_resume");
        let resume_value = self.ensure_block_parameter(resume_block, "catch_resume", payload_type.clone());
        self.current_block = current_block;
        self.current_label = current_label.clone();
        self.terminate(MirTerminator::Jump { target: dispatch_block, arguments: vec![payload] });
        self.flush_block(&current_label);

        self.current_block = dispatch_block;
        self.current_label = "catch_dispatch".to_string();
        self.instructions.clear();
        self.terminator = None;
        let raised_value = self.ensure_block_parameter(dispatch_block, "raised_effect", payload_type.clone());

        let arms = self.handler_stack[handler_index].arms.clone();
        let handler_exit = self.handler_stack[handler_index].exit;
        let continuation_index = self.continuations.len();
        self.continuations.push(MirContinuation {
            dispatch_block,
            resume_target: resume_block,
            resume_parameter: resume_value,
            resume_parameter_type: payload_type.clone(),
            handler_exit,
        });
        if arms.is_empty() {
            self.terminate(MirTerminator::Jump { target: handler_exit, arguments: Vec::new() });
            self.flush_block("catch_dispatch");
        }
        else {
            let arm_blocks: Vec<MirBlockRef> =
                arms.iter().enumerate().map(|(index, _)| self.new_block(&format!("catch_arm_{index}"))).collect();
            let no_match_block = self.new_block("catch_no_match");

            self.current_block = dispatch_block;
            self.current_label = "catch_dispatch".to_string();
            self.instructions.clear();
            self.terminator = None;
            self.terminate(MirTerminator::Jump { target: arm_blocks[0], arguments: Vec::new() });
            self.flush_block("catch_dispatch");

            let saved_bindings = self.bindings.clone();
            let saved_static_bindings = self.static_bindings.clone();

            for (index, arm) in arms.iter().enumerate() {
                let arm_block = arm_blocks[index];
                let next_target = arm_blocks.get(index + 1).copied().unwrap_or(no_match_block);
                let arm_label = format!("catch_arm_{index}");
                let needs_pattern_check = !matches!(arm.pattern, HirPattern::Wildcard | HirPattern::Variable(_) | HirPattern::Else);

                self.current_block = arm_block;
                self.current_label = arm_label.clone();
                self.instructions.clear();
                self.terminator = None;
                self.bindings = saved_bindings.clone();
                self.static_bindings = saved_static_bindings.clone();

                if needs_pattern_check {
                    let saved_arm_block = self.current_block;
                    let saved_arm_label = self.current_label.clone();
                    let body_or_guard_block = self.new_block(&format!("catch_arm_{index}_match"));
                    self.current_block = saved_arm_block;
                    self.current_label = saved_arm_label;
                    self.instructions.clear();
                    self.terminator = None;
                    self.bindings = saved_bindings.clone();
                    self.static_bindings = saved_static_bindings.clone();
                    let matched = self.lower_pattern_match_operand(&arm.pattern, MirOperand::Value(raised_value));
                    self.terminate(MirTerminator::Branch { condition: matched, then_target: body_or_guard_block, else_target: next_target });
                    self.flush_block(&arm_label);
                    self.current_block = body_or_guard_block;
                    self.current_label = format!("catch_arm_{index}_match");
                    self.instructions.clear();
                    self.terminator = None;
                    self.bindings = saved_bindings.clone();
                    self.static_bindings = saved_static_bindings.clone();
                }

                if let Some(guard) = &arm.guard {
                    let saved_guard_block = self.current_block;
                    let saved_guard_label = self.current_label.clone();
                    let body_block = self.new_block(&format!("catch_arm_{index}_body"));
                    self.current_block = saved_guard_block;
                    self.current_label = saved_guard_label;
                    self.instructions.clear();
                    self.terminator = None;
                    self.bindings = saved_bindings.clone();
                    self.static_bindings = saved_static_bindings.clone();
                    self.bind_catch_arm_pattern(&arm.pattern, MirOperand::Value(raised_value));

                    self.suspended_handler_depth += 1;
                    let guard_value = self.lower_expr_to_operand(guard);
                    self.suspended_handler_depth = self.suspended_handler_depth.saturating_sub(1);
                    self.terminate(MirTerminator::Branch { condition: guard_value, then_target: body_block, else_target: next_target });
                    let guard_label = self.current_label.clone();
                    self.flush_block(&guard_label);

                    self.current_block = body_block;
                    self.current_label = format!("catch_arm_{index}_body");
                    self.instructions.clear();
                    self.terminator = None;
                }

                self.bind_catch_arm_pattern(&arm.pattern, MirOperand::Value(raised_value));
                self.resume_stack.push(ResumeContext {
                    continuation: continuation_index,
                    target: resume_block,
                    parameter: resume_value,
                    parameter_name: "catch_resume",
                    parameter_type: payload_type.clone(),
                });
                self.suspended_handler_depth += 1;
                let arm_result = self.lower_expr_to_operand(&arm.body);
                self.suspended_handler_depth = self.suspended_handler_depth.saturating_sub(1);
                let _ = self.resume_stack.pop();
                if self.terminator.is_none() {
                    let ty = infer_builder_operand_type(&arm_result, &self.value_types);
                    let _ = self.ensure_handler_exit_parameter(handler_index, ty);
                    self.terminate(MirTerminator::Jump { target: handler_exit, arguments: vec![arm_result] });
                }
                let current_arm_label = self.current_label.clone();
                self.flush_block(&current_arm_label);
            }

            self.current_block = no_match_block;
            self.current_label = "catch_no_match".to_string();
            self.instructions.clear();
            self.terminator = None;
            let propagated = if handler_index > 0 {
                self.lower_handler_raise(handler_index - 1, Some(MirOperand::Value(raised_value)))
            }
            else {
                self.lower_uncaught_raise(Some(MirOperand::Value(raised_value)))
            };
            if self.terminator.is_none() {
                let ty = infer_builder_operand_type(&propagated, &self.value_types);
                let _ = self.ensure_handler_exit_parameter(handler_index, ty);
                self.terminate(MirTerminator::Jump { target: handler_exit, arguments: vec![propagated] });
            }
            let no_match_label = self.current_label.clone();
            self.flush_block(&no_match_label);
        }

        self.current_block = resume_block;
        self.current_label = "catch_resume".to_string();
        self.instructions.clear();
        self.terminator = None;
        MirOperand::Value(resume_value)
    }

    pub(super) fn lower_yield_expr(&mut self, value: Option<&HirExpr>) -> MirOperand {
        let payload = Some(value.map(|expr| self.lower_expr_to_operand(expr)).unwrap_or(MirOperand::Constant(MirConstant::Unit)));
        self.lower_perform_effect("yield_resume", MirEffectKind::Yield, payload, Some(ValkyrieType::Unit))
    }

    pub(super) fn lower_yield_from_expr(&mut self, value: &HirExpr) -> MirOperand {
        let payload = Some(self.lower_expr_to_operand(value));
        self.lower_perform_effect("yield_from_resume", MirEffectKind::DelegateYield, payload, Some(ValkyrieType::Unit))
    }

    pub(super) fn lower_await_expr(&mut self, value: &HirExpr) -> MirOperand {
        let payload = Some(self.lower_expr_to_operand(value));
        let resume_type =
            payload.as_ref().and_then(|operand| infer_builder_operand_type(operand, &self.value_types)).and_then(|ty| future_resume_type(&ty));
        self.lower_perform_effect("await_resume", MirEffectKind::Await, payload, resume_type)
    }

    pub(super) fn lower_awake_expr(&mut self, value: &HirExpr) -> MirOperand {
        let payload = Some(self.lower_expr_to_operand(value));
        self.lower_perform_effect_without_resume_parameter("awake_resume", MirEffectKind::AsyncSpawn, payload);
        MirOperand::Constant(MirConstant::Unit)
    }

    pub(super) fn lower_block_on_expr(&mut self, value: &HirExpr) -> MirOperand {
        let payload = Some(self.lower_expr_to_operand(value));
        let resume_type =
            payload.as_ref().and_then(|operand| infer_builder_operand_type(operand, &self.value_types)).and_then(|ty| future_resume_type(&ty));
        self.lower_perform_effect("block_resume", MirEffectKind::AsyncBlock, payload, resume_type)
    }

    pub(super) fn lower_raise_expr(&mut self, value: &HirExpr) -> MirOperand {
        let payload = Some(self.lower_expr_to_operand(value));
        if let Some(handler_index) = self.handler_stack.len().checked_sub(self.suspended_handler_depth.saturating_add(1)) {
            self.lower_handler_raise(handler_index, payload)
        }
        else {
            self.lower_uncaught_raise(payload)
        }
    }

    pub(super) fn lower_resume_expr(&mut self, value: &HirExpr) -> MirOperand {
        let resume_value = self.lower_expr_to_operand(value);
        if let Some(resume_context) = self.resume_stack.last().cloned() {
            let resume_type = infer_builder_operand_type(&resume_value, &self.value_types);
            if let Some(resume_type) = resume_type.clone() {
                self.continuations[resume_context.continuation].resume_parameter_type.get_or_insert(resume_type);
            }
            let parameter = self.ensure_block_parameter(
                resume_context.target,
                resume_context.parameter_name,
                resume_context.parameter_type.clone().or(resume_type.clone()),
            );
            debug_assert_eq!(parameter, resume_context.parameter);
            if let Some(resume_type) = resume_type {
                self.value_types.entry(resume_context.parameter).or_insert(resume_type);
            }
            self.terminate(MirTerminator::Jump { target: resume_context.target, arguments: vec![resume_value] });
            let label = self.current_label.clone();
            self.flush_block(&label);
            self.new_block("after_resume");
            self.instructions.clear();
            self.terminator = Some(MirTerminator::Unreachable);
        }
        else {
            self.terminate(MirTerminator::Unreachable);
            let label = self.current_label.clone();
            self.flush_block(&label);
            self.new_block("after_invalid_resume");
            self.instructions.clear();
            self.terminator = Some(MirTerminator::Unreachable);
        }
        MirOperand::Constant(MirConstant::Unit)
    }

    pub(super) fn lower_catch_expr(&mut self, expr: &HirExpr, arms: &[HirMatchArm]) -> MirOperand {
        let current_block = self.current_block;
        let current_label = self.current_label.clone();
        let exit_block = self.new_block("catch_exit");
        self.current_block = current_block;
        self.current_label = current_label.clone();
        self.instructions.clear();
        self.terminator = None;

        self.handler_stack.push(HandlerContext { arms: arms.to_vec(), exit: exit_block, exit_value: None });

        let expr_result = self.lower_expr_to_operand(expr);
        let handler_index = self.handler_stack.len() - 1;
        if self.terminator.is_none() {
            let ty = infer_builder_operand_type(&expr_result, &self.value_types);
            let _ = self.ensure_handler_exit_parameter(handler_index, ty);
            self.terminate(MirTerminator::Jump { target: exit_block, arguments: vec![expr_result] });
            let label = self.current_label.clone();
            self.flush_block(&label);
        }
        let handler_context = self.handler_stack.pop().expect("handler context should exist");

        self.current_block = exit_block;
        self.current_label = "catch_exit".to_string();
        self.instructions.clear();
        self.terminator = None;
        handler_context.exit_value.map(MirOperand::Value).unwrap_or(MirOperand::Constant(MirConstant::Unit))
    }
}

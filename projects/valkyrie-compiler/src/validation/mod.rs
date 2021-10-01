//! 跨 `HIR / MIR / LIR` 的编译器校验入口。

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    hir::validate_control_flow_module,
    lir::{LirEffectKind, LirFunction, LirModule, LirOperand, LirOperationKind, LirTerminator},
    mir::{MirConstant, MirEffectKind, MirFunction, MirInstructionKind, MirModule, MirOperand, MirTerminator, MirValueOrigin},
};
use valkyrie_parser::ParseError;
use valkyrie_types::hir::{HirModule, ValkyrieType};

/// 顶层控制流调度与一致性校验入口。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ControlFlowScheduler;

impl ControlFlowScheduler {
    /// 校验 `HIR` 控制流约束是否已闭合。
    pub fn validate_hir_module(module: &HirModule) -> Result<(), ParseError> {
        validate_control_flow_module(module)
    }

    /// 校验 `MIR` 控制流图的 block / terminator 一致性。
    pub fn validate_mir_module(module: &MirModule) -> Result<(), ParseError> {
        crate::mir::validation::validate_module(module)
    }

    /// 校验 `LIR` 控制流图的 block / terminator 一致性。
    pub fn validate_lir_module(module: &LirModule) -> Result<(), ParseError> {
        crate::lir::validation::validate_module(module)
    }

    /// 校验跨层级控制流骨架是否保持一致。
    pub fn validate_pipeline(hir: &HirModule, mir: &MirModule, lir: &LirModule) -> Result<(), ParseError> {
        let hir_functions: BTreeSet<&str> = hir.functions.iter().map(|function| function.name.as_str()).collect();
        let mir_functions: BTreeSet<&str> = mir.functions.iter().map(|function| function.symbol.as_str()).collect();
        let lir_functions: BTreeSet<&str> = lir.functions.iter().map(|function| function.symbol.as_str()).collect();

        if hir_functions != mir_functions || mir_functions != lir_functions {
            return Err(ParseError::invalid("控制流调度校验失败：`HIR / MIR / LIR` 的函数集合不一致"));
        }

        for mir_function in &mir.functions {
            let Some(lir_function) = lir.functions.iter().find(|candidate| candidate.symbol == mir_function.symbol)
            else {
                return Err(ParseError::invalid(format!("控制流调度校验失败：`LIR` 缺少函数 `{}`", mir_function.symbol)));
            };
            let mir_blocks: BTreeMap<_, _> = mir_function.blocks.iter().map(|block| (block.id, block)).collect();
            let lir_blocks: BTreeMap<_, _> = lir_function.blocks.iter().map(|block| (block.id, block)).collect();

            if mir_function.blocks.len() != lir_function.blocks.len() {
                return Err(ParseError::invalid(format!("控制流调度校验失败：函数 `{}` 的 `MIR / LIR` block 数量不一致", mir_function.symbol)));
            }

            for (mir_block, lir_block) in mir_function.blocks.iter().zip(lir_function.blocks.iter()) {
                if mir_block.id != lir_block.id
                    || mir_block.label != lir_block.label
                    || mir_block.parameters.len() != lir_block.parameters.len()
                {
                    return Err(ParseError::invalid(format!(
                        "控制流调度校验失败：函数 `{}` 的 block `{}` 参数形状不一致",
                        mir_function.symbol, mir_block.label
                    )));
                }

                validate_pipeline_block(
                    mir_function.symbol.as_str(),
                    mir_function,
                    lir_function,
                    mir_block,
                    lir_block,
                    &mir_blocks,
                    &lir_blocks,
                )?;
            }
            validate_pipeline_suspend_points(mir_function.symbol.as_str(), mir_function, lir_function, &mir_blocks, &lir_blocks)?;
            validate_pipeline_frame_layouts(mir_function.symbol.as_str(), mir_function, lir_function)?;
            validate_pipeline_continuations(mir_function.symbol.as_str(), mir_function, lir_function, &mir_blocks, &lir_blocks)?;
            validate_pipeline_case_chains(mir_function.symbol.as_str(), mir_function, lir_function, &mir_blocks, &lir_blocks)?;
        }

        Ok(())
    }
}

fn validate_pipeline_block(
    function_name: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_block: &crate::mir::MirBlock,
    lir_block: &crate::lir::LirBlock,
    mir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::mir::MirBlock>,
    lir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::lir::LirBlock>,
) -> Result<(), ParseError> {
    let mir_patterns: Vec<_> = mir_block
        .instructions
        .iter()
        .filter_map(|instruction| match &instruction.kind {
            MirInstructionKind::PatternMatch { pattern, .. } => Some(pattern),
            _ => None,
        })
        .collect();
    let lir_patterns: Vec<_> = lir_block
        .operations
        .iter()
        .filter_map(|operation| match &operation.kind {
            LirOperationKind::PatternMatch { pattern, .. } => Some(pattern),
            _ => None,
        })
        .collect();

    if mir_patterns != lir_patterns {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 block `{}` 在 `MIR / LIR` 间的 `PatternMatch` 序列不一致",
            mir_block.label
        )));
    }

    if mir_block.label == "catch_resume" {
        validate_pipeline_catch_resume_parameter_type(function_name, mir_function, lir_function, mir_block, lir_block)?;
    }

    match (&mir_block.terminator, &lir_block.terminator) {
        (MirTerminator::Return { .. }, LirTerminator::Return { .. }) | (MirTerminator::Unreachable, LirTerminator::Unreachable) => {}
        (
            MirTerminator::Jump { target: mir_target, arguments: mir_arguments },
            LirTerminator::Jump { target: lir_target, arguments: lir_arguments },
        ) => {
            if mir_target != lir_target || mir_arguments.len() != lir_arguments.len() {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的 block `{}` 在 `MIR / LIR` 间的 `Jump` 形状不一致",
                    mir_block.label
                )));
            }
            validate_pipeline_jump_argument_types(
                function_name,
                mir_function,
                lir_function,
                mir_block,
                lir_block,
                *mir_target,
                mir_arguments,
                lir_arguments,
                mir_blocks,
                lir_blocks,
            )?;
        }
        (
            MirTerminator::Branch { then_target: mir_then, else_target: mir_else, .. },
            LirTerminator::Branch { then_target: lir_then, else_target: lir_else, .. },
        ) => {
            if mir_then != lir_then || mir_else != lir_else {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的 block `{}` 在 `MIR / LIR` 间的 `Branch` 目标不一致",
                    mir_block.label
                )));
            }
        }
        (
            MirTerminator::PerformEffect { effect: mir_effect, payload: mir_payload, resume_target: mir_resume, .. },
            LirTerminator::PerformEffect { effect: lir_effect, payload: lir_payload, resume_target: lir_resume, .. },
        ) => {
            if !effect_kinds_match(*mir_effect, *lir_effect) || mir_resume != lir_resume {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的 block `{}` 在 `MIR / LIR` 间的 `PerformEffect` 形状不一致",
                    mir_block.label
                )));
            }
            validate_pipeline_effect_payload_shape(
                function_name,
                mir_block.label.as_str(),
                mir_function,
                lir_function,
                *mir_effect,
                mir_payload.as_ref(),
                lir_payload.as_ref(),
            )?;
            validate_pipeline_effect_resume_shape(
                function_name,
                mir_block.label.as_str(),
                mir_function,
                lir_function,
                *mir_effect,
                mir_payload.as_ref(),
                lir_payload.as_ref(),
                *mir_resume,
                mir_blocks,
                lir_blocks,
            )?;
        }
        _ => {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的 block `{}` 在 `MIR / LIR` 间的 terminator 类型不一致",
                mir_block.label
            )));
        }
    }

    Ok(())
}

fn validate_pipeline_catch_resume_parameter_type(
    function_name: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_block: &crate::mir::MirBlock,
    lir_block: &crate::lir::LirBlock,
) -> Result<(), ParseError> {
    let Some(mir_parameter) = mir_block.parameters.first()
    else {
        return Ok(());
    };
    let Some(lir_parameter) = lir_block.parameters.first()
    else {
        return Ok(());
    };
    if mir_parameter != lir_parameter {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 `catch_resume` block 参数引用在 `MIR / LIR` 间不一致"
        )));
    }
    let mir_type = mir_function.value_type(mir_parameter);
    let lir_type = lir_function.value_type(lir_parameter);
    if mir_type == lir_type {
        return Ok(());
    }
    Err(ParseError::invalid(format!(
        "控制流调度校验失败：函数 `{function_name}` 的 `catch_resume` 参数类型在 `MIR / LIR` 间不一致，`MIR` 为 `{}`，`LIR` 为 `{}`",
        mir_type.as_ref().map(display_type).unwrap_or_else(|| "unknown".to_string()),
        lir_type.as_ref().map(display_type).unwrap_or_else(|| "unknown".to_string())
    )))
}

fn validate_pipeline_continuations(
    function_name: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::mir::MirBlock>,
    lir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::lir::LirBlock>,
) -> Result<(), ParseError> {
    if mir_function.continuations.len() != lir_function.continuations.len() {
        return Err(ParseError::invalid(format!("控制流调度校验失败：函数 `{function_name}` 的 continuation 数量在 `MIR / LIR` 间不一致")));
    }
    for (index, (mir_continuation, lir_continuation)) in mir_function.continuations.iter().zip(lir_function.continuations.iter()).enumerate() {
        if mir_continuation.dispatch_block != lir_continuation.dispatch_block
            || mir_continuation.resume_target != lir_continuation.resume_target
            || mir_continuation.resume_parameter != lir_continuation.resume_parameter
            || mir_continuation.handler_exit != lir_continuation.handler_exit
        {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 continuation 结构在 `MIR / LIR` 间不一致",
                index + 1
            )));
        }
        if mir_continuation.resume_parameter_type != lir_continuation.resume_parameter_type {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 continuation 恢复类型在 `MIR / LIR` 间不一致，`MIR` 为 `{}`，`LIR` 为 `{}`",
                index + 1,
                mir_continuation
                    .resume_parameter_type
                    .as_ref()
                    .map(display_type)
                    .unwrap_or_else(|| "unknown".to_string()),
                lir_continuation
                    .resume_parameter_type
                    .as_ref()
                    .map(display_type)
                    .unwrap_or_else(|| "unknown".to_string())
            )));
        }
        validate_pipeline_continuation_target(
            function_name,
            index,
            mir_continuation.dispatch_block,
            mir_continuation.resume_target,
            mir_continuation.resume_parameter,
            mir_continuation.handler_exit,
            mir_blocks,
            "`MIR`",
        )?;
        validate_pipeline_continuation_target(
            function_name,
            index,
            lir_continuation.dispatch_block,
            lir_continuation.resume_target,
            lir_continuation.resume_parameter,
            lir_continuation.handler_exit,
            lir_blocks,
            "`LIR`",
        )?;
    }
    Ok(())
}

fn validate_pipeline_case_chains(
    function_name: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::mir::MirBlock>,
    lir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::lir::LirBlock>,
) -> Result<(), ParseError> {
    if mir_function.case_chains.len() != lir_function.case_chains.len() {
        return Err(ParseError::invalid(format!("控制流调度校验失败：函数 `{function_name}` 的 case chain 数量在 `MIR / LIR` 间不一致")));
    }
    for (index, (mir_case_chain, lir_case_chain)) in mir_function.case_chains.iter().zip(lir_function.case_chains.iter()).enumerate() {
        if mir_case_chain.dispatch_block != lir_case_chain.dispatch_block
            || mir_case_chain.first_arm != lir_case_chain.first_arm
            || mir_case_chain.no_match_block != lir_case_chain.no_match_block
            || mir_case_chain.exit_block != lir_case_chain.exit_block
            || mir_case_chain.produce_value != lir_case_chain.produce_value
            || mir_case_chain.arms.len() != lir_case_chain.arms.len()
        {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 case chain 结构在 `MIR / LIR` 间不一致",
                index + 1
            )));
        }
        if !mir_blocks.contains_key(&mir_case_chain.dispatch_block)
            || !mir_blocks.contains_key(&mir_case_chain.first_arm)
            || !mir_blocks.contains_key(&mir_case_chain.no_match_block)
            || !mir_blocks.contains_key(&mir_case_chain.exit_block)
            || !lir_blocks.contains_key(&lir_case_chain.dispatch_block)
            || !lir_blocks.contains_key(&lir_case_chain.first_arm)
            || !lir_blocks.contains_key(&lir_case_chain.no_match_block)
            || !lir_blocks.contains_key(&lir_case_chain.exit_block)
        {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 case chain 指向了不存在的 block",
                index + 1
            )));
        }
        for (arm_index, (mir_arm, lir_arm)) in mir_case_chain.arms.iter().zip(lir_case_chain.arms.iter()).enumerate() {
            if mir_arm.entry_block != lir_arm.entry_block
                || mir_arm.check_block != lir_arm.check_block
                || mir_arm.guard_block != lir_arm.guard_block
                || mir_arm.body_block != lir_arm.body_block
                || mir_arm.next_arm_target != lir_arm.next_arm_target
                || mir_arm.exit_target != lir_arm.exit_target
                || mir_arm.fallthrough_target != lir_arm.fallthrough_target
            {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 case chain 第 {} 个 arm 在 `MIR / LIR` 间不一致",
                    index + 1,
                    arm_index + 1
                )));
            }
        }
    }
    Ok(())
}

fn validate_pipeline_suspend_points(
    function_name: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::mir::MirBlock>,
    lir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::lir::LirBlock>,
) -> Result<(), ParseError> {
    if mir_function.suspend_points.len() != lir_function.suspend_points.len() {
        return Err(ParseError::invalid(format!("控制流调度校验失败：函数 `{function_name}` 的 suspend 点数量在 `MIR / LIR` 间不一致")));
    }
    for (index, (mir_suspend, lir_suspend)) in mir_function.suspend_points.iter().zip(lir_function.suspend_points.iter()).enumerate() {
        if mir_suspend.state_id != lir_suspend.state_id
            || !effect_kinds_match(mir_suspend.effect, lir_suspend.effect)
            || mir_suspend.suspend_block != lir_suspend.suspend_block
            || mir_suspend.resume_target != lir_suspend.resume_target
            || mir_suspend.resume_parameter_count != lir_suspend.resume_parameter_count
            || mir_suspend.payload_type != lir_suspend.payload_type
            || mir_suspend.spill_candidates != lir_suspend.spill_candidates
        {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 suspend 点元数据在 `MIR / LIR` 间不一致",
                index + 1
            )));
        }
        validate_pipeline_suspend_target(function_name, index, mir_suspend, mir_blocks, "`MIR`")?;
        validate_pipeline_suspend_target(function_name, index, lir_suspend, lir_blocks, "`LIR`")?;
    }
    Ok(())
}

fn validate_pipeline_frame_layouts(function_name: &str, mir_function: &MirFunction, lir_function: &LirFunction) -> Result<(), ParseError> {
    if mir_function.frame_layouts.len() != lir_function.frame_layouts.len() {
        return Err(ParseError::invalid(format!("控制流调度校验失败：函数 `{function_name}` 的 frame layout 数量在 `MIR / LIR` 间不一致")));
    }
    for (index, (mir_layout, lir_layout)) in mir_function.frame_layouts.iter().zip(lir_function.frame_layouts.iter()).enumerate() {
        if mir_layout.state_id != lir_layout.state_id
            || !effect_kinds_match(mir_layout.effect, lir_layout.effect)
            || mir_layout.resume_target != lir_layout.resume_target
            || mir_layout.slots.len() != lir_layout.slots.len()
        {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 frame layout 在 `MIR / LIR` 间不一致",
                index + 1
            )));
        }
        for (mir_slot, lir_slot) in mir_layout.slots.iter().zip(lir_layout.slots.iter()) {
            if mir_slot.slot_index != lir_slot.slot_index || mir_slot.value != lir_slot.value || mir_slot.value_type != lir_slot.value_type {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的第 {} 个 frame layout 槽位在 `MIR / LIR` 间不一致",
                    index + 1
                )));
            }
        }
    }
    Ok(())
}

fn validate_pipeline_suspend_target<TSuspendPoint, TBlock>(
    function_name: &str,
    index: usize,
    suspend_point: &TSuspendPoint,
    blocks: &BTreeMap<crate::mir::MirBlockRef, &TBlock>,
    layer: &str,
) -> Result<(), ParseError>
where
    TSuspendPoint: SuspendPointLike,
    TBlock: ContinuationBlockLike,
{
    if !blocks.contains_key(&suspend_point.suspend_block()) {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 suspend 点指向了不存在的挂起 block",
            index + 1
        )));
    }
    let Some(resume_block) = blocks.get(&suspend_point.resume_target())
    else {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 suspend 点指向了不存在的恢复 block",
            index + 1
        )));
    };
    if resume_block.parameters().len() != suspend_point.resume_parameter_count() {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 suspend 点恢复参数个数为 {}，但目标恢复 block 实际有 {} 个参数",
            index + 1,
            suspend_point.resume_parameter_count(),
            resume_block.parameters().len()
        )));
    }
    Ok(())
}

fn validate_pipeline_continuation_target<TBlock>(
    function_name: &str,
    index: usize,
    dispatch_block: crate::mir::MirBlockRef,
    resume_target: crate::mir::MirBlockRef,
    resume_parameter: crate::mir::MirValueRef,
    handler_exit: crate::mir::MirBlockRef,
    blocks: &BTreeMap<crate::mir::MirBlockRef, &TBlock>,
    layer: &str,
) -> Result<(), ParseError>
where
    TBlock: ContinuationBlockLike,
{
    let Some(resume_block) = blocks.get(&resume_target)
    else {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 continuation 指向了不存在的 resume block",
            index + 1
        )));
    };
    if !blocks.contains_key(&dispatch_block) {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 continuation 指向了不存在的 dispatch block",
            index + 1
        )));
    }
    if !blocks.contains_key(&handler_exit) {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 continuation 指向了不存在的 handler exit block",
            index + 1
        )));
    }
    if !resume_block.parameters().contains(&resume_parameter) {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 {layer} 第 {} 个 continuation 恢复参数不属于目标 resume block",
            index + 1
        )));
    }
    Ok(())
}

trait ContinuationBlockLike {
    fn parameters(&self) -> &[crate::mir::MirValueRef];
}

trait SuspendPointLike {
    fn suspend_block(&self) -> crate::mir::MirBlockRef;
    fn resume_target(&self) -> crate::mir::MirBlockRef;
    fn resume_parameter_count(&self) -> usize;
}

impl ContinuationBlockLike for crate::mir::MirBlock {
    fn parameters(&self) -> &[crate::mir::MirValueRef] {
        &self.parameters
    }
}

impl ContinuationBlockLike for crate::lir::LirBlock {
    fn parameters(&self) -> &[crate::mir::MirValueRef] {
        &self.parameters
    }
}

impl SuspendPointLike for crate::mir::ssa::MirSuspendPoint {
    fn suspend_block(&self) -> crate::mir::MirBlockRef {
        self.suspend_block
    }

    fn resume_target(&self) -> crate::mir::MirBlockRef {
        self.resume_target
    }

    fn resume_parameter_count(&self) -> usize {
        self.resume_parameter_count
    }
}

impl SuspendPointLike for crate::lir::LirSuspendPoint {
    fn suspend_block(&self) -> crate::mir::MirBlockRef {
        self.suspend_block
    }

    fn resume_target(&self) -> crate::mir::MirBlockRef {
        self.resume_target
    }

    fn resume_parameter_count(&self) -> usize {
        self.resume_parameter_count
    }
}

fn validate_pipeline_jump_argument_types(
    function_name: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_block: &crate::mir::MirBlock,
    lir_block: &crate::lir::LirBlock,
    target: crate::mir::MirBlockRef,
    mir_arguments: &[MirOperand],
    lir_arguments: &[LirOperand],
    mir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::mir::MirBlock>,
    lir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::lir::LirBlock>,
) -> Result<(), ParseError> {
    let Some(mir_target_block) = mir_blocks.get(&target)
    else {
        return Ok(());
    };
    let Some(lir_target_block) = lir_blocks.get(&target)
    else {
        return Ok(());
    };

    for (index, ((mir_argument, lir_argument), (mir_parameter, lir_parameter))) in mir_arguments
        .iter()
        .zip(lir_arguments.iter())
        .zip(mir_target_block.parameters.iter().zip(lir_target_block.parameters.iter()))
        .enumerate()
    {
        let mir_argument_type = infer_mir_operand_static_type(mir_function, mir_argument);
        let lir_argument_type = infer_lir_operand_static_type(lir_function, lir_argument);
        if let (Some(mir_argument_type), Some(lir_argument_type)) = (mir_argument_type.as_ref(), lir_argument_type.as_ref()) {
            if mir_argument_type != lir_argument_type {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的 block `{}` 在 `MIR / LIR` 间第 {} 个 Jump 参数类型不一致，`MIR` 为 `{}`，`LIR` 为 `{}`",
                    mir_block.label,
                    index + 1,
                    display_type(mir_argument_type),
                    display_type(lir_argument_type)
                )));
            }
        }

        let mir_parameter_type = mir_function.value_type(mir_parameter);
        let lir_parameter_type = lir_function.value_type(lir_parameter);
        if let (Some(mir_argument_type), Some(mir_parameter_type)) = (mir_argument_type.as_ref(), mir_parameter_type.as_ref()) {
            if mir_argument_type != mir_parameter_type {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的 `MIR` block `{}` 跳向 `{}` 时，第 {} 个 Jump 参数类型为 `{}`，目标参数类型为 `{}`",
                    mir_block.label,
                    mir_target_block.label,
                    index + 1,
                    display_type(mir_argument_type),
                    display_type(mir_parameter_type)
                )));
            }
        }
        if let (Some(lir_argument_type), Some(lir_parameter_type)) = (lir_argument_type.as_ref(), lir_parameter_type.as_ref()) {
            if lir_argument_type != lir_parameter_type {
                return Err(ParseError::invalid(format!(
                    "控制流调度校验失败：函数 `{function_name}` 的 `LIR` block `{}` 跳向 `{}` 时，第 {} 个 Jump 参数类型为 `{}`，目标参数类型为 `{}`",
                    lir_block.label,
                    lir_target_block.label,
                    index + 1,
                    display_type(lir_argument_type),
                    display_type(lir_parameter_type)
                )));
            }
        }
    }
    Ok(())
}

fn effect_kinds_match(mir_effect: MirEffectKind, lir_effect: LirEffectKind) -> bool {
    matches!(
        (mir_effect, lir_effect),
        (MirEffectKind::Raise, LirEffectKind::Raise)
            | (MirEffectKind::Yield, LirEffectKind::Yield)
            | (MirEffectKind::DelegateYield, LirEffectKind::DelegateYield)
            | (MirEffectKind::Await, LirEffectKind::Await)
            | (MirEffectKind::AsyncSpawn, LirEffectKind::AsyncSpawn)
            | (MirEffectKind::AsyncBlock, LirEffectKind::AsyncBlock)
    )
}

fn expected_mir_effect_resume_parameter_count(effect: MirEffectKind) -> usize {
    match effect {
        MirEffectKind::AsyncSpawn => 0,
        MirEffectKind::Raise | MirEffectKind::Yield | MirEffectKind::DelegateYield | MirEffectKind::Await | MirEffectKind::AsyncBlock => 1,
    }
}

fn mir_effect_payload_required(effect: MirEffectKind) -> bool {
    let _ = effect;
    true
}

fn validate_pipeline_effect_resume_shape(
    function_name: &str,
    block_label: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_effect: MirEffectKind,
    mir_payload: Option<&MirOperand>,
    lir_payload: Option<&LirOperand>,
    resume_target: crate::mir::MirBlockRef,
    mir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::mir::MirBlock>,
    lir_blocks: &BTreeMap<crate::mir::MirBlockRef, &crate::lir::LirBlock>,
) -> Result<(), ParseError> {
    let Some(mir_resume_block) = mir_blocks.get(&resume_target)
    else {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 指向了不存在的 `MIR` 恢复点"
        )));
    };
    let Some(lir_resume_block) = lir_blocks.get(&resume_target)
    else {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 指向了不存在的 `LIR` 恢复点"
        )));
    };
    let expected_parameter_count = expected_mir_effect_resume_parameter_count(mir_effect);
    let mir_parameter_count = mir_resume_block.parameters.len();
    let lir_parameter_count = lir_resume_block.parameters.len();
    if mir_parameter_count != expected_parameter_count
        || lir_parameter_count != expected_parameter_count
        || mir_parameter_count != lir_parameter_count
    {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 在 `MIR / LIR` 间的 effect 恢复点参数形状不一致"
        )));
    }
    let mir_payload_type = mir_payload.and_then(|payload| infer_mir_operand_static_type(mir_function, payload));
    let lir_payload_type = lir_payload.and_then(|payload| infer_lir_operand_static_type(lir_function, payload));
    let expected_mir_resume_type = infer_effect_resume_static_type(mir_effect, mir_payload_type.as_ref());
    let expected_lir_resume_type = infer_effect_resume_static_type(mir_effect, lir_payload_type.as_ref());
    if let Some(expected_type) = expected_mir_resume_type.as_ref() {
        validate_resume_block_parameter_static_type("`MIR`", function_name, block_label, mir_function, *mir_resume_block, expected_type)?;
    }
    if let Some(expected_type) = expected_lir_resume_type.as_ref() {
        validate_resume_block_parameter_static_type("`LIR`", function_name, block_label, lir_function, *lir_resume_block, expected_type)?;
    }
    if let (Some(mir_expected), Some(lir_expected)) = (expected_mir_resume_type.as_ref(), expected_lir_resume_type.as_ref()) {
        if mir_expected != lir_expected {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 在 `MIR / LIR` 间的 effect 恢复值静态类型不一致：`{}` vs `{}`",
                display_type(mir_expected),
                display_type(lir_expected)
            )));
        }
    }
    Ok(())
}

fn validate_pipeline_effect_payload_shape(
    function_name: &str,
    block_label: &str,
    mir_function: &MirFunction,
    lir_function: &LirFunction,
    mir_effect: MirEffectKind,
    mir_payload: Option<&MirOperand>,
    lir_payload: Option<&LirOperand>,
) -> Result<(), ParseError> {
    let mir_has_payload = mir_payload.is_some();
    let lir_has_payload = lir_payload.is_some();
    let expected_has_payload = mir_effect_payload_required(mir_effect);
    if mir_has_payload != lir_has_payload {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 在 `MIR / LIR` 间的 effect payload 形状不一致"
        )));
    }
    if expected_has_payload && !mir_has_payload {
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 缺少必需的 effect payload"
        )));
    }
    if let Some(payload_type) = mir_payload.and_then(|payload| infer_mir_operand_static_type(mir_function, payload)) {
        validate_effect_payload_static_type("`MIR`", function_name, block_label, mir_effect, &payload_type)?;
    }
    if let Some(payload_type) = lir_payload.and_then(|payload| infer_lir_operand_static_type(lir_function, payload)) {
        validate_effect_payload_static_type("`LIR`", function_name, block_label, mir_effect, &payload_type)?;
    }
    if let (Some(mir_payload_type), Some(lir_payload_type)) = (
        mir_payload.and_then(|payload| infer_mir_operand_static_type(mir_function, payload)),
        lir_payload.and_then(|payload| infer_lir_operand_static_type(lir_function, payload)),
    ) {
        if mir_payload_type != lir_payload_type {
            return Err(ParseError::invalid(format!(
                "控制流调度校验失败：函数 `{function_name}` 的 block `{block_label}` 在 `MIR / LIR` 间的 effect payload 静态类型不一致：`{}` vs `{}`",
                display_type(&mir_payload_type),
                display_type(&lir_payload_type)
            )));
        }
    }
    Ok(())
}

fn validate_effect_payload_static_type(
    stage: &str,
    function_name: &str,
    block_label: &str,
    effect: MirEffectKind,
    payload_type: &ValkyrieType,
) -> Result<(), ParseError> {
    if matches!(effect, MirEffectKind::Await | MirEffectKind::AsyncSpawn | MirEffectKind::AsyncBlock)
        && future_resume_type(payload_type).is_none()
    {
        let effect_name = match effect {
            MirEffectKind::Await => "`await`",
            MirEffectKind::AsyncSpawn => "`awake`",
            MirEffectKind::AsyncBlock => "`block`",
            _ => unreachable!(),
        };
        return Err(ParseError::invalid(format!(
            "控制流调度校验失败：{stage} 函数 `{function_name}` 的 block `{block_label}` 中 {effect_name} 的 effect payload 类型 `{}` 不满足 `Future<T>` / `Promise<T>` 形状",
            display_type(payload_type)
        )));
    }
    Ok(())
}

fn infer_mir_operand_static_type(function: &MirFunction, operand: &MirOperand) -> Option<ValkyrieType> {
    match operand {
        MirOperand::Constant(constant) => Some(infer_effect_constant_type(constant)),
        MirOperand::Value(value_ref) => function.value_types.get(value_ref).cloned().or_else(|| {
            let value = function.values.iter().find(|value| value.id == *value_ref)?;
            match &value.origin {
                MirValueOrigin::Parameter { index, .. } => function.param_types.get(*index).cloned(),
                _ => None,
            }
        }),
        MirOperand::Symbol(_) => None,
    }
}

fn infer_lir_operand_static_type(function: &LirFunction, operand: &LirOperand) -> Option<ValkyrieType> {
    match operand {
        LirOperand::Constant(constant) => Some(infer_effect_constant_type(constant)),
        LirOperand::Value(value_ref) => function.value_types.get(value_ref).cloned(),
        LirOperand::Symbol(_) => None,
    }
}

fn infer_effect_resume_static_type(effect: MirEffectKind, payload_type: Option<&ValkyrieType>) -> Option<ValkyrieType> {
    match effect {
        MirEffectKind::Yield | MirEffectKind::DelegateYield => Some(ValkyrieType::Unit),
        MirEffectKind::Await | MirEffectKind::AsyncBlock => payload_type.and_then(future_resume_type),
        MirEffectKind::AsyncSpawn | MirEffectKind::Raise => None,
    }
}

fn validate_resume_block_parameter_static_type(
    stage: &str,
    function_name: &str,
    block_label: &str,
    function: &impl HasValueTypes,
    resume_block: &impl HasBlockParameters,
    expected_type: &ValkyrieType,
) -> Result<(), ParseError> {
    let Some(parameter) = resume_block.parameters().first()
    else {
        return Ok(());
    };
    let Some(actual_type) = function.value_type(parameter)
    else {
        return Ok(());
    };
    if actual_type == *expected_type {
        return Ok(());
    }
    Err(ParseError::invalid(format!(
        "控制流调度校验失败：{stage} 函数 `{function_name}` 的 block `{block_label}` 所指向恢复点参数类型为 `{}`，期望 `{}`",
        display_type(&actual_type),
        display_type(expected_type)
    )))
}

trait HasValueTypes {
    fn value_type(&self, value: &crate::mir::MirValueRef) -> Option<ValkyrieType>;
}

impl HasValueTypes for MirFunction {
    fn value_type(&self, value: &crate::mir::MirValueRef) -> Option<ValkyrieType> {
        self.value_types.get(value).cloned()
    }
}

impl HasValueTypes for LirFunction {
    fn value_type(&self, value: &crate::mir::MirValueRef) -> Option<ValkyrieType> {
        self.value_types.get(value).cloned()
    }
}

trait HasBlockParameters {
    fn parameters(&self) -> &[crate::mir::MirValueRef];
}

impl HasBlockParameters for crate::mir::MirBlock {
    fn parameters(&self) -> &[crate::mir::MirValueRef] {
        &self.parameters
    }
}

impl HasBlockParameters for crate::lir::LirBlock {
    fn parameters(&self) -> &[crate::mir::MirValueRef] {
        &self.parameters
    }
}

fn infer_effect_constant_type(constant: &MirConstant) -> ValkyrieType {
    match constant {
        MirConstant::Int(_) => ValkyrieType::Integer64 { signed: true },
        MirConstant::Float64(_) => ValkyrieType::Float64,
        MirConstant::Bool(_) => ValkyrieType::Boolean,
        MirConstant::String(_) => ValkyrieType::Utf8,
        MirConstant::Unit => ValkyrieType::Unit,
    }
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

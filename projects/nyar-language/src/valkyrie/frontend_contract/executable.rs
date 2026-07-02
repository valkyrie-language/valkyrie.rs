//! Convert language MIR into backend-private [`nyar_types::ExecutableFunction`] views.

use std::collections::BTreeMap;

use nyar::{NyarType, QualifiedName};
use nyar_types::{
    Block, BlockRef, CarrierTable, CaseArm, CaseChain, Constant, Continuation, Diagnostic, EffectKind, ExecutableFunction,
    FrameLayout, FrameSlot, Instruction, InstructionKind, Operand, SuspendLoweringPlan, SuspendPoint, SuspendState,
    Terminator, Value, ValueOrigin, ValueRef,
};

use crate::{
    ArrayInitialization, MirBlock, MirBlockRef, MirConstant, MirEffectKind, MirFunction, MirInstruction, MirOperation, MirOperand,
    MirTerminator, MirValue, MirValueOrigin, MirValueRef, concretize_type_lossy,
    mir::{
        continuation_runtime::{SuspendLoweringPlan as MirSuspendLoweringPlan, SuspendState as MirSuspendState},
        ssa::{MirCaseArm, MirCaseChain, MirContinuation, MirDiagnostic, MirFrameLayout, MirFrameSlot, MirSuspendPoint},
    },
};

impl From<&MirFunction> for ExecutableFunction {
    fn from(function: &MirFunction) -> Self {
        mir_function_to_executable(function)
    }
}

impl From<MirFunction> for ExecutableFunction {
    fn from(function: MirFunction) -> Self {
        mir_function_to_executable(&function)
    }
}

/// Deep-convert a language [`MirFunction`] into a platform [`ExecutableFunction`].
pub fn mir_function_to_executable(function: &MirFunction) -> ExecutableFunction {
    let return_type = concretize_type_lossy(&function.return_type);
    let param_types = function.param_types.iter().map(concretize_type_lossy).collect();
    let value_types = function.value_types.iter().map(|(key, ty)| (convert_value_ref(*key), concretize_type_lossy(ty))).collect();

    let suspend_plan = function.suspend_plan.as_ref().map(convert_suspend_plan);
    #[allow(deprecated)]
    let state_machine = function.state_machine.as_ref().map(convert_suspend_plan).or_else(|| suspend_plan.clone());

    ExecutableFunction {
        symbol: function.symbol.clone(),
        return_type,
        param_types,
        value_types,
        entry: convert_block_ref(function.entry),
        values: function.values.iter().map(convert_value).collect(),
        suspend_points: function.suspend_points.iter().map(convert_suspend_point).collect(),
        frame_layouts: function.frame_layouts.iter().map(convert_frame_layout).collect(),
        continuations: function.continuations.iter().map(convert_continuation).collect(),
        case_chains: function.case_chains.iter().map(convert_case_chain).collect(),
        state_machine,
        suspend_plan,
        blocks: function.blocks.iter().map(convert_block).collect(),
        diagnostics: function.diagnostics.iter().map(convert_diagnostic).collect(),
    }
}

/// Convert many MIR functions keyed by qualified name.
pub fn mir_functions_to_executable_map<'a, I>(functions: I) -> BTreeMap<QualifiedName, ExecutableFunction>
where
    I: IntoIterator<Item = (&'a QualifiedName, &'a MirFunction)>,
{
    functions.into_iter().map(|(name, function)| (name.clone(), mir_function_to_executable(function))).collect()
}

fn convert_value_ref(value: MirValueRef) -> ValueRef {
    ValueRef(value.0)
}

fn convert_block_ref(block: MirBlockRef) -> BlockRef {
    BlockRef(block.0)
}

fn convert_effect(effect: MirEffectKind) -> EffectKind {
    match effect {
        MirEffectKind::Raise => EffectKind::Raise,
        MirEffectKind::Yield => EffectKind::Yield,
        MirEffectKind::DelegateYield => EffectKind::DelegateYield,
        MirEffectKind::Await => EffectKind::Await,
        MirEffectKind::AsyncSpawn => EffectKind::AsyncSpawn,
        MirEffectKind::AsyncBlock => EffectKind::AsyncBlock,
    }
}



fn convert_optional_type(ty: &Option<crate::types::hir::ValkyrieType>) -> Option<NyarType> {
    ty.as_ref().map(concretize_type_lossy)
}

fn convert_constant(constant: &MirConstant) -> Constant {
    match constant {
        MirConstant::Int(value) => Constant::Int(*value),
        MirConstant::Float64(value) => Constant::Float64(*value),
        MirConstant::Bool(value) => Constant::Bool(*value),
        MirConstant::Utf8(value) => Constant::Utf8(value.clone()),
        MirConstant::Utf16(value) => Constant::Utf16(value.clone()),
        MirConstant::Unit => Constant::Unit,
    }
}

fn convert_operand(operand: &MirOperand) -> Operand {
    match operand {
        MirOperand::Value(value) => Operand::Value(convert_value_ref(*value)),
        MirOperand::Constant(constant) => Operand::Constant(convert_constant(constant)),
        MirOperand::Symbol(path) => Operand::Symbol(path.clone()),
    }
}

fn convert_value_origin(origin: &MirValueOrigin) -> ValueOrigin {
    match origin {
        MirValueOrigin::Parameter { index, name } => ValueOrigin::Parameter { index: *index, name: name.clone() },
        MirValueOrigin::BlockParameter { block, name } => ValueOrigin::BlockParameter { block: convert_block_ref(*block), name: name.clone() },
        MirValueOrigin::LetBinding { name } => ValueOrigin::LetBinding { name: name.clone() },
        MirValueOrigin::MutRefBinding { name } => ValueOrigin::MutRefBinding { name: name.clone() },
        MirValueOrigin::PinMutRefBinding { name } => ValueOrigin::PinMutRefBinding { name: name.clone() },
        MirValueOrigin::Literal => ValueOrigin::Literal,
        MirValueOrigin::Path => ValueOrigin::Path,
        MirValueOrigin::CallResult => ValueOrigin::CallResult,
        MirValueOrigin::Temporary => ValueOrigin::Temporary,
    }
}

fn convert_value(value: &MirValue) -> Value {
    Value { id: convert_value_ref(value.id), origin: convert_value_origin(&value.origin) }
}

fn convert_instruction_kind(kind: &MirOperation) -> InstructionKind {
    match kind {
        MirOperation::LoadConstant { constant, ty } => {
            InstructionKind::LoadConstant { constant: convert_constant(constant), ty: convert_optional_type(ty) }
        }
        MirOperation::LoadSymbol { path } => InstructionKind::LoadSymbol { path: path.clone() },
        MirOperation::Copy { source } => InstructionKind::Copy { source: convert_operand(source) },
        MirOperation::StoreVar { name, value, ty } => {
            InstructionKind::StoreVar { name: name.clone(), value: convert_operand(value), ty: convert_optional_type(ty) }
        }
        MirOperation::Call { callee, arguments } => InstructionKind::Call {
            callee: convert_operand(callee),
            arguments: arguments.iter().map(convert_operand).collect(),
        },
        MirOperation::StructNew { type_name, fields } => InstructionKind::StructNew {
            type_name: type_name.clone(),
            fields: fields.iter().map(|(name, value)| (name.clone(), convert_operand(value))).collect(),
        },
        MirOperation::TupleNew { fields, .. } => InstructionKind::TupleNew {
            fields: fields.iter().map(convert_operand).collect(),
        },
        MirOperation::AggregateCopy { source, dest } => {
            InstructionKind::AggregateCopy { source: convert_operand(source), dest: convert_operand(dest) }
        }
        MirOperation::FieldGet { object, field } => {
            InstructionKind::FieldGet { object: convert_operand(object), field: field.clone() }
        }
        MirOperation::FieldSet { object, field, value } => InstructionKind::FieldSet {
            object: convert_operand(object),
            field: field.clone(),
            value: convert_operand(value),
        },
        MirOperation::SumNew { sum_type, type_args, variant, payload_type, payload } => InstructionKind::SumNew {
            sum_type: sum_type.clone(),
            type_args: type_args.iter().map(concretize_type_lossy).collect(),
            variant: variant.clone(),
            payload_type: payload_type.as_ref().map(concretize_type_lossy),
            payload: payload.as_ref().map(convert_operand),
        },
        MirOperation::SumPayloadGet { sum_type, type_args, variant, payload_type, object } => InstructionKind::SumPayloadGet {
            sum_type: sum_type.clone(),
            type_args: type_args.iter().map(concretize_type_lossy).collect(),
            variant: variant.clone(),
            payload_type: concretize_type_lossy(payload_type),
            object: convert_operand(object),
        },
        MirOperation::SumVariantIs { sum_type, type_args, variant, object } => InstructionKind::SumVariantIs {
            sum_type: sum_type.clone(),
            type_args: type_args.iter().map(concretize_type_lossy).collect(),
            variant: variant.clone(),
            object: convert_operand(object),
        },
        MirOperation::PatternMatch { value, pattern } => {
            InstructionKind::PatternMatch { value: convert_operand(value), pattern_debug: format!("{pattern:?}") }
        }
        MirOperation::ArrayNew { array_type, length, initialization } => InstructionKind::ArrayNew {
            array_type: concretize_type_lossy(array_type),
            length: convert_operand(length),
            initialization: match initialization {
                ArrayInitialization::Default => nyar_types::executable::ArrayInitialization::Default,
                ArrayInitialization::Fill(value) => nyar_types::executable::ArrayInitialization::Fill(convert_operand(value)),
            },
        },
        MirOperation::ArrayFromElements { array_type, elements } => InstructionKind::ArrayFromElements {
            array_type: concretize_type_lossy(array_type),
            elements: elements.iter().map(convert_operand).collect(),
        },
        MirOperation::ArrayGet { array, index } => {
            InstructionKind::ArrayGet { array: convert_operand(array), index: convert_operand(index) }
        }
        MirOperation::ArraySet { array, index, value } => InstructionKind::ArraySet {
            array: convert_operand(array),
            index: convert_operand(index),
            value: convert_operand(value),
        },
        MirOperation::ArrayLength { array } => InstructionKind::ArrayLength { array: convert_operand(array) },
    }
}

fn convert_instruction(instruction: &MirInstruction) -> Instruction {
}

fn convert_terminator(terminator: &MirTerminator) -> Terminator {
    match terminator {
        MirTerminator::Return { value } => Terminator::Return { value: value.as_ref().map(convert_operand) },
        MirTerminator::Jump { target, arguments } => {
            Terminator::Jump { target: convert_block_ref(*target), arguments: arguments.iter().map(convert_operand).collect() }
        }
        MirTerminator::Branch { condition, then_target, else_target } => Terminator::Branch {
            condition: convert_operand(condition),
            then_target: convert_block_ref(*then_target),
            else_target: convert_block_ref(*else_target),
        },
        MirTerminator::PerformEffect { effect, payload, resume_target } => Terminator::PerformEffect {
            effect: convert_effect(*effect),
            payload: payload.as_ref().map(convert_operand),
            resume_target: convert_block_ref(*resume_target),
        },
        MirTerminator::StateDispatch { state, cases, default_target } => Terminator::StateDispatch {
            state: convert_value_ref(*state),
            cases: cases.iter().map(|(id, target)| (*id, convert_block_ref(*target))).collect(),
            default_target: convert_block_ref(*default_target),
        },
        MirTerminator::YieldToRuntime { effect, payload, resume_state } => Terminator::YieldToRuntime {
            effect: convert_effect(*effect),
            payload: payload.as_ref().map(convert_operand),
            resume_state: *resume_state,
        },
        MirTerminator::Unreachable => Terminator::Unreachable,
    }
}

fn convert_block(block: &MirBlock) -> Block {
    Block {
        id: convert_block_ref(block.id),
        label: block.label.clone(),
        parameters: block.parameters.iter().copied().map(convert_value_ref).collect(),
        instructions: block.instructions.iter().map(convert_instruction).collect(),
        terminator: convert_terminator(&block.terminator),
    }
}

fn convert_continuation(continuation: &MirContinuation) -> Continuation {
    Continuation {
        dispatch_block: convert_block_ref(continuation.dispatch_block),
        resume_target: convert_block_ref(continuation.resume_target),
        resume_parameter: convert_value_ref(continuation.resume_parameter),
        resume_parameter_type: convert_optional_type(&continuation.resume_parameter_type),
        handler_exit: convert_block_ref(continuation.handler_exit),
        carrier_type: convert_optional_type(&continuation.carrier_type),
    }
}

fn convert_case_arm(arm: &MirCaseArm) -> CaseArm {
    CaseArm {
        entry_block: convert_block_ref(arm.entry_block),
        check_block: arm.check_block.map(convert_block_ref),
        guard_block: arm.guard_block.map(convert_block_ref),
        body_block: convert_block_ref(arm.body_block),
        next_arm_target: convert_block_ref(arm.next_arm_target),
        exit_target: convert_block_ref(arm.exit_target),
        fallthrough_target: arm.fallthrough_target.map(convert_block_ref),
    }
}

fn convert_case_chain(chain: &MirCaseChain) -> CaseChain {
    CaseChain {
        dispatch_block: convert_block_ref(chain.dispatch_block),
        first_arm: convert_block_ref(chain.first_arm),
        no_match_block: convert_block_ref(chain.no_match_block),
        exit_block: convert_block_ref(chain.exit_block),
        produce_value: chain.produce_value,
        arms: chain.arms.iter().map(convert_case_arm).collect(),
    }
}

fn convert_suspend_point(point: &MirSuspendPoint) -> SuspendPoint {
    SuspendPoint {
        state_id: point.state_id,
        effect: convert_effect(point.effect),
        suspend_block: convert_block_ref(point.suspend_block),
        resume_target: convert_block_ref(point.resume_target),
        resume_parameter_count: point.resume_parameter_count,
        payload_type: convert_optional_type(&point.payload_type),
        spill_candidates: point.spill_candidates.iter().copied().map(convert_value_ref).collect(),
        continuation_index: point.continuation_index,
        carrier_type: convert_optional_type(&point.carrier_type),
    }
}

fn convert_frame_slot(slot: &MirFrameSlot) -> FrameSlot {
    FrameSlot { slot_index: slot.slot_index, value: convert_value_ref(slot.value), value_type: convert_optional_type(&slot.value_type) }
}

fn convert_frame_layout(layout: &MirFrameLayout) -> FrameLayout {
    FrameLayout {
        state_id: layout.state_id,
        effect: convert_effect(layout.effect),
        resume_target: convert_block_ref(layout.resume_target),
        slots: layout.slots.iter().map(convert_frame_slot).collect(),
        carrier_type: convert_optional_type(&layout.carrier_type),
    }
}

fn convert_suspend_state(state: &MirSuspendState) -> SuspendState {
    SuspendState {
        state_id: state.state_id,
        effect: convert_effect(state.effect),
        suspend_block: convert_block_ref(state.suspend_block),
        resume_target: convert_block_ref(state.resume_target),
        resume_parameter_count: state.resume_parameter_count,
        resume_parameter_type: convert_optional_type(&state.resume_parameter_type),
        payload_type: convert_optional_type(&state.payload_type),
        spill_slots: state.spill_slots.iter().copied().map(convert_value_ref).collect(),
        frame_carrier: state.frame_carrier.clone(),
        continuation_index: state.continuation_index,
    }
}

fn convert_suspend_plan(plan: &MirSuspendLoweringPlan) -> SuspendLoweringPlan {
    SuspendLoweringPlan {
        function_symbol: plan.function_symbol.clone(),
        entry_block: convert_block_ref(plan.entry_block),
        states: plan.states.iter().map(convert_suspend_state).collect(),
        handler_dispatch_blocks: plan.handler_dispatch_blocks.iter().copied().map(convert_block_ref).collect(),
        carrier_table: CarrierTable::new(plan.function_symbol.clone()),
    }
}

fn convert_diagnostic(diagnostic: &MirDiagnostic) -> Diagnostic {
    match diagnostic {
        MirDiagnostic::PatternLoweringFailed { reason, .. } => Diagnostic::PatternLoweringFailed { reason: reason.clone() },
        MirDiagnostic::UnsupportedExpression { kind, .. } => {
            Diagnostic::PatternLoweringFailed { reason: format!("unsupported HIR expression: {kind}") }
        }
    }
}

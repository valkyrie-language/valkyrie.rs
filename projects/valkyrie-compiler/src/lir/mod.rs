#![doc = include_str!("readme.md")]

pub mod validation;

use std::collections::BTreeMap;

use crate::{
    hir::ValkyrieCompiler,
    mir::{
        MirBlock, MirBlockRef, MirBuiltinCall, MirConstant, MirDispatchKind, MirEffectKind, MirFunction, MirInstruction, MirInstructionKind,
        MirLowerer, MirModule, MirOperand, MirTerminator, MirValueRef,
    },
    validation::ControlFlowScheduler,
};
use valkyrie_parser::ParseError;
use valkyrie_types::{
    hir::{HirModule, HirPattern, ValkyrieType as HirType},
    NamePath,
};

/// Target-aware low-level representation module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirModule {
    /// Logical module name.
    pub name: String,
    /// Target lowering lane selected for this low-level view.
    pub lane: LirTargetLane,
    /// 结构体定义，保留字段类型供后端继续推断 `FieldGet`/`FieldSet`。
    pub structs: Vec<crate::mir::MirStruct>,
    /// Functions lowered into the selected lane.
    pub functions: Vec<LirFunction>,
}

impl ValkyrieCompiler {
    /// Parses source text and lowers it into LIR.
    pub fn compile_source_to_lir(&self, source: &str) -> Result<LirModule, ParseError> {
        let hir = self.compile_source(source)?;
        let mir = MirLowerer::lower_module(&hir);
        ControlFlowScheduler::validate_mir_module(&mir)?;
        let lir = LirLowerer::lower_module(&hir);
        ControlFlowScheduler::validate_lir_module(&lir)?;
        ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir)?;
        Ok(lir)
    }
}

/// Low-level function body grouped by basic blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirFunction {
    /// Symbol name of the lowered function.
    pub symbol: String,
    /// 函数参数类型列表，用于后端生成调用约定与方法签名。
    pub param_types: Vec<HirType>,
    /// 函数返回类型，用于后端判断调用是否返回 `void`。
    pub return_type: HirType,
    /// `SSA` 值的静态类型表，从 `MIR` 透传，供调度校验使用。
    pub value_types: BTreeMap<MirValueRef, HirType>,
    /// suspend 点元数据，从 `MIR` 透传，供 lane lowering / frame 构造使用。
    pub suspend_points: Vec<LirSuspendPoint>,
    /// frame layout 计划，从 `MIR` 透传，供 lane/runtime lowering 直接消费。
    pub frame_layouts: Vec<LirFrameLayout>,
    /// continuation 元数据，从 `MIR` 透传，供 lane lowering 与调度校验使用。
    pub continuations: Vec<LirContinuation>,
    /// case/match 链路元数据，从 `MIR` 透传，供跨 arm merge 与 fallthrough 校验使用。
    pub case_chains: Vec<LirCaseChain>,
    /// 显式 runtime frame 载体，作为 lane/runtime 消费 `frame_layouts` 的稳定边界。
    pub runtime_frames: Vec<LirRuntimeFrame>,
    /// 显式 runtime continuation 载体，作为 handler/runtime 消费恢复协议的稳定边界。
    pub runtime_continuations: Vec<LirRuntimeContinuation>,
    /// Entry block of the function.
    pub entry: MirBlockRef,
    /// All basic blocks that belong to the function.
    pub blocks: Vec<LirBlock>,
}

/// Low-level basic block.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirBlock {
    /// Stable block identifier.
    pub id: MirBlockRef,
    /// Human-readable block label.
    pub label: String,
    /// Incoming block parameters.
    pub parameters: Vec<MirValueRef>,
    /// Non-terminating low-level operations.
    pub operations: Vec<LirOperation>,
    /// Explicit block terminator.
    pub terminator: LirTerminator,
}

/// 低层 continuation 元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirContinuation {
    /// 对应的 handler dispatch block。
    pub dispatch_block: MirBlockRef,
    /// continuation 恢复时跳回的 block。
    pub resume_target: MirBlockRef,
    /// 恢复值进入的 block parameter。
    pub resume_parameter: MirValueRef,
    /// 当前已知的恢复值类型。
    pub resume_parameter_type: Option<HirType>,
    /// handler 正常结束时汇入的 exit block。
    pub handler_exit: MirBlockRef,
}

/// 低层 `case / match` 链路元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirCaseChain {
    /// 进入 case-like lowering 的分发块。
    pub dispatch_block: MirBlockRef,
    /// 第一个 arm 的入口块。
    pub first_arm: MirBlockRef,
    /// 全部 arm 未匹配时跳入的块。
    pub no_match_block: MirBlockRef,
    /// case-like 控制流最终汇入的 exit 块。
    pub exit_block: MirBlockRef,
    /// 是否为值语义 `match`。
    pub produce_value: bool,
    /// 各个 arm 的显式链路信息。
    pub arms: Vec<LirCaseArm>,
}

/// 单个低层 `case / match arm` 的控制流元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirCaseArm {
    /// arm 入口块。
    pub entry_block: MirBlockRef,
    /// 可选的 pattern check 块。
    pub check_block: Option<MirBlockRef>,
    /// 可选的 guard 求值块。
    pub guard_block: Option<MirBlockRef>,
    /// arm body 实际执行块。
    pub body_block: MirBlockRef,
    /// pattern/guard 失败后跳向的下一目标。
    pub next_arm_target: MirBlockRef,
    /// 正常完成后汇入的 exit 目标。
    pub exit_target: MirBlockRef,
    /// `fallthrough` 允许时应跳入的下一 arm 入口。
    pub fallthrough_target: Option<MirBlockRef>,
}

/// 低层 suspend 点元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirSuspendPoint {
    /// 状态机 lowering 使用的显式状态编号。
    pub state_id: u32,
    /// 触发的 effect 类型。
    pub effect: LirEffectKind,
    /// 发生挂起的 block。
    pub suspend_block: MirBlockRef,
    /// 恢复时跳回的 block。
    pub resume_target: MirBlockRef,
    /// 恢复点参数个数。
    pub resume_parameter_count: usize,
    /// 当前已知 payload 的静态类型。
    pub payload_type: Option<HirType>,
    /// 后续 frame / spill lowering 可直接使用的候选 SSA 值。
    pub spill_candidates: Vec<MirValueRef>,
}

/// 低层 frame layout 计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirFrameLayout {
    /// 对应 suspend 点的状态编号。
    pub state_id: u32,
    /// 对应的 effect 类型。
    pub effect: LirEffectKind,
    /// 恢复时跳回的 block。
    pub resume_target: MirBlockRef,
    /// 需要保存的槽位布局。
    pub slots: Vec<LirFrameSlot>,
}

/// 低层 frame spill 槽位。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirFrameSlot {
    /// frame 中的稳定槽位序号。
    pub slot_index: usize,
    /// 被保存的 SSA 值。
    pub value: MirValueRef,
    /// 当前已知的槽位静态类型。
    pub value_type: Option<HirType>,
}

/// 面向 lane/runtime 的显式 frame 承载实体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirRuntimeFrame {
    /// 稳定的 carrier 名称，供后续 lane lowering 映射到具体对象/结构体。
    pub carrier: String,
    /// 对应 suspend 点的状态编号。
    pub state_id: u32,
    /// 恢复时跳回的目标 block。
    pub resume_target: MirBlockRef,
    /// frame 中需要承载的槽位字段。
    pub slots: Vec<LirRuntimeSlot>,
}

/// runtime frame 中的单个字段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirRuntimeSlot {
    /// 稳定的字段名。
    pub field_name: String,
    /// 对应 `frame_layout` 中的槽位序号。
    pub slot_index: usize,
    /// 被保存的 SSA 值。
    pub value: MirValueRef,
    /// 当前已知的槽位静态类型。
    pub value_type: Option<HirType>,
}

/// 面向 lane/runtime 的显式 continuation 承载实体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirRuntimeContinuation {
    /// 稳定的 carrier 名称，供 lane lowering 映射到具体 continuation 对象。
    pub carrier: String,
    /// 对应的 handler dispatch block。
    pub dispatch_block: MirBlockRef,
    /// continuation 恢复时跳回的 block。
    pub resume_target: MirBlockRef,
    /// 恢复值进入的 block parameter。
    pub resume_parameter: MirValueRef,
    /// continuation 载体中承载恢复值的稳定字段名。
    pub resume_parameter_field: String,
    /// 当前已知的恢复值类型。
    pub resume_parameter_type: Option<HirType>,
    /// handler 正常结束时汇入的 exit block。
    pub handler_exit: MirBlockRef,
}

/// Single low-level operation with optional result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirOperation {
    /// Result value produced by this operation, if any.
    pub output: Option<MirValueRef>,
    /// Concrete low-level operation kind.
    pub kind: LirOperationKind,
}

/// Target lane selected for a low-level lowering route.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirTargetLane {
    /// `CLR` lane.
    Clr,
    /// `JVM` lane.
    Jvm,
    /// `WASM` lane.
    Wasm,
    /// Native object/image lane.
    Native,
    /// `CPU / VM` lane.
    Vm,
}

/// Low-level operand after target-lane shaping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirOperand {
    /// SSA value produced inside the current function.
    Value(MirValueRef),
    /// Immediate constant.
    Constant(MirConstant),
    /// Symbolic global or referenced item.
    Symbol(NamePath),
}

/// Dispatch kind preserved from `MIR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirDispatchKind {
    /// Closed static call.
    Static,
    /// Witness-based call.
    Witness,
    /// Effect handler call.
    EffectHandler,
}

/// Effect terminator category preserved from `MIR`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LirEffectKind {
    /// `raise expr`
    Raise,
    /// `yield expr`
    Yield,
    /// `yield from expr`
    DelegateYield,
    /// `expr.await`
    Await,
    /// `expr.awake`
    AsyncSpawn,
    /// `expr.block`
    AsyncBlock,
}

/// Concrete low-level operation family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirOperationKind {
    /// Materializes an immediate constant.
    LoadConstant {
        /// Constant payload.
        constant: MirConstant,
        /// Optional expected type carried from earlier literal lowering.
        ty: Option<HirType>,
    },
    /// Materializes a named symbol.
    LoadSymbol {
        /// Resolved symbol path.
        path: NamePath,
    },
    /// Moves or copies an operand.
    Move {
        /// Source operand.
        source: LirOperand,
    },
    /// 存储到命名变量槽位（可变变量）。
    ///
    /// `name` 是变量名，`value` 是要存储的值。
    /// 同名 `StoreVar` 复用同一局部槽位，确保循环中 header 能读到最新值。
    StoreVar {
        /// 变量名。
        name: String,
        /// 要存储的值。
        value: LirOperand,
        /// 变量声明类型注解。
        ty: Option<HirType>,
    },
    /// Lane-aware call operation.
    Call {
        /// Dispatch strategy chosen upstream.
        dispatch: LirDispatchKind,
        /// Callee operand.
        callee: LirOperand,
        /// Positional arguments.
        arguments: Vec<LirOperand>,
        /// 已确认的内建调用语义。
        builtin: Option<MirBuiltinCall>,
        /// Optional witness operand.
        witness: Option<LirOperand>,
        /// Optional effect operand.
        effect: Option<LirOperand>,
    },
    /// Array allocation.
    ArrayNew {
        /// Array element type.
        element_type: HirType,
        /// Array length.
        length: LirOperand,
    },
    /// 数组字面量构造。
    ///
    /// 这是数组构造自身的内建语义，不借用通用 `[]=` 语法糖。
    ArrayLiteral {
        /// 数组元素类型。
        element_type: HirType,
        /// 元素列表。
        items: Vec<LirOperand>,
    },
    /// Struct construction.
    StructNew {
        /// Struct type name.
        type_name: String,
        /// Field initializers.
        fields: Vec<(String, LirOperand)>,
    },
    /// Struct field load.
    FieldGet {
        /// Object operand.
        object: LirOperand,
        /// Field name.
        field: String,
    },
    /// Struct field store.
    FieldSet {
        /// Object operand.
        object: LirOperand,
        /// Field name.
        field: String,
        /// Stored value.
        value: LirOperand,
    },
    /// 显式保留模式判定，供后续 lane lowering 或 runtime 桥接使用。
    PatternMatch {
        /// 被判定的输入值。
        value: LirOperand,
        /// 需要匹配的源级模式。
        pattern: HirPattern,
    },
}

/// Explicit low-level block terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirTerminator {
    /// Returns from the current function.
    Return {
        /// Optional return value.
        value: Option<LirOperand>,
    },
    /// Unconditional jump with block arguments.
    Jump {
        /// Destination block.
        target: MirBlockRef,
        /// Outgoing block arguments.
        arguments: Vec<LirOperand>,
    },
    /// Conditional branch.
    Branch {
        /// Branch condition.
        condition: LirOperand,
        /// True edge destination.
        then_target: MirBlockRef,
        /// False edge destination.
        else_target: MirBlockRef,
    },
    /// Explicit effect suspension / delegation edge.
    PerformEffect {
        /// Effect category selected upstream.
        effect: LirEffectKind,
        /// Optional effect payload.
        payload: Option<LirOperand>,
        /// Resume target after the effect is handled.
        resume_target: MirBlockRef,
    },
    /// Terminates unreachable code.
    Unreachable,
}

/// Lowers `HIR` into the current lane-aware `LIR`.
pub struct LirLowerer;

impl LirLowerer {
    /// Lowers a module using the default `CLR` lane.
    pub fn lower_module(module: &HirModule) -> LirModule {
        let mir = MirLowerer::lower_module(module);
        let return_types = collect_return_types(module);
        lower_mir_module(&mir, &return_types, LirTargetLane::Clr)
    }

    /// Lowers a module for a specific target lane.
    pub fn lower_module_for_lane(module: &HirModule, lane: LirTargetLane) -> LirModule {
        let mir = MirLowerer::lower_module(module);
        let return_types = collect_return_types(module);
        lower_mir_module(&mir, &return_types, lane)
    }
}

fn collect_return_types(module: &HirModule) -> BTreeMap<&str, HirType> {
    module.functions.iter().map(|function| (function.name.as_str(), function.return_type.clone())).collect()
}

fn lower_mir_module(module: &MirModule, return_types: &BTreeMap<&str, HirType>, lane: LirTargetLane) -> LirModule {
    LirModule {
        name: module.name.clone(),
        lane,
        structs: module.structs.clone(),
        functions: module.functions.iter().map(|function| lower_mir_function(function, return_types, lane)).collect(),
    }
}

fn lower_mir_function(function: &MirFunction, return_types: &BTreeMap<&str, HirType>, lane: LirTargetLane) -> LirFunction {
    let return_type = return_types.get(function.symbol.as_str()).cloned().unwrap_or(HirType::Unit);
    let suspend_points = function
        .suspend_points
        .iter()
        .map(|suspend_point| LirSuspendPoint {
            state_id: suspend_point.state_id,
            effect: lower_effect_kind(suspend_point.effect),
            suspend_block: suspend_point.suspend_block,
            resume_target: suspend_point.resume_target,
            resume_parameter_count: suspend_point.resume_parameter_count,
            payload_type: suspend_point.payload_type.clone(),
            spill_candidates: suspend_point.spill_candidates.clone(),
        })
        .collect();
    let frame_layouts: Vec<_> = function
        .frame_layouts
        .iter()
        .map(|layout| LirFrameLayout {
            state_id: layout.state_id,
            effect: lower_effect_kind(layout.effect),
            resume_target: layout.resume_target,
            slots: layout
                .slots
                .iter()
                .map(|slot| LirFrameSlot { slot_index: slot.slot_index, value: slot.value, value_type: slot.value_type.clone() })
                .collect(),
        })
        .collect();
    let continuations: Vec<_> = function
        .continuations
        .iter()
        .map(|continuation| LirContinuation {
            dispatch_block: continuation.dispatch_block,
            resume_target: continuation.resume_target,
            resume_parameter: continuation.resume_parameter,
            resume_parameter_type: continuation.resume_parameter_type.clone(),
            handler_exit: continuation.handler_exit,
        })
        .collect();
    let case_chains: Vec<_> = function
        .case_chains
        .iter()
        .map(|case_chain| LirCaseChain {
            dispatch_block: case_chain.dispatch_block,
            first_arm: case_chain.first_arm,
            no_match_block: case_chain.no_match_block,
            exit_block: case_chain.exit_block,
            produce_value: case_chain.produce_value,
            arms: case_chain
                .arms
                .iter()
                .map(|arm| LirCaseArm {
                    entry_block: arm.entry_block,
                    check_block: arm.check_block,
                    guard_block: arm.guard_block,
                    body_block: arm.body_block,
                    next_arm_target: arm.next_arm_target,
                    exit_target: arm.exit_target,
                    fallthrough_target: arm.fallthrough_target,
                })
                .collect(),
        })
        .collect();
    let runtime_frames = lower_runtime_frames(function.symbol.as_str(), lane, &frame_layouts);
    let runtime_continuations = lower_runtime_continuations(function.symbol.as_str(), lane, &continuations);
    LirFunction {
        symbol: function.symbol.clone(),
        param_types: function.param_types.clone(),
        return_type,
        value_types: function.value_types.clone(),
        suspend_points,
        frame_layouts,
        continuations,
        case_chains,
        runtime_frames,
        runtime_continuations,
        entry: function.entry,
        blocks: function.blocks.iter().map(lower_mir_block).collect(),
    }
}

fn lower_runtime_frames(function_symbol: &str, lane: LirTargetLane, frame_layouts: &[LirFrameLayout]) -> Vec<LirRuntimeFrame> {
    let lane_prefix = runtime_lane_prefix(lane);
    frame_layouts
        .iter()
        .map(|layout| LirRuntimeFrame {
            carrier: format!("{function_symbol}${lane_prefix}_state_{}_frame", layout.state_id),
            state_id: layout.state_id,
            resume_target: layout.resume_target,
            slots: layout
                .slots
                .iter()
                .map(|slot| LirRuntimeSlot {
                    field_name: format!("slot_{}", slot.slot_index),
                    slot_index: slot.slot_index,
                    value: slot.value,
                    value_type: slot.value_type.clone(),
                })
                .collect(),
        })
        .collect()
}

fn lower_runtime_continuations(function_symbol: &str, lane: LirTargetLane, continuations: &[LirContinuation]) -> Vec<LirRuntimeContinuation> {
    let lane_prefix = runtime_lane_prefix(lane);
    continuations
        .iter()
        .enumerate()
        .map(|(index, continuation)| LirRuntimeContinuation {
            carrier: format!("{function_symbol}${lane_prefix}_continuation_{index}"),
            dispatch_block: continuation.dispatch_block,
            resume_target: continuation.resume_target,
            resume_parameter: continuation.resume_parameter,
            resume_parameter_field: "resume_value".to_string(),
            resume_parameter_type: continuation.resume_parameter_type.clone(),
            handler_exit: continuation.handler_exit,
        })
        .collect()
}

fn runtime_lane_prefix(lane: LirTargetLane) -> &'static str {
    match lane {
        LirTargetLane::Clr => "clr",
        LirTargetLane::Jvm => "jvm",
        LirTargetLane::Wasm => "wasm",
        LirTargetLane::Native => "native",
        LirTargetLane::Vm => "vm",
    }
}

fn lower_mir_block(block: &MirBlock) -> LirBlock {
    let operations = block.instructions.iter().map(lower_mir_instruction).collect();

    let terminator = match &block.terminator {
        MirTerminator::Return { value } => LirTerminator::Return { value: value.clone().map(lower_mir_operand) },
        MirTerminator::Jump { target, arguments } => {
            LirTerminator::Jump { target: *target, arguments: arguments.iter().cloned().map(lower_mir_operand).collect() }
        }
        MirTerminator::Branch { condition, then_target, else_target } => {
            LirTerminator::Branch { condition: lower_mir_operand(condition.clone()), then_target: *then_target, else_target: *else_target }
        }
        MirTerminator::PerformEffect { effect, payload, resume_target } => LirTerminator::PerformEffect {
            effect: lower_effect_kind(*effect),
            payload: payload.clone().map(lower_mir_operand),
            resume_target: *resume_target,
        },
        MirTerminator::Unreachable => LirTerminator::Unreachable,
    };

    LirBlock { id: block.id, label: block.label.clone(), parameters: block.parameters.clone(), operations, terminator }
}

fn lower_mir_instruction(instruction: &MirInstruction) -> LirOperation {
    let kind = match &instruction.kind {
        MirInstructionKind::LoadConstant { constant, ty } => LirOperationKind::LoadConstant { constant: constant.clone(), ty: ty.clone() },
        MirInstructionKind::LoadSymbol { path } => LirOperationKind::LoadSymbol { path: path.clone() },
        MirInstructionKind::Copy { source } => LirOperationKind::Move { source: lower_mir_operand(source.clone()) },
        MirInstructionKind::StoreVar { name, value, ty } => {
            LirOperationKind::StoreVar { name: name.clone(), value: lower_mir_operand(value.clone()), ty: ty.clone() }
        }
        MirInstructionKind::Call { dispatch, callee, arguments, builtin, witness, effect } => LirOperationKind::Call {
            dispatch: lower_dispatch_kind(*dispatch),
            callee: lower_mir_operand(callee.clone()),
            arguments: arguments.iter().cloned().map(lower_mir_operand).collect(),
            builtin: *builtin,
            witness: witness.clone().map(lower_mir_operand),
            effect: effect.clone().map(lower_mir_operand),
        },
        MirInstructionKind::ArrayNew { element_type, length } => {
            LirOperationKind::ArrayNew { element_type: element_type.clone(), length: lower_mir_operand(length.clone()) }
        }
        MirInstructionKind::ArrayLiteral { element_type, items } => {
            LirOperationKind::ArrayLiteral { element_type: element_type.clone(), items: items.iter().cloned().map(lower_mir_operand).collect() }
        }
        MirInstructionKind::StructNew { type_name, fields } => LirOperationKind::StructNew {
            type_name: type_name.clone(),
            fields: fields.iter().map(|(name, value)| (name.clone(), lower_mir_operand(value.clone()))).collect(),
        },
        MirInstructionKind::FieldGet { object, field } => {
            LirOperationKind::FieldGet { object: lower_mir_operand(object.clone()), field: field.clone() }
        }
        MirInstructionKind::FieldSet { object, field, value } => LirOperationKind::FieldSet {
            object: lower_mir_operand(object.clone()),
            field: field.clone(),
            value: lower_mir_operand(value.clone()),
        },
        MirInstructionKind::PatternMatch { value, pattern } => {
            LirOperationKind::PatternMatch { value: lower_mir_operand(value.clone()), pattern: pattern.clone() }
        }
    };

    LirOperation { output: instruction.output, kind }
}

fn lower_mir_operand(operand: MirOperand) -> LirOperand {
    match operand {
        MirOperand::Value(value) => LirOperand::Value(value),
        MirOperand::Constant(constant) => LirOperand::Constant(constant),
        MirOperand::Symbol(path) => LirOperand::Symbol(path),
    }
}

/// Preserves the upstream dispatch category when lowering `MIR` calls into `LIR`.
pub fn lower_dispatch_kind(dispatch: MirDispatchKind) -> LirDispatchKind {
    match dispatch {
        MirDispatchKind::Static => LirDispatchKind::Static,
        MirDispatchKind::Witness => LirDispatchKind::Witness,
        MirDispatchKind::EffectHandler => LirDispatchKind::EffectHandler,
    }
}

/// Preserves the upstream effect category when lowering `MIR` terminators into `LIR`.
pub fn lower_effect_kind(effect: MirEffectKind) -> LirEffectKind {
    match effect {
        MirEffectKind::Raise => LirEffectKind::Raise,
        MirEffectKind::Yield => LirEffectKind::Yield,
        MirEffectKind::DelegateYield => LirEffectKind::DelegateYield,
        MirEffectKind::Await => LirEffectKind::Await,
        MirEffectKind::AsyncSpawn => LirEffectKind::AsyncSpawn,
        MirEffectKind::AsyncBlock => LirEffectKind::AsyncBlock,
    }
}

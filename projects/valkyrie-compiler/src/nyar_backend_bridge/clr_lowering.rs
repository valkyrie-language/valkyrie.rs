//! `CLR` 路线 lowering。
//!
//! 这里负责把 `valkyrie-compiler` 的 `LIR` 收口成 `MSIL` 模块，
//! 并通过 `nyar::TargetLoweringLane` 暴露给上层编排。

use std::collections::BTreeMap;

use super::{
    clr_call_lowering::lower_special_operation,
    clr_effect_runtime::{
        allocate_runtime_scheduler_slots, collect_runtime_continuation_bindings, collect_runtime_frame_bindings,
        collect_runtime_frame_resume_loader_bindings, collect_runtime_handler_exit_targets, collect_runtime_resume_loader_bindings, emit_stloc,
        lower_runtime_continuation_prelude, lower_runtime_frame_resume_loader_prelude, lower_runtime_handler_exit_cleanup,
        lower_runtime_resume_loader_prelude,
    },
    clr_runtime_lowering::lower_runtime_carrier_types,
    clr_terminator_lowering::lower_terminator,
};
use crate::{
    lir::{LirFunction, LirModule, LirOperand, LirOperation, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirConstant, MirStruct, MirValueRef},
    symbols::is_main_symbol,
};
use clr_backend::{
    msil::MsilField, MsilAssembly, MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule,
    MsilOpcode, MsilType, MsilTypeDef,
};
use miette::miette;
use nyar::{
    abstractions::{BackendInputKind, BinaryTarget},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use valkyrie_types::hir::ValkyrieType as HirType;

/// 本地结构体/类布局信息。
#[derive(Debug, Clone)]
pub(super) struct LocalStructLayout {
    /// 类型简单名。
    pub(super) name: String,
    /// 类型限定名。
    pub(super) qualified_name: String,
    /// 字段声明顺序。
    pub(super) field_order: Vec<String>,
    /// 字段类型映射。
    pub(super) field_types: BTreeMap<String, HirType>,
}

/// `CLR` lane 的 `LIR -> MSIL` 承接器。
pub struct ClrLirLoweringLane {
    descriptor: TargetLoweringLaneDescriptor,
}

impl ClrLirLoweringLane {
    /// 创建一个新的 `CLR` lane lowering。
    pub fn new() -> Self {
        Self {
            descriptor: TargetLoweringLaneDescriptor {
                name: "clr-msil-lowering".to_string(),
                lane: TargetLane::Clr,
                input_kind: BackendInputKind::MsilText,
                target: BinaryTarget::clr(),
            },
        }
    }
}

impl Default for ClrLirLoweringLane {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetLoweringLane for ClrLirLoweringLane {
    type PartitionInput = LirModule;
    type BackendInput = MsilModule;

    fn descriptor(&self) -> &TargetLoweringLaneDescriptor {
        &self.descriptor
    }

    fn lower_partition(&self, partition: Self::PartitionInput) -> miette::Result<LaneLoweringResult<Self::BackendInput>> {
        if partition.lane != LirTargetLane::Clr {
            return Err(miette!(
                code = "nyar::clr::lowering::lane_mismatch",
                help = "请确认当前 `LIR` 分区已经选择 `CLR` 目标路线",
                "当前 lane 是 {:?}，不能进入 CLR lowering",
                partition.lane
            ));
        }

        let artifact_name = partition.name.clone();
        let input = lower_lir_to_msil(&partition);
        Ok(LaneLoweringResult { input, artifact_name })
    }
}

/// 将 `LIR` 模块降低为 `MSIL` 模块。
pub fn lower_lir_to_msil(lir: &LirModule) -> MsilModule {
    let local_structs = build_local_struct_layouts(&lir.structs);
    // 构建函数符号 -> 返回类型映射，用于在 `Call` 降级时判断调用是否返回 `void`。
    let return_types: BTreeMap<String, HirType> =
        lir.functions.iter().map(|function| (function.symbol.clone(), function.return_type.clone())).collect();
    let global_methods = lir.functions.iter().map(|function| lower_function(function, &lir.name, &return_types, &local_structs)).collect();
    let mut types: Vec<_> = lir.structs.iter().map(lower_struct_type).collect();
    types.extend(lower_runtime_carrier_types(lir));

    MsilModule { assembly: MsilAssembly { name: lir.name.clone(), externs: vec!["mscorlib".to_string()] }, types, global_methods }
}

fn build_local_struct_layouts(structs: &[MirStruct]) -> BTreeMap<String, LocalStructLayout> {
    let mut layouts = BTreeMap::new();
    for item in structs {
        let qualified_name = qualify_struct_name(item);
        let field_order = item.fields.iter().map(|field| field.name.clone()).collect::<Vec<_>>();
        let field_types = item.fields.iter().map(|field| (field.name.clone(), field.ty.clone())).collect::<BTreeMap<_, _>>();
        let layout = LocalStructLayout { name: item.name.clone(), qualified_name: qualified_name.clone(), field_order, field_types };
        layouts.insert(item.name.clone(), layout.clone());
        layouts.insert(qualified_name, layout);
    }
    layouts
}

fn lower_struct_type(item: &MirStruct) -> MsilTypeDef {
    let qualified_name = qualify_struct_name(item);
    let fields = item.fields.iter().map(|field| MsilField { name: field.name.clone(), ty: lower_hir_type(&field.ty) }).collect();
    let methods = vec![build_struct_constructor(item, &qualified_name)];
    MsilTypeDef { full_name: item.name.clone(), namespace: item.namespace.clone(), fields, methods, is_value_type: item.is_value_type }
}

fn build_struct_constructor(item: &MirStruct, qualified_name: &str) -> MsilMethodBody {
    let mut instructions = Vec::new();

    if !item.is_value_type {
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ldarg0, operand: None });
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Call,
            operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                owner: Some("[mscorlib]System.Object".to_string()),
                name: ".ctor".to_string(),
                signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
            })),
        });
    }

    for (index, field) in item.fields.iter().enumerate() {
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ldarg0, operand: None });
        instructions.push(load_argument_instruction(index + 1));
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Stfld,
            operand: Some(MsilInstructionOperand::Field(qualified_name.to_string(), field.name.clone())),
        });
    }

    instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });

    MsilMethodBody {
        method: MsilMethodRef {
            owner: Some(qualified_name.to_string()),
            name: ".ctor".to_string(),
            signature: MsilMethodSignature::new(MsilType::Void, item.fields.iter().map(|field| lower_hir_type(&field.ty)).collect()),
        },
        locals: Vec::new(),
        instructions,
        max_stack: 2,
        is_entry_point: false,
    }
}

fn qualify_struct_name(item: &MirStruct) -> String {
    if item.namespace.is_empty() {
        item.name.clone()
    }
    else {
        format!("{}.{}", item.namespace, item.name)
    }
}

fn lower_function(
    function: &LirFunction,
    module_name: &str,
    return_types: &BTreeMap<String, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> MsilMethodBody {
    let is_entry_point = is_main_symbol(&function.symbol);
    let mut instructions = Vec::new();
    let mut max_stack: u16 = 1;
    let runtime_namespace = format!("{}.runtime", module_name);
    let parameter_slots = collect_parameter_slots(function);
    let mut value_types = BTreeMap::new();
    let runtime_frame_bindings = collect_runtime_frame_bindings(function);
    let runtime_frame_resume_loader_bindings = collect_runtime_frame_resume_loader_bindings(function);
    let runtime_continuation_bindings = collect_runtime_continuation_bindings(function);
    let runtime_resume_loader_bindings = collect_runtime_resume_loader_bindings(function);
    let runtime_handler_exit_targets = collect_runtime_handler_exit_targets(function);

    // 为每个基本块分配 MSIL 标签，用于分支目标解析。
    let block_labels: BTreeMap<MirBlockRef, String> = function.blocks.iter().map(|block| (block.id, format!("BB{}", block.id.0))).collect();

    // 收集所有被引用为分支目标的基本块，确保它们的标签一定会被发射。
    let mut referenced_targets: std::collections::BTreeSet<MirBlockRef> = std::collections::BTreeSet::new();
    let block_params: BTreeMap<MirBlockRef, Vec<MirValueRef>> = function.blocks.iter().map(|b| (b.id, b.parameters.clone())).collect();

    for block in &function.blocks {
        match &block.terminator {
            LirTerminator::Jump { target, .. } => {
                referenced_targets.insert(*target);
            }
            LirTerminator::Branch { then_target, else_target, .. } => {
                referenced_targets.insert(*then_target);
                referenced_targets.insert(*else_target);
            }
            LirTerminator::PerformEffect { resume_target, .. } => {
                referenced_targets.insert(*resume_target);
            }
            _ => {}
        }
    }

    // 局部变量槽位映射：MirValueRef -> 槽位索引。
    // Move 操作会分配槽位并 emit stloc；后续引用通过 ldloc 加载。
    let mut local_slots: BTreeMap<MirValueRef, usize> = BTreeMap::new();
    let mut local_types: Vec<MsilType> = Vec::new();

    // 命名变量槽位映射：变量名 -> 槽位索引。
    // StoreVar 操作按变量名查找或分配槽位，同名 StoreVar 复用同一槽位，
    // 确保循环 header 通过 ldloc 总能读到最新值。
    let mut var_slots: BTreeMap<String, usize> = BTreeMap::new();

    if let Some(entry_block) = function.blocks.iter().find(|block| block.id == function.entry) {
        for (index, value_ref) in entry_block.parameters.iter().enumerate() {
            if let Some(parameter_type) = function.param_types.get(index) {
                value_types.insert(*value_ref, parameter_type.clone());
            }
        }
    }

    // 为非入口块参数分配局部槽位并确定局部类型（优先使用编译期提供的静态类型）。
    for block in &function.blocks {
        if block.id == function.entry {
            continue;
        }
        for param in &block.parameters {
            let slot = local_types.len();
            let ty = function.value_types.get(param).map(|t| lower_hir_type(t)).unwrap_or(MsilType::Object);
            local_types.push(ty);
            local_slots.insert(*param, slot);
        }
    }
    let runtime_scheduler_slots = allocate_runtime_scheduler_slots(&mut local_types);

    for block in &function.blocks {
        // 在块的第一条指令上打标签；若块为空，则插入一条带标签的 Nop。
        let mut block_label = block_labels.get(&block.id).cloned();
        let is_referenced = referenced_targets.contains(&block.id);
        if block.operations.is_empty() && matches!(block.terminator, LirTerminator::Unreachable) && !is_referenced {
            // 空且不可达且未被引用的块：跳过，不生成代码。
            continue;
        }

        let mut eval_stack: Vec<MirValueRef> = Vec::new();
        let mut first_op = true;
        if let Some(binding) = runtime_resume_loader_bindings.get(&block.id).copied() {
            lower_runtime_resume_loader_prelude(
                binding,
                runtime_scheduler_slots,
                &runtime_namespace,
                &mut instructions,
                &mut max_stack,
                &local_slots,
                block_label.take(),
            );
            first_op = false;
        }
        if let Some(binding) = runtime_frame_resume_loader_bindings.get(&block.id).copied() {
            lower_runtime_frame_resume_loader_prelude(
                binding,
                runtime_scheduler_slots,
                &runtime_namespace,
                &mut instructions,
                &mut max_stack,
                &local_slots,
                block_label.take(),
            );
            first_op = false;
        }
        if let Some(binding) = runtime_continuation_bindings.get(&block.id).copied() {
            lower_runtime_continuation_prelude(
                binding,
                runtime_scheduler_slots,
                &runtime_namespace,
                &mut instructions,
                &mut max_stack,
                &parameter_slots,
                &mut local_slots,
                &mut local_types,
                &mut eval_stack,
                &value_types,
                block_label.take(),
            );
            first_op = false;
        }
        if runtime_handler_exit_targets.contains(&block.id) {
            lower_runtime_handler_exit_cleanup(runtime_scheduler_slots, &mut instructions, &mut max_stack, block_label.take());
            first_op = false;
        }
        for operation in &block.operations {
            let label = if first_op { block_label.take() } else { None };
            first_op = false;
            lower_operation(
                operation,
                &mut instructions,
                &mut max_stack,
                &parameter_slots,
                &mut local_slots,
                &mut local_types,
                &mut var_slots,
                &mut eval_stack,
                return_types,
                local_structs,
                &mut value_types,
                label,
            );
        }

        // 若块没有操作，但需要标签，则插入 Nop 承载标签。
        if first_op {
            if let Some(label) = block_label {
                instructions.push(MsilInstruction { label: Some(label), opcode: MsilOpcode::Nop, operand: None });
            }
        }

        lower_terminator(
            &block.terminator,
            &mut instructions,
            &mut max_stack,
            &parameter_slots,
            &mut local_slots,
            &mut local_types,
            &mut eval_stack,
            &function.return_type,
            &value_types,
            &block_labels,
            &block_params,
            runtime_scheduler_slots,
            &runtime_namespace,
            runtime_frame_bindings.get(&block.id).copied(),
            &runtime_continuation_bindings,
            &runtime_resume_loader_bindings,
            &runtime_handler_exit_targets,
        );
    }

    if instructions.is_empty() {
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
    }

    apply_tail_call_optimization(&mut instructions);

    MsilMethodBody {
        method: MsilMethodRef {
            owner: Some(module_name.to_string()),
            name: function.symbol.clone(),
            signature: MsilMethodSignature::new(
                lower_entry_return_type(&function.return_type, is_entry_point),
                function.param_types.iter().map(lower_hir_type).collect(),
            ),
        },
        locals: local_types,
        instructions,
        max_stack,
        is_entry_point,
    }
}

fn lower_entry_return_type(ty: &HirType, is_entry_point: bool) -> MsilType {
    if !is_entry_point {
        return lower_hir_type(ty);
    }

    match ty {
        HirType::Void | HirType::Unit => MsilType::Void,
        HirType::Integer32 { signed: true } => MsilType::Int32,
        HirType::Integer32 { signed: false } => MsilType::UInt32,
        HirType::Named(name) if name.to_string() == "ExitCode" => MsilType::Int32,
        _ => MsilType::Int32,
    }
}

/// 识别尾位置调用，并在 `call` / `callvirt` 前插入 `tail.` 前缀。
///
/// 当前按照最小充分条件匹配：
/// `... -> call|callvirt -> ret`
/// 这样既覆盖自尾递归，也覆盖跨函数互递归。
fn apply_tail_call_optimization(instructions: &mut Vec<MsilInstruction>) {
    let mut index = 0usize;
    while index + 1 < instructions.len() {
        let call_instruction = &instructions[index];
        let ret_instruction = &instructions[index + 1];
        let is_tail_position =
            matches!(call_instruction.opcode, MsilOpcode::Call | MsilOpcode::Callvirt) && matches!(ret_instruction.opcode, MsilOpcode::Ret);
        let already_prefixed = index > 0 && matches!(instructions[index - 1].opcode, MsilOpcode::Tail);

        if is_tail_position && !already_prefixed {
            let label = instructions[index].label.take();
            instructions.insert(index, MsilInstruction { label, opcode: MsilOpcode::Tail, operand: None });
            index += 3;
        }
        else {
            index += 1;
        }
    }
}

fn collect_parameter_slots(function: &LirFunction) -> BTreeMap<MirValueRef, usize> {
    let mut slots = BTreeMap::new();
    let Some(entry_block) = function.blocks.iter().find(|block| block.id == function.entry)
    else {
        return slots;
    };
    for (index, parameter) in entry_block.parameters.iter().enumerate() {
        slots.insert(*parameter, index);
    }
    slots
}

fn lower_operation(
    operation: &LirOperation,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    var_slots: &mut BTreeMap<String, usize>,
    eval_stack: &mut Vec<MirValueRef>,
    return_types: &BTreeMap<String, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
    label: Option<String>,
) {
    // 记录 label 锚点：label 必须放在本操作 emit 的第一条指令上，
    // 确保分支目标指向块的第一条指令（而非最后一条，如 add/stloc/call）。
    let anchor = label.as_ref().map(|_| instructions.len());

    if lower_special_operation(
        operation,
        instructions,
        max_stack,
        parameter_slots,
        local_slots,
        local_types,
        eval_stack,
        return_types,
        local_structs,
        value_types,
    ) {
        if let Some(label) = label {
            if let Some(anchor) = anchor {
                if anchor < instructions.len() {
                    instructions[anchor].label = Some(label);
                }
            }
        }
        return;
    }

    match &operation.kind {
        LirOperationKind::LoadConstant { constant, ty } => {
            lower_constant(constant, instructions, max_stack, None);
            if let Some(output) = operation.output {
                if let Some(inferred_type) = ty.clone().or_else(|| infer_constant_type(constant)) {
                    value_types.insert(output, inferred_type);
                }
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
        }
        LirOperationKind::LoadSymbol { path } => {
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Ldsfld,
                operand: Some(MsilInstructionOperand::Symbol(path.to_string())),
            });
            *max_stack = (*max_stack).max(1);
            if let Some(output) = operation.output {
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
        }
        LirOperationKind::Move { source } => {
            // 先分配局部变量槽位（在调用 lower_operand 之前，避免借用冲突）。
            let slot = local_types.len();
            local_types.push(infer_operand_msil_type(source, value_types));

            // 若 source 是 Value 且不在栈顶，先溢出 eval_stack。
            let need_spill = match source {
                LirOperand::Value(v) => eval_stack.last() != Some(v),
                _ => false,
            };
            if need_spill {
                spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            }

            // 加载源值到求值栈。
            lower_operand(source, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            // emit stloc 将值存储到局部变量。
            emit_stloc(slot, instructions, None);

            // stloc 消费了栈顶值，若 source 是被追踪的 Value，则从 eval_stack 弹出。
            if let LirOperand::Value(_) = source {
                eval_stack.pop();
            }

            // 记录 output -> slot 映射，后续引用通过 ldloc 加载。
            if let Some(output) = operation.output {
                local_slots.insert(output, slot);
                if let Some(inferred_type) = infer_operand_type(source, value_types) {
                    value_types.insert(output, inferred_type);
                }
            }
        }
        LirOperationKind::StoreVar { name, value, ty } => {
            // 按变量名查找或分配槽位：同名 StoreVar 复用同一槽位。
            let slot = match var_slots.get(name).copied() {
                Some(existing) => existing,
                None => {
                    let new_slot = local_types.len();
                    let declared = ty.as_ref().map(lower_hir_type).unwrap_or_else(|| infer_operand_msil_type(value, value_types));
                    local_types.push(declared);
                    var_slots.insert(name.clone(), new_slot);
                    new_slot
                }
            };

            // 若 value 是 Value 且不在栈顶，先溢出 eval_stack。
            let need_spill = match value {
                LirOperand::Value(v) => eval_stack.last() != Some(v),
                _ => false,
            };
            if need_spill {
                spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            }

            // 加载要存储的值到求值栈。
            lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            // emit stloc 将值存储到命名槽位。
            emit_stloc(slot, instructions, None);

            // stloc 消费了栈顶值，若 value 是被追踪的 Value，则从 eval_stack 弹出。
            if let LirOperand::Value(_) = value {
                eval_stack.pop();
            }

            // 记录 output -> slot 映射，后续引用通过 ldloc 加载最新值。
            if let Some(output) = operation.output {
                local_slots.insert(output, slot);
                if let Some(inferred_type) = ty.clone().or_else(|| infer_operand_type(value, value_types)) {
                    value_types.insert(output, inferred_type);
                }
            }
        }
        LirOperationKind::PatternMatch { .. } => {
            lower_constant(&MirConstant::Bool(false), instructions, max_stack, None);
            if let Some(output) = operation.output {
                value_types.insert(output, HirType::Boolean);
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
        }
        LirOperationKind::Call { .. }
        | LirOperationKind::ArrayNew { .. }
        | LirOperationKind::ArrayLiteral { .. }
        | LirOperationKind::StructNew { .. }
        | LirOperationKind::FieldGet { .. }
        | LirOperationKind::FieldSet { .. } => {
            unreachable!("special CLR lowering operations should be handled before the main match")
        }
    }

    // 将 label 放在第一条 emitted 指令上，确保分支目标指向块的第一条指令。
    if let Some(label) = label {
        if let Some(anchor) = anchor {
            if anchor < instructions.len() {
                instructions[anchor].label = Some(label);
            }
        }
    }
}

fn load_argument_instruction(index: usize) -> MsilInstruction {
    let (opcode, operand) = match index {
        0 => (MsilOpcode::Ldarg0, None),
        1 => (MsilOpcode::Ldarg1, None),
        2 => (MsilOpcode::Ldarg2, None),
        3 => (MsilOpcode::Ldarg3, None),
        _ => (MsilOpcode::Ldarg, Some(MsilInstructionOperand::Integer(index as i64))),
    };
    MsilInstruction { label: None, opcode, operand }
}

fn infer_constant_type(constant: &MirConstant) -> Option<HirType> {
    Some(match constant {
        MirConstant::Int(value) if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => HirType::Integer32 { signed: true },
        MirConstant::Int(_) => HirType::Integer64 { signed: true },
        MirConstant::Bool(_) => HirType::Boolean,
        MirConstant::Float64(_) => HirType::Float64,
        MirConstant::String(_) => HirType::Utf8,
        MirConstant::Unit => HirType::Unit,
    })
}

pub(super) fn infer_operand_type(operand: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> Option<HirType> {
    match operand {
        LirOperand::Value(value_ref) => value_types.get(value_ref).cloned(),
        LirOperand::Constant(constant) => infer_constant_type(constant),
        LirOperand::Symbol(_) => None,
    }
}

pub(super) fn lower_hir_type(ty: &HirType) -> MsilType {
    match ty {
        HirType::Void | HirType::Unit => MsilType::Void,
        HirType::Boolean => MsilType::Bool,
        HirType::Character => MsilType::Char,
        HirType::Integer8 { signed } => {
            if *signed {
                MsilType::Int8
            }
            else {
                MsilType::UInt8
            }
        }
        HirType::Integer16 { signed } => {
            if *signed {
                MsilType::Int16
            }
            else {
                MsilType::UInt16
            }
        }
        HirType::Integer32 { signed } => {
            if *signed {
                MsilType::Int32
            }
            else {
                MsilType::UInt32
            }
        }
        HirType::Integer64 { signed } => {
            if *signed {
                MsilType::Int64
            }
            else {
                MsilType::UInt64
            }
        }
        HirType::Float32 => MsilType::Float32,
        HirType::Float64 => MsilType::Float64,
        HirType::Utf8 | HirType::Utf16 => MsilType::String,
        HirType::Named(name) => MsilType::Named(name.to_string()),
        HirType::Array(item) => MsilType::sz_array(lower_hir_type(item)),
        _ => MsilType::Object,
    }
}

fn infer_operand_msil_type(operand: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> MsilType {
    infer_operand_type(operand, value_types).map(|ty| lower_hir_type(&ty)).unwrap_or(MsilType::Object)
}

/// emit stloc 指令：根据槽位索引选择短格式或长格式。

pub(super) fn lower_local_slot(slot: usize, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16) {
    let (opcode, operand) = match slot {
        0 => (MsilOpcode::Ldloc0, None),
        1 => (MsilOpcode::Ldloc1, None),
        2 => (MsilOpcode::Ldloc2, None),
        3 => (MsilOpcode::Ldloc3, None),
        _ => (MsilOpcode::Ldloc, Some(MsilInstructionOperand::Integer(slot as i64))),
    };
    instructions.push(MsilInstruction { label: None, opcode, operand });
    *max_stack = (*max_stack).max(1);
}

/// 将求值栈中的所有值溢出到局部变量。
///
/// 当需要访问非栈顶的值时，MSIL 无法直接索引栈中位置，
/// 必须先弹出上方所有值（存入局部变量），再加载目标值。
pub(super) fn spill_eval_stack(
    eval_stack: &mut Vec<MirValueRef>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    instructions: &mut Vec<MsilInstruction>,
    value_types: &BTreeMap<MirValueRef, HirType>,
) {
    while let Some(value) = eval_stack.pop() {
        let slot = local_types.len();
        let spilled_type = value_types.get(&value).map(lower_hir_type).unwrap_or(MsilType::Object);
        local_types.push(spilled_type);
        emit_stloc(slot, instructions, None);
        local_slots.insert(value, slot);
    }
}

pub(super) fn lower_constant(constant: &MirConstant, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16, label: Option<String>) {
    match constant {
        MirConstant::Int(value) => {
            let opcode = match *value {
                0 => MsilOpcode::LdcI4_0,
                1 => MsilOpcode::LdcI4_1,
                2 => MsilOpcode::LdcI4_2,
                3 => MsilOpcode::LdcI4_3,
                4 => MsilOpcode::LdcI4_4,
                5 => MsilOpcode::LdcI4_5,
                6 => MsilOpcode::LdcI4_6,
                7 => MsilOpcode::LdcI4_7,
                8 => MsilOpcode::LdcI4_8,
                -1 => MsilOpcode::LdcI4M1,
                _ if *value >= i8::MIN as i64 && *value <= i8::MAX as i64 => MsilOpcode::LdcI4S,
                _ if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => MsilOpcode::LdcI4,
                _ => MsilOpcode::LdcI8,
            };
            let operand = match opcode {
                MsilOpcode::LdcI4 | MsilOpcode::LdcI4S | MsilOpcode::LdcI8 => Some(MsilInstructionOperand::Integer(*value)),
                _ => None,
            };
            instructions.push(MsilInstruction { label, opcode, operand });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::Bool(value) => {
            instructions.push(MsilInstruction { label, opcode: if *value { MsilOpcode::LdcI4_1 } else { MsilOpcode::LdcI4_0 }, operand: None });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::Float64(value) => {
            instructions.push(MsilInstruction {
                label,
                opcode: MsilOpcode::LdcR8,
                operand: Some(MsilInstructionOperand::Float(value.0.to_string())),
            });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::String(value) => {
            instructions.push(MsilInstruction {
                label,
                opcode: MsilOpcode::Ldstr,
                operand: Some(MsilInstructionOperand::StringLiteral(value.clone())),
            });
            *max_stack = (*max_stack).max(1);
        }
        MirConstant::Unit => {
            instructions.push(MsilInstruction { label, opcode: MsilOpcode::Ldnull, operand: None });
            *max_stack = (*max_stack).max(1);
        }
    }
}

pub(super) fn lower_operand(
    operand: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
    eval_stack: &mut Vec<MirValueRef>,
) {
    match operand {
        LirOperand::Value(value_ref) => {
            // 若该值已在求值栈顶，直接复用，避免重复 ldloc。
            if eval_stack.last() == Some(value_ref) {
                return;
            }
            lower_value_ref(value_ref, instructions, max_stack, parameter_slots, local_slots);
            eval_stack.push(*value_ref);
        }
        LirOperand::Constant(constant) => {
            lower_constant(constant, instructions, max_stack, None);
            // 常量没有 MirValueRef，无法在 eval_stack 中追踪。
        }
        LirOperand::Symbol(path) => {
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Ldsfld,
                operand: Some(MsilInstructionOperand::Symbol(path.to_string())),
            });
            *max_stack = (*max_stack).max(1);
            // 符号没有 MirValueRef，无法在 eval_stack 中追踪。
        }
    }
}

pub(super) fn lower_value_ref(
    value_ref: &MirValueRef,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &BTreeMap<MirValueRef, usize>,
) {
    // 优先检查参数槽位。
    if let Some(parameter_index) = parameter_slots.get(value_ref).copied() {
        let (opcode, operand) = match parameter_index {
            0 => (MsilOpcode::Ldarg0, None),
            1 => (MsilOpcode::Ldarg1, None),
            2 => (MsilOpcode::Ldarg2, None),
            3 => (MsilOpcode::Ldarg3, None),
            _ => (MsilOpcode::Ldarg, Some(MsilInstructionOperand::Integer(parameter_index as i64))),
        };
        instructions.push(MsilInstruction { label: None, opcode, operand });
        *max_stack = (*max_stack).max(1);
        return;
    }

    // 检查局部变量槽位。
    if let Some(slot) = local_slots.get(value_ref).copied() {
        let (opcode, operand) = match slot {
            0 => (MsilOpcode::Ldloc0, None),
            1 => (MsilOpcode::Ldloc1, None),
            2 => (MsilOpcode::Ldloc2, None),
            3 => (MsilOpcode::Ldloc3, None),
            _ => (MsilOpcode::Ldloc, Some(MsilInstructionOperand::Integer(slot as i64))),
        };
        instructions.push(MsilInstruction { label: None, opcode, operand });
        *max_stack = (*max_stack).max(1);
        return;
    }

    // 兜底：值未在任何槽位中，发射 `nop`，避免污染求值栈导致 IL 无效。
    instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Nop, operand: None });
}

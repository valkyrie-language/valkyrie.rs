//! `CLR` 路线 lowering。
//!
//! 这里负责把 `valkyrie-compiler` 的 `LIR` 收口成 `MSIL` 模块，
//! 并通过 `nyar::TargetLoweringLane` 暴露给上层编排。

use std::collections::BTreeMap;

use super::clr_runtime_lowering::lower_runtime_carrier_types;
use crate::{
    lir::{LirFunction, LirModule, LirOperand, LirOperation, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirBuiltinBinaryOp, MirBuiltinCall, MirBuiltinCompareOp, MirConstant, MirStruct, MirValueRef},
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
use valkyrie_types::{hir::ValkyrieType as HirType, Identifier};

/// 本地结构体/类布局信息。
#[derive(Debug, Clone)]
struct LocalStructLayout {
    /// 类型简单名。
    name: String,
    /// 类型限定名。
    qualified_name: String,
    /// 字段声明顺序。
    field_order: Vec<String>,
    /// 字段类型映射。
    field_types: BTreeMap<String, HirType>,
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
    let is_entry_point = function.symbol == "main";
    let mut instructions = Vec::new();
    let mut max_stack: u16 = 1;
    let parameter_slots = collect_parameter_slots(function);
    let mut value_types = BTreeMap::new();

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

    for block in &function.blocks {
        // 在块的第一条指令上打标签；若块为空，则插入一条带标签的 Nop。
        let block_label = block_labels.get(&block.id).cloned();
        let is_referenced = referenced_targets.contains(&block.id);
        if block.operations.is_empty() && matches!(block.terminator, LirTerminator::Unreachable) && !is_referenced {
            // 空且不可达且未被引用的块：跳过，不生成代码。
            continue;
        }

        let mut eval_stack: Vec<MirValueRef> = Vec::new();
        let mut first_op = true;
        for operation in &block.operations {
            let label = if first_op { block_label.clone() } else { None };
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
            &mut var_slots,
            &mut eval_stack,
            &function.return_type,
            &value_types,
            &block_labels,
            &block_params,
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
    for block in &function.blocks {
        for (index, parameter) in block.parameters.iter().enumerate() {
            slots.entry(*parameter).or_insert(index);
        }
    }
    slots
}

/// 运算符方法的内建降级策略。
///
/// `Simple`：单个 opcode 即可完成计算。
/// `NegatedComparison`：先做比较，再与 0 比较，用于 `!=`/`<=`/`>=`。
/// `LogicalNot`：直接对布尔结果执行 `ldc.i4.0 + ceq`。
enum IntrinsicCallLowering {
    /// 单条 opcode 即可完成计算。
    Simple(MsilOpcode),
    /// 先执行比较 opcode，再 `ldc.i4.0 + ceq` 取反。
    NegatedComparison(MsilOpcode),
    /// 对单个布尔参数执行逻辑取反。
    LogicalNot,
    /// 数组长度：`ldlen` + `conv.i4`。
    ArrayLength,
    /// 透明包装类型构造：直接保留参数值。
    Identity,
}

fn intrinsic_from_builtin(builtin: MirBuiltinCall) -> Option<IntrinsicCallLowering> {
    Some(match builtin {
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Add) => IntrinsicCallLowering::Simple(MsilOpcode::Add),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Sub) => IntrinsicCallLowering::Simple(MsilOpcode::Sub),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Mul) => IntrinsicCallLowering::Simple(MsilOpcode::Mul),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Div) => IntrinsicCallLowering::Simple(MsilOpcode::Div),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Rem) => IntrinsicCallLowering::Simple(MsilOpcode::Rem),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq) => IntrinsicCallLowering::Simple(MsilOpcode::Ceq),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Lt) => IntrinsicCallLowering::Simple(MsilOpcode::Clt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Gt) => IntrinsicCallLowering::Simple(MsilOpcode::Cgt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ne) => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Ceq),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Le) => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Cgt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ge) => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Clt),
        MirBuiltinCall::NumericNeg => IntrinsicCallLowering::Simple(MsilOpcode::Neg),
        MirBuiltinCall::LogicalAnd => IntrinsicCallLowering::Simple(MsilOpcode::And),
        MirBuiltinCall::LogicalOr => IntrinsicCallLowering::Simple(MsilOpcode::Or),
        MirBuiltinCall::LogicalNot => IntrinsicCallLowering::LogicalNot,
        MirBuiltinCall::ArrayLength => IntrinsicCallLowering::ArrayLength,
        MirBuiltinCall::Identity => IntrinsicCallLowering::Identity,
        MirBuiltinCall::ArrayGet | MirBuiltinCall::ArraySet => return None,
    })
}

fn infer_intrinsic_output_type(
    op_kind: &IntrinsicCallLowering,
    arguments: &[LirOperand],
    value_types: &BTreeMap<MirValueRef, HirType>,
) -> Option<HirType> {
    match op_kind {
        IntrinsicCallLowering::Simple(opcode) => match opcode {
            MsilOpcode::Ceq | MsilOpcode::Clt | MsilOpcode::Cgt => Some(HirType::Boolean),
            _ => arguments.first().and_then(|argument| infer_operand_type(argument, value_types)),
        },
        IntrinsicCallLowering::NegatedComparison(_) | IntrinsicCallLowering::LogicalNot => Some(HirType::Boolean),
        IntrinsicCallLowering::ArrayLength => Some(HirType::Integer32 { signed: true }),
        IntrinsicCallLowering::Identity => arguments.first().and_then(|argument| infer_operand_type(argument, value_types)),
    }
}

/// 判断 callee 的返回类型是否为 void。
///
/// 对于 `Symbol` callee（函数名），从 `return_types` 映射中查找。
/// 对于 `Value` 或 `Constant` callee（闭包等），默认认为不返回 void。
fn callee_returns_void(callee: &LirOperand, return_types: &BTreeMap<String, HirType>) -> bool {
    let LirOperand::Symbol(path) = callee
    else {
        return false;
    };
    let symbol_name = path.to_string();
    let return_type = match return_types.get(&symbol_name) {
        Some(ty) => ty,
        None => return false,
    };
    matches!(return_type, HirType::Unit | HirType::Void)
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
        LirOperationKind::Call { callee, arguments, builtin, .. } => {
            if matches!(builtin, Some(MirBuiltinCall::ArrayGet)) {
                lower_array_subscript_access(
                    &arguments[0],
                    &arguments[1],
                    operation.output,
                    instructions,
                    max_stack,
                    parameter_slots,
                    local_slots,
                    local_types,
                    eval_stack,
                    value_types,
                );
            }
            else if matches!(builtin, Some(MirBuiltinCall::ArraySet)) {
                lower_array_subscript_store(
                    &arguments[0],
                    &arguments[1],
                    &arguments[2],
                    instructions,
                    max_stack,
                    parameter_slots,
                    local_slots,
                    local_types,
                    eval_stack,
                    value_types,
                );
            }
            else if let Some(op_kind) = builtin.and_then(intrinsic_from_builtin) {
                if let Some(output) = operation.output {
                    if let Some(inferred_type) = infer_intrinsic_output_type(&op_kind, arguments, value_types) {
                        value_types.insert(output, inferred_type);
                    }
                }
                lower_intrinsic_call(
                    op_kind,
                    arguments,
                    operation.output,
                    instructions,
                    max_stack,
                    parameter_slots,
                    local_slots,
                    local_types,
                    eval_stack,
                    value_types,
                    None,
                );
            }
            else {
                // 常规 call：先溢出 eval_stack，再逐个压入参数。
                spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
                for argument in arguments {
                    lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                }
                let callee_operand = match callee {
                    LirOperand::Symbol(path) => Some(MsilInstructionOperand::Symbol(path.to_string())),
                    LirOperand::Value(_) | LirOperand::Constant(_) => {
                        lower_operand(callee, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                        None
                    }
                };
                instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Call, operand: callee_operand });
                *max_stack = (*max_stack).max((arguments.len() + 1) as u16);
                // call 消费所有参数，产出 1 个返回值。清空 eval_stack 并压入结果。
                // 但若 callee 返回 void，则不应将结果推入 eval_stack（CLR 的 call 指令对 void 方法不在栈上留下返回值）。
                eval_stack.clear();
                if let Some(output) = operation.output {
                    // 检查 callee 的返回类型是否为 void。
                    let returns_void = callee_returns_void(callee, return_types);
                    if !returns_void {
                        if let Some(inferred_type) = infer_call_return_type(callee, return_types) {
                            value_types.insert(output, inferred_type);
                        }
                        eval_stack.push(output);
                        *max_stack = (*max_stack).max(eval_stack.len() as u16);
                    }
                }
            }
        }
        LirOperationKind::ArrayNew { element_type, length } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            lower_operand(length, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Newarr,
                operand: Some(MsilInstructionOperand::Type(lower_hir_type(element_type).to_string())),
            });
            *max_stack = (*max_stack).max(1);
            eval_stack.clear();
            if let Some(output) = operation.output {
                value_types.insert(output, HirType::Array(Box::new(element_type.clone())));
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
        }
        LirOperationKind::ArrayLiteral { element_type, items } => {
            lower_array_literal(
                element_type,
                items,
                operation.output,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                value_types,
            );
        }
        LirOperationKind::StructNew { type_name, fields } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            let ordered_fields = order_struct_fields(type_name, fields, local_structs);
            let parameter_types = ordered_fields
                .iter()
                .map(|(field_name, value)| {
                    local_structs
                        .get(type_name)
                        .and_then(|layout| layout.field_types.get(field_name))
                        .map(lower_hir_type)
                        .or_else(|| infer_operand_type(value, value_types).map(|ty| lower_hir_type(&ty)))
                        .unwrap_or(MsilType::Object)
                })
                .collect::<Vec<_>>();
            for (_, value) in &ordered_fields {
                lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            }
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Newobj,
                operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                    owner: Some(normalize_local_type_name(type_name, local_structs).unwrap_or_else(|| type_name.clone())),
                    name: ".ctor".to_string(),
                    signature: MsilMethodSignature::new(MsilType::Void, parameter_types),
                })),
            });
            *max_stack = (*max_stack).max(ordered_fields.len() as u16);
            eval_stack.clear();
            if let Some(output) = operation.output {
                value_types.insert(output, HirType::Named(Identifier::new(extract_simple_type_name(type_name))));
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
        }
        LirOperationKind::FieldGet { object, field } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            if field == "length" {
                // 当前 `LIR` 的 FieldGet 不携带 owner/type，像 `array.length` 这类内建长度访问
                // 到 `PE` 阶段已无法恢复成真实字段 token，因此在 lowering 阶段直接降为数组长度指令。
                instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ldlen, operand: None });
                instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::ConvI4, operand: None });
                *max_stack = (*max_stack).max(1);
                eval_stack.clear();
                if let Some(output) = operation.output {
                    value_types.insert(output, HirType::Integer32 { signed: true });
                    eval_stack.push(output);
                    *max_stack = (*max_stack).max(eval_stack.len() as u16);
                }
            }
            else {
                let field_owner = infer_field_owner_name(object, field, value_types, local_structs).unwrap_or_default();
                instructions.push(MsilInstruction {
                    label: None,
                    opcode: MsilOpcode::Ldfld,
                    operand: Some(MsilInstructionOperand::Field(field_owner, field.clone())),
                });
                *max_stack = (*max_stack).max(1);
                eval_stack.clear();
                if let Some(output) = operation.output {
                    if let Some(inferred_type) = infer_field_output_type(object, field, value_types, local_structs) {
                        value_types.insert(output, inferred_type);
                    }
                    eval_stack.push(output);
                    *max_stack = (*max_stack).max(eval_stack.len() as u16);
                }
            }
        }
        LirOperationKind::FieldSet { object, field, value } => {
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            let field_owner = infer_field_owner_name(object, field, value_types, local_structs).unwrap_or_default();
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Stfld,
                operand: Some(MsilInstructionOperand::Field(field_owner, field.clone())),
            });
            *max_stack = (*max_stack).max(2);
            eval_stack.clear();
        }
        LirOperationKind::PatternMatch { .. } => {
            lower_constant(&MirConstant::Bool(false), instructions, max_stack, None);
            if let Some(output) = operation.output {
                value_types.insert(output, HirType::Boolean);
                eval_stack.push(output);
                *max_stack = (*max_stack).max(eval_stack.len() as u16);
            }
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

fn infer_operand_type(operand: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> Option<HirType> {
    match operand {
        LirOperand::Value(value_ref) => value_types.get(value_ref).cloned(),
        LirOperand::Constant(constant) => infer_constant_type(constant),
        LirOperand::Symbol(_) => None,
    }
}

fn infer_call_return_type(callee: &LirOperand, return_types: &BTreeMap<String, HirType>) -> Option<HirType> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    return_types.get(&path.to_string()).cloned()
}

fn infer_subscript_output_type(object: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> Option<HirType> {
    match infer_operand_type(object, value_types)? {
        HirType::Array(item) => Some(*item),
        _ => None,
    }
}

fn is_direct_array_operand(operand: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> bool {
    matches!(infer_operand_type(operand, value_types), Some(HirType::Array(_)))
}

fn lower_array_subscript_access(
    object: &LirOperand,
    index: &LirOperand,
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    // 数组下标访问需要按 `array, index` 的顺序入栈。
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    match index {
        LirOperand::Value(value_ref) => {
            lower_value_ref(value_ref, instructions, max_stack, parameter_slots, local_slots);
        }
        LirOperand::Constant(constant) => {
            lower_constant(constant, instructions, max_stack, None);
        }
        LirOperand::Symbol(path) => {
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Ldsfld,
                operand: Some(MsilInstructionOperand::Symbol(path.to_string())),
            });
            *max_stack = (*max_stack).max(1);
        }
    }

    let opcode =
        infer_subscript_output_type(object, value_types).map(|ty| array_load_opcode(&lower_hir_type(&ty))).unwrap_or(MsilOpcode::LdelemRef);
    instructions.push(MsilInstruction { label: None, opcode, operand: None });
    *max_stack = (*max_stack).max(2);

    if let LirOperand::Value(_) = object {
        eval_stack.pop();
    }

    if let Some(output) = output {
        if let Some(inferred_type) = infer_subscript_output_type(object, value_types) {
            value_types.insert(output, inferred_type);
        }
        eval_stack.push(output);
    }
}

fn lower_array_subscript_store(
    object: &LirOperand,
    index: &LirOperand,
    value: &LirOperand,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    lower_operand(index, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    let opcode =
        infer_subscript_output_type(object, value_types).map(|ty| array_store_opcode(&lower_hir_type(&ty))).unwrap_or(MsilOpcode::StelemRef);
    instructions.push(MsilInstruction { label: None, opcode, operand: None });
    *max_stack = (*max_stack).max(3);
    eval_stack.clear();
}

fn lower_array_literal(
    element_type: &HirType,
    items: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) {
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
    lower_constant(&MirConstant::Int(items.len() as i64), instructions, max_stack, None);
    instructions.push(MsilInstruction {
        label: None,
        opcode: MsilOpcode::Newarr,
        operand: Some(MsilInstructionOperand::Type(lower_hir_type(element_type).to_string())),
    });
    *max_stack = (*max_stack).max(1);

    let array_slot = local_types.len();
    local_types.push(MsilType::sz_array(lower_hir_type(element_type)));
    emit_stloc(array_slot, instructions, None);

    let store_opcode = array_store_opcode(&lower_hir_type(element_type));
    for (index, item) in items.iter().enumerate() {
        lower_local_slot(array_slot, instructions, max_stack);
        lower_constant(&MirConstant::Int(index as i64), instructions, max_stack, None);
        lower_operand(item, instructions, max_stack, parameter_slots, local_slots, eval_stack);
        instructions.push(MsilInstruction { label: None, opcode: store_opcode, operand: None });
        *max_stack = (*max_stack).max(3);
        eval_stack.clear();
    }

    if let Some(output) = output {
        value_types.insert(output, HirType::Array(Box::new(element_type.clone())));
        local_slots.insert(output, array_slot);
        lower_value_ref(&output, instructions, max_stack, parameter_slots, local_slots);
        eval_stack.push(output);
    }
}

fn try_lower_direct_array_subscript_call(
    callee: &LirOperand,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &mut BTreeMap<MirValueRef, HirType>,
) -> bool {
    let LirOperand::Symbol(path) = callee
    else {
        return false;
    };
    if path.parts().len() != 1 {
        return false;
    }

    match path.parts()[0].as_str() {
        "suffix []" if arguments.len() == 2 && is_direct_array_operand(&arguments[0], value_types) => {
            lower_array_subscript_access(
                &arguments[0],
                &arguments[1],
                output,
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                value_types,
            );
            true
        }
        "suffix []=" if arguments.len() == 3 && is_direct_array_operand(&arguments[0], value_types) => {
            lower_array_subscript_store(
                &arguments[0],
                &arguments[1],
                &arguments[2],
                instructions,
                max_stack,
                parameter_slots,
                local_slots,
                local_types,
                eval_stack,
                value_types,
            );
            true
        }
        _ => false,
    }
}

fn array_load_opcode(ty: &MsilType) -> MsilOpcode {
    match ty {
        MsilType::Bool | MsilType::Int8 | MsilType::UInt8 => MsilOpcode::LdelemU1,
        MsilType::Int32 | MsilType::UInt32 => MsilOpcode::LdelemI4,
        _ => MsilOpcode::LdelemRef,
    }
}

fn array_store_opcode(ty: &MsilType) -> MsilOpcode {
    match ty {
        MsilType::Bool | MsilType::Int8 | MsilType::UInt8 => MsilOpcode::StelemI1,
        MsilType::Int32 | MsilType::UInt32 => MsilOpcode::StelemI4,
        _ => MsilOpcode::StelemRef,
    }
}

fn infer_field_owner_name(
    object: &LirOperand,
    field: &str,
    value_types: &BTreeMap<MirValueRef, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> Option<String> {
    if let Some(local_type_name) = infer_named_type_name(object, value_types) {
        return local_structs.get(&local_type_name).map(|layout| layout.qualified_name.clone());
    }

    // 当值类型推断暂时缺失时，退化到“字段名唯一”匹配，避免本地字段被误当成外部引用。
    let matches = local_structs
        .values()
        .filter(|layout| layout.field_types.contains_key(field))
        .map(|layout| layout.qualified_name.clone())
        .collect::<std::collections::BTreeSet<_>>();
    if matches.len() == 1 {
        return matches.into_iter().next();
    }
    None
}

fn infer_field_output_type(
    object: &LirOperand,
    field: &str,
    value_types: &BTreeMap<MirValueRef, HirType>,
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> Option<HirType> {
    let local_type_name = infer_named_type_name(object, value_types)?;
    local_structs.get(&local_type_name)?.field_types.get(field).cloned()
}

fn infer_named_type_name(object: &LirOperand, value_types: &BTreeMap<MirValueRef, HirType>) -> Option<String> {
    match infer_operand_type(object, value_types)? {
        HirType::Named(name) => Some(name.to_string()),
        _ => None,
    }
}

fn normalize_local_type_name(type_name: &str, local_structs: &BTreeMap<String, LocalStructLayout>) -> Option<String> {
    let layout = local_structs.get(type_name)?;
    if type_name == layout.name {
        Some(layout.qualified_name.clone())
    }
    else {
        Some(type_name.to_string())
    }
}

fn extract_simple_type_name(type_name: &str) -> &str {
    type_name.rsplit('.').next().unwrap_or(type_name)
}

fn order_struct_fields(
    type_name: &str,
    fields: &[(String, LirOperand)],
    local_structs: &BTreeMap<String, LocalStructLayout>,
) -> Vec<(String, LirOperand)> {
    let Some(layout) = local_structs.get(type_name)
    else {
        return fields.to_vec();
    };

    let supplied_fields = fields.iter().cloned().collect::<BTreeMap<_, _>>();
    let mut ordered = Vec::with_capacity(fields.len());
    for field_name in &layout.field_order {
        if let Some(value) = supplied_fields.get(field_name) {
            ordered.push((field_name.clone(), value.clone()));
        }
    }

    if ordered.len() == fields.len() {
        ordered
    }
    else {
        fields.to_vec()
    }
}

fn lower_hir_type(ty: &HirType) -> MsilType {
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
fn emit_stloc(slot: usize, instructions: &mut Vec<MsilInstruction>, label: Option<String>) {
    let (opcode, operand) = match slot {
        0 => (MsilOpcode::Stloc0, None),
        1 => (MsilOpcode::Stloc1, None),
        2 => (MsilOpcode::Stloc2, None),
        3 => (MsilOpcode::Stloc3, None),
        _ => (MsilOpcode::Stloc, Some(MsilInstructionOperand::Integer(slot as i64))),
    };
    instructions.push(MsilInstruction { label, opcode, operand });
}

fn lower_local_slot(slot: usize, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16) {
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
fn spill_eval_stack(
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

/// 降级内建运算符方法：若操作数恰好在求值栈顶，直接 emit 指令序列；
/// 否则先溢出求值栈到局部变量，再逐个加载操作数后 emit opcode。
fn lower_intrinsic_call(
    op_kind: IntrinsicCallLowering,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    value_types: &BTreeMap<MirValueRef, HirType>,
    label: Option<String>,
) {
    let arg_count = arguments.len();
    // 快速路径：所有参数都是 Value 且恰好匹配 eval_stack 顶部 N 个。
    let all_values = arguments.iter().all(|arg| matches!(arg, LirOperand::Value(_)));
    if all_values && eval_stack.len() >= arg_count && arg_count >= 1 {
        let matches = (0..arg_count).all(|i| {
            let arg_val = match &arguments[i] {
                LirOperand::Value(v) => *v,
                _ => unreachable!(),
            };
            let stack_val = eval_stack[eval_stack.len() - arg_count + i];
            arg_val == stack_val
        });
        if matches {
            // 操作数已在栈上，直接 emit 内建指令序列。
            emit_intrinsic_call(op_kind, label, instructions, max_stack, arg_count, eval_stack, output);
            return;
        }
    }

    // 慢速路径：先溢出 eval_stack 到局部变量，确保所有 Value 都可通过 local_slots 访问。
    spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);

    // 逐个加载操作数到求值栈。
    for argument in arguments {
        lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    }

    emit_intrinsic_call(op_kind, label, instructions, max_stack, arg_count, eval_stack, output);
}

/// 发射内建运算符方法对应的指令序列。
///
/// `Simple`：单条 opcode。
/// `NegatedComparison`：比较 opcode + `ldc.i4.0` + `ceq`。
/// `LogicalNot`：`ldc.i4.0 + ceq`。
fn emit_intrinsic_call(
    op_kind: IntrinsicCallLowering,
    label: Option<String>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    arg_count: usize,
    eval_stack: &mut Vec<MirValueRef>,
    output: Option<MirValueRef>,
) {
    match op_kind {
        IntrinsicCallLowering::Simple(opcode) => {
            instructions.push(MsilInstruction { label, opcode, operand: None });
            *max_stack = (*max_stack).max(arg_count as u16);
        }
        IntrinsicCallLowering::NegatedComparison(opcode) => {
            instructions.push(MsilInstruction { label, opcode, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ceq, operand: None });
            *max_stack = (*max_stack).max(2);
        }
        IntrinsicCallLowering::LogicalNot => {
            instructions.push(MsilInstruction { label, opcode: MsilOpcode::LdcI4_0, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ceq, operand: None });
            *max_stack = (*max_stack).max(2);
        }
        IntrinsicCallLowering::ArrayLength => {
            // ldlen 推送 native int，conv.i4 转换为 int32。
            instructions.push(MsilInstruction { label, opcode: MsilOpcode::Ldlen, operand: None });
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::ConvI4, operand: None });
            *max_stack = (*max_stack).max(1);
        }
        IntrinsicCallLowering::Identity => {
            *max_stack = (*max_stack).max(arg_count as u16);
        }
    }

    for _ in 0..arg_count {
        eval_stack.pop();
    }

    if let Some(output) = output {
        eval_stack.push(output);
    }
}

fn lower_constant(constant: &MirConstant, instructions: &mut Vec<MsilInstruction>, max_stack: &mut u16, label: Option<String>) {
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

fn lower_operand(
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

fn lower_value_ref(
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

fn lower_terminator(
    terminator: &LirTerminator,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    _var_slots: &mut BTreeMap<String, usize>,
    eval_stack: &mut Vec<MirValueRef>,
    return_type: &HirType,
    value_types: &BTreeMap<MirValueRef, HirType>,
    block_labels: &BTreeMap<MirBlockRef, String>,
    block_params: &BTreeMap<MirBlockRef, Vec<MirValueRef>>,
) {
    match terminator {
        LirTerminator::Return { value } => {
            let returns_void = matches!(return_type, HirType::Unit | HirType::Void);
            if !returns_void {
                if let Some(value_operand) = value {
                    let on_top = match value_operand {
                        LirOperand::Value(v) => eval_stack.last() == Some(v),
                        _ => false,
                    };
                    if !on_top {
                        // 若值不在栈顶，先溢出 eval_stack 再加载。
                        spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
                        lower_operand(value_operand, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                    }
                }
            }
            else {
                eval_stack.clear();
            }
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
        }
        LirTerminator::Jump { target, arguments } => {
            // 在跳转前将参数值写入目标块的参数槽位，保持 SSA 块参数语义。
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            // 找到目标块的参数列表。
            let target_params = block_params.get(target).cloned().unwrap_or_default();
            // 逐个参数写入对应的局部槽位。
            for (index, arg) in arguments.iter().enumerate() {
                lower_operand(arg, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                // 通过 `local_slots` 查找目标块参数的槽位。
                // 这里依赖前置步骤已为非入口块参数分配槽位。
                // 我们需要定位目标块的第 `index` 个参数的 MirValueRef。
                if let Some(param_ref) = target_params.get(index) {
                    let slot = *local_slots.get(param_ref).expect("缺少目标块参数的本地槽位");
                    emit_stloc(slot, instructions, None);
                }
                // 每次 stloc 消耗一个栈顶值。
                eval_stack.clear();
            }
            // 发出实际跳转。
            let target_label = block_labels.get(target).cloned().unwrap_or_else(|| format!("BB{}", target.0));
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Br,
                operand: Some(MsilInstructionOperand::BranchTarget(target_label)),
            });
        }
        LirTerminator::Branch { condition, then_target, else_target, .. } => {
            // 分支前无条件清空求值栈，再加载条件，确保所有分支目标的栈高度一致。
            spill_eval_stack(eval_stack, local_slots, local_types, instructions, value_types);
            lower_operand(condition, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            let then_label = block_labels.get(then_target).cloned().unwrap_or_else(|| format!("BB{}", then_target.0));
            let else_label = block_labels.get(else_target).cloned().unwrap_or_else(|| format!("BB{}", else_target.0));
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Brtrue,
                operand: Some(MsilInstructionOperand::BranchTarget(then_label)),
            });
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Br,
                operand: Some(MsilInstructionOperand::BranchTarget(else_label)),
            });
            // `brtrue` 会消费条件值；这里同步清空内部栈跟踪，避免后续块继续把旧条件当成仍在栈顶。
            eval_stack.clear();
        }
        LirTerminator::PerformEffect { effect, .. } => {
            panic!("CLR backend 暂未支持 {:?} effect terminator lowering", effect);
        }
        LirTerminator::Unreachable => {
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
        }
    }
}

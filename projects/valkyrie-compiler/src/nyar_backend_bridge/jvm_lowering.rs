//! `JVM` 路线 lowering。
//!
//! 这里把 lane-aware `LIR` 收口成最小可编码的 `JVM class` 模型。
//! 当前刻意只实现一条很窄的路径：
//! - 顶层函数 -> 单个静态方法
//! - 参数与局部值先按单槽位整型处理
//! - 控制流仍由 `LIR` 基本块表达，`JVM` 细节只在本 crate 内部消化

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use super::{
    jvm_host_bridge::is_jvm_host_bridge_symbol,
    jvm_intrinsics::{JvmBinaryNumericOp, JvmIntComparison},
    jvm_operation_lowering::{infer_operation_output_type, lower_operand, lower_operand_with_hint, lower_operation},
};
use crate::{
    lir::{LirFunction, LirModule, LirOperand, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirBuiltinCall, MirConstant, MirValueRef},
    symbols::{is_main_symbol, mangle_emitted_symbol},
};
use jvm_backend::{JvmClassFile, JvmCodeBody, JvmInstruction, JvmMethodDescriptor, JvmMethodSignature, JvmTypeDescriptor};
use miette::{miette, Result};
use nyar::{
    abstractions::{BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use valkyrie_types::hir::{TraitObject, ValkyrieType};

const JVM_ACC_PUBLIC: u16 = 0x0001;
const JVM_ACC_STATIC: u16 = 0x0008;
const DEFAULT_MAX_STACK: u16 = 16;

/// `JVM` lane 的 `LIR -> ClassFile` 承接器。
pub struct JvmLirLoweringLane {
    descriptor: TargetLoweringLaneDescriptor,
}

impl JvmLirLoweringLane {
    /// 创建一个新的 `JVM` lane lowering。
    pub fn new() -> Self {
        Self {
            descriptor: TargetLoweringLaneDescriptor {
                name: "jvm-class-lowering".to_string(),
                lane: TargetLane::Jvm,
                input_kind: BackendInputKind::JvmClassFile,
                target: BinaryTarget::new(TargetFamily::Jvm, BinaryArch::Any, BinaryFlavor::ManagedClr),
            },
        }
    }
}

impl Default for JvmLirLoweringLane {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetLoweringLane for JvmLirLoweringLane {
    type PartitionInput = LirModule;
    type BackendInput = JvmClassFile;

    fn descriptor(&self) -> &TargetLoweringLaneDescriptor {
        &self.descriptor
    }

    fn lower_partition(&self, partition: Self::PartitionInput) -> Result<LaneLoweringResult<Self::BackendInput>> {
        if partition.lane != LirTargetLane::Jvm {
            return Err(miette!(
                code = "nyar::jvm::lowering::lane_mismatch",
                help = "请确认当前 `LIR` 分区已经选择 `JVM` 目标路线",
                "当前 lane 是 {:?}，不能进入 JVM lowering",
                partition.lane
            ));
        }

        let artifact_name = partition.name.clone();
        let input = lower_lir_to_jvm_class(&partition)?;
        Ok(LaneLoweringResult { input, artifact_name })
    }
}

/// 将 `LIR` 模块降低为单个 `JVM class`。
pub fn lower_lir_to_jvm_class(lir: &LirModule) -> Result<JvmClassFile> {
    let internal_name = module_internal_name(&lir.name);
    let reachable_symbols = collect_reachable_function_symbols(lir);
    let signatures = collect_function_signatures(lir, &reachable_symbols)?;
    let field_types = collect_field_types(lir)?;
    let mut class_file = JvmClassFile::new(internal_name.clone());
    // 按 (name, descriptor) 去重，避免合并多源文件时同名方法冲突。
    // 保留首次出现的方法，后续重复定义被跳过。
    let mut seen: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    let mut methods = Vec::with_capacity(reachable_symbols.len());
    for function in &lir.functions {
        if !reachable_symbols.contains(&function.symbol) || is_jvm_host_bridge_symbol(&function.symbol) {
            continue;
        }
        let method = lower_function(function, &internal_name, &signatures, &field_types)?;
        let key = (method.name.clone(), method.descriptor.to_string());
        if seen.insert(key) {
            methods.push(method);
        }
    }
    class_file.methods = methods;
    class_file.optimize_static_self_tail_recursion().map_err(|error| miette!("JVM 尾递归循环化失败：{error}"))?;
    Ok(class_file)
}

fn lower_function(
    function: &LirFunction,
    owner: &str,
    signatures: &BTreeMap<String, JvmMethodDescriptor>,
    field_types: &BTreeMap<(String, String), JvmTypeDescriptor>,
) -> Result<JvmMethodSignature> {
    let descriptor = signatures.get(&function.symbol).cloned().ok_or_else(|| miette!("缺少函数 `{}` 的 JVM 描述符", function.symbol))?;
    let reachable_block_ids = collect_reachable_blocks(function);
    let reachable_blocks: Vec<&crate::lir::LirBlock> = function.blocks.iter().filter(|block| reachable_block_ids.contains(&block.id)).collect();
    let block_labels: BTreeMap<MirBlockRef, String> = reachable_blocks.iter().map(|block| (block.id, format!("BB{}", block.id.0))).collect();
    // 按参数的 JVM 类型槽位计算入口局部变量起始偏移。
    // `long` / `double` 占 2 槽，若仅按参数数量计算会导致 `max_locals` 不足。
    let entry_parameters = function.param_types.iter().map(|ty| jvm_type_descriptor(ty).map(|d| d.slot_count()).unwrap_or(1)).sum::<u16>();
    let mut context = FunctionLoweringContext::new(function.symbol.as_str(), owner, signatures, field_types, &block_labels, entry_parameters);
    context.reserve_block_parameters(&reachable_blocks, function);
    context.reserve_operation_outputs(&reachable_blocks);
    // 先传播变量声明类型，再推断 block 参数，避免局部数组值被提前回退成 `Int`。
    context.propagate_var_decl_types(&reachable_blocks);
    context.propagate_return_operand_types(&reachable_blocks, function);
    // 在操作输出类型已知后，通过跳转参数推断非入口 block 的参数类型。
    context.infer_block_parameter_types(&reachable_blocks, function);
    // 预分配所有 StoreVar 操作的变量槽位，确保方法调用内建函数能找到接收者变量。
    context.preallocate_var_slots(&reachable_blocks);

    let referenced_targets = collect_referenced_targets(&reachable_blocks);
    let mut instructions = Vec::new();

    for block in reachable_blocks {
        let emit_label = referenced_targets.contains(&block.id) || block.id == function.entry || block.operations.is_empty();
        if emit_label {
            instructions.push(JvmInstruction::Label(block_labels[&block.id].clone()));
        }

        for operation in &block.operations {
            lower_operation(operation, &mut context, &mut instructions)?;
        }

        lower_terminator(&block.terminator, &function.return_type, &mut context, &mut instructions)?;
    }

    if instructions.is_empty() {
        // 空方法体：按返回类型压入默认值后返回，避免 JVM 栈下溢。
        push_default_value(&function.return_type, &mut instructions);
        instructions.push(default_return_instruction(&function.return_type)?);
    }

    Ok(JvmMethodSignature {
        name: emitted_function_name(&function.symbol),
        descriptor,
        access_flags: JVM_ACC_PUBLIC | JVM_ACC_STATIC,
        code: Some(JvmCodeBody { max_stack: DEFAULT_MAX_STACK, max_locals: context.max_locals(), instructions }),
    })
}

fn collect_reachable_function_symbols(lir: &LirModule) -> BTreeSet<String> {
    let function_symbols: BTreeSet<String> = lir.functions.iter().map(|function| function.symbol.clone()).collect();
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();

    if let Some(entry_symbol) = lir.functions.iter().find(|function| is_main_symbol(&function.symbol)).map(|function| function.symbol.clone()) {
        queue.push_back(entry_symbol);
    }
    else {
        for function in &lir.functions {
            queue.push_back(function.symbol.clone());
        }
    }

    while let Some(symbol) = queue.pop_front() {
        if !reachable.insert(symbol.clone()) {
            continue;
        }
        let Some(function) = lir.functions.iter().find(|function| function.symbol == symbol)
        else {
            continue;
        };

        for block in &function.blocks {
            for operation in &block.operations {
                let LirOperationKind::Call { callee, .. } = &operation.kind
                else {
                    continue;
                };
                let LirOperand::Symbol(path) = callee
                else {
                    continue;
                };
                let next_symbol = logical_symbol_name(path);
                let Some(next_symbol) = next_symbol
                else {
                    continue;
                };
                if let Some(resolved_symbol) = resolve_reachable_symbol(&next_symbol, &function_symbols) {
                    if !is_jvm_host_bridge_symbol(resolved_symbol) {
                        queue.push_back(resolved_symbol.to_string());
                    }
                }
            }
        }
    }

    reachable
}

fn resolve_reachable_symbol<'a>(symbol: &'a str, function_symbols: &'a BTreeSet<String>) -> Option<&'a str> {
    if function_symbols.contains(symbol) {
        return Some(symbol);
    }

    let mut matches = function_symbols.iter().filter(|key| key.rsplit("::").next() == Some(symbol));
    let first = matches.next()?;
    if matches.next().is_some() {
        None
    }
    else {
        Some(first.as_str())
    }
}

fn collect_function_signatures(lir: &LirModule, reachable_symbols: &BTreeSet<String>) -> Result<BTreeMap<String, JvmMethodDescriptor>> {
    lir.functions
        .iter()
        .filter(|function| reachable_symbols.contains(&function.symbol))
        .map(|function| Ok((function.symbol.clone(), build_method_descriptor(function)?)))
        .collect()
}

pub(super) fn emitted_function_name(symbol: &str) -> String {
    if is_main_symbol(symbol) {
        "main".to_string()
    }
    else {
        mangle_emitted_symbol(symbol)
    }
}

pub(super) fn logical_symbol_name(path: &valkyrie_types::NamePath) -> Option<String> {
    if path.is_empty() {
        None
    }
    else {
        Some(path.parts().iter().map(|segment| segment.as_str()).collect::<Vec<_>>().join("::"))
    }
}

fn collect_field_types(lir: &LirModule) -> Result<BTreeMap<(String, String), JvmTypeDescriptor>> {
    let mut field_types = BTreeMap::new();
    for item in &lir.structs {
        for field in &item.fields {
            let jvm_ty = jvm_type_descriptor(&field.ty)?;
            field_types.insert((item.name.clone(), field.name.clone()), jvm_ty);
        }
    }
    Ok(field_types)
}

fn build_method_descriptor(function: &LirFunction) -> Result<JvmMethodDescriptor> {
    let parameter_count = entry_block(function)?.parameters.len();
    // 使用函数参数类型列表生成正确的 JVM 描述符。
    // 若 param_types 长度不足，缺失部分按 Int 处理。
    let mut parameter_types = Vec::with_capacity(parameter_count);
    for i in 0..parameter_count {
        let ty = match function.param_types.get(i) {
            Some(hir_ty) => jvm_type_descriptor(hir_ty)?,
            None => JvmTypeDescriptor::Int,
        };
        parameter_types.push(ty);
    }
    Ok(JvmMethodDescriptor::new(parameter_types, jvm_type_descriptor(&function.return_type)?))
}

fn entry_block(function: &LirFunction) -> Result<&crate::lir::LirBlock> {
    function.blocks.iter().find(|block| block.id == function.entry).ok_or_else(|| miette!("函数 `{}` 缺少入口块", function.symbol))
}

fn collect_reachable_blocks(function: &LirFunction) -> BTreeSet<MirBlockRef> {
    let blocks_by_id: BTreeMap<MirBlockRef, &crate::lir::LirBlock> = function.blocks.iter().map(|block| (block.id, block)).collect();
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::from([function.entry]);
    while let Some(block_id) = queue.pop_front() {
        if !reachable.insert(block_id) {
            continue;
        }
        let Some(block) = blocks_by_id.get(&block_id).copied()
        else {
            continue;
        };
        match &block.terminator {
            LirTerminator::Jump { target, .. } => {
                queue.push_back(*target);
            }
            LirTerminator::Branch { then_target, else_target, .. } => {
                queue.push_back(*then_target);
                queue.push_back(*else_target);
            }
            LirTerminator::PerformEffect { resume_target, .. } => {
                queue.push_back(*resume_target);
            }
            LirTerminator::Return { .. } | LirTerminator::Unreachable => {}
        }
    }
    reachable
}

fn collect_referenced_targets(blocks: &[&crate::lir::LirBlock]) -> BTreeSet<MirBlockRef> {
    let mut referenced_targets = BTreeSet::new();
    for block in blocks {
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
            LirTerminator::Return { .. } | LirTerminator::Unreachable => {}
        }
    }
    referenced_targets
}

pub(super) struct FunctionLoweringContext<'a> {
    function_symbol: &'a str,
    pub(super) owner: &'a str,
    pub(super) signatures: &'a BTreeMap<String, JvmMethodDescriptor>,
    field_types: &'a BTreeMap<(String, String), JvmTypeDescriptor>,
    block_labels: &'a BTreeMap<MirBlockRef, String>,
    block_parameter_slots: BTreeMap<MirBlockRef, Vec<u16>>,
    pub(super) value_slots: BTreeMap<MirValueRef, u16>,
    /// 每个 SSA 值对应的 JVM 类型描述符，用于选择正确的 load/store 指令。
    pub(super) value_types: BTreeMap<MirValueRef, JvmTypeDescriptor>,
    var_slots: BTreeMap<String, u16>,
    /// 命名变量对应的 JVM 类型描述符，用于选择正确的 load/store 指令。
    pub(super) var_types: BTreeMap<String, JvmTypeDescriptor>,
    next_slot: u16,
    label_seed: usize,
}

impl<'a> FunctionLoweringContext<'a> {
    fn new(
        function_symbol: &'a str,
        owner: &'a str,
        signatures: &'a BTreeMap<String, JvmMethodDescriptor>,
        field_types: &'a BTreeMap<(String, String), JvmTypeDescriptor>,
        block_labels: &'a BTreeMap<MirBlockRef, String>,
        entry_parameters: u16,
    ) -> Self {
        Self {
            function_symbol,
            owner,
            signatures,
            field_types,
            block_labels,
            block_parameter_slots: BTreeMap::new(),
            value_slots: BTreeMap::new(),
            value_types: BTreeMap::new(),
            var_slots: BTreeMap::new(),
            var_types: BTreeMap::new(),
            next_slot: entry_parameters,
            label_seed: 0,
        }
    }

    fn reserve_block_parameters(&mut self, blocks: &[&crate::lir::LirBlock], function: &LirFunction) {
        for block in blocks {
            let mut slots = Vec::with_capacity(block.parameters.len());
            for (index, parameter) in block.parameters.iter().enumerate() {
                let slot = if block.id == function.entry {
                    // 入口 block 的参数槽位按 JVM 类型槽位递增分配。
                    // `long` / `double` 占 2 槽，后续参数需跳过额外槽位。
                    let mut slot = 0u16;
                    for param_ty in function.param_types.iter().take(index) {
                        slot += jvm_type_descriptor(param_ty).map(|d| d.slot_count()).unwrap_or(1);
                    }
                    slot
                }
                else {
                    self.allocate_slot()
                };
                slots.push(slot);
                self.value_slots.entry(*parameter).or_insert(slot);
                // 入口 block 的参数类型来自函数参数类型列表。
                if block.id == function.entry {
                    if let Some(param_ty) = function.param_types.get(index) {
                        if let Ok(jvm_ty) = jvm_type_descriptor(param_ty) {
                            self.value_types.insert(*parameter, jvm_ty);
                        }
                    }
                }
            }
            self.block_parameter_slots.insert(block.id, slots);
        }
    }

    /// 通过跳转参数推断非入口 block 的参数类型。
    /// 遍历所有 Jump/Branch terminator，根据跳转参数的类型设置目标 block 参数的类型。
    fn infer_block_parameter_types(&mut self, blocks: &[&crate::lir::LirBlock], function: &LirFunction) {
        for block in blocks {
            match &block.terminator {
                LirTerminator::Jump { target, arguments } => {
                    self.infer_target_parameter_types(*target, arguments, function);
                }
                LirTerminator::Branch { then_target, else_target, .. } => {
                    // Branch 没有 block 参数，但保险起见仍尝试推断。
                    let empty: &[LirOperand] = &[];
                    self.infer_target_parameter_types(*then_target, empty, function);
                    self.infer_target_parameter_types(*else_target, empty, function);
                }
                _ => {}
            }
        }
    }

    fn infer_target_parameter_types(&mut self, target: MirBlockRef, arguments: &[LirOperand], function: &LirFunction) {
        // 找到目标 block 的参数列表。
        let target_block = match function.blocks.iter().find(|b| b.id == target) {
            Some(block) => block,
            None => return,
        };
        for (param, arg) in target_block.parameters.iter().zip(arguments.iter()) {
            // 如果参数类型已设置，跳过。
            if self.value_types.contains_key(param) {
                continue;
            }
            // 从跳转参数推断类型。
            let ty = operand_type(arg, self);
            self.value_types.insert(*param, ty);
        }
    }

    /// 预留所有操作输出的局部槽位，并推断每个输出的 JVM 类型。
    ///
    /// `LIR` 的 block 顺序不一定与数据流顺序一致：某个 block 可能在产生值的 block 之前被处理。
    /// 若不预先分配槽位，`slot_for_value` 在加载未处理 block 产生的值时会失败。
    /// 这里在生成字节码前遍历所有 block 的所有操作，为每个 `output` 预分配槽位并记录类型。
    ///
    /// 类型推断进行两遍：第一遍处理所有非 Move 操作，第二遍处理 Move 操作
    /// （此时 source 值的类型可能已在第一遍中被记录）。
    fn reserve_operation_outputs(&mut self, blocks: &[&crate::lir::LirBlock]) {
        // 第一遍：处理所有非 Move 操作，记录输出类型。
        for block in blocks {
            for operation in &block.operations {
                if let Some(output) = operation.output {
                    self.slot_for_output(output);
                    if !matches!(operation.kind, LirOperationKind::Move { .. }) {
                        let ty = match &operation.kind {
                            LirOperationKind::FieldGet { object, field } => self.infer_field_get_type(object, field),
                            LirOperationKind::Call { arguments, builtin: Some(MirBuiltinCall::ArrayGet), .. } => {
                                match arguments.first().map(|operand| operand_type(operand, self)) {
                                    Some(JvmTypeDescriptor::Array(item)) => *item,
                                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                                }
                            }
                            LirOperationKind::Call { builtin: Some(MirBuiltinCall::ArraySet), .. } => JvmTypeDescriptor::Void,
                            LirOperationKind::StoreVar { value, ty, .. } => {
                                ty.as_ref().and_then(|hir_ty| jvm_type_descriptor(hir_ty).ok()).unwrap_or_else(|| operand_type(value, self))
                            }
                            _ => infer_operation_output_type(operation, self.signatures),
                        };
                        self.value_types.insert(output, ty);
                    }
                }
            }
        }
        // 第二遍：处理 Move 操作，从已记录的 source 值类型推断输出类型。
        for block in blocks {
            for operation in &block.operations {
                if let Some(output) = operation.output {
                    if let LirOperationKind::Move { source } = &operation.kind {
                        let ty = match source {
                            LirOperand::Value(v) => self.value_types.get(v).cloned().unwrap_or(JvmTypeDescriptor::Int),
                            LirOperand::Constant(constant) => constant_type(constant),
                            LirOperand::Symbol(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                        };
                        self.value_types.insert(output, ty);
                    }
                }
            }
        }
    }

    pub(super) fn allocate_slot(&mut self) -> u16 {
        let slot = self.next_slot;
        // JVM 局部槽位统一按 2 槽保留，避免 `long/double` 与后续单槽值发生重叠。
        self.next_slot += 2;
        slot
    }

    /// 传播变量声明类型到源 SSA 值。
    ///
    /// 遍历所有 `StoreVar` 操作，若变量有声明类型注解（`ty`），则：
    /// 1. 将变量类型设置为声明类型对应的 JVM 类型。
    /// 2. 若源值（`value`）的类型为 `Object`（默认回退类型），则用声明类型覆盖之。
    ///    这修正了 `array()` 等内建调用的返回类型，使其与变量声明类型一致。
    fn propagate_var_decl_types(&mut self, blocks: &[&crate::lir::LirBlock]) {
        for block in blocks {
            for operation in &block.operations {
                if let LirOperationKind::StoreVar { name, value, ty } = &operation.kind {
                    let Some(hir_ty) = ty
                    else {
                        continue;
                    };
                    let Ok(jvm_ty) = jvm_type_descriptor(hir_ty)
                    else {
                        continue;
                    };
                    // 设置变量类型。
                    self.var_types.insert(name.clone(), jvm_ty.clone());
                    // 若源值类型为 Object（默认回退），用声明类型覆盖。
                    if let LirOperand::Value(v) = value {
                        let needs_override = match self.value_types.get(v) {
                            Some(JvmTypeDescriptor::Object(ref name)) if name == "java/lang/Object" => true,
                            None => true,
                            _ => false,
                        };
                        if needs_override {
                            self.value_types.insert(*v, jvm_ty.clone());
                        }
                    }
                }
            }
        }
    }

    /// 预分配所有 `StoreVar` 操作的变量槽位。
    ///
    /// 遍历所有 block 的所有 `StoreVar` 操作，为每个变量名预分配槽位。
    /// 这确保方法调用内建函数（如 `x.length()`）能在主 lowering 循环前找到接收者变量。
    fn preallocate_var_slots(&mut self, blocks: &[&crate::lir::LirBlock]) {
        for block in blocks {
            for operation in &block.operations {
                if let LirOperationKind::StoreVar { name, .. } = &operation.kind {
                    self.slot_for_var(name);
                }
            }
        }
    }

    /// 当返回值直接来自 SSA 常量时，用函数返回类型反向修正其 JVM 类型。
    ///
    /// 这类值往往没有显式 `ty`，若不在 lowering 前补齐，`return 0` 会被当成 `Int`
    /// 发码，最终在 `long` / `double` 返回路径上触发 verifier 错误。
    fn propagate_return_operand_types(&mut self, blocks: &[&crate::lir::LirBlock], function: &LirFunction) {
        let Ok(return_ty) = jvm_type_descriptor(&function.return_type)
        else {
            return;
        };
        for block in blocks {
            let LirTerminator::Return { value: Some(LirOperand::Value(value)) } = &block.terminator
            else {
                continue;
            };
            let needs_override = match self.value_types.get(value) {
                Some(JvmTypeDescriptor::Object(ref name)) if name == "java/lang/Object" => true,
                Some(JvmTypeDescriptor::Int)
                    if matches!(return_ty, JvmTypeDescriptor::Long | JvmTypeDescriptor::Double | JvmTypeDescriptor::Float) =>
                {
                    true
                }
                None => true,
                _ => false,
            };
            if needs_override {
                self.value_types.insert(*value, return_ty.clone());
            }
        }
    }

    fn max_locals(&self) -> u16 {
        self.next_slot.max(1)
    }

    pub(super) fn slot_for_value(&self, value: MirValueRef) -> Result<u16> {
        self.value_slots.get(&value).copied().ok_or_else(|| {
            miette!(
                "函数 `{}` 找不到值 {:?} 对应的局部槽位；已知值槽位: {:?}",
                self.function_symbol,
                value,
                self.value_slots.keys().copied().collect::<Vec<_>>()
            )
        })
    }

    /// 获取 SSA 值的 JVM 类型，未知时默认按 Int 处理。
    pub(super) fn type_for_value(&self, value: MirValueRef) -> JvmTypeDescriptor {
        self.value_types.get(&value).cloned().unwrap_or(JvmTypeDescriptor::Int)
    }

    pub(super) fn infer_field_get_type(&self, object: &LirOperand, field: &str) -> JvmTypeDescriptor {
        if field == "length" {
            return JvmTypeDescriptor::Int;
        }
        let object_ty = operand_type(object, self);
        match object_ty {
            JvmTypeDescriptor::Object(type_name) => self
                .field_types
                .get(&(type_name, field.to_string()))
                .cloned()
                .unwrap_or(JvmTypeDescriptor::Object("java/lang/Object".to_string())),
            JvmTypeDescriptor::Array(_) if field == "length" => JvmTypeDescriptor::Int,
            _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        }
    }

    pub(super) fn slot_for_output(&mut self, value: MirValueRef) -> u16 {
        if let Some(slot) = self.value_slots.get(&value).copied() {
            return slot;
        }
        let slot = self.allocate_slot();
        self.value_slots.insert(value, slot);
        slot
    }

    pub(super) fn slot_for_var(&mut self, name: &str) -> u16 {
        if let Some(slot) = self.var_slots.get(name).copied() {
            return slot;
        }
        let slot = self.allocate_slot();
        self.var_slots.insert(name.to_string(), slot);
        slot
    }

    /// 查找命名变量的槽位，不分配新槽位。
    pub(super) fn try_slot_for_var(&self, name: &str) -> Option<u16> {
        self.var_slots.get(name).copied()
    }

    /// 获取命名变量的 JVM 类型，未知时默认按 Int 处理。
    pub(super) fn type_for_var(&self, name: &str) -> JvmTypeDescriptor {
        self.var_types.get(name).cloned().unwrap_or(JvmTypeDescriptor::Int)
    }

    /// 通过槽位反查值类型，用于 block 参数存储。
    /// 遍历 value_slots 找到对应的 MirValueRef，再查 value_types。
    fn type_for_slot(&self, slot: u16) -> JvmTypeDescriptor {
        for (value, value_slot) in &self.value_slots {
            if *value_slot == slot {
                if let Some(ty) = self.value_types.get(value) {
                    return ty.clone();
                }
            }
        }
        JvmTypeDescriptor::Int
    }

    pub(super) fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("__{}_{}", prefix, self.label_seed);
        self.label_seed += 1;
        label
    }
}

/// 推断操作数的 JVM 类型描述符。
pub(super) fn infer_operand_type(operand: &LirOperand, _signatures: &BTreeMap<String, JvmMethodDescriptor>) -> JvmTypeDescriptor {
    match operand {
        LirOperand::Value(_) => {
            // 值的类型需要从上下文获取，这里返回 Int 作为占位。
            // 实际类型在 reserve_operation_outputs 中已被记录。
            JvmTypeDescriptor::Int
        }
        LirOperand::Constant(constant) => constant_type(constant),
        LirOperand::Symbol(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
    }
}

/// 判断操作数是否为字符串类型。
pub(super) fn operand_is_string(operand: &LirOperand, context: &FunctionLoweringContext<'_>) -> bool {
    match operand {
        LirOperand::Value(v) => matches!(context.type_for_value(*v), JvmTypeDescriptor::Object(ref name) if name == "java/lang/String"),
        LirOperand::Constant(MirConstant::String(_)) => true,
        _ => false,
    }
}

pub(super) fn constant_type(constant: &MirConstant) -> JvmTypeDescriptor {
    match constant {
        MirConstant::Int(value) if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => JvmTypeDescriptor::Int,
        MirConstant::Int(_) => JvmTypeDescriptor::Long,
        MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
        MirConstant::Float64(_) => JvmTypeDescriptor::Double,
        MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
    }
}

pub(super) fn operand_type(operand: &LirOperand, context: &FunctionLoweringContext<'_>) -> JvmTypeDescriptor {
    match operand {
        LirOperand::Value(v) => context.type_for_value(*v),
        LirOperand::Constant(constant) => constant_type(constant),
        LirOperand::Symbol(path) => {
            if path.parts().len() == 1 {
                path.parts()
                    .last()
                    .map(|segment| segment.to_string())
                    .and_then(|name| context.var_types.get(&name).cloned())
                    .unwrap_or(JvmTypeDescriptor::Object("java/lang/Object".to_string()))
            }
            else {
                JvmTypeDescriptor::Object("java/lang/Object".to_string())
            }
        }
    }
}

pub(super) fn string_value_of_descriptor(ty: &JvmTypeDescriptor) -> JvmMethodDescriptor {
    let parameter = match ty {
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        _ => ty.clone(),
    };
    JvmMethodDescriptor::new(vec![parameter], JvmTypeDescriptor::Object("java/lang/String".to_string()))
}

pub(super) fn is_reference_descriptor(ty: &JvmTypeDescriptor) -> bool {
    matches!(ty, JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_))
}

pub(super) fn push_default_descriptor_value(ty: &JvmTypeDescriptor, instructions: &mut Vec<JvmInstruction>) {
    match ty {
        JvmTypeDescriptor::Void => {}
        JvmTypeDescriptor::Long => {
            instructions.push(JvmInstruction::LConst0);
        }
        JvmTypeDescriptor::Float => {
            instructions.push(JvmInstruction::FConst0);
        }
        JvmTypeDescriptor::Double => {
            instructions.push(JvmInstruction::DConst0);
        }
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => {
            instructions.push(JvmInstruction::AConstNull);
        }
        _ => {
            instructions.push(JvmInstruction::IConst(0));
        }
    }
}

pub(super) fn array_load_instruction(ty: &JvmTypeDescriptor) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            JvmInstruction::IALoad
        }
        _ => JvmInstruction::AALoad,
    }
}

pub(super) fn array_store_instruction(ty: &JvmTypeDescriptor) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            JvmInstruction::IAStore
        }
        _ => JvmInstruction::AAStore,
    }
}

pub(super) fn array_new_instruction(ty: &JvmTypeDescriptor) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            JvmInstruction::NewIntArray
        }
        JvmTypeDescriptor::Object(name) => JvmInstruction::ANewArray(name.clone()),
        JvmTypeDescriptor::Array(item) => match item.as_ref() {
            JvmTypeDescriptor::Object(name) => JvmInstruction::ANewArray(format!("[L{name};")),
            nested => JvmInstruction::ANewArray(format!("[{nested}")),
        },
        _ => JvmInstruction::ANewArray("java/lang/Object".to_string()),
    }
}

pub(super) fn numeric_binary_instruction(ty: &JvmTypeDescriptor, op: JvmBinaryNumericOp) -> Result<JvmInstruction> {
    Ok(match ty {
        JvmTypeDescriptor::Long => match op {
            JvmBinaryNumericOp::Add => JvmInstruction::LAdd,
            JvmBinaryNumericOp::Sub => JvmInstruction::LSub,
            JvmBinaryNumericOp::Mul => JvmInstruction::LMul,
            JvmBinaryNumericOp::Div => JvmInstruction::LDiv,
            JvmBinaryNumericOp::Rem => JvmInstruction::LRem,
        },
        JvmTypeDescriptor::Float => match op {
            JvmBinaryNumericOp::Add => JvmInstruction::FAdd,
            JvmBinaryNumericOp::Sub => JvmInstruction::FSub,
            JvmBinaryNumericOp::Mul => JvmInstruction::FMul,
            JvmBinaryNumericOp::Div => JvmInstruction::FDiv,
            JvmBinaryNumericOp::Rem => JvmInstruction::FRem,
        },
        JvmTypeDescriptor::Double => match op {
            JvmBinaryNumericOp::Add => JvmInstruction::DAdd,
            JvmBinaryNumericOp::Sub => JvmInstruction::DSub,
            JvmBinaryNumericOp::Mul => JvmInstruction::DMul,
            JvmBinaryNumericOp::Div => JvmInstruction::DDiv,
            JvmBinaryNumericOp::Rem => JvmInstruction::DRem,
        },
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            match op {
                JvmBinaryNumericOp::Add => JvmInstruction::IAdd,
                JvmBinaryNumericOp::Sub => JvmInstruction::ISub,
                JvmBinaryNumericOp::Mul => JvmInstruction::IMul,
                JvmBinaryNumericOp::Div => JvmInstruction::IDiv,
                JvmBinaryNumericOp::Rem => JvmInstruction::IRem,
            }
        }
        _ => {
            return Err(miette!("JVM 数值二元运算暂不支持类型 {:?}", ty));
        }
    })
}

pub(super) fn numeric_neg_instruction(ty: &JvmTypeDescriptor) -> Result<JvmInstruction> {
    Ok(match ty {
        JvmTypeDescriptor::Long => JvmInstruction::LNeg,
        JvmTypeDescriptor::Float => JvmInstruction::FNeg,
        JvmTypeDescriptor::Double => JvmInstruction::DNeg,
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            JvmInstruction::INeg
        }
        _ => {
            return Err(miette!("JVM 一元负号暂不支持类型 {:?}", ty));
        }
    })
}

pub(super) fn emit_compare_branch(
    ty: &JvmTypeDescriptor,
    compare: JvmIntComparison,
    true_label: &str,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    match ty {
        JvmTypeDescriptor::Long => {
            instructions.push(JvmInstruction::LCmp);
            instructions.push(match compare {
                JvmIntComparison::Eq => JvmInstruction::IfEq(true_label.to_string()),
                JvmIntComparison::Ne => JvmInstruction::IfNe(true_label.to_string()),
                JvmIntComparison::Lt => JvmInstruction::IfLt(true_label.to_string()),
                JvmIntComparison::Le => JvmInstruction::IfLe(true_label.to_string()),
                JvmIntComparison::Gt => JvmInstruction::IfGt(true_label.to_string()),
                JvmIntComparison::Ge => JvmInstruction::IfGe(true_label.to_string()),
            });
        }
        JvmTypeDescriptor::Float => {
            instructions.push(match compare {
                JvmIntComparison::Gt | JvmIntComparison::Ge => JvmInstruction::FCmpL,
                _ => JvmInstruction::FCmpG,
            });
            instructions.push(match compare {
                JvmIntComparison::Eq => JvmInstruction::IfEq(true_label.to_string()),
                JvmIntComparison::Ne => JvmInstruction::IfNe(true_label.to_string()),
                JvmIntComparison::Lt => JvmInstruction::IfLt(true_label.to_string()),
                JvmIntComparison::Le => JvmInstruction::IfLe(true_label.to_string()),
                JvmIntComparison::Gt => JvmInstruction::IfGt(true_label.to_string()),
                JvmIntComparison::Ge => JvmInstruction::IfGe(true_label.to_string()),
            });
        }
        JvmTypeDescriptor::Double => {
            instructions.push(match compare {
                JvmIntComparison::Gt | JvmIntComparison::Ge => JvmInstruction::DCmpL,
                _ => JvmInstruction::DCmpG,
            });
            instructions.push(match compare {
                JvmIntComparison::Eq => JvmInstruction::IfEq(true_label.to_string()),
                JvmIntComparison::Ne => JvmInstruction::IfNe(true_label.to_string()),
                JvmIntComparison::Lt => JvmInstruction::IfLt(true_label.to_string()),
                JvmIntComparison::Le => JvmInstruction::IfLe(true_label.to_string()),
                JvmIntComparison::Gt => JvmInstruction::IfGt(true_label.to_string()),
                JvmIntComparison::Ge => JvmInstruction::IfGe(true_label.to_string()),
            });
        }
        _ => {
            instructions.push(match compare {
                JvmIntComparison::Eq => JvmInstruction::IfICmpEq(true_label.to_string()),
                JvmIntComparison::Ne => JvmInstruction::IfICmpNe(true_label.to_string()),
                JvmIntComparison::Lt => JvmInstruction::IfICmpLt(true_label.to_string()),
                JvmIntComparison::Le => JvmInstruction::IfICmpLe(true_label.to_string()),
                JvmIntComparison::Gt => JvmInstruction::IfICmpGt(true_label.to_string()),
                JvmIntComparison::Ge => JvmInstruction::IfICmpGe(true_label.to_string()),
            });
        }
    }
    Ok(())
}

/// 根据类型选择正确的存储指令。
pub(super) fn store_instruction_for_type(ty: &JvmTypeDescriptor, slot: u16) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Long => JvmInstruction::LStore(slot),
        JvmTypeDescriptor::Float => JvmInstruction::FStore(slot),
        JvmTypeDescriptor::Double => JvmInstruction::DStore(slot),
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::AStore(slot),
        _ => JvmInstruction::IStore(slot),
    }
}

/// 根据类型选择正确的加载指令。
pub(super) fn load_instruction_for_type(ty: &JvmTypeDescriptor, slot: u16) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Long => JvmInstruction::LLoad(slot),
        JvmTypeDescriptor::Float => JvmInstruction::FLoad(slot),
        JvmTypeDescriptor::Double => JvmInstruction::DLoad(slot),
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::ALoad(slot),
        _ => JvmInstruction::ILoad(slot),
    }
}

fn lower_terminator(
    terminator: &LirTerminator,
    return_type: &ValkyrieType,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    match terminator {
        LirTerminator::Return { value } => {
            if let Some(value) = value {
                let return_ty = jvm_type_descriptor(return_type).ok();
                lower_operand_with_hint(value, return_ty.as_ref(), context, instructions)?;
            }
            else {
                // 无返回值的 `Return` terminator：按返回类型压入默认值，避免 JVM 栈下溢。
                push_default_value(return_type, instructions);
            }
            instructions.push(return_instruction_for_type(return_type)?);
        }
        LirTerminator::Jump { target, arguments } => {
            lower_jump_arguments(*target, arguments, context, instructions)?;
            instructions.push(JvmInstruction::Goto(block_label(*target, context)?));
        }
        LirTerminator::Branch { condition, then_target, else_target } => {
            lower_operand(condition, context, instructions)?;
            instructions.push(JvmInstruction::IfNe(block_label(*then_target, context)?));
            instructions.push(JvmInstruction::Goto(block_label(*else_target, context)?));
        }
        LirTerminator::PerformEffect { effect, .. } => {
            return Err(miette!("JVM backend 暂未支持 `{:?}` effect terminator lowering", effect));
        }
        LirTerminator::Unreachable => {
            // `Unreachable` terminator：按返回类型压入默认值后返回，避免 JVM 栈下溢。
            push_default_value(return_type, instructions);
            instructions.push(default_return_instruction(return_type)?);
        }
    }
    Ok(())
}

fn lower_jump_arguments(
    target: MirBlockRef,
    arguments: &[LirOperand],
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    let target_slots = target_parameter_slots(target, context)?;
    if arguments.len() > target_slots.len() {
        return Err(miette!(
            "块参数数量与跳转参数数量不匹配：目标块 `{:?}` 最多接受 {} 个参数，实际收到 {} 个参数",
            target,
            target_slots.len(),
            arguments.len()
        ));
    }
    for (argument, slot) in arguments.iter().zip(target_slots.iter()) {
        let ty = context.type_for_slot(*slot);
        lower_operand_with_hint(argument, Some(&ty), context, instructions)?;
    }
    // 按逆序存储已提供的参数（栈是 LIFO）；未显式传入的目标块参数继续复用既有槽位值。
    for slot in target_slots.iter().take(arguments.len()).rev() {
        let ty = context.type_for_slot(*slot);
        instructions.push(store_instruction_for_type(&ty, *slot));
    }
    Ok(())
}

fn target_parameter_slots(target: MirBlockRef, context: &FunctionLoweringContext<'_>) -> Result<Vec<u16>> {
    context
        .block_parameter_slots
        .iter()
        .find(|(block, _)| **block == target)
        .map(|(_, slots)| slots.clone())
        .ok_or_else(|| miette!("JVM lowering 内部错误：缺少目标块参数槽位信息"))
}

fn block_label(target: MirBlockRef, context: &FunctionLoweringContext<'_>) -> Result<String> {
    context.block_labels.get(&target).cloned().ok_or_else(|| miette!("找不到块 {:?} 的标签", target))
}

fn return_instruction_for_type(return_type: &ValkyrieType) -> Result<JvmInstruction> {
    default_return_instruction(return_type)
}

/// 按返回类型压入默认值，用于无返回值的 `Return` / `Unreachable` terminator。
fn push_default_value(return_type: &ValkyrieType, instructions: &mut Vec<JvmInstruction>) {
    if let Ok(descriptor) = jvm_type_descriptor(return_type) {
        push_default_descriptor_value(&descriptor, instructions);
        return;
    }

    instructions.push(JvmInstruction::AConstNull);
}

fn default_return_instruction(return_type: &ValkyrieType) -> Result<JvmInstruction> {
    if let Ok(descriptor) = jvm_type_descriptor(return_type) {
        return Ok(match descriptor {
            JvmTypeDescriptor::Void => JvmInstruction::Return,
            JvmTypeDescriptor::Long => JvmInstruction::LReturn,
            JvmTypeDescriptor::Double => JvmInstruction::DReturn,
            JvmTypeDescriptor::Float => JvmInstruction::FReturn,
            JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::AReturn,
            _ => JvmInstruction::IReturn,
        });
    }

    Ok(match return_type {
        ValkyrieType::Tuple(_)
        | ValkyrieType::Function(_)
        | ValkyrieType::TraitObject(TraitObject { .. })
        | ValkyrieType::Apply(_, _)
        | ValkyrieType::r#SelfType => JvmInstruction::AReturn,
        _ => return Err(miette!("JVM 最小 lowering 暂不支持类型 {:?} 的默认返回指令", return_type)),
    })
}

pub(super) fn jvm_type_descriptor(ty: &ValkyrieType) -> Result<JvmTypeDescriptor> {
    Ok(match ty {
        ValkyrieType::Integer8 { .. }
        | ValkyrieType::Integer16 { .. }
        | ValkyrieType::Integer32 { .. }
        | ValkyrieType::Boolean
        | ValkyrieType::Character => JvmTypeDescriptor::Int,
        ValkyrieType::Integer64 { .. } => JvmTypeDescriptor::Long,
        ValkyrieType::Float32 => JvmTypeDescriptor::Float,
        ValkyrieType::Float64 => JvmTypeDescriptor::Double,
        ValkyrieType::Utf8 | ValkyrieType::Utf16 => JvmTypeDescriptor::Object("java/lang/String".to_string()),
        ValkyrieType::Unit | ValkyrieType::Void => JvmTypeDescriptor::Void,
        ValkyrieType::Array(item) => JvmTypeDescriptor::Array(Box::new(jvm_type_descriptor(item)?)),
        // 命名类型：识别常见的整型/浮点型别名，其余按 Object 处理。
        ValkyrieType::Named(name) => match name.as_str() {
            "int" | "i32" | "uint" | "u32" | "usize" | "isize" | "Int32" | "UInt32" | "Size" | "Offset" => JvmTypeDescriptor::Int,
            "long" | "i64" | "u64" | "Int64" | "UInt64" => JvmTypeDescriptor::Long,
            "float" | "f32" | "Float32" => JvmTypeDescriptor::Float,
            "double" | "f64" | "Float64" => JvmTypeDescriptor::Double,
            "bool" | "Boolean" | "char" | "Char" | "Character" => JvmTypeDescriptor::Int,
            "string" | "String" | "str" | "utf8" | "Utf8" => JvmTypeDescriptor::Object("java/lang/String".to_string()),
            _ => JvmTypeDescriptor::Object(name.as_str().to_string()),
        },
        ValkyrieType::r#SelfType => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        // 泛型应用类型按类型擦除映射为 Object。
        ValkyrieType::Apply(_, _) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        // 推断类型暂按 Int 处理，待类型推断完善后修正。
        ValkyrieType::AutoType => JvmTypeDescriptor::Int,
        // 函数类型按类型擦除映射为 Object，JVM 运行时通过方法句柄调用。
        ValkyrieType::Function(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        // 元组类型按类型擦除映射为 Object。
        ValkyrieType::Tuple(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        _ => return Err(miette!("JVM 最小 lowering 暂不支持类型 {:?} 的描述符映射", ty)),
    })
}

fn module_internal_name(module_name: &str) -> String {
    let sanitized = module_name
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '_' => ch,
            '.' | ':' | '/' | '\\' | '-' => '/',
            _ => '_',
        })
        .collect::<String>()
        .trim_matches('/')
        .to_string();
    if sanitized.is_empty() {
        "nyar/Main".to_string()
    }
    else if sanitized.contains('/') {
        sanitized
    }
    else {
        format!("nyar/{sanitized}")
    }
}

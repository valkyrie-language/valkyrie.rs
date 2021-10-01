//! `JVM` 路线 lowering。
//!
//! 这里把 lane-aware `LIR` 收口成最小可编码的 `JVM class` 模型。
//! 当前刻意只实现一条很窄的路径：
//! - 顶层函数 -> 单个静态方法
//! - 参数与局部值先按单槽位整型处理
//! - 控制流仍由 `LIR` 基本块表达，`JVM` 细节只在本 crate 内部消化

use std::collections::{BTreeMap, BTreeSet, VecDeque};

use crate::{
    lir::{LirDispatchKind, LirFunction, LirModule, LirOperand, LirOperation, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirBuiltinBinaryOp, MirBuiltinCall, MirBuiltinCompareOp, MirConstant, MirValueRef},
};
use jvm_backend::{
    class::JvmFieldRef, JvmClassFile, JvmCodeBody, JvmInstruction, JvmMethodDescriptor, JvmMethodRef, JvmMethodSignature, JvmTypeDescriptor,
};
use miette::{miette, Result};
use nyar::{
    abstractions::{BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use valkyrie_types::{
    hir::{TraitObject, ValkyrieType},
    NamePath,
};

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
    let block_labels: BTreeMap<MirBlockRef, String> = function.blocks.iter().map(|block| (block.id, format!("BB{}", block.id.0))).collect();
    // 按参数的 JVM 类型槽位计算入口局部变量起始偏移。
    // `long` / `double` 占 2 槽，若仅按参数数量计算会导致 `max_locals` 不足。
    let entry_parameters = function.param_types.iter().map(|ty| jvm_type_descriptor(ty).map(|d| d.slot_count()).unwrap_or(1)).sum::<u16>();
    let mut context = FunctionLoweringContext::new(owner, signatures, field_types, &block_labels, entry_parameters);
    context.reserve_block_parameters(function);
    context.reserve_operation_outputs(function);
    // 先传播变量声明类型，再推断 block 参数，避免局部数组值被提前回退成 `Int`。
    context.propagate_var_decl_types(function);
    // 在操作输出类型已知后，通过跳转参数推断非入口 block 的参数类型。
    context.infer_block_parameter_types(function);
    // 预分配所有 StoreVar 操作的变量槽位，确保方法调用内建函数能找到接收者变量。
    context.preallocate_var_slots(function);

    let referenced_targets = collect_referenced_targets(function);
    let mut instructions = Vec::new();

    for block in &function.blocks {
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
        name: function.symbol.clone(),
        descriptor,
        access_flags: JVM_ACC_PUBLIC | JVM_ACC_STATIC,
        code: Some(JvmCodeBody { max_stack: DEFAULT_MAX_STACK, max_locals: context.max_locals(), instructions }),
    })
}

fn collect_reachable_function_symbols(lir: &LirModule) -> BTreeSet<String> {
    let function_symbols: BTreeSet<String> = lir.functions.iter().map(|function| function.symbol.clone()).collect();
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();

    if function_symbols.contains("main") {
        queue.push_back("main".to_string());
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
                let Some(next_symbol) = path.parts().last().map(|segment| segment.to_string())
                else {
                    continue;
                };
                if function_symbols.contains(&next_symbol) && !is_jvm_host_bridge_symbol(&next_symbol) {
                    queue.push_back(next_symbol);
                }
            }
        }
    }

    reachable
}

fn collect_function_signatures(lir: &LirModule, reachable_symbols: &BTreeSet<String>) -> Result<BTreeMap<String, JvmMethodDescriptor>> {
    lir.functions
        .iter()
        .filter(|function| reachable_symbols.contains(&function.symbol))
        .map(|function| Ok((function.symbol.clone(), build_method_descriptor(function)?)))
        .collect()
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

fn collect_referenced_targets(function: &LirFunction) -> BTreeSet<MirBlockRef> {
    let mut referenced_targets = BTreeSet::new();
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
            LirTerminator::Return { .. } | LirTerminator::Unreachable => {}
        }
    }
    referenced_targets
}

struct FunctionLoweringContext<'a> {
    owner: &'a str,
    signatures: &'a BTreeMap<String, JvmMethodDescriptor>,
    field_types: &'a BTreeMap<(String, String), JvmTypeDescriptor>,
    block_labels: &'a BTreeMap<MirBlockRef, String>,
    block_parameter_slots: BTreeMap<MirBlockRef, Vec<u16>>,
    value_slots: BTreeMap<MirValueRef, u16>,
    /// 每个 SSA 值对应的 JVM 类型描述符，用于选择正确的 load/store 指令。
    value_types: BTreeMap<MirValueRef, JvmTypeDescriptor>,
    var_slots: BTreeMap<String, u16>,
    /// 命名变量对应的 JVM 类型描述符，用于选择正确的 load/store 指令。
    var_types: BTreeMap<String, JvmTypeDescriptor>,
    next_slot: u16,
    label_seed: usize,
}

impl<'a> FunctionLoweringContext<'a> {
    fn new(
        owner: &'a str,
        signatures: &'a BTreeMap<String, JvmMethodDescriptor>,
        field_types: &'a BTreeMap<(String, String), JvmTypeDescriptor>,
        block_labels: &'a BTreeMap<MirBlockRef, String>,
        entry_parameters: u16,
    ) -> Self {
        Self {
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

    fn reserve_block_parameters(&mut self, function: &LirFunction) {
        for block in &function.blocks {
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
    fn infer_block_parameter_types(&mut self, function: &LirFunction) {
        for block in &function.blocks {
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
    fn reserve_operation_outputs(&mut self, function: &LirFunction) {
        // 第一遍：处理所有非 Move 操作，记录输出类型。
        for block in &function.blocks {
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
        for block in &function.blocks {
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

    fn allocate_slot(&mut self) -> u16 {
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
    fn propagate_var_decl_types(&mut self, function: &LirFunction) {
        for block in &function.blocks {
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
    fn preallocate_var_slots(&mut self, function: &LirFunction) {
        for block in &function.blocks {
            for operation in &block.operations {
                if let LirOperationKind::StoreVar { name, .. } = &operation.kind {
                    self.slot_for_var(name);
                }
            }
        }
    }

    fn max_locals(&self) -> u16 {
        self.next_slot.max(1)
    }

    fn slot_for_value(&self, value: MirValueRef) -> Result<u16> {
        self.value_slots.get(&value).copied().ok_or_else(|| miette!("找不到值 {:?} 对应的局部槽位", value))
    }

    /// 获取 SSA 值的 JVM 类型，未知时默认按 Int 处理。
    fn type_for_value(&self, value: MirValueRef) -> JvmTypeDescriptor {
        self.value_types.get(&value).cloned().unwrap_or(JvmTypeDescriptor::Int)
    }

    fn infer_field_get_type(&self, object: &LirOperand, field: &str) -> JvmTypeDescriptor {
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

    fn infer_subscript_type(&self, object: &LirOperand) -> JvmTypeDescriptor {
        match operand_type(object, self) {
            JvmTypeDescriptor::Array(item) => *item,
            _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        }
    }

    fn slot_for_output(&mut self, value: MirValueRef) -> u16 {
        if let Some(slot) = self.value_slots.get(&value).copied() {
            return slot;
        }
        let slot = self.allocate_slot();
        self.value_slots.insert(value, slot);
        slot
    }

    fn slot_for_var(&mut self, name: &str) -> u16 {
        if let Some(slot) = self.var_slots.get(name).copied() {
            return slot;
        }
        let slot = self.allocate_slot();
        self.var_slots.insert(name.to_string(), slot);
        slot
    }

    /// 查找命名变量的槽位，不分配新槽位。
    fn try_slot_for_var(&self, name: &str) -> Option<u16> {
        self.var_slots.get(name).copied()
    }

    /// 获取命名变量的 JVM 类型，未知时默认按 Int 处理。
    fn type_for_var(&self, name: &str) -> JvmTypeDescriptor {
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

    fn fresh_label(&mut self, prefix: &str) -> String {
        let label = format!("__{}_{}", prefix, self.label_seed);
        self.label_seed += 1;
        label
    }
}

fn lower_operation(operation: &LirOperation, context: &mut FunctionLoweringContext<'_>, instructions: &mut Vec<JvmInstruction>) -> Result<()> {
    match &operation.kind {
        LirOperationKind::LoadConstant { constant, .. } => {
            lower_constant(constant, instructions)?;
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::LoadSymbol { path } => {
            if let Some(var_name) = local_symbol_name(path) {
                if let Some(slot) = context.try_slot_for_var(&var_name) {
                    let ty = context.type_for_var(&var_name);
                    if let Some(output) = operation.output {
                        context.value_types.insert(output, ty.clone());
                    }
                    instructions.push(load_instruction_for_type(&ty, slot));
                    store_output(operation.output, context, instructions);
                    return Ok(());
                }
            }
            // 模块路径等非局部符号暂用 null 占位，待后续支持完整的外部符号解析。
            instructions.push(JvmInstruction::AConstNull);
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::Move { source } => {
            lower_operand(source, context, instructions)?;
            // 根据 source 类型动态选择存储指令，避免预推断类型不完整导致的类型不匹配。
            if let Some(output) = operation.output {
                let slot = context.slot_for_output(output);
                let ty = operand_type(source, context);
                instructions.push(store_instruction_for_type(&ty, slot));
                context.value_types.insert(output, ty);
            }
        }
        LirOperationKind::StoreVar { name, value, ty: _ } => {
            lower_operand(value, context, instructions)?;
            let slot = context.slot_for_var(name);
            // 优先使用变量已记录的类型（来自 propagate_var_decl_types），否则从 value 推断。
            let ty = context.var_types.get(name).cloned().unwrap_or_else(|| operand_type(value, context));
            context.var_types.insert(name.clone(), ty.clone());
            instructions.push(store_instruction_for_type(&ty, slot));
            if let Some(output) = operation.output {
                context.value_slots.insert(output, slot);
                context.value_types.insert(output, ty);
            }
        }
        LirOperationKind::Call { dispatch, callee, arguments, builtin, witness, effect } => {
            if witness.is_some() || effect.is_some() {
                return Err(miette!("JVM 最小 lowering 暂不支持 witness / effect 调用"));
            }
            if *dispatch != LirDispatchKind::Static {
                return Err(miette!("JVM 最小 lowering 暂只支持静态调用"));
            }
            if matches!(builtin, Some(MirBuiltinCall::ArrayGet)) {
                let array_ty = operand_type(&arguments[0], context);
                let element_ty = match array_ty {
                    JvmTypeDescriptor::Array(item) => *item,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                };
                lower_operand(&arguments[0], context, instructions)?;
                lower_operand(&arguments[1], context, instructions)?;
                instructions.push(array_load_instruction(&element_ty));
                store_output(operation.output, context, instructions);
            }
            else if matches!(builtin, Some(MirBuiltinCall::ArraySet)) {
                let array_ty = operand_type(&arguments[0], context);
                let element_ty = match array_ty {
                    JvmTypeDescriptor::Array(item) => *item,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                };
                lower_operand(&arguments[0], context, instructions)?;
                lower_operand(&arguments[1], context, instructions)?;
                lower_operand(&arguments[2], context, instructions)?;
                instructions.push(array_store_instruction(&element_ty));
            }
            else if let Some(intrinsic) = builtin.and_then(jvm_intrinsic_from_builtin) {
                lower_intrinsic_call(intrinsic, arguments, operation.output, context, instructions)?;
            }
            else if let Some((intrinsic, receiver_path)) = try_method_intrinsic(callee) {
                // 方法调用风格的内建函数（如 `x.length()`）：
                // 接收者嵌入在路径中，参数列表为空。
                // 将接收者从路径中提取，作为第一个参数传入内建 lowering。
                let receiver_operand = load_receiver_operand(&receiver_path, context, instructions)?;
                let mut combined_args = vec![receiver_operand];
                combined_args.extend_from_slice(arguments);
                lower_intrinsic_call(intrinsic, &combined_args, operation.output, context, instructions)?;
            }
            else {
                lower_static_call(callee, arguments, operation.output, context, instructions)?;
            }
        }
        LirOperationKind::ArrayNew { element_type, length } => {
            lower_operand(length, context, instructions)?;
            let element_descriptor = jvm_type_descriptor(element_type)?;
            instructions.push(array_new_instruction(&element_descriptor));
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::ArrayLiteral { element_type, items } => {
            let element_descriptor = jvm_type_descriptor(element_type)?;
            let array_descriptor = JvmTypeDescriptor::Array(Box::new(element_descriptor.clone()));
            let slot = if let Some(output) = operation.output {
                context.value_types.insert(output, array_descriptor.clone());
                context.slot_for_output(output)
            }
            else {
                context.allocate_slot()
            };

            instructions.push(JvmInstruction::IConst(items.len() as i32));
            instructions.push(array_new_instruction(&element_descriptor));
            instructions.push(store_instruction_for_type(&array_descriptor, slot));

            for (index, item) in items.iter().enumerate() {
                instructions.push(load_instruction_for_type(&array_descriptor, slot));
                instructions.push(JvmInstruction::IConst(index as i32));
                lower_operand(item, context, instructions)?;
                instructions.push(array_store_instruction(&element_descriptor));
            }
        }
        LirOperationKind::StructNew { type_name: _, fields: _ } => {
            instructions.push(JvmInstruction::AConstNull);
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::FieldGet { object, field } => {
            let field_ty = context.infer_field_get_type(object, field);
            if let Some(output) = operation.output {
                context.value_types.insert(output, field_ty.clone());
            }
            if field == "length" {
                lower_operand(object, context, instructions)?;
                instructions.push(JvmInstruction::ArrayLength);
            }
            else {
                push_default_descriptor_value(&field_ty, instructions);
            }
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::FieldSet { object: _, field: _, value: _ } => {}
        LirOperationKind::PatternMatch { .. } => {
            instructions.push(JvmInstruction::IConst(0));
            if let Some(output) = operation.output {
                context.value_types.insert(output, JvmTypeDescriptor::Boolean);
            }
            store_output(operation.output, context, instructions);
        }
    }
    Ok(())
}

/// 根据操作类型推断其输出的 JVM 类型描述符。
fn infer_operation_output_type(operation: &LirOperation, signatures: &BTreeMap<String, JvmMethodDescriptor>) -> JvmTypeDescriptor {
    match &operation.kind {
        LirOperationKind::LoadConstant { constant, ty } => {
            ty.as_ref().and_then(|hir_ty| jvm_type_descriptor(hir_ty).ok()).unwrap_or_else(|| constant_type(constant))
        }
        LirOperationKind::LoadSymbol { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        LirOperationKind::Move { source } => infer_operand_type(source, signatures),
        LirOperationKind::StoreVar { name, ty, .. } => {
            // 若有声明类型注解，使用之；否则返回 Int 作为占位。
            if let Some(hir_ty) = ty {
                if let Ok(jvm_ty) = jvm_type_descriptor(hir_ty) {
                    return jvm_ty;
                }
            }
            let _ = name;
            JvmTypeDescriptor::Int
        }
        LirOperationKind::Call { callee, builtin, .. } => {
            if let Some(intrinsic) = builtin.and_then(jvm_intrinsic_from_builtin) {
                return intrinsic_output_type(intrinsic);
            }
            // 内建调用（如 infix +, infix == 等）返回 int/bool。
            match try_intrinsic_call(callee) {
                Some(JvmIntrinsicCallLowering::ArrayLiteral) => {
                    // array() 返回 Object[]，具体元素类型由 propagate_var_decl_types 修正。
                    JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string())))
                }
                Some(JvmIntrinsicCallLowering::StringSplit) => {
                    // split() 返回 String[]。
                    JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string())))
                }
                Some(
                    JvmIntrinsicCallLowering::StringTrim
                    | JvmIntrinsicCallLowering::StringToLower
                    | JvmIntrinsicCallLowering::StringToUpper
                    | JvmIntrinsicCallLowering::StringSlice
                    | JvmIntrinsicCallLowering::StringReplace,
                ) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                Some(_) => JvmTypeDescriptor::Int,
                None => {
                    if let LirOperand::Symbol(path) = callee {
                        if let Some(segment) = path.parts().last() {
                            let key: &str = segment.as_str();
                            if let Some(descriptor) = signatures.get(key) {
                                return descriptor.return_type.clone();
                            }
                        }
                    }
                    JvmTypeDescriptor::Object("java/lang/Object".to_string())
                }
            }
        }
        LirOperationKind::StructNew { type_name, .. } => JvmTypeDescriptor::Object(type_name.clone()),
        LirOperationKind::FieldGet { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        LirOperationKind::FieldSet { .. } => JvmTypeDescriptor::Void,
        LirOperationKind::PatternMatch { .. } => JvmTypeDescriptor::Boolean,
        LirOperationKind::ArrayNew { element_type, .. } => {
            // 数组创建：返回与元素类型匹配的数组类型。
            match jvm_type_descriptor(element_type) {
                Ok(jvm_ty) => JvmTypeDescriptor::Array(Box::new(jvm_ty)),
                Err(_) => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
            }
        }
        LirOperationKind::ArrayLiteral { element_type, .. } => match jvm_type_descriptor(element_type) {
            Ok(jvm_ty) => JvmTypeDescriptor::Array(Box::new(jvm_ty)),
            Err(_) => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
        },
    }
}

/// 推断操作数的 JVM 类型描述符。
fn infer_operand_type(operand: &LirOperand, _signatures: &BTreeMap<String, JvmMethodDescriptor>) -> JvmTypeDescriptor {
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
fn operand_is_string(operand: &LirOperand, context: &FunctionLoweringContext<'_>) -> bool {
    match operand {
        LirOperand::Value(v) => matches!(context.type_for_value(*v), JvmTypeDescriptor::Object(ref name) if name == "java/lang/String"),
        LirOperand::Constant(MirConstant::String(_)) => true,
        _ => false,
    }
}

fn constant_type(constant: &MirConstant) -> JvmTypeDescriptor {
    match constant {
        MirConstant::Int(value) if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => JvmTypeDescriptor::Int,
        MirConstant::Int(_) => JvmTypeDescriptor::Long,
        MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
        MirConstant::Float64(_) => JvmTypeDescriptor::Double,
        MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
    }
}

fn operand_type(operand: &LirOperand, context: &FunctionLoweringContext<'_>) -> JvmTypeDescriptor {
    match operand {
        LirOperand::Value(v) => context.type_for_value(*v),
        LirOperand::Constant(constant) => constant_type(constant),
        LirOperand::Symbol(path) => local_symbol_name(path)
            .and_then(|name| context.var_types.get(&name).cloned())
            .unwrap_or(JvmTypeDescriptor::Object("java/lang/Object".to_string())),
    }
}

fn string_value_of_descriptor(ty: &JvmTypeDescriptor) -> JvmMethodDescriptor {
    let parameter = match ty {
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        _ => ty.clone(),
    };
    JvmMethodDescriptor::new(vec![parameter], JvmTypeDescriptor::Object("java/lang/String".to_string()))
}

fn is_reference_descriptor(ty: &JvmTypeDescriptor) -> bool {
    matches!(ty, JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_))
}

fn push_default_descriptor_value(ty: &JvmTypeDescriptor, instructions: &mut Vec<JvmInstruction>) {
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

fn array_load_instruction(ty: &JvmTypeDescriptor) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            JvmInstruction::IALoad
        }
        _ => JvmInstruction::AALoad,
    }
}

fn array_store_instruction(ty: &JvmTypeDescriptor) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Int | JvmTypeDescriptor::Byte | JvmTypeDescriptor::Short | JvmTypeDescriptor::Char | JvmTypeDescriptor::Boolean => {
            JvmInstruction::IAStore
        }
        _ => JvmInstruction::AAStore,
    }
}

fn array_new_instruction(ty: &JvmTypeDescriptor) -> JvmInstruction {
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

fn numeric_binary_instruction(ty: &JvmTypeDescriptor, op: JvmBinaryNumericOp) -> Result<JvmInstruction> {
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

fn numeric_neg_instruction(ty: &JvmTypeDescriptor) -> Result<JvmInstruction> {
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

fn emit_compare_branch(
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
fn store_instruction_for_type(ty: &JvmTypeDescriptor, slot: u16) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Long => JvmInstruction::LStore(slot),
        JvmTypeDescriptor::Float => JvmInstruction::FStore(slot),
        JvmTypeDescriptor::Double => JvmInstruction::DStore(slot),
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::AStore(slot),
        _ => JvmInstruction::IStore(slot),
    }
}

/// 根据类型选择正确的加载指令。
fn load_instruction_for_type(ty: &JvmTypeDescriptor, slot: u16) -> JvmInstruction {
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
                lower_operand(value, context, instructions)?;
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
    for argument in arguments {
        lower_operand(argument, context, instructions)?;
    }
    let target_slots = target_parameter_slots(target, context)?;
    if arguments.len() > target_slots.len() {
        return Err(miette!(
            "块参数数量与跳转参数数量不匹配：目标块 `{:?}` 最多接受 {} 个参数，实际收到 {} 个参数",
            target,
            target_slots.len(),
            arguments.len()
        ));
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

fn store_output(output: Option<MirValueRef>, context: &mut FunctionLoweringContext<'_>, instructions: &mut Vec<JvmInstruction>) {
    if let Some(output) = output {
        let slot = context.slot_for_output(output);
        let ty = context.type_for_value(output);
        instructions.push(store_instruction_for_type(&ty, slot));
    }
}

fn lower_operand(operand: &LirOperand, context: &mut FunctionLoweringContext<'_>, instructions: &mut Vec<JvmInstruction>) -> Result<()> {
    match operand {
        LirOperand::Value(value) => {
            let slot = context.slot_for_value(*value)?;
            let ty = context.type_for_value(*value);
            instructions.push(load_instruction_for_type(&ty, slot));
        }
        LirOperand::Constant(constant) => lower_constant(constant, instructions)?,
        LirOperand::Symbol(path) => {
            if let Some(var_name) = local_symbol_name(path) {
                if let Some(slot) = context.try_slot_for_var(&var_name) {
                    let ty = context.type_for_var(&var_name);
                    instructions.push(load_instruction_for_type(&ty, slot));
                    return Ok(());
                }
            }
            // 符号作为操作数时暂用 null 占位，待后续支持完整的外部符号解析。
            instructions.push(JvmInstruction::AConstNull);
        }
    }
    Ok(())
}

fn local_symbol_name(path: &NamePath) -> Option<String> {
    if path.parts().len() == 1 {
        path.parts().last().map(|segment| segment.to_string())
    }
    else {
        None
    }
}

fn lower_constant(constant: &MirConstant, instructions: &mut Vec<JvmInstruction>) -> Result<()> {
    match constant {
        MirConstant::Int(value) => {
            let value = i32::try_from(*value).map_err(|_| miette!("JVM 最小 lowering 暂只支持 `i32` 常量"))?;
            instructions.push(JvmInstruction::IConst(value));
        }
        MirConstant::Float64(value) => match value.0 {
            0.0 => {
                instructions.push(JvmInstruction::DConst0);
            }
            1.0 => {
                instructions.push(JvmInstruction::DConst1);
            }
            value => {
                instructions.push(JvmInstruction::LdcDouble(value.to_bits()));
            }
        },
        MirConstant::Bool(value) => {
            instructions.push(JvmInstruction::IConst(i32::from(*value)));
        }
        MirConstant::Unit => {
            instructions.push(JvmInstruction::IConst(0));
        }
        MirConstant::String(value) => {
            instructions.push(JvmInstruction::LdcString(value.clone()));
        }
    }
    Ok(())
}

#[derive(Clone, Copy)]
enum JvmIntrinsicCallLowering {
    BinaryNumeric(JvmBinaryNumericOp),
    Compare(JvmIntComparison),
    NumericNeg,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    /// 数组/字符串长度，对应 `arraylength` 指令，返回 int。
    ArrayLength,
    /// 数组字面量构造，对应 `newarray` / `anewarray` 指令序列。
    ArrayLiteral,
    /// 字符串去除首尾空白，对应 `invokevirtual java/lang/String.trim()`。
    StringTrim,
    /// 字符串转小写，对应 `invokevirtual java/lang/String.toLowerCase()`。
    StringToLower,
    /// 字符串转大写，对应 `invokevirtual java/lang/String.toUpperCase()`。
    StringToUpper,
    /// 字符串前缀匹配，对应 `invokevirtual java/lang/String.startsWith(Ljava/lang/String;)Z`。
    StringStartsWith,
    /// 字符串后缀匹配，对应 `invokevirtual java/lang/String.endsWith(Ljava/lang/String;)Z`。
    StringEndsWith,
    /// 字符串包含判断，对应 `invokevirtual java/lang/String.contains(Ljava/lang/CharSequence;)Z`。
    StringContains,
    /// 字符串切片，对应 `invokevirtual java/lang/String.substring(II)Ljava/lang/String;`。
    StringSlice,
    /// 字符串替换，对应 `invokevirtual java/lang/String.replace(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)Ljava/lang/String;`。
    StringReplace,
    /// 字符串分割，对应 `invokevirtual java/lang/String.split(Ljava/lang/String;)[Ljava/lang/String;`。
    StringSplit,
}

#[derive(Clone, Copy)]
enum JvmBinaryNumericOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Clone, Copy)]
enum JvmIntComparison {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

fn try_intrinsic_call(callee: &LirOperand) -> Option<JvmIntrinsicCallLowering> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    if path.parts().len() != 1 {
        return None;
    }
    Some(match path.parts()[0].as_str() {
        "array" => JvmIntrinsicCallLowering::ArrayLiteral,
        "trim" => JvmIntrinsicCallLowering::StringTrim,
        "to_lower" | "to_lowercase" | "lowercase" => JvmIntrinsicCallLowering::StringToLower,
        "to_upper" | "to_uppercase" | "uppercase" => JvmIntrinsicCallLowering::StringToUpper,
        "starts_with" | "startsWith" => JvmIntrinsicCallLowering::StringStartsWith,
        "ends_with" | "endsWith" => JvmIntrinsicCallLowering::StringEndsWith,
        "contains" => JvmIntrinsicCallLowering::StringContains,
        "slice" | "substring" => JvmIntrinsicCallLowering::StringSlice,
        "replace" => JvmIntrinsicCallLowering::StringReplace,
        "split" => JvmIntrinsicCallLowering::StringSplit,
        _ => return None,
    })
}

fn jvm_intrinsic_from_builtin(builtin: MirBuiltinCall) -> Option<JvmIntrinsicCallLowering> {
    Some(match builtin {
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Add) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Add),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Sub) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Sub),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Mul) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Mul),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Div) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Div),
        MirBuiltinCall::BinaryNumeric(MirBuiltinBinaryOp::Rem) => JvmIntrinsicCallLowering::BinaryNumeric(JvmBinaryNumericOp::Rem),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Eq),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ne) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Ne),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Lt) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Lt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Le) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Le),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Gt) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Gt),
        MirBuiltinCall::Compare(MirBuiltinCompareOp::Ge) => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Ge),
        MirBuiltinCall::NumericNeg => JvmIntrinsicCallLowering::NumericNeg,
        MirBuiltinCall::LogicalAnd => JvmIntrinsicCallLowering::LogicalAnd,
        MirBuiltinCall::LogicalOr => JvmIntrinsicCallLowering::LogicalOr,
        MirBuiltinCall::LogicalNot => JvmIntrinsicCallLowering::LogicalNot,
        MirBuiltinCall::ArrayLength => JvmIntrinsicCallLowering::ArrayLength,
        MirBuiltinCall::ArrayGet | MirBuiltinCall::ArraySet | MirBuiltinCall::Identity => return None,
    })
}

fn intrinsic_output_type(intrinsic: JvmIntrinsicCallLowering) -> JvmTypeDescriptor {
    match intrinsic {
        JvmIntrinsicCallLowering::BinaryNumeric(_) | JvmIntrinsicCallLowering::NumericNeg => JvmTypeDescriptor::Int,
        JvmIntrinsicCallLowering::Compare(_)
        | JvmIntrinsicCallLowering::LogicalAnd
        | JvmIntrinsicCallLowering::LogicalOr
        | JvmIntrinsicCallLowering::LogicalNot => JvmTypeDescriptor::Boolean,
        JvmIntrinsicCallLowering::ArrayLength => JvmTypeDescriptor::Int,
        JvmIntrinsicCallLowering::ArrayLiteral => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
        JvmIntrinsicCallLowering::StringTrim
        | JvmIntrinsicCallLowering::StringToLower
        | JvmIntrinsicCallLowering::StringToUpper
        | JvmIntrinsicCallLowering::StringSlice
        | JvmIntrinsicCallLowering::StringReplace => JvmTypeDescriptor::Object("java/lang/String".to_string()),
        JvmIntrinsicCallLowering::StringStartsWith | JvmIntrinsicCallLowering::StringEndsWith | JvmIntrinsicCallLowering::StringContains => {
            JvmTypeDescriptor::Boolean
        }
        JvmIntrinsicCallLowering::StringSplit => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
    }
}

/// 尝试将方法调用风格的内建函数（如 `x.length()`）解析为内建调用。
///
/// 当 callee 是恰好两段路径且最后一段是内建函数名时，返回内建类型和接收者路径。
/// 例如 `NamePath(["targets", "length"])` 返回 `(ArrayLength, NamePath(["targets"]))`。
/// 多段路径（如 `request.project.length`）暂不支持，需后续添加字段访问支持。
fn try_method_intrinsic(callee: &LirOperand) -> Option<(JvmIntrinsicCallLowering, NamePath)> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    if path.parts().len() != 2 {
        return None;
    }
    let last_segment = path.parts().last()?;
    let intrinsic = match last_segment.as_str() {
        "len" | "length" => JvmIntrinsicCallLowering::ArrayLength,
        "trim" => JvmIntrinsicCallLowering::StringTrim,
        "to_lower" | "to_lowercase" | "lowercase" => JvmIntrinsicCallLowering::StringToLower,
        "to_upper" | "to_uppercase" | "uppercase" => JvmIntrinsicCallLowering::StringToUpper,
        "starts_with" | "startsWith" => JvmIntrinsicCallLowering::StringStartsWith,
        "ends_with" | "endsWith" => JvmIntrinsicCallLowering::StringEndsWith,
        "contains" => JvmIntrinsicCallLowering::StringContains,
        "slice" | "substring" => JvmIntrinsicCallLowering::StringSlice,
        "replace" => JvmIntrinsicCallLowering::StringReplace,
        "split" => JvmIntrinsicCallLowering::StringSplit,
        _ => return None,
    };
    let receiver_segments = path.parts()[..path.parts().len() - 1].to_vec();
    Some((intrinsic, NamePath::new(receiver_segments)))
}

/// 加载接收者操作数到栈上，并返回对应的 `LirOperand::Value`。
///
/// 接收者路径为单段时，从命名变量槽位加载。
fn load_receiver_operand(
    receiver_path: &NamePath,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<LirOperand> {
    if receiver_path.parts().len() != 1 {
        return Err(miette!("方法调用内建函数的接收者路径仅支持单段，收到 {:?}", receiver_path));
    }
    let var_name = receiver_path.parts()[0].as_str();
    let slot = context.try_slot_for_var(var_name).ok_or_else(|| miette!("方法调用内建函数的接收者变量 `{}` 未找到槽位", var_name))?;
    let ty = context.type_for_var(var_name);
    instructions.push(load_instruction_for_type(&ty, slot));
    // 分配临时槽位存储接收者值，创建合成 MirValueRef 供内建 lowering 使用。
    let temp_slot = context.allocate_slot();
    let temp_ref = MirValueRef(u32::MAX / 2 + temp_slot as u32);
    instructions.push(store_instruction_for_type(&ty, temp_slot));
    context.value_slots.insert(temp_ref, temp_slot);
    context.value_types.insert(temp_ref, ty);
    Ok(LirOperand::Value(temp_ref))
}

fn lower_intrinsic_call(
    intrinsic: JvmIntrinsicCallLowering,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    match intrinsic {
        JvmIntrinsicCallLowering::BinaryNumeric(op) => {
            if arguments.len() != 2 {
                return Err(miette!("二元数值内建调用参数数量错误"));
            }
            // 对于 Add 操作，检查是否为字符串拼接。
            if matches!(op, JvmBinaryNumericOp::Add) {
                let lhs_ty = operand_type(&arguments[0], context);
                let rhs_ty = operand_type(&arguments[1], context);
                let lhs_is_string = operand_is_string(&arguments[0], context);
                let rhs_is_string = operand_is_string(&arguments[1], context);
                if lhs_is_string || rhs_is_string || is_reference_descriptor(&lhs_ty) || is_reference_descriptor(&rhs_ty) {
                    // 字符串拼接：左操作数.concat(右操作数)
                    // 非字符串操作数先用 String.valueOf 转换为字符串。
                    lower_operand(&arguments[0], context, instructions)?;
                    if !lhs_is_string {
                        instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                            owner: "java/lang/String".to_string(),
                            name: "valueOf".to_string(),
                            descriptor: string_value_of_descriptor(&lhs_ty),
                        }));
                    }
                    lower_operand(&arguments[1], context, instructions)?;
                    if !rhs_is_string {
                        instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                            owner: "java/lang/String".to_string(),
                            name: "valueOf".to_string(),
                            descriptor: string_value_of_descriptor(&rhs_ty),
                        }));
                    }
                    instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                        owner: "java/lang/String".to_string(),
                        name: "concat".to_string(),
                        descriptor: JvmMethodDescriptor::new(
                            vec![JvmTypeDescriptor::Object("java/lang/String".to_string())],
                            JvmTypeDescriptor::Object("java/lang/String".to_string()),
                        ),
                    }));
                    // 字符串拼接结果类型为 String，更新输出类型以确保使用正确的存储指令。
                    if let Some(output) = output {
                        context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
                    }
                    store_required_output(output, context, instructions)?;
                    return Ok(());
                }
            }
            let lhs_ty = operand_type(&arguments[0], context);
            let rhs_ty = operand_type(&arguments[1], context);
            if is_reference_descriptor(&lhs_ty) || is_reference_descriptor(&rhs_ty) {
                // 宽容处理非数值参与的数值运算：返回 0 作为占位，避免编译失败。
                if let Some(output) = output {
                    context.value_types.insert(output, JvmTypeDescriptor::Int);
                }
                instructions.push(JvmInstruction::IConst(0));
                store_required_output(output, context, instructions)?;
                return Ok(());
            }
            let result_ty = lhs_ty;
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(numeric_binary_instruction(&result_ty, op)?);
            if let Some(output) = output {
                context.value_types.insert(output, result_ty);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::Compare(compare) => {
            if arguments.len() != 2 {
                return Err(miette!("数值比较内建调用参数数量错误"));
            }
            let compare_ty = operand_type(&arguments[0], context);
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            let true_label = context.fresh_label("cmp_true");
            let end_label = context.fresh_label("cmp_end");
            emit_compare_branch(&compare_ty, compare, &true_label, instructions)?;
            instructions.push(JvmInstruction::IConst(0));
            instructions.push(JvmInstruction::Goto(end_label.clone()));
            instructions.push(JvmInstruction::Label(true_label));
            instructions.push(JvmInstruction::IConst(1));
            instructions.push(JvmInstruction::Label(end_label));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::NumericNeg => {
            if arguments.len() != 1 {
                return Err(miette!("数值取负内建调用参数数量错误"));
            }
            let value_ty = operand_type(&arguments[0], context);
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(numeric_neg_instruction(&value_ty)?);
            if let Some(output) = output {
                context.value_types.insert(output, value_ty);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::LogicalAnd => {
            if arguments.len() != 2 {
                return Err(miette!("逻辑与内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::IAnd);
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Boolean);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::LogicalOr => {
            if arguments.len() != 2 {
                return Err(miette!("逻辑或内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::IOr);
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Boolean);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::LogicalNot => {
            if arguments.len() != 1 {
                return Err(miette!("逻辑取反内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            let true_label = context.fresh_label("not_true");
            let end_label = context.fresh_label("not_end");
            instructions.push(JvmInstruction::IfEq(true_label.clone()));
            instructions.push(JvmInstruction::IConst(0));
            instructions.push(JvmInstruction::Goto(end_label.clone()));
            instructions.push(JvmInstruction::Label(true_label));
            instructions.push(JvmInstruction::IConst(1));
            instructions.push(JvmInstruction::Label(end_label));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::ArrayLength => {
            if arguments.len() != 1 {
                return Err(miette!("数组长度内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            // 根据参数类型选择正确的长度指令：
            // - String 类型使用 `invokevirtual java/lang/String.length()I`
            // - 数组类型使用 `arraylength`
            let arg_ty = infer_operand_type(&arguments[0], context.signatures);
            let arg_ty = match &arguments[0] {
                LirOperand::Value(v) => context.type_for_value(*v),
                _ => arg_ty,
            };
            if matches!(arg_ty, JvmTypeDescriptor::Object(ref name) if name == "java/lang/String") {
                instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                    owner: "java/lang/String".to_string(),
                    name: "length".to_string(),
                    descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Int),
                }));
            }
            else {
                instructions.push(JvmInstruction::ArrayLength);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::ArrayLiteral => {
            // 数组字面量构造：`array()` 或 `array(a, b, c)`。
            // 元素类型优先从输出值的类型推断（由 propagate_var_decl_types 设置），
            // 其次从参数推断，最后回退到 Object。
            let element_type = if let Some(output) = output {
                let array_ty = context.type_for_value(output);
                match array_ty {
                    JvmTypeDescriptor::Array(item) => *item,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                }
            }
            else if !arguments.is_empty() {
                // 从第一个参数推断元素类型。
                match &arguments[0] {
                    LirOperand::Value(v) => context.type_for_value(*v),
                    LirOperand::Constant(MirConstant::String(_)) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                    LirOperand::Constant(MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit) => JvmTypeDescriptor::Int,
                    LirOperand::Constant(MirConstant::Float64(_)) => JvmTypeDescriptor::Double,
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                }
            }
            else {
                JvmTypeDescriptor::Object("java/lang/Object".to_string())
            };
            // 压入数组大小。
            instructions.push(JvmInstruction::IConst(arguments.len() as i32));
            // 创建对应元素类型的数组。
            instructions.push(array_new_instruction(&element_type));
            // 逐个存储元素。
            for (index, argument) in arguments.iter().enumerate() {
                instructions.push(JvmInstruction::Dup);
                instructions.push(JvmInstruction::IConst(index as i32));
                lower_operand(argument, context, instructions)?;
                instructions.push(array_store_instruction(&element_type));
            }
            // 设置输出类型为数组。
            if let Some(output) = output {
                let array_ty = JvmTypeDescriptor::Array(Box::new(element_type));
                context.value_types.insert(output, array_ty);
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringTrim => {
            if arguments.len() != 1 {
                return Err(miette!("字符串 trim 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "trim".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Object("java/lang/String".to_string())),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringToLower => {
            if arguments.len() != 1 {
                return Err(miette!("字符串 to_lower 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "toLowerCase".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Object("java/lang/String".to_string())),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringToUpper => {
            if arguments.len() != 1 {
                return Err(miette!("字符串 to_upper 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "toUpperCase".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Object("java/lang/String".to_string())),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringStartsWith => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 starts_with 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "startsWith".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".to_string())], JvmTypeDescriptor::Int),
            }));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringEndsWith => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 ends_with 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "endsWith".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".to_string())], JvmTypeDescriptor::Int),
            }));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringContains => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 contains 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "contains".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/lang/CharSequence".to_string())],
                    JvmTypeDescriptor::Int,
                ),
            }));
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringSlice => {
            if arguments.len() != 3 {
                return Err(miette!("字符串 slice 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            lower_operand(&arguments[2], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "substring".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Int, JvmTypeDescriptor::Int],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringReplace => {
            if arguments.len() != 3 {
                return Err(miette!("字符串 replace 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            lower_operand(&arguments[2], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "replace".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/lang/CharSequence".to_string()),
                        JvmTypeDescriptor::Object("java/lang/CharSequence".to_string()),
                    ],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
            }
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::StringSplit => {
            if arguments.len() != 2 {
                return Err(miette!("字符串 split 内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/lang/String".to_string(),
                name: "split".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/lang/String".to_string())],
                    JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
                ),
            }));
            if let Some(output) = output {
                let array_ty = JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string())));
                context.value_types.insert(output, array_ty);
            }
            store_required_output(output, context, instructions)?;
        }
    }
    Ok(())
}

fn store_required_output(
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    let output = output.ok_or_else(|| miette!("当前 JVM 最小 lowering 要求值产生操作必须带输出槽位"))?;
    let slot = context.slot_for_output(output);
    let ty = context.type_for_value(output);
    instructions.push(store_instruction_for_type(&ty, slot));
    Ok(())
}

fn lower_static_call(
    callee: &LirOperand,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    let symbol = symbol_name(callee)?;
    if try_lower_host_bridge_call(&symbol, arguments, output, context, instructions)? {
        return Ok(());
    }
    // 若被调符号不在已知函数签名表中，视为外部调用或结构体构造，
    // 根据实际参数类型生成描述符，返回类型默认为 Object。
    let descriptor = context.signatures.get(&symbol).cloned().unwrap_or_else(|| {
        let parameter_types = arguments.iter().map(|arg| operand_type(arg, context)).collect();
        JvmMethodDescriptor::new(parameter_types, JvmTypeDescriptor::Object("java/lang/Object".to_string()))
    });
    for (index, argument) in arguments.iter().enumerate() {
        lower_operand(argument, context, instructions)?;
        // 若参数实际类型为 Object 但期望类型为更具体的引用类型，插入 checkcast。
        // 这修正了合成 getter（如 get_target）返回 Object 但实际值为 String 的类型不匹配。
        if let Some(expected_ty) = descriptor.parameter_types.get(index) {
            let actual_ty = operand_type(argument, context);
            if needs_checkcast(&actual_ty, expected_ty) {
                if let JvmTypeDescriptor::Object(class_name) = expected_ty {
                    instructions.push(JvmInstruction::CheckCast(class_name.clone()));
                }
            }
        }
    }
    instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
        owner: context.owner.to_string(),
        name: symbol,
        descriptor: descriptor.clone(),
    }));

    if descriptor.return_type != JvmTypeDescriptor::Void {
        // 根据方法返回类型设置输出类型，确保后续存储指令类型正确。
        if let Some(output) = output {
            context.value_types.insert(output, descriptor.return_type.clone());
        }
        store_required_output(output, context, instructions)?;
    }
    Ok(())
}

fn try_lower_host_bridge_call(
    symbol: &str,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<bool> {
    match symbol {
        "__console_write" => {
            emit_print_stream_call("out", "print", arguments, context, instructions)?;
            return Ok(true);
        }
        "__console_write_line" => {
            emit_print_stream_call("out", "println", arguments, context, instructions)?;
            return Ok(true);
        }
        "__console_error_line" => {
            emit_print_stream_call("err", "println", arguments, context, instructions)?;
            return Ok(true);
        }
        "__console_read" => {
            instructions.push(JvmInstruction::GetStatic(JvmFieldRef {
                owner: "java/lang/System".to_string(),
                name: "in".to_string(),
                descriptor: JvmTypeDescriptor::Object("java/io/InputStream".to_string()),
            }));
            instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
                owner: "java/io/InputStream".to_string(),
                name: "read".to_string(),
                descriptor: JvmMethodDescriptor::new(vec![], JvmTypeDescriptor::Int),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__system_get_property" => {
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/lang/System".to_string(),
                name: "getProperty".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/lang/String".to_string())],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__path_of" => {
            lower_operand(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/lang/String", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Path".to_string(),
                name: "of".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/lang/String".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
                    ],
                    JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/nio/file/Path".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__file_exists" => {
            emit_path_of_argument(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/nio/file/LinkOption", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "exists".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/LinkOption".to_string()))),
                    ],
                    JvmTypeDescriptor::Int,
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__is_directory" => {
            emit_path_of_argument(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/nio/file/LinkOption", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "isDirectory".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/LinkOption".to_string()))),
                    ],
                    JvmTypeDescriptor::Int,
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__mkdirs" => {
            emit_path_of_argument(&arguments[0], context, instructions)?;
            emit_empty_object_array("java/nio/file/attribute/FileAttribute", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "createDirectories".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/attribute/FileAttribute".to_string()))),
                    ],
                    JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                ),
            }));
            instructions.push(JvmInstruction::Pop);
            instructions.push(JvmInstruction::IConst(1));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Int);
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__file_read_all_text" => {
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::CheckCast("java/nio/file/Path".to_string()));
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "readString".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![JvmTypeDescriptor::Object("java/nio/file/Path".to_string())],
                    JvmTypeDescriptor::Object("java/lang/String".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/lang/String".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            return Ok(true);
        }
        "__file_write_all_text" => {
            lower_operand(&arguments[0], context, instructions)?;
            instructions.push(JvmInstruction::CheckCast("java/nio/file/Path".to_string()));
            lower_operand(&arguments[1], context, instructions)?;
            emit_empty_object_array("java/nio/file/OpenOption", instructions);
            instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                owner: "java/nio/file/Files".to_string(),
                name: "writeString".to_string(),
                descriptor: JvmMethodDescriptor::new(
                    vec![
                        JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                        JvmTypeDescriptor::Object("java/lang/CharSequence".to_string()),
                        JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/nio/file/OpenOption".to_string()))),
                    ],
                    JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
                ),
            }));
            if let Some(output) = output {
                context.value_types.insert(output, JvmTypeDescriptor::Object("java/nio/file/Path".to_string()));
                store_required_output(Some(output), context, instructions)?;
            }
            else {
                instructions.push(JvmInstruction::Pop);
            }
            return Ok(true);
        }
        _ => {}
    }

    Ok(false)
}

fn emit_print_stream_call(
    stream_field: &str,
    method_name: &str,
    arguments: &[LirOperand],
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    instructions.push(JvmInstruction::GetStatic(JvmFieldRef {
        owner: "java/lang/System".to_string(),
        name: stream_field.to_string(),
        descriptor: JvmTypeDescriptor::Object("java/io/PrintStream".to_string()),
    }));
    lower_operand(&arguments[0], context, instructions)?;
    instructions.push(JvmInstruction::InvokeVirtual(JvmMethodRef {
        owner: "java/io/PrintStream".to_string(),
        name: method_name.to_string(),
        descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Object("java/lang/String".to_string())], JvmTypeDescriptor::Void),
    }));
    Ok(())
}

fn emit_path_of_argument(
    argument: &LirOperand,
    context: &mut FunctionLoweringContext<'_>,
    instructions: &mut Vec<JvmInstruction>,
) -> Result<()> {
    lower_operand(argument, context, instructions)?;
    emit_empty_object_array("java/lang/String", instructions);
    instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
        owner: "java/nio/file/Path".to_string(),
        name: "of".to_string(),
        descriptor: JvmMethodDescriptor::new(
            vec![
                JvmTypeDescriptor::Object("java/lang/String".to_string()),
                JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/String".to_string()))),
            ],
            JvmTypeDescriptor::Object("java/nio/file/Path".to_string()),
        ),
    }));
    Ok(())
}

fn emit_empty_object_array(class_name: &str, instructions: &mut Vec<JvmInstruction>) {
    instructions.push(JvmInstruction::IConst(0));
    instructions.push(JvmInstruction::ANewArray(class_name.to_string()));
}

fn is_jvm_host_bridge_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "__console_write"
            | "__console_write_line"
            | "__console_error_line"
            | "__console_read"
            | "__console_read_line"
            | "__system_get_property"
            | "__file_exists"
            | "__mkdirs"
            | "__is_directory"
            | "__path_of"
            | "__file_read_all_text"
            | "__file_write_all_text"
    )
}

fn symbol_name(callee: &LirOperand) -> Result<String> {
    let LirOperand::Symbol(path) = callee
    else {
        eprintln!("[DEBUG symbol_name] non-symbol callee: {:?}", callee);
        return Err(miette!("JVM 最小 lowering 暂只支持符号静态调用"));
    };
    last_path_segment(path).ok_or_else(|| miette!("无法解析被调符号 `{}`", path))
}

/// 判断是否需要在参数前插入 `checkcast` 指令。
///
/// 当实际类型为 `Object("java/lang/Object")`（默认回退类型），
/// 而期望类型为更具体的对象类型时，需要插入 `checkcast` 以通过 JVM 字节码验证。
fn needs_checkcast(actual: &JvmTypeDescriptor, expected: &JvmTypeDescriptor) -> bool {
    match (actual, expected) {
        (JvmTypeDescriptor::Object(actual_name), JvmTypeDescriptor::Object(expected_name)) => {
            actual_name == "java/lang/Object" && expected_name != "java/lang/Object"
        }
        _ => false,
    }
}

fn last_path_segment(path: &NamePath) -> Option<String> {
    path.parts().last().map(|segment| segment.to_string())
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

fn jvm_type_descriptor(ty: &ValkyrieType) -> Result<JvmTypeDescriptor> {
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

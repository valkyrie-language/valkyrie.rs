//! `JVM` 路线 lowering。
//!
//! 这里把 lane-aware `LIR` 收口成最小可编码的 `JVM class` 模型。
//! 当前刻意只实现一条很窄的路径：
//! - 顶层函数 -> 单个静态方法
//! - 参数与局部值先按单槽位整型处理
//! - 控制流仍由 `LIR` 基本块表达，`JVM` 细节只在本 crate 内部消化

use std::collections::{BTreeMap, BTreeSet};

use miette::{miette, Result};
use nyar::{
    abstractions::{BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use valkyrie_compiler::{
    lir::{LirDispatchKind, LirFunction, LirModule, LirOperand, LirOperation, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirConstant, MirValueRef},
};
use valkyrie_types::{hir::HirType, NamePath};

use crate::{JvmClassFile, JvmCodeBody, JvmInstruction, JvmMethodDescriptor, JvmMethodRef, JvmMethodSignature, JvmTypeDescriptor};

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
    let signatures = collect_function_signatures(lir)?;
    let mut class_file = JvmClassFile::new(internal_name.clone());
    // 按 (name, descriptor) 去重，避免合并多源文件时同名方法冲突。
    // 保留首次出现的方法，后续重复定义被跳过。
    let mut seen: std::collections::BTreeSet<(String, String)> = std::collections::BTreeSet::new();
    let mut methods = Vec::with_capacity(lir.functions.len());
    for function in &lir.functions {
        let method = lower_function(function, &internal_name, &signatures)?;
        let key = (method.name.clone(), method.descriptor.to_string());
        if seen.insert(key) {
            methods.push(method);
        }
    }
    class_file.methods = methods;
    class_file.optimize_static_self_tail_recursion().map_err(|error| miette!("JVM 尾递归循环化失败：{error}"))?;
    Ok(class_file)
}

fn lower_function(function: &LirFunction, owner: &str, signatures: &BTreeMap<String, JvmMethodDescriptor>) -> Result<JvmMethodSignature> {
    let descriptor = signatures.get(&function.symbol).cloned().ok_or_else(|| miette!("缺少函数 `{}` 的 JVM 描述符", function.symbol))?;
    let block_labels: BTreeMap<MirBlockRef, String> = function.blocks.iter().map(|block| (block.id, format!("BB{}", block.id.0))).collect();
    // 按参数的 JVM 类型槽位计算入口局部变量起始偏移。
    // `long` / `double` 占 2 槽，若仅按参数数量计算会导致 `max_locals` 不足。
    let entry_parameters = function.param_types.iter().map(|ty| jvm_type_descriptor(ty).map(|d| d.slot_count()).unwrap_or(1)).sum::<u16>();
    let mut context = FunctionLoweringContext::new(owner, signatures, &block_labels, entry_parameters);
    context.reserve_block_parameters(function);
    context.reserve_operation_outputs(function);
    // 在操作输出类型已知后，通过跳转参数推断非入口 block 的参数类型。
    context.infer_block_parameter_types(function);
    // 传播变量声明类型到源 SSA 值，用于修正 array() 等内建调用的返回类型。
    context.propagate_var_decl_types(function);
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

fn collect_function_signatures(lir: &LirModule) -> Result<BTreeMap<String, JvmMethodDescriptor>> {
    lir.functions.iter().map(|function| Ok((function.symbol.clone(), build_method_descriptor(function)?))).collect()
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

fn entry_block(function: &LirFunction) -> Result<&valkyrie_compiler::lir::LirBlock> {
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
            LirTerminator::Return { .. } | LirTerminator::Unreachable => {}
        }
    }
    referenced_targets
}

struct FunctionLoweringContext<'a> {
    owner: &'a str,
    signatures: &'a BTreeMap<String, JvmMethodDescriptor>,
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
        block_labels: &'a BTreeMap<MirBlockRef, String>,
        entry_parameters: u16,
    ) -> Self {
        Self {
            owner,
            signatures,
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
            let ty = infer_operand_type(arg, self.signatures);
            // 对于 Value 类型的参数，需要查询其已记录的类型。
            if let LirOperand::Value(v) = arg {
                if let Some(existing_ty) = self.value_types.get(v) {
                    self.value_types.insert(*param, existing_ty.clone());
                    continue;
                }
            }
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
                        let ty = infer_operation_output_type(operation, self.signatures);
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
                            LirOperand::Constant(constant) => match constant {
                                MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
                                MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                            },
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
        self.next_slot += 1;
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
        LirOperationKind::LoadConstant { constant } => {
            lower_constant(constant, instructions)?;
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::LoadSymbol { path } => {
            // 模块路径等符号暂用 null 占位，待后续支持完整的外部符号解析。
            let _ = path;
            instructions.push(JvmInstruction::AConstNull);
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::Move { source } => {
            lower_operand(source, context, instructions)?;
            // 根据 source 类型动态选择存储指令，避免预推断类型不完整导致的类型不匹配。
            if let Some(output) = operation.output {
                let slot = context.slot_for_output(output);
                let ty = match source {
                    LirOperand::Value(v) => context.type_for_value(*v),
                    LirOperand::Constant(constant) => match constant {
                        MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
                        MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                    },
                    LirOperand::Symbol(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                };
                instructions.push(store_instruction_for_type(&ty, slot));
                context.value_types.insert(output, ty);
            }
        }
        LirOperationKind::StoreVar { name, value, ty: _ } => {
            lower_operand(value, context, instructions)?;
            let slot = context.slot_for_var(name);
            // 优先使用变量已记录的类型（来自 propagate_var_decl_types），否则从 value 推断。
            let ty = context.var_types.get(name).cloned().unwrap_or_else(|| match value {
                LirOperand::Value(v) => context.type_for_value(*v),
                LirOperand::Constant(constant) => match constant {
                    MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
                    MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                },
                LirOperand::Symbol(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
            });
            context.var_types.insert(name.clone(), ty.clone());
            instructions.push(store_instruction_for_type(&ty, slot));
            if let Some(output) = operation.output {
                context.value_slots.insert(output, slot);
                context.value_types.insert(output, ty);
            }
        }
        LirOperationKind::Call { dispatch, callee, arguments, witness, effect } => {
            if witness.is_some() || effect.is_some() {
                return Err(miette!("JVM 最小 lowering 暂不支持 witness / effect 调用"));
            }
            if *dispatch != LirDispatchKind::Static {
                return Err(miette!("JVM 最小 lowering 暂只支持静态调用"));
            }
            if let Some(intrinsic) = try_intrinsic_call(callee) {
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
        LirOperationKind::Subscript { object, index } => {
            // 数组下标访问：压入数组和索引，然后 emit aaload。
            lower_operand(object, context, instructions)?;
            lower_operand(index, context, instructions)?;
            instructions.push(JvmInstruction::AALoad);
            store_output(operation.output, context, instructions);
        }
        LirOperationKind::StoreSubscript { object, index, value } => {
            // 数组元素存储：压入数组、索引、值，然后 emit aastore。
            lower_operand(object, context, instructions)?;
            lower_operand(index, context, instructions)?;
            lower_operand(value, context, instructions)?;
            instructions.push(JvmInstruction::AAStore);
        }
        LirOperationKind::ArrayNew { element_type, length } => {
            // 数组创建：压入长度，然后 emit anewarray。
            lower_operand(length, context, instructions)?;
            let class_name = match jvm_type_descriptor(element_type) {
                Ok(JvmTypeDescriptor::Object(name)) => name,
                _ => "java/lang/Object".to_string(),
            };
            instructions.push(JvmInstruction::ANewArray(class_name));
            store_output(operation.output, context, instructions);
        }
        // TODO: 结构体相关操作尚未在 JVM 后端实现
        _ => {}
    }
    Ok(())
}

/// 根据操作类型推断其输出的 JVM 类型描述符。
fn infer_operation_output_type(operation: &LirOperation, signatures: &BTreeMap<String, JvmMethodDescriptor>) -> JvmTypeDescriptor {
    match &operation.kind {
        LirOperationKind::LoadConstant { constant } => match constant {
            MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
            MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
        },
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
        LirOperationKind::Call { callee, .. } => {
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
                        if let Some(segment) = path.0.last() {
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
        LirOperationKind::Subscript { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        LirOperationKind::StoreSubscript { .. } => JvmTypeDescriptor::Void,
        LirOperationKind::StructNew { type_name, .. } => JvmTypeDescriptor::Object(type_name.clone()),
        LirOperationKind::FieldGet { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        LirOperationKind::FieldSet { .. } => JvmTypeDescriptor::Void,
        LirOperationKind::ArrayNew { element_type, .. } => {
            // 数组创建：返回与元素类型匹配的数组类型。
            match jvm_type_descriptor(element_type) {
                Ok(jvm_ty) => JvmTypeDescriptor::Array(Box::new(jvm_ty)),
                Err(_) => JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object("java/lang/Object".to_string()))),
            }
        }
    }
}

/// 推断操作数的 JVM 类型描述符。
fn infer_operand_type(operand: &LirOperand, signatures: &BTreeMap<String, JvmMethodDescriptor>) -> JvmTypeDescriptor {
    match operand {
        LirOperand::Value(_) => {
            // 值的类型需要从上下文获取，这里返回 Int 作为占位。
            // 实际类型在 reserve_operation_outputs 中已被记录。
            JvmTypeDescriptor::Int
        }
        LirOperand::Constant(constant) => match constant {
            MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
            MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
        },
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

/// 根据类型选择正确的存储指令。
fn store_instruction_for_type(ty: &JvmTypeDescriptor, slot: u16) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Long => JvmInstruction::IStore(slot),
        JvmTypeDescriptor::Float => JvmInstruction::IStore(slot),
        JvmTypeDescriptor::Double => JvmInstruction::IStore(slot),
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::AStore(slot),
        _ => JvmInstruction::IStore(slot),
    }
}

/// 根据类型选择正确的加载指令。
fn load_instruction_for_type(ty: &JvmTypeDescriptor, slot: u16) -> JvmInstruction {
    match ty {
        JvmTypeDescriptor::Object(_) | JvmTypeDescriptor::Array(_) => JvmInstruction::ALoad(slot),
        _ => JvmInstruction::ILoad(slot),
    }
}

fn lower_terminator(
    terminator: &LirTerminator,
    return_type: &HirType,
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
    if target_slots.len() != arguments.len() {
        return Err(miette!("块参数数量与跳转参数数量不匹配"));
    }
    // 按逆序存储参数（栈是 LIFO），并根据参数类型选择正确的存储指令。
    for slot in target_slots.iter().rev() {
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
            // 符号作为操作数时暂用 null 占位，待后续支持完整的外部符号解析。
            let _ = path;
            instructions.push(JvmInstruction::AConstNull);
        }
    }
    Ok(())
}

fn lower_constant(constant: &MirConstant, instructions: &mut Vec<JvmInstruction>) -> Result<()> {
    match constant {
        MirConstant::Int(value) => {
            let value = i32::try_from(*value).map_err(|_| miette!("JVM 最小 lowering 暂只支持 `i32` 常量"))?;
            instructions.push(JvmInstruction::IConst(value));
        }
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
    BinaryInt(JvmBinaryIntOp),
    Compare(JvmIntComparison),
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
enum JvmBinaryIntOp {
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
    if path.0.len() != 1 {
        return None;
    }
    Some(match path.0[0].as_str() {
        "infix +" => JvmIntrinsicCallLowering::BinaryInt(JvmBinaryIntOp::Add),
        "infix -" => JvmIntrinsicCallLowering::BinaryInt(JvmBinaryIntOp::Sub),
        "infix *" => JvmIntrinsicCallLowering::BinaryInt(JvmBinaryIntOp::Mul),
        "infix /" => JvmIntrinsicCallLowering::BinaryInt(JvmBinaryIntOp::Div),
        "infix %" => JvmIntrinsicCallLowering::BinaryInt(JvmBinaryIntOp::Rem),
        "infix ==" => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Eq),
        "infix !=" => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Ne),
        "infix <" => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Lt),
        "infix <=" => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Le),
        "infix >" => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Gt),
        "infix >=" => JvmIntrinsicCallLowering::Compare(JvmIntComparison::Ge),
        "prefix !" => JvmIntrinsicCallLowering::LogicalNot,
        "len" | "length" => JvmIntrinsicCallLowering::ArrayLength,
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
    if path.0.len() != 2 {
        return None;
    }
    let last_segment = path.0.last()?;
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
    let receiver_segments = path.0[..path.0.len() - 1].to_vec();
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
    if receiver_path.0.len() != 1 {
        return Err(miette!("方法调用内建函数的接收者路径仅支持单段，收到 {:?}", receiver_path));
    }
    let var_name = receiver_path.0[0].as_str();
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
        JvmIntrinsicCallLowering::BinaryInt(op) => {
            if arguments.len() != 2 {
                return Err(miette!("二元整数内建调用参数数量错误"));
            }
            // 对于 Add 操作，检查是否为字符串拼接。
            if matches!(op, JvmBinaryIntOp::Add) {
                let lhs_is_string = operand_is_string(&arguments[0], context);
                let rhs_is_string = operand_is_string(&arguments[1], context);
                if lhs_is_string || rhs_is_string {
                    // 字符串拼接：左操作数.concat(右操作数)
                    // 非字符串操作数先用 String.valueOf 转换为字符串。
                    lower_operand(&arguments[0], context, instructions)?;
                    if !lhs_is_string {
                        instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                            owner: "java/lang/String".to_string(),
                            name: "valueOf".to_string(),
                            descriptor: JvmMethodDescriptor::new(
                                vec![JvmTypeDescriptor::Int],
                                JvmTypeDescriptor::Object("java/lang/String".to_string()),
                            ),
                        }));
                    }
                    lower_operand(&arguments[1], context, instructions)?;
                    if !rhs_is_string {
                        instructions.push(JvmInstruction::InvokeStatic(JvmMethodRef {
                            owner: "java/lang/String".to_string(),
                            name: "valueOf".to_string(),
                            descriptor: JvmMethodDescriptor::new(
                                vec![JvmTypeDescriptor::Int],
                                JvmTypeDescriptor::Object("java/lang/String".to_string()),
                            ),
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
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            instructions.push(match op {
                JvmBinaryIntOp::Add => JvmInstruction::IAdd,
                JvmBinaryIntOp::Sub => JvmInstruction::ISub,
                JvmBinaryIntOp::Mul => JvmInstruction::IMul,
                JvmBinaryIntOp::Div => JvmInstruction::IDiv,
                JvmBinaryIntOp::Rem => JvmInstruction::IRem,
            });
            store_required_output(output, context, instructions)?;
        }
        JvmIntrinsicCallLowering::Compare(compare) => {
            if arguments.len() != 2 {
                return Err(miette!("整数比较内建调用参数数量错误"));
            }
            lower_operand(&arguments[0], context, instructions)?;
            lower_operand(&arguments[1], context, instructions)?;
            let true_label = context.fresh_label("cmp_true");
            let end_label = context.fresh_label("cmp_end");
            instructions.push(match compare {
                JvmIntComparison::Eq => JvmInstruction::IfICmpEq(true_label.clone()),
                JvmIntComparison::Ne => JvmInstruction::IfICmpNe(true_label.clone()),
                JvmIntComparison::Lt => JvmInstruction::IfICmpLt(true_label.clone()),
                JvmIntComparison::Le => JvmInstruction::IfICmpLe(true_label.clone()),
                JvmIntComparison::Gt => JvmInstruction::IfICmpGt(true_label.clone()),
                JvmIntComparison::Ge => JvmInstruction::IfICmpGe(true_label.clone()),
            });
            instructions.push(JvmInstruction::IConst(0));
            instructions.push(JvmInstruction::Goto(end_label.clone()));
            instructions.push(JvmInstruction::Label(true_label));
            instructions.push(JvmInstruction::IConst(1));
            instructions.push(JvmInstruction::Label(end_label));
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
            let element_class = if let Some(output) = output {
                let array_ty = context.type_for_value(output);
                match array_ty {
                    JvmTypeDescriptor::Array(item) => match *item {
                        JvmTypeDescriptor::Object(ref name) => name.clone(),
                        _ => "java/lang/Object".to_string(),
                    },
                    _ => "java/lang/Object".to_string(),
                }
            }
            else if !arguments.is_empty() {
                // 从第一个参数推断元素类型。
                let first_ty = match &arguments[0] {
                    LirOperand::Value(v) => context.type_for_value(*v),
                    LirOperand::Constant(MirConstant::String(_)) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                    _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
                };
                match first_ty {
                    JvmTypeDescriptor::Object(ref name) => name.clone(),
                    _ => "java/lang/Object".to_string(),
                }
            }
            else {
                "java/lang/Object".to_string()
            };
            // 压入数组大小。
            instructions.push(JvmInstruction::IConst(arguments.len() as i32));
            // 创建对象数组。
            instructions.push(JvmInstruction::ANewArray(element_class.clone()));
            // 逐个存储元素。
            for (index, argument) in arguments.iter().enumerate() {
                instructions.push(JvmInstruction::Dup);
                instructions.push(JvmInstruction::IConst(index as i32));
                lower_operand(argument, context, instructions)?;
                instructions.push(JvmInstruction::AAStore);
            }
            // 设置输出类型为数组。
            if let Some(output) = output {
                let array_ty = JvmTypeDescriptor::Array(Box::new(JvmTypeDescriptor::Object(element_class)));
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
    // 若被调符号不在已知函数签名表中，视为外部调用或结构体构造，
    // 根据实际参数类型生成描述符，返回类型默认为 Object。
    let descriptor = context.signatures.get(&symbol).cloned().unwrap_or_else(|| {
        let parameter_types = arguments
            .iter()
            .map(|arg| match arg {
                LirOperand::Value(v) => context.type_for_value(*v),
                LirOperand::Constant(constant) => match constant {
                    MirConstant::Int(_) | MirConstant::Bool(_) | MirConstant::Unit => JvmTypeDescriptor::Int,
                    MirConstant::String(_) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                },
                LirOperand::Symbol(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
            })
            .collect();
        JvmMethodDescriptor::new(parameter_types, JvmTypeDescriptor::Object("java/lang/Object".to_string()))
    });
    for (index, argument) in arguments.iter().enumerate() {
        lower_operand(argument, context, instructions)?;
        // 若参数实际类型为 Object 但期望类型为更具体的引用类型，插入 checkcast。
        // 这修正了合成 getter（如 get_target）返回 Object 但实际值为 String 的类型不匹配。
        if let Some(expected_ty) = descriptor.parameter_types.get(index) {
            let actual_ty = match argument {
                LirOperand::Value(v) => context.type_for_value(*v),
                LirOperand::Constant(MirConstant::String(_)) => JvmTypeDescriptor::Object("java/lang/String".to_string()),
                LirOperand::Constant(_) => JvmTypeDescriptor::Int,
                LirOperand::Symbol(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
            };
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
    path.0.last().map(|segment| segment.to_string())
}

fn return_instruction_for_type(return_type: &HirType) -> Result<JvmInstruction> {
    default_return_instruction(return_type)
}

/// 按返回类型压入默认值，用于无返回值的 `Return` / `Unreachable` terminator。
fn push_default_value(return_type: &HirType, instructions: &mut Vec<JvmInstruction>) {
    match return_type {
        HirType::Unit | HirType::Void => {
            // void 方法不需要压入返回值。
        }
        HirType::Integer32 | HirType::Boolean | HirType::Integer64 | HirType::Infer => {
            instructions.push(JvmInstruction::IConst(0));
        }
        HirType::Float32 | HirType::Float64 => {
            instructions.push(JvmInstruction::IConst(0));
        }
        // 对象/数组/字符串等引用类型压入 null。
        _ => {
            instructions.push(JvmInstruction::AConstNull);
        }
    }
}

fn default_return_instruction(return_type: &HirType) -> Result<JvmInstruction> {
    Ok(match return_type {
        HirType::Unit | HirType::Void => JvmInstruction::Return,
        HirType::Integer32 | HirType::Boolean | HirType::Infer => JvmInstruction::IReturn,
        HirType::Integer64 => JvmInstruction::LReturn,
        HirType::Float32 => JvmInstruction::FReturn,
        HirType::Float64 => JvmInstruction::DReturn,
        // 所有引用类型（含元组、函数类型）均使用 areturn 返回 null。
        HirType::Utf8
        | HirType::Utf16
        | HirType::Named(_)
        | HirType::SelfType
        | HirType::Array(_)
        | HirType::Apply(_, _)
        | HirType::TraitObject { .. }
        | HirType::Tuple(_)
        | HirType::Function { .. } => JvmInstruction::AReturn,
        _ => return Err(miette!("JVM 最小 lowering 暂不支持类型 {:?} 的默认返回指令", return_type)),
    })
}

fn jvm_type_descriptor(ty: &HirType) -> Result<JvmTypeDescriptor> {
    Ok(match ty {
        HirType::Integer32 | HirType::Boolean => JvmTypeDescriptor::Int,
        HirType::Integer64 => JvmTypeDescriptor::Long,
        HirType::Float32 => JvmTypeDescriptor::Float,
        HirType::Float64 => JvmTypeDescriptor::Double,
        HirType::Utf8 | HirType::Utf16 => JvmTypeDescriptor::Object("java/lang/String".to_string()),
        HirType::Unit | HirType::Void => JvmTypeDescriptor::Void,
        HirType::Array(item) => JvmTypeDescriptor::Array(Box::new(jvm_type_descriptor(item)?)),
        // 命名类型：识别常见的整型/浮点型别名，其余按 Object 处理。
        HirType::Named(name) => match name.as_str() {
            "int" | "i32" | "uint" | "u32" | "usize" | "isize" | "Int32" | "UInt32" | "Size" | "Offset" => JvmTypeDescriptor::Int,
            "long" | "i64" | "u64" | "Int64" | "UInt64" => JvmTypeDescriptor::Long,
            "float" | "f32" | "Float32" => JvmTypeDescriptor::Float,
            "double" | "f64" | "Float64" => JvmTypeDescriptor::Double,
            "bool" | "Boolean" => JvmTypeDescriptor::Int,
            "string" | "String" | "str" | "utf8" | "Utf8" => JvmTypeDescriptor::Object("java/lang/String".to_string()),
            _ => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        },
        HirType::SelfType => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        // 泛型应用类型按类型擦除映射为 Object。
        HirType::Apply(_, _) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        // 推断类型暂按 Int 处理，待类型推断完善后修正。
        HirType::Infer => JvmTypeDescriptor::Int,
        // 函数类型按类型擦除映射为 Object，JVM 运行时通过方法句柄调用。
        HirType::Function { .. } => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
        // 元组类型按类型擦除映射为 Object。
        HirType::Tuple(_) => JvmTypeDescriptor::Object("java/lang/Object".to_string()),
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
        "valkyrie/Main".to_string()
    }
    else if sanitized.contains('/') {
        sanitized
    }
    else {
        format!("valkyrie/{sanitized}")
    }
}

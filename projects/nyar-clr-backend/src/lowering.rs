//! `CLR` 路线 lowering。
//!
//! 这里负责把 `valkyrie-compiler` 的 `LIR` 收口成 `MSIL` 模块，
//! 并通过 `nyar::TargetLoweringLane` 暴露给上层编排。

use std::collections::BTreeMap;

use miette::miette;
use nyar::{
    abstractions::{BackendInputKind, BinaryTarget},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use valkyrie_compiler::{
    lir::{LirFunction, LirModule, LirOperand, LirOperation, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirConstant, MirStruct, MirValueRef},
};
use valkyrie_types::hir::HirType;

use crate::msil::{
    MsilAssembly, MsilField, MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule,
    MsilOpcode, MsilType, MsilTypeDef,
};

/// 将 `HIR` 类型映射为 `MSIL` 类型（降级专用，不报错）。
///
/// 与 `interop::map_hir_type_to_msil` 不同，此函数在遇到不支持的类型时回退为 `object`，
/// 而非返回错误。这是因为 lowering 阶段需要容错处理，避免因单个类型映射失败而中断整个编译。
///
/// `qualified_name_map` 用于将用户定义类型的简单名解析为命名空间限定名，
/// 确保不同命名空间下的同名类型（如 `core.text.TextSpan` 和 `std.data.text.v.TextSpan`）
/// 在 `Field` token 解析时能正确区分。
fn map_hir_type_to_msil_for_lowering(ty: &HirType, qualified_name_map: &BTreeMap<String, String>) -> MsilType {
    match ty {
        HirType::Integer32 => MsilType::Int32,
        HirType::Integer64 => MsilType::Int64,
        HirType::Float32 => MsilType::Float32,
        HirType::Float64 => MsilType::Float64,
        HirType::Boolean => MsilType::Bool,
        // `utf8` 和 `utf16` 在 CLR 中都映射为 `System.String`。
        HirType::Utf8 | HirType::Utf16 => MsilType::String,
        HirType::Unit | HirType::Void => MsilType::Void,
        HirType::Array(inner) => MsilType::sz_array(map_hir_type_to_msil_for_lowering(inner, qualified_name_map)),
        HirType::Named(name) => match name.as_str() {
            "i32" | "int" => MsilType::Int32,
            "i64" | "long" => MsilType::Int64,
            "f32" | "float" => MsilType::Float32,
            "f64" | "double" => MsilType::Float64,
            "bool" | "boolean" => MsilType::Bool,
            "u8" | "byte" => MsilType::UInt8,
            "u16" | "ushort" => MsilType::UInt16,
            "u32" | "uint" => MsilType::UInt32,
            "u64" | "ulong" => MsilType::UInt64,
            "i8" | "sbyte" => MsilType::Int8,
            "i16" | "short" => MsilType::Int16,
            "char" => MsilType::Char,
            "utf8" | "utf16" => MsilType::String,
            "unit" | "void" => MsilType::Void,
            "ExitCode" => MsilType::Int32,
            "usize" | "isize" => MsilType::UInt32,
            // 用户定义的类型：用命名空间限定名，确保字段 token 解析能区分同名类型。
            _ => MsilType::Named(qualified_name_map.get(name.as_str()).cloned().unwrap_or_else(|| name.to_string())),
        },
        _ => MsilType::Object,
    }
}

/// 将 `HIR` 类型映射为 `CLR` 类型名（供 `newarr` 等指令的 `Type` 操作数使用）。
///
/// 返回带程序集前缀的 `ECMA-335` 标准类型名，如 `[mscorlib]System.Int32`、`[mscorlib]System.String` 等。
/// PE writer 的 `parse_external_owner` 要求格式为 `[assembly]FullName`。
fn map_hir_type_to_clr_type_name(ty: &HirType) -> String {
    match ty {
        HirType::Integer32 => "[mscorlib]System.Int32".to_string(),
        HirType::Integer64 => "[mscorlib]System.Int64".to_string(),
        HirType::Float32 => "[mscorlib]System.Single".to_string(),
        HirType::Float64 => "[mscorlib]System.Double".to_string(),
        HirType::Boolean => "[mscorlib]System.Boolean".to_string(),
        HirType::Utf8 | HirType::Utf16 => "[mscorlib]System.String".to_string(),
        HirType::Named(name) => match name.as_str() {
            "i32" | "int" => "[mscorlib]System.Int32".to_string(),
            "i64" | "long" => "[mscorlib]System.Int64".to_string(),
            "f32" | "float" => "[mscorlib]System.Single".to_string(),
            "f64" | "double" => "[mscorlib]System.Double".to_string(),
            "bool" | "boolean" => "[mscorlib]System.Boolean".to_string(),
            "u8" | "byte" => "[mscorlib]System.Byte".to_string(),
            "u16" | "ushort" => "[mscorlib]System.UInt16".to_string(),
            "u32" | "uint" => "[mscorlib]System.UInt32".to_string(),
            "u64" | "ulong" => "[mscorlib]System.UInt64".to_string(),
            "i8" | "sbyte" => "[mscorlib]System.SByte".to_string(),
            "i16" | "short" => "[mscorlib]System.Int16".to_string(),
            "char" => "[mscorlib]System.Char".to_string(),
            "utf8" | "utf16" => "[mscorlib]System.String".to_string(),
            "usize" | "isize" => "[mscorlib]System.UInt32".to_string(),
            // 用户定义类型保留原名，供 PE writer 通过 TypeDef token 解析。
            _ => name.to_string(),
        },
        _ => "[mscorlib]System.Object".to_string(),
    }
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
    // 构建函数符号 -> 返回类型映射，用于在 `Call` 降级时判断调用是否返回 `void`。
    let return_types: BTreeMap<String, HirType> =
        lir.functions.iter().map(|function| (function.symbol.clone(), function.return_type.clone())).collect();

    // 构建简单名 -> 命名空间限定名映射（如 "TextSpan" -> "core.text.TextSpan"）。
    // 用于将 `HirType::Named("TextSpan")` 解析为正确的限定名，
    // 确保不同命名空间下的同名类型在字段 token 解析时能正确区分。
    // 当存在同名类型时，后定义的覆盖先定义的（BTreeMap::insert 语义）。
    let qualified_name_map: BTreeMap<String, String> = lir
        .structs
        .iter()
        .map(|structure| {
            let qualified =
                if structure.namespace.is_empty() { structure.name.clone() } else { format!("{}.{}", structure.namespace, structure.name) };
            (structure.name.clone(), qualified)
        })
        .collect();

    // 构建结构体字段类型映射：(限定名, 字段名) -> 字段类型。
    // 用于在 `FieldGet` 降级时追踪输出值的类型，确保后续字段访问能正确解析 type_name。
    let field_types: BTreeMap<(String, String), HirType> = lir
        .structs
        .iter()
        .flat_map(|structure| {
            let qualified =
                if structure.namespace.is_empty() { structure.name.clone() } else { format!("{}.{}", structure.namespace, structure.name) };
            structure.fields.iter().map(move |field| ((qualified.clone(), field.name.clone()), field.ty.clone()))
        })
        .collect();

    let global_methods =
        lir.functions.iter().map(|function| lower_function(function, &lir.name, &return_types, &field_types, &qualified_name_map)).collect();

    // 将 LIR 结构体定义降级为 MSIL 类型定义。
    // 类型通过命名空间限定名唯一区分，无需去重。
    let types: Vec<MsilTypeDef> = lir.structs.iter().map(|structure| lower_struct_to_type_def(structure, &qualified_name_map)).collect();

    MsilModule { assembly: MsilAssembly { name: lir.name.clone(), externs: vec!["mscorlib".to_string()] }, types, global_methods }
}

/// 将 `MirStruct` 降级为 `MsilTypeDef`。
///
/// 生成类型定义、字段列表和自动构造函数。
/// 构造函数按字段声明顺序接收参数，逐个 `stfld` 赋值。
/// 字段引用使用命名空间限定名，确保不同命名空间下的同名类型字段能正确解析。
fn lower_struct_to_type_def(mir_struct: &MirStruct, qualified_name_map: &BTreeMap<String, String>) -> MsilTypeDef {
    let fields = mir_struct
        .fields
        .iter()
        .map(|field| MsilField { name: field.name.clone(), ty: map_hir_type_to_msil_for_lowering(&field.ty, qualified_name_map) })
        .collect();
    let ctor = generate_constructor(mir_struct, qualified_name_map);
    MsilTypeDef {
        full_name: mir_struct.name.clone(),
        namespace: mir_struct.namespace.clone(),
        fields,
        methods: vec![ctor],
        is_value_type: mir_struct.is_value_type,
    }
}

/// 为结构体生成自动构造函数。
///
/// 构造函数签名：`void .ctor(field1_type, field2_type, ...)`
/// 函数体：`ldarg.0; call [mscorlib]System.Object::.ctor();` 然后对每个字段 `ldarg.0; ldarg N; stfld`。
/// 字段引用使用命名空间限定名，确保 token 解析能匹配正确的 TypeDef。
fn generate_constructor(mir_struct: &MirStruct, qualified_name_map: &BTreeMap<String, String>) -> MsilMethodBody {
    let param_types: Vec<MsilType> =
        mir_struct.fields.iter().map(|field| map_hir_type_to_msil_for_lowering(&field.ty, qualified_name_map)).collect();
    let mut instructions = Vec::new();

    // 构造函数的 owner 和字段引用使用命名空间限定名。
    let qualified_name =
        if mir_struct.namespace.is_empty() { mir_struct.name.clone() } else { format!("{}.{}", mir_struct.namespace, mir_struct.name) };

    // 调用基类构造函数：`ldarg.0; call [mscorlib]System.Object::.ctor()`。
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

    // 对每个字段：`ldarg.0; ldarg (index+1); stfld`。
    for (index, field) in mir_struct.fields.iter().enumerate() {
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ldarg0, operand: None });
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Ldarg,
            operand: Some(MsilInstructionOperand::Integer((index + 1) as i64)),
        });
        instructions.push(MsilInstruction {
            label: None,
            opcode: MsilOpcode::Stfld,
            operand: Some(MsilInstructionOperand::Field(qualified_name.clone(), field.name.clone())),
        });
    }

    instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });

    MsilMethodBody {
        method: MsilMethodRef {
            owner: Some(qualified_name),
            name: ".ctor".to_string(),
            signature: MsilMethodSignature::new(MsilType::Void, param_types),
        },
        locals: Vec::new(),
        instructions,
        max_stack: 2,
        is_entry_point: false,
    }
}

fn lower_function(
    function: &LirFunction,
    module_name: &str,
    return_types: &BTreeMap<String, HirType>,
    field_types: &BTreeMap<(String, String), HirType>,
    qualified_name_map: &BTreeMap<String, String>,
) -> MsilMethodBody {
    let is_entry_point = function.symbol == "main";
    let mut instructions = Vec::new();
    let mut max_stack: u16 = 1;
    let parameter_slots = collect_parameter_slots(function);

    // 值类型映射：MirValueRef -> MsilType。
    // 用于在创建局部变量时推断正确的类型，避免所有局部都声明为 int32。
    // 初始化时填入参数类型。
    let mut value_types: BTreeMap<MirValueRef, MsilType> = BTreeMap::new();
    for (value_ref, slot) in &parameter_slots {
        if let Some(param_ty) = function.param_types.get(*slot) {
            value_types.insert(*value_ref, map_hir_type_to_msil_for_lowering(param_ty, qualified_name_map));
        }
    }

    // 为每个基本块分配 MSIL 标签，用于分支目标解析。
    let block_labels: BTreeMap<MirBlockRef, String> = function.blocks.iter().map(|block| (block.id, format!("BB{}", block.id.0))).collect();

    // 收集所有被引用为分支目标的基本块，确保它们的标签一定会被发射。
    let mut referenced_targets: std::collections::BTreeSet<MirBlockRef> = std::collections::BTreeSet::new();
    for block in &function.blocks {
        match &block.terminator {
            LirTerminator::Jump { target, .. } => {
                referenced_targets.insert(*target);
            }
            LirTerminator::Branch { then_target, else_target, .. } => {
                referenced_targets.insert(*then_target);
                referenced_targets.insert(*else_target);
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

    for block in &function.blocks {
        // 在块的第一条指令上打标签；若块为空，则插入一条带标签的 Nop。
        let block_label = block_labels.get(&block.id).cloned();
        let is_referenced = referenced_targets.contains(&block.id);
        eprintln!("  [DEBUG-BLOCK] {} BB{} ops={} term={:?}", function.symbol, block.id.0, block.operations.len(), block.terminator);
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
                &mut value_types,
                return_types,
                field_types,
                qualified_name_map,
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
            &mut value_types,
            &mut var_slots,
            &mut eval_stack,
            &block_labels,
        );
    }

    if instructions.is_empty() {
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
    }

    apply_tail_call_optimization(&mut instructions);

    // 构建方法签名：使用真实的返回类型和参数类型。
    let return_msil_type = if is_entry_point {
        // 入口点 `main` 的返回类型固定为 `int32`（CLR 入口约定）。
        MsilType::Int32
    }
    else {
        map_hir_type_to_msil_for_lowering(&function.return_type, qualified_name_map)
    };
    let param_msil_types: Vec<MsilType> =
        function.param_types.iter().map(|ty| map_hir_type_to_msil_for_lowering(ty, qualified_name_map)).collect();

    MsilMethodBody {
        method: MsilMethodRef {
            owner: Some(module_name.to_string()),
            name: function.symbol.clone(),
            signature: MsilMethodSignature::new(return_msil_type, param_msil_types),
        },
        locals: local_types,
        instructions,
        max_stack,
        is_entry_point,
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
}

/// 尝试将 callee 符号识别为可内建降级的运算符方法并返回策略。
///
/// `ast_to_hir` 将 `1 + 2` 降级为 `Call { callee: Path("infix +"), ... }`，
/// `-value` / `!value` 也会收敛成对应的 `prefix` 方法调用。
/// 在 CLR 中这些内建方法有对应 opcode（`add`/`neg`/`ceq` 等），
/// 不需要生成 `call` 指令。
fn try_intrinsic_call(callee: &LirOperand) -> Option<IntrinsicCallLowering> {
    let LirOperand::Symbol(path) = callee
    else {
        return None;
    };
    if path.0.len() != 1 {
        return None;
    }
    let name = path.0[0].as_str();
    Some(match name {
        "infix +" => IntrinsicCallLowering::Simple(MsilOpcode::Add),
        "infix -" => IntrinsicCallLowering::Simple(MsilOpcode::Sub),
        "infix *" => IntrinsicCallLowering::Simple(MsilOpcode::Mul),
        "infix /" => IntrinsicCallLowering::Simple(MsilOpcode::Div),
        "infix %" => IntrinsicCallLowering::Simple(MsilOpcode::Rem),
        "infix ==" => IntrinsicCallLowering::Simple(MsilOpcode::Ceq),
        "infix <" => IntrinsicCallLowering::Simple(MsilOpcode::Clt),
        "infix >" => IntrinsicCallLowering::Simple(MsilOpcode::Cgt),
        "infix !=" => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Ceq),
        "infix <=" => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Cgt),
        "infix >=" => IntrinsicCallLowering::NegatedComparison(MsilOpcode::Clt),
        "prefix -" => IntrinsicCallLowering::Simple(MsilOpcode::Neg),
        "prefix !" => IntrinsicCallLowering::LogicalNot,
        "len" => IntrinsicCallLowering::ArrayLength,
        _ => return None,
    })
}

/// 尝试将实例方法调用降级为 CLR 内建调用。
///
/// MIR 将 `obj.method()` 转换为 `method(obj, ...)`，此处根据接收者类型
/// 生成对应的 CLR 指令：
/// - 数组 `length()` → `ldlen` + `conv.i4`
/// - 字符串 `length()` → `call System.String::get_Length()`
/// - 字符串 `starts_with(s)` → `call System.String::StartsWith(string)`
/// - 字符串 `ends_with(s)` → `call System.String::EndsWith(string)`
/// - 字符串 `contains(s)` → `call System.String::Contains(string)`
/// - 字符串 `trim()` → `call System.String::Trim()`
/// - 字符串 `replace(old, new)` → `call System.String::Replace(string, string)`
/// - 字符串 `slice(start, end)` → `call System.String::Substring(int32, int32)`
///
/// 返回 `true` 表示已处理，`false` 表示非实例方法调用，需走常规降级路径。
#[allow(clippy::too_many_arguments)]
fn try_lower_instance_method_call(
    callee: &LirOperand,
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    value_types: &mut BTreeMap<MirValueRef, MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    label: Option<String>,
) -> bool {
    let LirOperand::Symbol(path) = callee
    else {
        return false;
    };
    if path.0.len() != 1 {
        return false;
    }
    let method_name = path.0[0].as_str();
    if arguments.is_empty() {
        return false;
    }

    let receiver_type = infer_operand_type(&arguments[0], value_types);

    // 数组 length() → ldlen + conv.i4
    if method_name == "length" && matches!(receiver_type, MsilType::SzArray(_)) {
        spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
        lower_operand(&arguments[0], instructions, max_stack, parameter_slots, local_slots, eval_stack);
        instructions.push(MsilInstruction { label, opcode: MsilOpcode::Ldlen, operand: None });
        instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::ConvI4, operand: None });
        *max_stack = (*max_stack).max(1);
        eval_stack.pop();
        if let Some(output) = output {
            eval_stack.push(output);
            value_types.insert(output, MsilType::Int32);
        }
        return true;
    }

    // 字符串方法调用
    if receiver_type == MsilType::String {
        let (clr_method, return_type, param_count) = match method_name {
            "length" => ("get_Length", MsilType::Int32, 0),
            "starts_with" => ("StartsWith", MsilType::Bool, 1),
            "ends_with" => ("EndsWith", MsilType::Bool, 1),
            "contains" => ("Contains", MsilType::Bool, 1),
            "trim" => ("Trim", MsilType::String, 0),
            "replace" => ("Replace", MsilType::String, 2),
            "slice" => ("Substring", MsilType::String, 2),
            _ => return false,
        };

        if arguments.len() != param_count + 1 {
            return false;
        }

        spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
        // 加载接收者
        lower_operand(&arguments[0], instructions, max_stack, parameter_slots, local_slots, eval_stack);
        // 加载参数
        for argument in &arguments[1..] {
            lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
        }

        let signature = match method_name {
            "length" => MsilMethodSignature::new(MsilType::Int32, Vec::new()),
            "starts_with" | "ends_with" | "contains" => MsilMethodSignature::new(MsilType::Bool, vec![MsilType::String]),
            "trim" => MsilMethodSignature::new(MsilType::String, Vec::new()),
            "replace" => MsilMethodSignature::new(MsilType::String, vec![MsilType::String, MsilType::String]),
            "slice" => MsilMethodSignature::new(MsilType::String, vec![MsilType::Int32, MsilType::Int32]),
            _ => return false,
        };

        instructions.push(MsilInstruction {
            label,
            opcode: MsilOpcode::Call,
            operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                owner: Some("[mscorlib]System.String".to_string()),
                name: clr_method.to_string(),
                signature,
            })),
        });
        *max_stack = (*max_stack).max((arguments.len() + 1) as u16);

        // call 消费接收者和参数，产出返回值。
        eval_stack.clear();
        if let Some(output) = output {
            value_types.insert(output, return_type);
            eval_stack.push(output);
        }
        return true;
    }

    false
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

/// 判断是否为字符串相等比较调用。
///
/// 当 callee 是 `infix ==` 且至少一个参数是字符串类型时，返回 `true`。
/// CLR 中字符串 `==` 不能用 `ceq`（比较引用），需调用 `System.String::op_Equality`。
/// `ldelem.ref` 返回 `object`，但实际可能是字符串，因此只要任一操作数是 `string` 就使用值比较。
fn is_string_equality_call(callee: &LirOperand, arguments: &[LirOperand], value_types: &BTreeMap<MirValueRef, MsilType>) -> bool {
    let LirOperand::Symbol(path) = callee
    else {
        return false;
    };
    if path.to_string() != "infix ==" {
        return false;
    }
    if arguments.len() != 2 {
        return false;
    }
    // 只要任一操作数是字符串类型，就使用 `op_Equality` 进行值比较。
    // 另一操作数可能是 `object`（来自 `ldelem.ref`），CLR 运行时会自动处理类型转换。
    let lhs_type = infer_operand_type(&arguments[0], value_types);
    let rhs_type = infer_operand_type(&arguments[1], value_types);
    lhs_type == MsilType::String || rhs_type == MsilType::String
}

/// 降级字符串相等比较：调用 `System.String::op_Equality(string, string): bool`。
fn lower_string_equality_call(
    arguments: &[LirOperand],
    output: Option<MirValueRef>,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    value_types: &mut BTreeMap<MirValueRef, MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    label: Option<String>,
) {
    // 先溢出 eval_stack，再逐个压入参数。
    spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
    for argument in arguments {
        lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    }
    // 调用 `[mscorlib]System.String::op_Equality(string, string): bool`。
    instructions.push(MsilInstruction {
        label,
        opcode: MsilOpcode::Call,
        operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
            owner: Some("[mscorlib]System.String".to_string()),
            name: "op_Equality".to_string(),
            signature: MsilMethodSignature::new(MsilType::Bool, vec![MsilType::String, MsilType::String]),
        })),
    });
    *max_stack = (*max_stack).max(2);
    eval_stack.clear();
    if let Some(output) = output {
        value_types.insert(output, MsilType::Bool);
        eval_stack.push(output);
    }
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
    value_types: &mut BTreeMap<MirValueRef, MsilType>,
    return_types: &BTreeMap<String, HirType>,
    field_types: &BTreeMap<(String, String), HirType>,
    qualified_name_map: &BTreeMap<String, String>,
    mut label: Option<String>,
) {
    // 记录 label 锚点：label 必须放在本操作 emit 的第一条指令上，
    // 确保分支目标指向块的第一条指令（而非最后一条，如 add/stloc/call）。
    let anchor = label.as_ref().map(|_| instructions.len());
    eprintln!("  [DEBUG-OP] label={:?} anchor={:?} kind={:?}", label, anchor, operation.kind);

    match &operation.kind {
        LirOperationKind::LoadConstant { constant } => {
            lower_constant(constant, instructions, max_stack, None);
            if let Some(output) = operation.output {
                let ty = infer_constant_type(constant);
                value_types.insert(output, ty);
                eval_stack.push(output);
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
            }
        }
        LirOperationKind::Move { source } => {
            // 推断源值类型，用于局部变量声明。
            let source_type = infer_operand_type(source, value_types);
            // 先分配局部变量槽位（在调用 lower_operand 之前，避免借用冲突）。
            let slot = local_types.len();
            local_types.push(source_type.clone());

            // 若 source 是 Value 且不在栈顶，先溢出 eval_stack。
            let need_spill = match source {
                LirOperand::Value(v) => eval_stack.last() != Some(v),
                _ => false,
            };
            if need_spill {
                spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
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
                value_types.insert(output, source_type);
            }
        }
        LirOperationKind::StoreVar { name, value, ty } => {
            // 推断值类型，用于局部变量声明。
            // 优先使用 `let` 语句的类型注解（如 `let target: LegionPublishTarget = ...`），
            // 因为从值推断可能丢失结构体类型信息（如数组下标访问默认返回 Object）。
            let value_type = match ty {
                Some(annotated) => map_hir_type_to_msil_for_lowering(annotated, qualified_name_map),
                None => infer_operand_type(value, value_types),
            };
            // 按变量名查找或分配槽位：同名 StoreVar 复用同一槽位。
            let slot = match var_slots.get(name).copied() {
                Some(existing) => existing,
                None => {
                    let new_slot = local_types.len();
                    local_types.push(value_type.clone());
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
                spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
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
                value_types.insert(output, value_type);
            }
        }
        LirOperationKind::Call { callee, arguments, .. } => {
            // 识别可内建降级的运算符方法，直接 emit 对应指令序列。
            // 但对于字符串相等比较 `infix ==`，需要调用 `System.String::op_Equality` 而非 `ceq`。
            let is_string_equality = is_string_equality_call(callee, arguments, value_types);
            if !is_string_equality {
                // 尝试实例方法调用降级（如 `length(args)`、`starts_with(left, "/")`）。
                // MIR 将 `obj.method()` 转换为 `method(obj)`，此处根据接收者类型生成对应的 CLR 调用。
                if try_lower_instance_method_call(
                    callee,
                    arguments,
                    operation.output,
                    instructions,
                    max_stack,
                    parameter_slots,
                    local_slots,
                    local_types,
                    value_types,
                    eval_stack,
                    label.take(),
                ) {
                    return;
                }
                if let Some(op_kind) = try_intrinsic_call(callee) {
                    lower_intrinsic_call(
                        op_kind,
                        arguments,
                        operation.output,
                        instructions,
                        max_stack,
                        parameter_slots,
                        local_slots,
                        local_types,
                        value_types,
                        eval_stack,
                        label.take(),
                    );
                    return;
                }
            }

            if is_string_equality {
                // 字符串相等比较：调用 `System.String::op_Equality(string, string): bool`。
                lower_string_equality_call(
                    arguments,
                    operation.output,
                    instructions,
                    max_stack,
                    parameter_slots,
                    local_slots,
                    local_types,
                    value_types,
                    eval_stack,
                    label.take(),
                );
            }
            else {
                // 常规 call：先溢出 eval_stack，再逐个压入参数。
                spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
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
                        // 从 return_types 映射推断 callee 的返回类型，用于值类型追踪。
                        let return_msil_type = if let LirOperand::Symbol(path) = callee {
                            return_types
                                .get(&path.to_string())
                                .map(|ty| map_hir_type_to_msil_for_lowering(ty, qualified_name_map))
                                .unwrap_or(MsilType::Object)
                        }
                        else {
                            MsilType::Object
                        };
                        value_types.insert(output, return_msil_type);
                        eval_stack.push(output);
                    }
                }
            }
        }
        LirOperationKind::Subscript { object, index } => {
            // 数组下标访问：先加载 object（数组），再加载 index，emit ldelem。
            // CLR 求值栈顺序：array, index → ldelem → value
            // 先溢出 eval_stack，确保 object 和 index 按顺序压栈。
            spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

            // 推断数组元素类型，选择正确的 ldelem 指令。
            let array_type = infer_operand_type(object, value_types);
            let (load_opcode, result_type) = match &array_type {
                MsilType::SzArray(elem) if **elem == MsilType::UInt8 => (MsilOpcode::LdelemU1, MsilType::UInt8),
                MsilType::SzArray(elem) => (MsilOpcode::LdelemRef, (**elem).clone()),
                _ => (MsilOpcode::LdelemRef, MsilType::Object),
            };

            // 加载数组对象到求值栈。
            lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            // 加载索引到求值栈。
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

            // emit ldelem：从数组中加载元素。
            instructions.push(MsilInstruction { label: None, opcode: load_opcode, operand: None });
            *max_stack = (*max_stack).max(2);

            // ldelem 消费了数组（eval_stack 顶部的 Value）和索引。
            if let LirOperand::Value(_) = object {
                eval_stack.pop();
            }

            // 结果压入 eval_stack。
            if let Some(output) = operation.output {
                value_types.insert(output, result_type);
                eval_stack.push(output);
            }
        }
        LirOperationKind::StoreSubscript { object, index, value } => {
            // 数组元素存储：`array[index] = value`。
            // CLR 求值栈顺序：array, index, value → stelem → (无返回值)
            // 先溢出 eval_stack，确保 object、index、value 按顺序压栈。
            spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

            // 推断数组元素类型，选择正确的 stelem 指令。
            let array_type = infer_operand_type(object, value_types);
            let store_opcode = match &array_type {
                MsilType::SzArray(elem) if **elem == MsilType::UInt8 => MsilOpcode::StelemI1,
                _ => MsilOpcode::StelemRef,
            };

            // 加载数组对象到求值栈。
            lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            // 加载索引到求值栈。
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

            // 加载要存储的值到求值栈。
            match value {
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

            // emit stelem：将值存入数组指定位置。
            // 消费 array、index、value，无返回值。
            instructions.push(MsilInstruction { label: None, opcode: store_opcode, operand: None });
            *max_stack = (*max_stack).max(3);

            // stelem 消费了数组（eval_stack 顶部的 Value）。
            if let LirOperand::Value(_) = object {
                eval_stack.pop();
            }
        }
        LirOperationKind::StructNew { type_name, fields } => {
            // 结构体构造：将所有字段值压栈，然后 emit newobj。
            spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

            // 按字段顺序加载所有字段值到求值栈。
            for (_, value) in fields {
                lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            }

            // 构造函数签名：参数类型 = 字段类型列表。
            let param_types: Vec<MsilType> = fields.iter().map(|(_, v)| infer_operand_type(v, value_types)).collect();

            // 将简单类型名解析为命名空间限定名，确保 newobj 的 owner 与 TypeDef 匹配。
            let qualified_type_name = qualified_name_map.get(type_name.as_str()).cloned().unwrap_or_else(|| type_name.clone());

            // emit newobj：消费所有参数，产生新对象。
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Newobj,
                operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                    owner: Some(qualified_type_name.clone()),
                    name: ".ctor".to_string(),
                    signature: MsilMethodSignature::new(MsilType::Void, param_types),
                })),
            });
            *max_stack = (*max_stack).max(fields.len() as u16 + 1);

            // newobj 消费了所有字段值，从 eval_stack 中弹出。
            for _ in 0..fields.len() {
                eval_stack.pop();
            }

            // newobj 在栈上留下新对象。
            if let Some(output) = operation.output {
                eval_stack.push(output);
                value_types.insert(output, MsilType::Named(qualified_type_name));
            }
        }
        LirOperationKind::FieldGet { object, field } => {
            // 字段读取：加载对象，emit ldfld。
            spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

            lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            // 推断对象类型以构建字段引用。
            let object_type = infer_operand_type(object, value_types);

            // 特殊处理：字符串的 `length` 字段在 CLR 中是属性，
            // 通过 `System.String::get_Length()` 方法访问，而非 `ldfld`。
            if object_type == MsilType::String && field == "length" {
                instructions.push(MsilInstruction {
                    label: None,
                    opcode: MsilOpcode::Call,
                    operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                        owner: Some("[mscorlib]System.String".to_string()),
                        name: "get_Length".to_string(),
                        signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()),
                    })),
                });
                *max_stack = (*max_stack).max(1);

                // get_Length 在栈上留下 int32。
                if let Some(output) = operation.output {
                    eval_stack.push(output);
                    value_types.insert(output, MsilType::Int32);
                }
            }
            else {
                let type_name = match object_type {
                    MsilType::Named(name) => name,
                    _ => String::new(),
                };

                instructions.push(MsilInstruction {
                    label: None,
                    opcode: MsilOpcode::Ldfld,
                    operand: Some(MsilInstructionOperand::Field(type_name.clone(), field.clone())),
                });
                *max_stack = (*max_stack).max(1);

                // ldfld 在栈上留下字段值。
                if let Some(output) = operation.output {
                    eval_stack.push(output);
                    // 查找字段类型并记录，确保后续字段访问能正确推断对象类型。
                    if let Some(field_ty) = field_types.get(&(type_name, field.clone())) {
                        value_types.insert(output, map_hir_type_to_msil_for_lowering(field_ty, qualified_name_map));
                    }
                }
            }
        }
        LirOperationKind::FieldSet { object, field, value } => {
            // 字段写入：加载对象和值，emit stfld。
            spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

            lower_operand(object, instructions, max_stack, parameter_slots, local_slots, eval_stack);
            lower_operand(value, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            let type_name = match infer_operand_type(object, value_types) {
                MsilType::Named(name) => name,
                _ => String::new(),
            };

            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Stfld,
                operand: Some(MsilInstructionOperand::Field(type_name, field.clone())),
            });
            *max_stack = (*max_stack).max(2);

            // stfld 消费对象和值。
            if let LirOperand::Value(_) = object {
                eval_stack.pop();
            }
        }
        LirOperationKind::ArrayNew { element_type, length } => {
            // 数组创建：加载长度到栈，emit newarr。
            // CLR 求值栈顺序：length → newarr → (留下新数组)
            spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

            // 加载数组长度到求值栈。
            lower_operand(length, instructions, max_stack, parameter_slots, local_slots, eval_stack);

            // 将 HirType 映射为 CLR 类型名，供 PE writer 解析 TypeRef token。
            let clr_type_name = map_hir_type_to_clr_type_name(element_type);

            // emit newarr：消费长度，产生新数组。
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Newarr,
                operand: Some(MsilInstructionOperand::Type(clr_type_name)),
            });
            *max_stack = (*max_stack).max(2);

            // newarr 消费了长度，从 eval_stack 中弹出。
            if let LirOperand::Value(_) = length {
                eval_stack.pop();
            }

            // newarr 在栈上留下新数组。
            if let Some(output) = operation.output {
                eval_stack.push(output);
                let array_elem = map_hir_type_to_msil_for_lowering(element_type, qualified_name_map);
                value_types.insert(output, MsilType::sz_array(array_elem));
            }
        }
    }

    // 将 label 放在第一条 emitted 指令上，确保分支目标指向块的第一条指令。
    if let Some(lbl) = label {
        if let Some(anchor) = anchor {
            if anchor < instructions.len() {
                instructions[anchor].label = Some(lbl.clone());
                eprintln!("  [DEBUG] applied label '{lbl}' at instruction index {anchor}");
            }
            else {
                eprintln!("  [DEBUG] anchor {anchor} >= instructions.len() {} for label '{lbl}'", instructions.len());
            }
        }
        else {
            eprintln!("  [DEBUG] anchor is None for label '{lbl}'");
        }
    }
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

/// 将求值栈中的所有值溢出到局部变量。
///
/// 当需要访问非栈顶的值时，MSIL 无法直接索引栈中位置，
/// 必须先弹出上方所有值（存入局部变量），再加载目标值。
fn spill_eval_stack(
    eval_stack: &mut Vec<MirValueRef>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    value_types: &BTreeMap<MirValueRef, MsilType>,
    instructions: &mut Vec<MsilInstruction>,
) {
    while let Some(value) = eval_stack.pop() {
        let slot = local_types.len();
        // 从 value_types 映射查找值的类型，默认回退为 int32。
        let ty = value_types.get(&value).cloned().unwrap_or(MsilType::Int32);
        local_types.push(ty);
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
    value_types: &BTreeMap<MirValueRef, MsilType>,
    eval_stack: &mut Vec<MirValueRef>,
    label: Option<String>,
) {
    // 记录 anchor：label 必须放在本操作 emit 的第一条指令上，
    // 确保分支目标指向块的第一条指令（而非最后一条，如 intrinsic opcode）。
    let anchor = label.as_ref().map(|_| instructions.len());
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
            emit_intrinsic_call(op_kind, None, instructions, max_stack, arg_count, eval_stack, output);
            apply_anchor_label(label, anchor, instructions);
            return;
        }
    }

    // 慢速路径：先溢出 eval_stack 到局部变量，确保所有 Value 都可通过 local_slots 访问。
    spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);

    // 逐个加载操作数到求值栈。
    for argument in arguments {
        lower_operand(argument, instructions, max_stack, parameter_slots, local_slots, eval_stack);
    }

    emit_intrinsic_call(op_kind, None, instructions, max_stack, arg_count, eval_stack, output);
    apply_anchor_label(label, anchor, instructions);
}

/// 将 label 放在 anchor 位置的第一条 emitted 指令上。
fn apply_anchor_label(label: Option<String>, anchor: Option<usize>, instructions: &mut [MsilInstruction]) {
    if let Some(lbl) = label {
        if let Some(anchor) = anchor {
            if anchor < instructions.len() {
                eprintln!("  [DEBUG-ANCHOR] applied label '{lbl}' at anchor {anchor}");
                instructions[anchor].label = Some(lbl);
            } else {
                eprintln!("  [DEBUG-ANCHOR] anchor {anchor} >= len {} for label '{lbl}'", instructions.len());
            }
        } else {
            eprintln!("  [DEBUG-ANCHOR] anchor is None for label '{lbl}'");
        }
    }
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

/// 推断常量的 `MSIL` 类型。
fn infer_constant_type(constant: &MirConstant) -> MsilType {
    match constant {
        MirConstant::Int(_) => MsilType::Int32,
        MirConstant::Bool(_) => MsilType::Bool,
        MirConstant::String(_) => MsilType::String,
        MirConstant::Unit => MsilType::Object,
    }
}

/// 推断操作数的 `MSIL` 类型。
///
/// 对于 `Value`，从 `value_types` 映射中查找；
/// 对于 `Constant`，从常量值推断；
/// 对于 `Symbol`，默认为 `object`。
fn infer_operand_type(operand: &LirOperand, value_types: &BTreeMap<MirValueRef, MsilType>) -> MsilType {
    match operand {
        LirOperand::Value(v) => value_types.get(v).cloned().unwrap_or(MsilType::Int32),
        LirOperand::Constant(c) => infer_constant_type(c),
        LirOperand::Symbol(_) => MsilType::Object,
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

    // 兜底：值未在任何槽位中，说明值已在求值栈上（由前一条指令产生），无需发射额外指令。
    // 如果此处发射 ldloc.0 等占位指令，会导致求值栈多出一个错误值，产生 InvalidProgramException。
}

fn lower_terminator(
    terminator: &LirTerminator,
    instructions: &mut Vec<MsilInstruction>,
    max_stack: &mut u16,
    parameter_slots: &BTreeMap<MirValueRef, usize>,
    local_slots: &mut BTreeMap<MirValueRef, usize>,
    local_types: &mut Vec<MsilType>,
    value_types: &BTreeMap<MirValueRef, MsilType>,
    _var_slots: &mut BTreeMap<String, usize>,
    eval_stack: &mut Vec<MirValueRef>,
    block_labels: &BTreeMap<MirBlockRef, String>,
) {
    match terminator {
        LirTerminator::Return { value } => {
            if let Some(value_operand) = value {
                let on_top = match value_operand {
                    LirOperand::Value(v) => eval_stack.last() == Some(v),
                    _ => false,
                };
                if !on_top {
                    // 若值不在栈顶，先溢出 eval_stack 再加载。
                    spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
                    lower_operand(value_operand, instructions, max_stack, parameter_slots, local_slots, eval_stack);
                }
            }
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
        }
        LirTerminator::Jump { target, .. } => {
            let target_label = block_labels.get(target).cloned().unwrap_or_else(|| format!("BB{}", target.0));
            instructions.push(MsilInstruction {
                label: None,
                opcode: MsilOpcode::Br,
                operand: Some(MsilInstructionOperand::BranchTarget(target_label)),
            });
        }
        LirTerminator::Branch { condition, then_target, else_target, .. } => {
            // 若条件不在栈顶，先溢出 eval_stack 再加载。
            let on_top = match condition {
                LirOperand::Value(v) => eval_stack.last() == Some(v),
                _ => false,
            };
            if !on_top {
                spill_eval_stack(eval_stack, local_slots, local_types, value_types, instructions);
            }
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
        }
        LirTerminator::Unreachable => {
            instructions.push(MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None });
        }
    }
}

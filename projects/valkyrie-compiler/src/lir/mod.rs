#![doc = include_str!("readme.md")]

use std::collections::BTreeMap;

use crate::mir::{
    MirBlock, MirBlockRef, MirConstant, MirDispatchKind, MirField, MirFunction, MirInstruction, MirInstructionKind, MirLowerer, MirModule,
    MirOperand, MirStruct, MirTerminator, MirValueRef,
};
use valkyrie_types::{
    hir::{HirModule, HirType},
    NamePath,
};

/// Target-aware low-level representation module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirModule {
    /// Logical module name.
    pub name: String,
    /// Target lowering lane selected for this low-level view.
    pub lane: LirTargetLane,
    /// Functions lowered into the selected lane.
    pub functions: Vec<LirFunction>,
    /// 结构体类型定义，供后端生成 `TypeDef`/`Field` 表。
    pub structs: Vec<MirStruct>,
    /// 外部函数导入声明，供 `WASM` 等后端生成 `Import` 段。
    pub imports: Vec<LirImport>,
    /// `using` 导入的模块路径列表，供后端进行跨模块名称解析。
    pub module_imports: Vec<String>,
}

/// 外部函数导入声明。
///
/// 描述一个从宿主环境导入的函数，例如 `WASM` 的 `import` 段条目
/// 或 `CLR` 的外部方法引用。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirImport {
    /// 导入模块名（如 `"env"`、`"wasi_snapshot_preview1"`）。
    pub module: String,
    /// 导入字段名（如 `"fd_write"`、`"read_file"`）。
    pub field: String,
    /// 函数符号名，用于在 `Call` 指令中匹配 `callee`。
    pub symbol: String,
    /// 参数类型列表。
    pub param_types: Vec<HirType>,
    /// 返回类型。
    pub return_type: HirType,
}

/// Low-level function body grouped by basic blocks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LirFunction {
    /// Symbol name of the lowered function.
    pub symbol: String,
    /// 函数返回类型，用于后端判断调用是否返回 `void`。
    pub return_type: HirType,
    /// 函数参数类型列表，用于后端生成方法签名。
    pub param_types: Vec<HirType>,
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

/// Concrete low-level operation family.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LirOperationKind {
    /// Materializes an immediate constant.
    LoadConstant {
        /// Constant payload.
        constant: MirConstant,
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
    /// `ty` 是变量声明时的类型注解，用于类型推断（如空数组字面量的元素类型）。
    /// 同名 `StoreVar` 复用同一局部槽位，确保循环中 header 能读到最新值。
    StoreVar {
        /// 变量名。
        name: String,
        /// 要存储的值。
        value: LirOperand,
        /// 变量声明类型注解，来自 `let` 语句的 `ty` 字段。
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
        /// Optional witness operand.
        witness: Option<LirOperand>,
        /// Optional effect operand.
        effect: Option<LirOperand>,
    },
    /// 下标访问操作。
    Subscript {
        /// 被索引的对象。
        object: LirOperand,
        /// 索引表达式。
        index: LirOperand,
    },
    /// 数组元素赋值操作：`object[index] = value`。
    StoreSubscript {
        /// 被写入的数组对象。
        object: LirOperand,
        /// 索引表达式。
        index: LirOperand,
        /// 要写入的值。
        value: LirOperand,
    },
    /// 结构体构造：`TypeName { field1: value1, ... }`。
    StructNew {
        /// 结构体类型名。
        type_name: String,
        /// 字段初始化列表：(字段名, 字段值)。
        fields: Vec<(String, LirOperand)>,
    },
    /// 字段读取：`object.field`。
    FieldGet {
        /// 被访问字段的对象。
        object: LirOperand,
        /// 字段名。
        field: String,
    },
    /// 字段写入：`object.field = value`。
    FieldSet {
        /// 被写入字段的对象。
        object: LirOperand,
        /// 字段名。
        field: String,
        /// 要写入的值。
        value: LirOperand,
    },
    /// 数组创建：`new [ElementType](length)`。
    ///
    /// 创建一个指定元素类型和长度的一维零基数组。
    /// 输出是新创建的数组实例。
    ArrayNew {
        /// 数组元素类型。
        element_type: HirType,
        /// 数组长度。
        length: LirOperand,
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
        functions: module.functions.iter().map(|function| lower_mir_function(function, return_types)).collect(),
        structs: module.structs.clone(),
        imports: Vec::new(),
        module_imports: module.imports.clone(),
    }
}

fn lower_mir_function(function: &MirFunction, return_types: &BTreeMap<&str, HirType>) -> LirFunction {
    let return_type = return_types.get(function.symbol.as_str()).cloned().unwrap_or(HirType::Unit);
    LirFunction {
        symbol: function.symbol.clone(),
        return_type,
        param_types: function.param_types.clone(),
        entry: function.entry,
        blocks: function.blocks.iter().map(lower_mir_block).collect(),
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
        MirTerminator::Unreachable => LirTerminator::Unreachable,
    };

    LirBlock { id: block.id, label: block.label.clone(), parameters: block.parameters.clone(), operations, terminator }
}

fn lower_mir_instruction(instruction: &MirInstruction) -> LirOperation {
    let kind = match &instruction.kind {
        MirInstructionKind::LoadConstant { constant } => LirOperationKind::LoadConstant { constant: constant.clone() },
        MirInstructionKind::LoadSymbol { path } => LirOperationKind::LoadSymbol { path: path.clone() },
        MirInstructionKind::Copy { source } => LirOperationKind::Move { source: lower_mir_operand(source.clone()) },
        MirInstructionKind::StoreVar { name, value, ty } => {
            LirOperationKind::StoreVar { name: name.clone(), value: lower_mir_operand(value.clone()), ty: ty.clone() }
        }
        MirInstructionKind::Call { dispatch, callee, arguments, witness, effect } => LirOperationKind::Call {
            dispatch: lower_dispatch_kind(*dispatch),
            callee: lower_mir_operand(callee.clone()),
            arguments: arguments.iter().cloned().map(lower_mir_operand).collect(),
            witness: witness.clone().map(lower_mir_operand),
            effect: effect.clone().map(lower_mir_operand),
        },
        MirInstructionKind::Subscript { object, index } => {
            LirOperationKind::Subscript { object: lower_mir_operand(object.clone()), index: lower_mir_operand(index.clone()) }
        }
        MirInstructionKind::StoreSubscript { object, index, value } => LirOperationKind::StoreSubscript {
            object: lower_mir_operand(object.clone()),
            index: lower_mir_operand(index.clone()),
            value: lower_mir_operand(value.clone()),
        },
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
        MirInstructionKind::ArrayNew { element_type, length } => {
            LirOperationKind::ArrayNew { element_type: element_type.clone(), length: lower_mir_operand(length.clone()) }
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

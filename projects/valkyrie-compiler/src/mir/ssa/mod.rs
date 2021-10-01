#![allow(missing_docs)]

use std::collections::BTreeMap;

use valkyrie_types::{
    hir::{HirExpr, HirExprKind, HirFunction, HirLiteral, HirMatchArm, HirModule, HirPattern, HirStatement, HirStatementKind, ValkyrieType},
    Identifier, NamePath,
};

mod control_flow_lowering;
mod effect_lowering;
mod frame_planning;
mod match_lowering;
mod pattern_lowering;
mod suspend_analysis;
/// `MIR` 降低测试辅助入口。
pub mod test_support;

/// `SSA` 形式的 `MIR` 模块。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirModule {
    pub name: String,
    pub functions: Vec<MirFunction>,
    /// 模块中定义的结构体类型，供后端生成 `TypeDef`/`Field` 表。
    pub structs: Vec<MirStruct>,
    /// `using` 导入的模块路径列表，供后端进行跨模块名称解析。
    pub imports: Vec<String>,
}

/// `MIR` 结构体类型定义。
///
/// 携带后端生成 `TypeDef`/`Field` 表所需的最小信息：
/// 类型名、字段列表、是否值类型。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirStruct {
    /// 类型名。
    pub name: String,
    /// 所属命名空间（点分隔，如 `core.text`，空串表示全局命名空间）。
    pub namespace: String,
    /// 字段列表。
    pub fields: Vec<MirField>,
    /// 是否为值类型（`structure` 关键字声明）。
    /// `false` 表示引用类型（`class` 关键字声明）。
    pub is_value_type: bool,
}

/// `MIR` 结构体字段定义。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirField {
    /// 字段名。
    pub name: String,
    /// 字段类型。
    pub ty: ValkyrieType,
}

/// `SSA` 形式的 `MIR` 函数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFunction {
    pub symbol: String,
    /// 函数返回类型，用于后端判断调用是否返回 `void`。
    pub return_type: ValkyrieType,
    /// 函数参数类型列表，用于后端生成方法签名。
    pub param_types: Vec<ValkyrieType>,
    /// `SSA` 值的静态类型表，供调度校验与后续 lowering 使用。
    pub value_types: BTreeMap<MirValueRef, ValkyrieType>,
    pub entry: MirBlockRef,
    pub values: Vec<MirValue>,
    /// 显式 suspend 点元数据，为后续 frame / spill / state lowering 提供稳定入口。
    pub suspend_points: Vec<MirSuspendPoint>,
    /// 显式 frame layout 计划，供 lane/runtime lowering 直接消费。
    pub frame_layouts: Vec<MirFrameLayout>,
    /// 显式 continuation 元数据，供 handler/runtime lowering 与跨层校验使用。
    pub continuations: Vec<MirContinuation>,
    /// 显式 case/match 链路元数据，供跨 arm merge 与 fallthrough 校验使用。
    pub case_chains: Vec<MirCaseChain>,
    pub blocks: Vec<MirBlock>,
}

/// `MIR` 基本块。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirBlock {
    pub id: MirBlockRef,
    pub label: String,
    pub parameters: Vec<MirValueRef>,
    pub instructions: Vec<MirInstruction>,
    pub terminator: MirTerminator,
}

/// `MIR` 值槽。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirValue {
    pub id: MirValueRef,
    pub origin: MirValueOrigin,
}

/// `catch / resume` 等可恢复控制流的 continuation 元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirContinuation {
    /// 当前 continuation 对应的 handler dispatch block。
    pub dispatch_block: MirBlockRef,
    /// continuation 恢复时跳回的 resume block。
    pub resume_target: MirBlockRef,
    /// resume block 的显式参数位。
    pub resume_parameter: MirValueRef,
    /// 当前已知的恢复值类型。
    pub resume_parameter_type: Option<ValkyrieType>,
    /// handler 完成后汇入的 exit block。
    pub handler_exit: MirBlockRef,
}

/// `match / case` 链路的显式 merge 元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirCaseChain {
    /// 发起 case-like lowering 的分发块。
    pub dispatch_block: MirBlockRef,
    /// 第一个 arm 的入口块。
    pub first_arm: MirBlockRef,
    /// 全部 arm 都未匹配时跳入的块。
    pub no_match_block: MirBlockRef,
    /// case-like 控制流最终汇入的 exit 块。
    pub exit_block: MirBlockRef,
    /// 是否为值语义 `match`。
    pub produce_value: bool,
    /// 每个 arm 的显式链路信息。
    pub arms: Vec<MirCaseArm>,
}

/// 单个 `case / match arm` 的显式控制流元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirCaseArm {
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

/// `yield / await / block / raise` 等 suspend 点元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirSuspendPoint {
    /// 状态机 lowering 使用的显式状态编号。
    pub state_id: u32,
    /// 触发的 effect 类型。
    pub effect: MirEffectKind,
    /// 发生挂起的 block。
    pub suspend_block: MirBlockRef,
    /// 恢复时跳回的 block。
    pub resume_target: MirBlockRef,
    /// 恢复点参数个数。
    pub resume_parameter_count: usize,
    /// 当前已知 payload 的静态类型。
    pub payload_type: Option<ValkyrieType>,
    /// 挂起时需要后续做 spill 分析的候选 SSA 值。
    pub spill_candidates: Vec<MirValueRef>,
}

/// 单个 suspend 状态对应的 frame layout 计划。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFrameLayout {
    /// 对应 suspend 点的显式状态编号。
    pub state_id: u32,
    /// 该状态对应的 effect 类型。
    pub effect: MirEffectKind,
    /// 恢复后跳回的目标 block。
    pub resume_target: MirBlockRef,
    /// 需要保存到 frame 的槽位布局。
    pub slots: Vec<MirFrameSlot>,
}

/// frame 中的单个 spill 槽位。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFrameSlot {
    /// frame 中的稳定槽位序号。
    pub slot_index: usize,
    /// 被保存的 SSA 值。
    pub value: MirValueRef,
    /// 当前已知的槽位静态类型。
    pub value_type: Option<ValkyrieType>,
}

/// `MIR` 指令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirInstruction {
    pub output: Option<MirValueRef>,
    pub kind: MirInstructionKind,
}

/// `MIR` 值引用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirValueRef(pub u32);

/// `MIR` 基本块引用。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirBlockRef(pub u32);

/// `MIR` 值来源。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirValueOrigin {
    Parameter { index: usize, name: String },
    BlockParameter { block: MirBlockRef, name: String },
    LetBinding { name: String },
    Literal,
    Path,
    CallResult,
    Temporary,
}

/// `MIR` 操作数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirOperand {
    Value(MirValueRef),
    Constant(MirConstant),
    Symbol(NamePath),
}

/// `MIR` 常量。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirConstant {
    Int(i64),
    Float64(ordered_float::OrderedFloat<f64>),
    Bool(bool),
    String(String),
    Unit,
}

/// `MIR` 调用分发种类。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirDispatchKind {
    Static,
    Witness,
    EffectHandler,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBuiltinBinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBuiltinCompareOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirBuiltinCall {
    BinaryNumeric(MirBuiltinBinaryOp),
    Compare(MirBuiltinCompareOp),
    NumericNeg,
    LogicalAnd,
    LogicalOr,
    LogicalNot,
    ArrayGet,
    ArraySet,
    ArrayLength,
    Identity,
}

/// `MIR` effect terminator category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirEffectKind {
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

/// `MIR` 指令种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirInstructionKind {
    LoadConstant {
        constant: MirConstant,
        ty: Option<ValkyrieType>,
    },
    LoadSymbol {
        path: NamePath,
    },
    Copy {
        source: MirOperand,
    },
    /// 存储到命名变量槽位（可变变量）。
    ///
    /// `name` 是变量名，`value` 是要存储的值。
    /// `ty` 是变量声明时的类型注解，用于类型推断（如空数组字面量的元素类型）。
    /// 同名 `StoreVar` 复用同一局部槽位，确保循环中 header 能读到最新值。
    StoreVar {
        name: String,
        value: MirOperand,
        /// 变量声明类型注解，来自 `let` 语句的 `ty` 字段。
        ty: Option<ValkyrieType>,
    },
    Call {
        dispatch: MirDispatchKind,
        callee: MirOperand,
        arguments: Vec<MirOperand>,
        builtin: Option<MirBuiltinCall>,
        witness: Option<MirOperand>,
        effect: Option<MirOperand>,
    },
    /// 结构体构造：`TypeName { field1: value1, field2: value2, ... }`。
    ///
    /// `type_name` 是结构体类型名。
    /// `fields` 是字段名与字段值的有序列表。
    /// 输出是新构造的结构体实例。
    StructNew {
        /// 结构体类型名。
        type_name: String,
        /// 字段初始化列表：(字段名, 字段值)。
        fields: Vec<(String, MirOperand)>,
    },
    /// 字段读取：`object.field`。
    ///
    /// 读取 `object` 对象的 `field` 字段值。
    /// 输出是字段值。
    FieldGet {
        /// 被访问字段的对象。
        object: MirOperand,
        /// 字段名。
        field: String,
    },
    /// 字段写入：`object.field = value`。
    ///
    /// 将 `value` 写入 `object` 对象的 `field` 字段。
    /// 无输出值（结果为 `Unit`）。
    FieldSet {
        /// 被写入字段的对象。
        object: MirOperand,
        /// 字段名。
        field: String,
        /// 要写入的值。
        value: MirOperand,
    },
    /// 显式保留模式判定语义，供 handler/case dispatch 使用。
    PatternMatch {
        /// 被判定的输入值。
        value: MirOperand,
        /// 需要匹配的源级模式。
        pattern: HirPattern,
    },
    /// 数组创建：`new [ElementType](length)`。
    ///
    /// 创建一个指定元素类型和长度的一维零基数组。
    /// 输出是新创建的数组实例。
    ArrayNew {
        /// 数组元素类型。
        element_type: ValkyrieType,
        /// 数组长度。
        length: MirOperand,
    },
    /// 数组字面量构造。
    ///
    /// 这是数组构造自身的内建语义，不借用通用 `[]=` 语法糖。
    ArrayLiteral {
        /// 数组元素类型。
        element_type: ValkyrieType,
        /// 元素列表。
        items: Vec<MirOperand>,
    },
}

/// `MIR` 终结符。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirTerminator {
    Return { value: Option<MirOperand> },
    Jump { target: MirBlockRef, arguments: Vec<MirOperand> },
    Branch { condition: MirOperand, then_target: MirBlockRef, else_target: MirBlockRef },
    PerformEffect { effect: MirEffectKind, payload: Option<MirOperand>, resume_target: MirBlockRef },
    Unreachable,
}

pub struct MirLowerer;

impl MirLowerer {
    pub fn lower_module(module: &HirModule) -> MirModule {
        let return_types = collect_module_return_types(&module.functions);
        let struct_field_layouts = collect_struct_field_layouts(&module.structs);
        let struct_parent_index = collect_struct_parent_index(&module.structs);
        let structs = module.structs.iter().map(lower_struct).collect();
        let imports = module.imports.iter().map(|path| path.to_string()).collect();
        MirModule {
            name: module.name.to_string(),
            functions: module
                .functions
                .iter()
                .map(|function| lower_function(function, &return_types, &struct_field_layouts, &struct_parent_index))
                .collect(),
            structs,
            imports,
        }
    }
}

/// 将 `HirStruct` 降级为 `MirStruct`，提取后端生成 `TypeDef`/`Field` 表所需的最小信息。
fn lower_struct(hir_struct: &valkyrie_types::hir::HirStruct) -> MirStruct {
    let fields = hir_struct.fields.iter().map(|field| MirField { name: field.name.to_string(), ty: field.ty.clone() }).collect();
    let namespace = hir_struct.namespace.iter().map(|part| part.as_str().to_string()).collect::<Vec<_>>().join(".");
    MirStruct { name: hir_struct.name.to_string(), namespace, fields, is_value_type: hir_struct.is_value_type }
}

fn collect_module_return_types(functions: &[HirFunction]) -> BTreeMap<String, ValkyrieType> {
    functions.iter().map(|function| (function.name.to_string(), function.return_type.clone())).collect()
}

fn collect_struct_field_layouts(structs: &[valkyrie_types::hir::HirStruct]) -> BTreeMap<String, Vec<(String, ValkyrieType)>> {
    structs
        .iter()
        .map(|hir_struct| {
            (hir_struct.name.to_string(), hir_struct.fields.iter().map(|field| (field.name.to_string(), field.ty.clone())).collect())
        })
        .collect()
}

fn collect_struct_parent_index(structs: &[valkyrie_types::hir::HirStruct]) -> BTreeMap<String, Vec<String>> {
    structs
        .iter()
        .map(|hir_struct| {
            (
                hir_struct.name.to_string(),
                hir_struct.parents.iter().filter_map(|parent| parent.name.parts().last().map(|name| name.to_string())).collect(),
            )
        })
        .collect()
}

fn lower_function(
    function: &HirFunction,
    return_types: &BTreeMap<String, ValkyrieType>,
    struct_field_layouts: &BTreeMap<String, Vec<(String, ValkyrieType)>>,
    struct_parent_index: &BTreeMap<String, Vec<String>>,
) -> MirFunction {
    let mut builder = MirBuilder::new(return_types.clone(), struct_field_layouts.clone(), struct_parent_index.clone());

    // 绑定参数，并收集参数值用于入口块的 parameters 字段。
    let mut param_values = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        let value = builder.next_value(MirValueOrigin::Parameter { index, name: param.name.name.to_string() });
        builder.bindings.insert(param.name.name.to_string(), MirOperand::Value(value));
        builder.value_types.insert(value, param.ty.clone());
        param_values.push(value);
    }

    // 将函数参数设置到入口块的 parameters 字段，
    // 使 CLR 后端能通过 collect_parameter_slots 识别参数槽位（ldarg）。
    builder.blocks[0].parameters = param_values;

    // 处理函数体内的语句
    for statement in &function.body.statements {
        builder.lower_statement(statement);
        // 若某条语句触发了 return（或其它终结操作），后续语句均为不可达代码，
        // 不再降低以避免覆盖已设置的终结符。
        if builder.terminator.is_some() {
            break;
        }
    }

    // 处理尾表达式
    if builder.terminator.is_none() {
        if let Some(expr) = &function.body.expr {
            let operand = builder.lower_expr_to_operand(expr);
            // 如果尾表达式是 return，terminator 已经被设置
            if builder.terminator.is_none() {
                builder.terminate(MirTerminator::Return { value: Some(operand) });
            }
        }
        else {
            builder.terminate(MirTerminator::Return { value: None });
        }
    }

    // 将当前块刷新到 blocks
    let current_label = builder.current_label.clone();
    builder.flush_block(&current_label);

    let param_types = function.params.iter().map(|p| p.ty.clone()).collect();
    let mut mir_function = MirFunction {
        symbol: function.name.to_string(),
        return_type: function.return_type.clone(),
        param_types,
        value_types: builder.value_types,
        entry: builder.entry,
        values: builder.values,
        suspend_points: builder.suspend_points,
        frame_layouts: Vec::new(),
        continuations: builder.continuations,
        case_chains: builder.case_chains,
        blocks: builder.blocks,
    };
    suspend_analysis::analyze_suspend_points(&mut mir_function);
    frame_planning::plan_frame_layouts(&mut mir_function);
    mir_function
}

/// MIR 构建器：管理基本块创建和控制流。
struct MirBuilder {
    entry: MirBlockRef,
    current_block: MirBlockRef,
    current_label: String,
    values: Vec<MirValue>,
    instructions: Vec<MirInstruction>,
    blocks: Vec<MirBlock>,
    suspend_points: Vec<MirSuspendPoint>,
    continuations: Vec<MirContinuation>,
    case_chains: Vec<MirCaseChain>,
    bindings: BTreeMap<String, MirOperand>,
    value_types: BTreeMap<MirValueRef, ValkyrieType>,
    return_types: BTreeMap<String, ValkyrieType>,
    struct_field_layouts: BTreeMap<String, Vec<(String, ValkyrieType)>>,
    struct_parent_index: BTreeMap<String, Vec<String>>,
    static_bindings: BTreeMap<String, HirExpr>,
    terminator: Option<MirTerminator>,
    value_seed: u32,
    state_seed: u32,
    /// 循环上下文栈。
    /// `break` 跳转到 exit，`continue` 跳转到 header。
    loop_stack: Vec<LoopContext>,
    /// `catch` handler 上下文栈。
    handler_stack: Vec<HandlerContext>,
    /// `resume` continuation 上下文栈。
    resume_stack: Vec<ResumeContext>,
    /// 当前正在执行的 handler arm 层数。
    /// arm 内部再次 `raise` 时需要跳过本层 handler，避免递归回灌到同一个 handler。
    suspended_handler_depth: usize,
    /// `match` arm 内 `fallthrough` 的目标栈。
    fallthrough_stack: Vec<FallthroughContext>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LoopContext {
    label: Option<String>,
    header: MirBlockRef,
    exit: MirBlockRef,
    exit_value: Option<MirValueRef>,
    carried_values: Vec<String>,
    carried_value_refs: BTreeMap<String, MirValueRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct HandlerContext {
    arms: Vec<HirMatchArm>,
    exit: MirBlockRef,
    exit_value: Option<MirValueRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResumeContext {
    continuation: usize,
    target: MirBlockRef,
    parameter: MirValueRef,
    parameter_name: &'static str,
    parameter_type: Option<ValkyrieType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FallthroughContext {
    target: MirBlockRef,
}

impl MirBuilder {
    fn new(
        return_types: BTreeMap<String, ValkyrieType>,
        struct_field_layouts: BTreeMap<String, Vec<(String, ValkyrieType)>>,
        struct_parent_index: BTreeMap<String, Vec<String>>,
    ) -> Self {
        let entry = MirBlockRef(0);
        Self {
            entry,
            current_block: entry,
            current_label: "entry".to_string(),
            values: Vec::new(),
            instructions: Vec::new(),
            blocks: vec![MirBlock {
                id: entry,
                label: "entry".to_string(),
                parameters: Vec::new(),
                instructions: Vec::new(),
                terminator: MirTerminator::Unreachable,
            }],
            suspend_points: Vec::new(),
            continuations: Vec::new(),
            case_chains: Vec::new(),
            bindings: BTreeMap::new(),
            value_types: BTreeMap::new(),
            return_types,
            struct_field_layouts,
            struct_parent_index,
            static_bindings: BTreeMap::new(),
            terminator: None,
            value_seed: 0,
            state_seed: 0,
            loop_stack: Vec::new(),
            handler_stack: Vec::new(),
            resume_stack: Vec::new(),
            suspended_handler_depth: 0,
            fallthrough_stack: Vec::new(),
        }
    }

    fn next_value(&mut self, origin: MirValueOrigin) -> MirValueRef {
        let id = MirValueRef(self.value_seed);
        self.value_seed += 1;
        self.values.push(MirValue { id, origin });
        id
    }

    fn next_state_id(&mut self) -> u32 {
        let id = self.state_seed;
        self.state_seed += 1;
        id
    }

    fn terminate(&mut self, terminator: MirTerminator) {
        self.terminator = Some(terminator);
    }

    fn new_block(&mut self, label: &str) -> MirBlockRef {
        let id = MirBlockRef(self.blocks.len() as u32);
        self.blocks.push(MirBlock {
            id,
            label: label.to_string(),
            parameters: Vec::new(),
            instructions: Vec::new(),
            terminator: MirTerminator::Unreachable,
        });
        self.current_block = id;
        self.current_label = label.to_string();
        id
    }

    fn flush_block(&mut self, label: &str) {
        // 保存到 current_block 对应的基本块（按 ID 索引，ID 即创建顺序）。
        let block = &mut self.blocks[self.current_block.0 as usize];
        block.label = label.to_string();
        block.instructions = self.instructions.clone();
        block.terminator = self.terminator.clone().unwrap_or(MirTerminator::Unreachable);
        // 清空缓冲，防止指令/终结符泄漏到下一个块。
        self.instructions.clear();
        self.terminator = None;
    }

    fn ensure_loop_exit_parameter(&mut self, loop_index: usize, ty: Option<ValkyrieType>) -> MirValueRef {
        if let Some(value) = self.loop_stack[loop_index].exit_value {
            return value;
        }

        let exit_block = self.loop_stack[loop_index].exit;
        if let Some(value) = self.blocks[exit_block.0 as usize].parameters.first().copied() {
            if let Some(ty) = ty {
                self.value_types.entry(value).or_insert(ty);
            }
            self.loop_stack[loop_index].exit_value = Some(value);
            return value;
        }

        let value = self.next_value(MirValueOrigin::Temporary);
        if let Some(ty) = ty {
            self.value_types.insert(value, ty);
        }
        self.blocks[exit_block.0 as usize].parameters.push(value);
        self.loop_stack[loop_index].exit_value = Some(value);
        value
    }

    fn ensure_block_parameter(&mut self, block: MirBlockRef, name: &str, ty: Option<ValkyrieType>) -> MirValueRef {
        if let Some(value) = self.blocks[block.0 as usize].parameters.first().copied() {
            if let Some(ty) = ty {
                self.value_types.entry(value).or_insert(ty);
            }
            return value;
        }

        let value = self.next_value(MirValueOrigin::BlockParameter { block, name: name.to_string() });
        if let Some(ty) = ty {
            self.value_types.insert(value, ty);
        }
        self.blocks[block.0 as usize].parameters.push(value);
        value
    }

    fn ensure_handler_exit_parameter(&mut self, handler_index: usize, ty: Option<ValkyrieType>) -> MirValueRef {
        if let Some(value) = self.handler_stack[handler_index].exit_value {
            return value;
        }

        let exit_block = self.handler_stack[handler_index].exit;
        if let Some(value) = self.blocks[exit_block.0 as usize].parameters.first().copied() {
            if let Some(ty) = ty {
                self.value_types.entry(value).or_insert(ty);
            }
            self.handler_stack[handler_index].exit_value = Some(value);
            return value;
        }

        let value = self.next_value(MirValueOrigin::Temporary);
        if let Some(ty) = ty {
            self.value_types.insert(value, ty);
        }
        self.blocks[exit_block.0 as usize].parameters.push(value);
        self.handler_stack[handler_index].exit_value = Some(value);
        value
    }

    fn resolve_loop_index(&self, label: Option<&Identifier>) -> Option<usize> {
        match label {
            Some(label) => self
                .loop_stack
                .iter()
                .enumerate()
                .rev()
                .find_map(|(index, context)| context.label.as_deref().filter(|name| *name == label.as_str()).map(|_| index)),
            None => self.loop_stack.len().checked_sub(1),
        }
    }

    fn bind_catch_arm_pattern(&mut self, pattern: &HirPattern, payload: MirOperand) {
        match pattern {
            HirPattern::Else | HirPattern::Wildcard => {}
            _ => self.bind_pattern_from_operand(pattern, payload, None),
        }
    }

    fn lower_statement(&mut self, statement: &HirStatement) {
        match &statement.kind {
            HirStatementKind::Let { pattern, initializer, ty, .. } => {
                self.record_static_binding(pattern, initializer.as_deref());
                if let Some(expr) = initializer.as_deref() {
                    self.bind_pattern_from_expr(pattern, expr, ty.clone());
                }
                else {
                    self.bind_pattern_from_operand(pattern, MirOperand::Constant(MirConstant::Unit), ty.clone());
                }
            }
            HirStatementKind::Expr(expression) => {
                let _ = self.lower_expr_to_operand(expression);
            }
        }
    }

    fn lower_static_call(&mut self, name: &str, arguments: Vec<MirOperand>, origin: MirValueOrigin) -> MirValueRef {
        let value = self.next_value(origin);
        self.instructions.push(MirInstruction {
            output: Some(value),
            kind: MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(NamePath::new(vec![Identifier::new(name)])),
                arguments,
                builtin: None,
                witness: None,
                effect: None,
            },
        });
        if let Some(return_type) = self.return_types.get(name).cloned() {
            self.value_types.insert(value, return_type);
        }
        value
    }

    fn lower_expr_to_operand(&mut self, expr: &HirExpr) -> MirOperand {
        self.lower_expr_to_operand_with_hint(expr, None)
    }

    fn lower_expr_to_operand_with_hint(&mut self, expr: &HirExpr, expected_type: Option<&ValkyrieType>) -> MirOperand {
        match &expr.kind {
            HirExprKind::Literal(literal) => {
                let (constant, ty) = lower_literal(literal, expected_type);
                let value = self.next_value(MirValueOrigin::Literal);
                self.instructions
                    .push(MirInstruction { output: Some(value), kind: MirInstructionKind::LoadConstant { constant, ty: ty.clone() } });
                if let Some(ty) = ty {
                    self.value_types.insert(value, ty);
                }
                MirOperand::Value(value)
            }
            HirExprKind::Variable(identifier) => self
                .bindings
                .get(identifier.name.as_str())
                .cloned()
                .unwrap_or_else(|| MirOperand::Symbol(NamePath::new(vec![identifier.name.clone()]))),
            HirExprKind::Path(path) => {
                let value = self.next_value(MirValueOrigin::Path);
                self.instructions.push(MirInstruction { output: Some(value), kind: MirInstructionKind::LoadSymbol { path: path.clone() } });
                MirOperand::Value(value)
            }
            HirExprKind::Call { callee, args, resolved } => {
                // 检测实例方法调用：`obj.method()` 其中 `obj` 的根是已绑定的变量（参数或 let 绑定）。
                // 此时不应将 `obj.method` 解析为静态路径，而应将 `obj` 作为接收者参数传入，
                // 方法名作为单组件 callee，使后端能正确识别为实例方法调用。
                // 例如 `args.length()` 降级为 `length(args)`，`left.starts_with("/")` 降级为 `starts_with(left, "/")`。
                //
                // 处理两种 callee 形式：
                // 1. `FieldAccess`：直接字段访问表达式
                // 2. `Path`：`extract_dotted_path` 将点分路径统一解析为 `Path`，
                //    需在此检测首段是否为绑定变量，以区分方法调用与模块路径调用。
                if let Some((receiver_operand, method_name)) = self.extract_method_call(callee) {
                    let mut arguments = args.iter().map(|arg| self.lower_expr_to_operand(arg)).collect::<Vec<_>>();
                    arguments.insert(0, receiver_operand);
                    let callee = MirOperand::Symbol(NamePath::new(vec![method_name.clone()]));
                    let value = self.next_value(MirValueOrigin::CallResult);
                    self.instructions.push(MirInstruction {
                        output: Some(value),
                        kind: MirInstructionKind::Call {
                            dispatch: MirDispatchKind::Static,
                            callee,
                            arguments,
                            builtin: None,
                            witness: None,
                            effect: None,
                        },
                    });
                    if let Some(return_type) = self.return_types.get(method_name.as_str()).cloned() {
                        self.value_types.insert(value, return_type);
                    }
                    return MirOperand::Value(value);
                }
                let callee = lower_callee_operand(callee, resolved.as_ref(), self);
                let arguments = args.iter().map(|arg| self.lower_expr_to_operand(arg)).collect::<Vec<_>>();
                let builtin = resolve_builtin_call(&callee, &arguments, &self.value_types);
                let value = self.next_value(MirValueOrigin::CallResult);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::Call {
                        dispatch: MirDispatchKind::Static,
                        callee: callee.clone(),
                        arguments: arguments.clone(),
                        builtin,
                        witness: None,
                        effect: None,
                    },
                });
                if let Some(ty) = builtin
                    .and_then(|builtin| builtin_call_output_type(builtin, &arguments, &self.value_types))
                    .or_else(|| resolved.as_ref().map(|call| call.return_type.clone()))
                    .or_else(|| match &callee {
                        MirOperand::Symbol(path) => self.return_types.get(&path.to_string()).cloned(),
                        _ => None,
                    })
                {
                    self.value_types.insert(value, ty);
                }
                MirOperand::Value(value)
            }
            HirExprKind::ArrayNew { element_type, length } => {
                // 数组创建：求值长度，生成 ArrayNew 指令。
                let length_operand = self.lower_expr_to_operand(length);
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::ArrayNew { element_type: element_type.clone(), length: length_operand },
                });
                self.value_types.insert(value, ValkyrieType::Array(Box::new(element_type.clone())));
                MirOperand::Value(value)
            }
            HirExprKind::ArrayLiteral { items } => {
                let array_item_type = match expected_type {
                    Some(ValkyrieType::Array(item)) => Some(item.as_ref()),
                    _ => None,
                };
                let element_type = infer_array_literal_element_type(items, array_item_type);
                let item_operands =
                    items.iter().map(|item| self.lower_expr_to_operand_with_hint(item, Some(&element_type))).collect::<Vec<_>>();
                let array_value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(array_value),
                    kind: MirInstructionKind::ArrayLiteral { element_type: element_type.clone(), items: item_operands },
                });
                self.value_types.insert(array_value, ValkyrieType::Array(Box::new(element_type)));
                MirOperand::Value(array_value)
            }
            HirExprKind::Construct { name, args, resolved } => {
                // 结构体构造：收集字段名与字段值，生成 StructNew 指令。
                let mut fields = Vec::with_capacity(args.len());
                for arg in args {
                    if let HirExprKind::FieldInit { name, value } = &arg.kind {
                        let value_operand = self.lower_expr_to_operand(value);
                        fields.push((name.to_string(), value_operand));
                    }
                }
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions
                    .push(MirInstruction { output: Some(value), kind: MirInstructionKind::StructNew { type_name: name.to_string(), fields } });
                self.value_types
                    .insert(value, resolved.as_ref().map(|call| call.return_type.clone()).unwrap_or_else(|| ValkyrieType::Named(name.clone())));
                MirOperand::Value(value)
            }
            HirExprKind::FieldAccess { object, field } => {
                // 字段读取：求值对象，生成 FieldGet 指令。
                let object_operand = self.lower_expr_to_operand(object);
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::FieldGet { object: object_operand, field: field.to_string() },
                });
                MirOperand::Value(value)
            }
            HirExprKind::StoreField { object, field, value } => {
                // 字段写入：求值对象和值，生成 FieldSet 指令。
                let object_operand = self.lower_expr_to_operand(object);
                let value_operand = self.lower_expr_to_operand(value);
                self.instructions.push(MirInstruction {
                    output: None,
                    kind: MirInstructionKind::FieldSet { object: object_operand, field: field.to_string(), value: value_operand },
                });
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Return(value) => {
                let terminand = value.as_deref().map(|e| self.lower_expr_to_operand(e));
                self.terminate(MirTerminator::Return { value: terminand });
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Assign { target, value } => {
                // 求值右侧表达式得到操作数。
                let operand = self.lower_expr_to_operand(value);
                // 为赋值创建新的 SSA 值，存储到命名槽位（复用原 let 的槽位）。
                let name = target.as_str().to_string();
                let new_value = self.next_value(MirValueOrigin::LetBinding { name: name.clone() });
                self.instructions.push(MirInstruction {
                    output: Some(new_value),
                    kind: MirInstructionKind::StoreVar { name: name.clone(), value: operand.clone(), ty: None },
                });
                if let Some(ty) = infer_builder_operand_type(&operand, &self.value_types) {
                    self.value_types.insert(new_value, ty);
                }
                // 更新绑定指向新值。
                self.bindings.insert(name, MirOperand::Value(new_value));
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::If { condition, then_branch, else_branch } => self.lower_if_expr(condition, then_branch, else_branch),
            HirExprKind::Block(body) => self.lower_block_expr(body),
            HirExprKind::Loop { label, pattern, iterator, condition, body, .. } => {
                self.lower_loop_expr(label, pattern, iterator, condition, body)
            }
            HirExprKind::Break { label, expr } => self.lower_break_expr(label, expr),
            HirExprKind::Continue { label } => self.lower_continue_expr(label),
            HirExprKind::Match { scrutinee, arms } => self.lower_match_expr(scrutinee, arms),
            HirExprKind::Case { scrutinee, arms } => self.lower_case_expr(scrutinee, arms),
            HirExprKind::Yield(value) => self.lower_yield_expr(value.as_deref()),
            HirExprKind::YieldFrom(value) => self.lower_yield_from_expr(value),
            HirExprKind::Await(value) => self.lower_await_expr(value),
            HirExprKind::Awake(value) => self.lower_awake_expr(value),
            HirExprKind::BlockOn(value) => self.lower_block_on_expr(value),
            HirExprKind::Raise(value) => self.lower_raise_expr(value),
            HirExprKind::Resume(value) => self.lower_resume_expr(value),
            HirExprKind::Catch { expr, arms } => self.lower_catch_expr(expr, arms),
            HirExprKind::Fallthrough => self.lower_fallthrough_expr(),
            _ => {
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::LoadSymbol { path: NamePath::new(vec![Identifier::new("unsupported_hir_expr")]) },
                });
                MirOperand::Value(value)
            }
        }
    }

    fn resolve_static_expr(&self, expr: &HirExpr) -> Option<HirExpr> {
        match &expr.kind {
            HirExprKind::Variable(identifier) => self.static_bindings.get(identifier.name.as_str()).cloned().or_else(|| Some(expr.clone())),
            _ => Some(expr.clone()),
        }
    }

    fn resolve_static_iterable_items(&self, expr: &HirExpr) -> Option<Vec<HirExpr>> {
        let resolved = self.resolve_static_expr(expr)?;
        match resolved.kind {
            HirExprKind::ArrayLiteral { items } => Some(items),
            HirExprKind::Call { callee, args, .. } if callee_name_matches(&callee.kind, "array") => Some(args),
            HirExprKind::Call { callee, args, resolved: call_resolved } if callee_name_matches(&callee.kind, "tuple") => {
                Some(vec![HirExpr { kind: HirExprKind::Call { callee, args, resolved: call_resolved }, span: resolved.span }])
            }
            _ => None,
        }
    }

    /// 检测 callee 是否为实例方法调用，返回 `(接收者操作数, 方法名)`。
    ///
    /// 处理两种 callee 形式：
    /// 1. `FieldAccess { object, field }`：当 `object` 的根变量在 `bindings` 中时，
    ///    将 `object` 作为接收者，`field` 作为方法名。
    /// 2. `Path([first, second, ...])`：当 `first` 在 `bindings` 中时，
    ///    将 `first` 对应的绑定值作为接收者，`second` 作为方法名。
    ///    仅处理 2 段路径；多段路径（如 `obj.field.method`）需要字段访问支持，暂不处理。
    fn extract_method_call(&mut self, callee: &HirExpr) -> Option<(MirOperand, Identifier)> {
        match &callee.kind {
            HirExprKind::FieldAccess { object, field } => {
                let root_name = root_variable_name(object)?;
                if !self.bindings.contains_key(root_name) {
                    return None;
                }
                let receiver_operand = self.lower_expr_to_operand(object);
                Some((receiver_operand, field.clone()))
            }
            HirExprKind::Path(path) if path.parts().len() == 2 => {
                let receiver_name = &path.parts()[0];
                if !self.bindings.contains_key(receiver_name.as_str()) {
                    return None;
                }
                let receiver_operand = self.bindings.get(receiver_name.as_str()).cloned()?;
                let method_name = path.parts()[1].clone();
                Some((receiver_operand, method_name))
            }
            _ => None,
        }
    }
}

fn infer_array_literal_element_type(items: &[HirExpr], hint: Option<&ValkyrieType>) -> ValkyrieType {
    if let Some(hint) = hint {
        return hint.clone();
    }
    match items.first().map(|item| &item.kind) {
        Some(HirExprKind::Literal(HirLiteral::Bool(_))) => ValkyrieType::Boolean,
        Some(HirExprKind::Literal(HirLiteral::String(_))) => ValkyrieType::Utf8,
        Some(HirExprKind::Literal(HirLiteral::Float64(_))) => ValkyrieType::Float64,
        Some(HirExprKind::Literal(HirLiteral::Integer64(_))) => ValkyrieType::Integer32 { signed: true },
        _ => ValkyrieType::Integer32 { signed: true },
    }
}

fn infer_builder_constant_type(constant: &MirConstant) -> Option<ValkyrieType> {
    Some(match constant {
        MirConstant::Int(value) if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => ValkyrieType::Integer32 { signed: true },
        MirConstant::Int(_) => ValkyrieType::Integer64 { signed: true },
        MirConstant::Float64(_) => ValkyrieType::Float64,
        MirConstant::Bool(_) => ValkyrieType::Boolean,
        MirConstant::String(_) => ValkyrieType::Utf8,
        MirConstant::Unit => ValkyrieType::Unit,
    })
}

fn infer_builder_operand_type(operand: &MirOperand, value_types: &BTreeMap<MirValueRef, ValkyrieType>) -> Option<ValkyrieType> {
    match operand {
        MirOperand::Value(value_ref) => value_types.get(value_ref).cloned(),
        MirOperand::Constant(constant) => infer_builder_constant_type(constant),
        MirOperand::Symbol(_) => None,
    }
}

fn named_type_name(ty: &ValkyrieType) -> Option<&str> {
    match ty {
        ValkyrieType::Named(name) => Some(name.as_str()),
        ValkyrieType::Apply(base, _) => named_type_name(base),
        _ => None,
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

fn plain_type_pattern_matches(actual_type: &ValkyrieType, pattern_name: &NamePath) -> bool {
    let Some(expected_name) = pattern_name.parts().last().map(|identifier| identifier.as_str())
    else {
        return false;
    };

    match actual_type {
        ValkyrieType::Void => expected_name == "void",
        ValkyrieType::Unit => expected_name == "unit",
        ValkyrieType::Boolean => expected_name == "bool",
        ValkyrieType::Integer8 { signed } => expected_name == if *signed { "i8" } else { "u8" },
        ValkyrieType::Integer16 { signed } => expected_name == if *signed { "i16" } else { "u16" },
        ValkyrieType::Integer32 { signed } => expected_name == if *signed { "i32" } else { "u32" },
        ValkyrieType::Integer64 { signed } => expected_name == if *signed { "i64" } else { "u64" },
        ValkyrieType::Integer128 { signed } => expected_name == if *signed { "i128" } else { "u128" },
        ValkyrieType::Float32 => expected_name == "f32",
        ValkyrieType::Float64 => expected_name == "f64",
        ValkyrieType::Character => expected_name == "char",
        ValkyrieType::Utf8 => expected_name == "utf8",
        ValkyrieType::Utf16 => expected_name == "utf16",
        ValkyrieType::Named(name) => name.as_str() == expected_name,
        ValkyrieType::Apply(base, _) => plain_type_pattern_matches(base, pattern_name),
        ValkyrieType::Array(_) => expected_name == "array",
        ValkyrieType::Tuple(_) => expected_name == "tuple",
        ValkyrieType::Function(_) => expected_name == "function",
        ValkyrieType::TraitObject(_) => expected_name == "trait_object",
        ValkyrieType::Associated(_) => expected_name == "associated",
        ValkyrieType::AutoType => expected_name == "auto",
        ValkyrieType::SelfType => expected_name == "Self",
        ValkyrieType::Generic(generic) => generic.name.as_str() == expected_name,
        ValkyrieType::TypeLambda(_) => expected_name == "type_lambda",
    }
}

fn resolve_builtin_call(
    callee: &MirOperand,
    arguments: &[MirOperand],
    value_types: &BTreeMap<MirValueRef, ValkyrieType>,
) -> Option<MirBuiltinCall> {
    let MirOperand::Symbol(path) = callee
    else {
        return None;
    };
    let first = arguments.first().and_then(|arg| infer_builder_operand_type(arg, value_types));
    let second = arguments.get(1).and_then(|arg| infer_builder_operand_type(arg, value_types));
    match path.to_string().as_str() {
        "suffix []" if matches!(first, Some(ValkyrieType::Array(_))) && is_builder_integer_type(second.as_ref()) => {
            Some(MirBuiltinCall::ArrayGet)
        }
        "suffix []=" if matches!(first, Some(ValkyrieType::Array(_))) && is_builder_integer_type(second.as_ref()) => {
            Some(MirBuiltinCall::ArraySet)
        }
        "len" | "length" if matches!(first, Some(ValkyrieType::Array(_))) => Some(MirBuiltinCall::ArrayLength),
        "prefix -" if is_builder_numeric_type(first.as_ref()) => Some(MirBuiltinCall::NumericNeg),
        "infix &&" if matches!(first, Some(ValkyrieType::Boolean)) && matches!(second, Some(ValkyrieType::Boolean)) => {
            Some(MirBuiltinCall::LogicalAnd)
        }
        "infix ||" if matches!(first, Some(ValkyrieType::Boolean)) && matches!(second, Some(ValkyrieType::Boolean)) => {
            Some(MirBuiltinCall::LogicalOr)
        }
        "prefix !" if matches!(first, Some(ValkyrieType::Boolean)) => Some(MirBuiltinCall::LogicalNot),
        "ExitCode" if is_builder_integer_type(first.as_ref()) => Some(MirBuiltinCall::Identity),
        "infix +" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Add),
        "infix -" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Sub),
        "infix *" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Mul),
        "infix /" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Div),
        "infix %" => resolve_binary_numeric_builtin(first.as_ref(), second.as_ref(), MirBuiltinBinaryOp::Rem),
        "infix ==" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Eq),
        "infix !=" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Ne),
        "infix <" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Lt),
        "infix <=" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Le),
        "infix >" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Gt),
        "infix >=" => resolve_compare_builtin(first.as_ref(), second.as_ref(), MirBuiltinCompareOp::Ge),
        _ => None,
    }
}

fn resolve_binary_numeric_builtin(lhs: Option<&ValkyrieType>, rhs: Option<&ValkyrieType>, op: MirBuiltinBinaryOp) -> Option<MirBuiltinCall> {
    if lhs == rhs && is_builder_numeric_type(lhs) {
        Some(MirBuiltinCall::BinaryNumeric(op))
    }
    else {
        None
    }
}

fn resolve_compare_builtin(lhs: Option<&ValkyrieType>, rhs: Option<&ValkyrieType>, op: MirBuiltinCompareOp) -> Option<MirBuiltinCall> {
    if lhs == rhs && (is_builder_numeric_type(lhs) || matches!(lhs, Some(ValkyrieType::Boolean))) {
        Some(MirBuiltinCall::Compare(op))
    }
    else {
        None
    }
}

fn builtin_call_output_type(
    builtin: MirBuiltinCall,
    arguments: &[MirOperand],
    value_types: &BTreeMap<MirValueRef, ValkyrieType>,
) -> Option<ValkyrieType> {
    match builtin {
        MirBuiltinCall::BinaryNumeric(_) | MirBuiltinCall::NumericNeg | MirBuiltinCall::Identity => {
            arguments.first().and_then(|argument| infer_builder_operand_type(argument, value_types))
        }
        MirBuiltinCall::Compare(_) | MirBuiltinCall::LogicalAnd | MirBuiltinCall::LogicalOr | MirBuiltinCall::LogicalNot => {
            Some(ValkyrieType::Boolean)
        }
        MirBuiltinCall::ArrayGet => match arguments.first().and_then(|argument| infer_builder_operand_type(argument, value_types))? {
            ValkyrieType::Array(item) => Some(*item),
            _ => None,
        },
        MirBuiltinCall::ArraySet => Some(ValkyrieType::Unit),
        MirBuiltinCall::ArrayLength => Some(ValkyrieType::Integer32 { signed: true }),
    }
}

fn is_builder_numeric_type(ty: Option<&ValkyrieType>) -> bool {
    matches!(
        ty,
        Some(
            ValkyrieType::Integer8 { .. }
                | ValkyrieType::Integer16 { .. }
                | ValkyrieType::Integer32 { .. }
                | ValkyrieType::Integer64 { .. }
                | ValkyrieType::Integer128 { .. }
                | ValkyrieType::Float32
                | ValkyrieType::Float64
        )
    )
}

fn is_builder_integer_type(ty: Option<&ValkyrieType>) -> bool {
    matches!(
        ty,
        Some(
            ValkyrieType::Integer8 { .. }
                | ValkyrieType::Integer16 { .. }
                | ValkyrieType::Integer32 { .. }
                | ValkyrieType::Integer64 { .. }
                | ValkyrieType::Integer128 { .. }
        )
    )
}

/// 判断 callee 名称是否匹配预期。
///
/// 同时支持 `HirExprKind::Path`（多部分路径）和 `HirExprKind::Variable`（单部分名称），
/// 因为 `lower_name_expression` 会将单部分名称降级为 `Variable`。
fn callee_name_matches(kind: &HirExprKind, expected: &str) -> bool {
    match kind {
        HirExprKind::Path(path) if path.to_string() == expected => true,
        HirExprKind::Variable(ident) if ident.name.as_str() == expected => true,
        _ => false,
    }
}

fn lower_callee_operand(expr: &HirExpr, resolved: Option<&valkyrie_types::hir::HirResolvedCall>, builder: &mut MirBuilder) -> MirOperand {
    if let Some(resolved) = resolved {
        return MirOperand::Symbol(resolved.symbol.clone());
    }
    match &expr.kind {
        HirExprKind::Path(path) => MirOperand::Symbol(path.clone()),
        HirExprKind::Variable(identifier) => builder
            .bindings
            .get(identifier.name.as_str())
            .cloned()
            .unwrap_or_else(|| MirOperand::Symbol(NamePath::new(vec![identifier.name.clone()]))),
        HirExprKind::GenericApply { callee, .. } => lower_callee_operand(callee, None, builder),
        // 字段访问链作为 callee 时，尝试解析为符号路径。
        // 例如 `std.iterator.collect_array` 会被降级为嵌套 FieldAccess，
        // 此处将其重新组合为 NamePath，以便后端生成静态调用。
        HirExprKind::FieldAccess { .. } => match try_resolve_as_path(expr) {
            Some(path) => MirOperand::Symbol(path),
            None => builder.lower_expr_to_operand(expr),
        },
        _ => builder.lower_expr_to_operand(expr),
    }
}

/// 尝试将表达式解析为 `NamePath`。
///
/// 递归处理 `Variable` 和 `FieldAccess` 链，将其组合为完整路径。
/// 例如 `std.iterator.collect_array` 会被解析为 `NamePath(["std", "iterator", "collect_array"])`。
/// 若表达式不是纯路径形式（如对象是复杂表达式），返回 `None`。
fn try_resolve_as_path(expr: &HirExpr) -> Option<NamePath> {
    match &expr.kind {
        HirExprKind::Variable(identifier) => Some(NamePath::new(vec![identifier.name.clone()])),
        HirExprKind::Path(path) => Some(path.clone()),
        HirExprKind::FieldAccess { object, field } => {
            let mut path = try_resolve_as_path(object)?;
            path.append(field.clone());
            Some(path)
        }
        _ => None,
    }
}

/// 递归提取 `FieldAccess` 链的根变量名。
///
/// 例如 `request.target.length` 的根变量是 `request`。
/// 若表达式不是 `Variable` 或 `FieldAccess` 链，返回 `None`。
/// 用于区分实例方法调用（`obj.method()`）和静态路径调用（`module.function()`）。
fn root_variable_name(expr: &HirExpr) -> Option<&str> {
    match &expr.kind {
        HirExprKind::Variable(identifier) => Some(identifier.name.as_str()),
        HirExprKind::FieldAccess { object, .. } => root_variable_name(object),
        _ => None,
    }
}

fn lower_literal(literal: &HirLiteral, expected_type: Option<&ValkyrieType>) -> (MirConstant, Option<ValkyrieType>) {
    match literal {
        HirLiteral::Integer64(value) => (
            MirConstant::Int(*value),
            Some(match expected_type {
                Some(ValkyrieType::Integer32 { signed }) => ValkyrieType::Integer32 { signed: *signed },
                Some(ValkyrieType::Integer64 { signed }) => ValkyrieType::Integer64 { signed: *signed },
                _ if *value >= i32::MIN as i64 && *value <= i32::MAX as i64 => ValkyrieType::Integer32 { signed: true },
                _ => ValkyrieType::Integer64 { signed: true },
            }),
        ),
        HirLiteral::Float64(value) => (MirConstant::Float64(*value), Some(ValkyrieType::Float64)),
        HirLiteral::Bool(value) => (MirConstant::Bool(*value), Some(ValkyrieType::Boolean)),
        HirLiteral::String(value) => (
            MirConstant::String(
                value
                    .segments
                    .iter()
                    .map(|segment| match segment {
                        valkyrie_types::hir::HirStringSegment::Text(text) => text.clone(),
                        valkyrie_types::hir::HirStringSegment::Interpolation { expr, .. } => {
                            format!("${{{}}}", render_interpolation_expr(expr))
                        }
                    })
                    .collect::<String>(),
            ),
            Some(ValkyrieType::Utf8),
        ),
        HirLiteral::Unit => (MirConstant::Unit, Some(ValkyrieType::Unit)),
    }
}

fn render_interpolation_expr(expr: &HirExpr) -> String {
    match &expr.kind {
        HirExprKind::Variable(identifier) => identifier.name.to_string(),
        HirExprKind::Path(path) => path.to_string(),
        HirExprKind::Literal(HirLiteral::Integer64(value)) => value.to_string(),
        HirExprKind::Literal(HirLiteral::Bool(value)) => value.to_string(),
        HirExprKind::Literal(HirLiteral::String(value)) => value
            .segments
            .iter()
            .map(|segment| match segment {
                valkyrie_types::hir::HirStringSegment::Text(text) => text.clone(),
                valkyrie_types::hir::HirStringSegment::Interpolation { .. } => "${...}".to_string(),
            })
            .collect(),
        HirExprKind::Call { callee, .. } => format!("{}(...)", render_interpolation_expr(callee)),
        _ => "...".to_string(),
    }
}

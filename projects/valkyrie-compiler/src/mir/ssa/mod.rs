#![allow(missing_docs)]

use std::collections::BTreeMap;

use valkyrie_types::{
    hir::{HirExpr, HirExprKind, HirFunction, HirLiteral, HirModule, HirPattern, HirStatement, HirStatementKind, HirType},
    Identifier, NamePath,
};

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
    pub ty: HirType,
}

/// `SSA` 形式的 `MIR` 函数。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MirFunction {
    pub symbol: String,
    /// 函数返回类型，用于后端判断调用是否返回 `void`。
    pub return_type: HirType,
    /// 函数参数类型列表，用于后端生成方法签名。
    pub param_types: Vec<HirType>,
    pub entry: MirBlockRef,
    pub values: Vec<MirValue>,
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

/// `MIR` 指令种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirInstructionKind {
    LoadConstant {
        constant: MirConstant,
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
        ty: Option<HirType>,
    },
    Call {
        dispatch: MirDispatchKind,
        callee: MirOperand,
        arguments: Vec<MirOperand>,
        witness: Option<MirOperand>,
        effect: Option<MirOperand>,
    },
    /// 数组下标访问：`object[index]`。
    ///
    /// `object` 是被索引的数组操作数，`index` 是索引操作数。
    /// 结果是数组元素值。
    Subscript {
        /// 被索引的数组对象。
        object: MirOperand,
        /// 索引表达式结果。
        index: MirOperand,
    },
    /// 数组元素赋值：`object[index] = value`。
    ///
    /// 将 `value` 写入 `object` 数组的 `index` 位置。
    /// 无输出值（结果为 `Unit`）。
    StoreSubscript {
        /// 被写入的数组对象。
        object: MirOperand,
        /// 索引表达式结果。
        index: MirOperand,
        /// 要写入的值。
        value: MirOperand,
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
    /// 数组创建：`new [ElementType](length)`。
    ///
    /// 创建一个指定元素类型和长度的一维零基数组。
    /// 输出是新创建的数组实例。
    ArrayNew {
        /// 数组元素类型。
        element_type: HirType,
        /// 数组长度。
        length: MirOperand,
    },
}

/// `MIR` 终结符。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MirTerminator {
    Return { value: Option<MirOperand> },
    Jump { target: MirBlockRef, arguments: Vec<MirOperand> },
    Branch { condition: MirOperand, then_target: MirBlockRef, else_target: MirBlockRef },
    Unreachable,
}

pub struct MirLowerer;

impl MirLowerer {
    pub fn lower_module(module: &HirModule) -> MirModule {
        let structs = module.structs.iter().map(lower_struct).collect();
        let imports = module.imports.iter().map(|path| path.to_string()).collect();
        MirModule { name: module.name.to_string(), functions: module.functions.iter().map(lower_function).collect(), structs, imports }
    }
}

/// 将 `HirStruct` 降级为 `MirStruct`，提取后端生成 `TypeDef`/`Field` 表所需的最小信息。
fn lower_struct(hir_struct: &valkyrie_types::hir::HirStruct) -> MirStruct {
    let fields = hir_struct.fields.iter().map(|field| MirField { name: field.name.to_string(), ty: field.ty.clone() }).collect();
    let namespace = hir_struct.namespace.iter().map(|part| part.as_str().to_string()).collect::<Vec<_>>().join(".");
    MirStruct { name: hir_struct.name.to_string(), namespace, fields, is_value_type: hir_struct.is_value_type }
}

fn lower_function(function: &HirFunction) -> MirFunction {
    let mut builder = MirBuilder::new();

    // 绑定参数，并收集参数值用于入口块的 parameters 字段。
    let mut param_values = Vec::new();
    for (index, param) in function.params.iter().enumerate() {
        let value = builder.next_value(MirValueOrigin::Parameter { index, name: param.name.name.to_string() });
        builder.bindings.insert(param.name.name.to_string(), MirOperand::Value(value));
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
    builder.flush_block("entry");

    let param_types = function.params.iter().map(|p| p.ty.clone()).collect();
    MirFunction {
        symbol: function.name.to_string(),
        return_type: function.return_type.clone(),
        param_types,
        entry: builder.entry,
        values: builder.values,
        blocks: builder.blocks,
    }
}

/// MIR 构建器：管理基本块创建和控制流。
struct MirBuilder {
    entry: MirBlockRef,
    current_block: MirBlockRef,
    current_label: String,
    values: Vec<MirValue>,
    instructions: Vec<MirInstruction>,
    blocks: Vec<MirBlock>,
    bindings: BTreeMap<String, MirOperand>,
    static_bindings: BTreeMap<String, HirExpr>,
    terminator: Option<MirTerminator>,
    value_seed: u32,
    /// 循环上下文栈：(header, exit) 块 ID。
    /// `break` 跳转到 exit，`continue` 跳转到 header。
    loop_stack: Vec<(MirBlockRef, MirBlockRef)>,
}

impl MirBuilder {
    fn new() -> Self {
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
            bindings: BTreeMap::new(),
            static_bindings: BTreeMap::new(),
            terminator: None,
            value_seed: 0,
            loop_stack: Vec::new(),
        }
    }

    fn next_value(&mut self, origin: MirValueOrigin) -> MirValueRef {
        let id = MirValueRef(self.value_seed);
        self.value_seed += 1;
        self.values.push(MirValue { id, origin });
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

    fn record_static_binding(&mut self, pattern: &HirPattern, initializer: Option<&HirExpr>) {
        match pattern {
            HirPattern::Variable(identifier) => {
                if let Some(expr) = initializer {
                    self.static_bindings.insert(identifier.name.to_string(), expr.clone());
                }
                else {
                    self.static_bindings.remove(identifier.name.as_str());
                }
            }
            HirPattern::Tuple(items) => {
                for item in items {
                    self.record_static_binding(item, None);
                }
            }
            _ => {}
        }
    }

    fn bind_pattern_from_expr(&mut self, pattern: &HirPattern, expr: &HirExpr, ty: Option<HirType>) {
        match pattern {
            HirPattern::Tuple(items) => {
                if let Some(tuple_items) = tuple_literal_items(expr) {
                    for (item_pattern, item_expr) in items.iter().zip(tuple_items.iter()) {
                        self.bind_pattern_from_expr(item_pattern, item_expr, None);
                    }
                    return;
                }
            }
            HirPattern::Wildcard => {
                let _ = self.lower_expr_to_operand(expr);
                return;
            }
            _ => {}
        }

        let operand = self.lower_expr_to_operand(expr);
        self.bind_pattern_from_operand(pattern, operand, ty);
    }

    fn bind_pattern_from_operand(&mut self, pattern: &HirPattern, operand: MirOperand, ty: Option<HirType>) {
        match pattern {
            HirPattern::Wildcard => {}
            HirPattern::Variable(identifier) => {
                let name = identifier.name.to_string();
                let value = self.next_value(MirValueOrigin::LetBinding { name: name.clone() });
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::StoreVar { name: name.clone(), value: operand, ty },
                });
                self.bindings.insert(name, MirOperand::Value(value));
            }
            HirPattern::Tuple(items) => {
                for (index, item_pattern) in items.iter().enumerate() {
                    let extracted = self.lower_static_call(&format!("tuple_get_{index}"), vec![operand.clone()], MirValueOrigin::Temporary);
                    self.bind_pattern_from_operand(item_pattern, MirOperand::Value(extracted), None);
                }
            }
            _ => {
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::LoadSymbol { path: NamePath::new(vec![Identifier::new("unsupported_pattern")]) },
                });
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
                witness: None,
                effect: None,
            },
        });
        value
    }

    fn lower_expr_to_operand(&mut self, expr: &HirExpr) -> MirOperand {
        match &expr.kind {
            HirExprKind::Literal(literal) => {
                let constant = lower_literal(literal);
                let value = self.next_value(MirValueOrigin::Literal);
                self.instructions.push(MirInstruction { output: Some(value), kind: MirInstructionKind::LoadConstant { constant } });
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
            HirExprKind::Call { callee, args } => {
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
                    let callee = MirOperand::Symbol(NamePath::new(vec![method_name]));
                    let value = self.next_value(MirValueOrigin::CallResult);
                    self.instructions.push(MirInstruction {
                        output: Some(value),
                        kind: MirInstructionKind::Call { dispatch: MirDispatchKind::Static, callee, arguments, witness: None, effect: None },
                    });
                    return MirOperand::Value(value);
                }
                let callee = lower_callee_operand(callee, self);
                let arguments = args.iter().map(|arg| self.lower_expr_to_operand(arg)).collect();
                let value = self.next_value(MirValueOrigin::CallResult);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::Call { dispatch: MirDispatchKind::Static, callee, arguments, witness: None, effect: None },
                });
                MirOperand::Value(value)
            }
            HirExprKind::Subscript { object, index } => {
                // 求值对象和索引，生成 Subscript 指令。
                let object_operand = self.lower_expr_to_operand(object);
                let index_operand = self.lower_expr_to_operand(index);
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::Subscript { object: object_operand, index: index_operand },
                });
                MirOperand::Value(value)
            }
            HirExprKind::StoreSubscript { object, index, value } => {
                // 数组元素赋值：求值对象、索引、值，生成 StoreSubscript 指令。
                let object_operand = self.lower_expr_to_operand(object);
                let index_operand = self.lower_expr_to_operand(index);
                let value_operand = self.lower_expr_to_operand(value);
                self.instructions.push(MirInstruction {
                    output: None,
                    kind: MirInstructionKind::StoreSubscript { object: object_operand, index: index_operand, value: value_operand },
                });
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::ArrayNew { element_type, length } => {
                // 数组创建：求值长度，生成 ArrayNew 指令。
                let length_operand = self.lower_expr_to_operand(length);
                let value = self.next_value(MirValueOrigin::Temporary);
                self.instructions.push(MirInstruction {
                    output: Some(value),
                    kind: MirInstructionKind::ArrayNew { element_type: element_type.clone(), length: length_operand },
                });
                MirOperand::Value(value)
            }
            HirExprKind::Construct { name, args } => {
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
                    kind: MirInstructionKind::StoreVar { name: name.clone(), value: operand, ty: None },
                });
                // 更新绑定指向新值。
                self.bindings.insert(name, MirOperand::Value(new_value));
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::If { condition, then_branch, else_branch } => {
                // 在当前块中求值条件，条件指令与前置指令留在同一块。
                let cond_val = self.lower_expr_to_operand(condition);

                // 保存 if 之前的 bindings，供 then/else 分支共享。
                let pre_if_bindings = self.bindings.clone();

                // 记录条件块 ID（当前块）。
                let cond_block_id = self.current_block;

                // 创建 then、else、merge 块（顺序创建，ID 即索引）。
                let then_block = self.new_block("then");
                let else_block = self.new_block("else");
                let merge_block = self.new_block("merge");

                // 回到条件块，设置 Branch 终结符并刷新。
                self.current_block = cond_block_id;
                self.terminate(MirTerminator::Branch { condition: cond_val, then_target: then_block, else_target: else_block });
                self.flush_block("cond");

                // 切换到 then 块，降低 then 分支。
                self.current_block = then_block;
                self.bindings = pre_if_bindings.clone();

                for statement in &then_branch.statements {
                    self.lower_statement(statement);
                }
                if let Some(tail) = &then_branch.expr {
                    let _ = self.lower_expr_to_operand(tail);
                }

                let then_returns = self.terminator.is_some();

                if self.terminator.is_none() {
                    self.terminate(MirTerminator::Jump { target: merge_block, arguments: Vec::new() });
                }
                self.flush_block("then");

                // 切换到 else 块，降低 else 分支。
                self.current_block = else_block;
                self.bindings = pre_if_bindings.clone();

                let else_returns;
                if let Some(else_body) = else_branch {
                    for statement in &else_body.statements {
                        self.lower_statement(statement);
                    }
                    if let Some(tail) = &else_body.expr {
                        let _ = self.lower_expr_to_operand(tail);
                    }
                    else_returns = self.terminator.is_some();
                }
                else {
                    else_returns = false;
                }

                if self.terminator.is_none() {
                    self.terminate(MirTerminator::Jump { target: merge_block, arguments: Vec::new() });
                }
                self.flush_block("else");

                // 切换到 merge 块，后续代码将追加到此块。
                self.current_block = merge_block;
                self.bindings = pre_if_bindings;

                // 若两个分支都 return，则 merge 块不可达。
                if then_returns && else_returns {
                    self.terminate(MirTerminator::Unreachable);
                }

                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Loop { pattern, iterator, condition, body, .. } => {
                // 静态可展开循环：`loop x in array(...)` 形式且迭代物可在编译期求值时，直接展开。
                if let (Some(loop_pattern), Some(iterator_expr)) = (pattern.as_ref(), iterator.as_deref()) {
                    if let Some(iteration_items) = self.resolve_static_iterable_items(iterator_expr) {
                        for item in iteration_items {
                            self.bind_pattern_from_expr(loop_pattern, &item, None);
                            for statement in &body.statements {
                                self.lower_statement(statement);
                                // 若循环体内触发了 return/break/continue，终结符已被设置，
                                // 后续迭代均为不可达代码，直接退出展开。
                                if self.terminator.is_some() {
                                    break;
                                }
                            }
                            if self.terminator.is_some() {
                                break;
                            }
                            if let Some(tail) = &body.expr {
                                let _ = self.lower_expr_to_operand(tail);
                            }
                            if self.terminator.is_some() {
                                break;
                            }
                        }
                        return MirOperand::Constant(MirConstant::Unit);
                    }
                }

                // 保存循环前的 bindings，用于 header 和 body 恢复。
                let pre_loop_bindings = self.bindings.clone();

                // 预计算 header 块 ID（此时无其他块创建，可安全推算）。
                let loop_header_id = MirBlockRef(self.blocks.len() as u32);

                // 1. 终止当前块为 Jump -> header，然后 flush。
                self.terminate(MirTerminator::Jump { target: loop_header_id, arguments: Vec::new() });
                let current_label = self.current_label.clone();
                self.flush_block(&current_label);
                self.terminator = None;

                // 2. 预创建 header、body、exit 三个块，确保 ID 不会被循环体内嵌套的
                //    if/loop 块抢占。若不预创建 exit 块，循环体内的 if 会先分配到
                //    exit 的预计算 ID，导致 Branch 的 else_target 错乱（指向 if-then
                //    而非循环出口），从而引发无限循环。
                self.new_block("loop_header");
                let loop_body_id = self.new_block("loop_body");
                let loop_exit_id = self.new_block("loop_exit");

                // 3. 回到 header 块，求值条件，设置 Branch(body, exit)。
                self.current_block = loop_header_id;
                self.instructions.clear();
                self.terminator = None;
                self.bindings = pre_loop_bindings.clone();

                let cond_val = self.lower_expr_to_operand(condition.as_ref().unwrap_or(&Box::new(HirExpr {
                    kind: HirExprKind::Literal(HirLiteral::Bool(true)),
                    span: valkyrie_types::SourceSpan::new(valkyrie_types::SourceID::default(), 0, 0),
                })));
                self.terminate(MirTerminator::Branch { condition: cond_val, then_target: loop_body_id, else_target: loop_exit_id });
                self.flush_block("loop_header");
                self.terminator = None;

                // 4. 切换到 body 块，降低循环体，设置 Jump -> header。
                self.current_block = loop_body_id;
                self.instructions.clear();
                self.terminator = None;
                self.bindings = pre_loop_bindings.clone();

                // 将循环上下文压栈，供 break/continue 使用。
                self.loop_stack.push((loop_header_id, loop_exit_id));

                for statement in &body.statements {
                    self.lower_statement(statement);
                }
                if let Some(tail) = &body.expr {
                    let _ = self.lower_expr_to_operand(tail);
                }

                // 弹出循环上下文。
                self.loop_stack.pop();

                if self.terminator.is_none() {
                    self.terminate(MirTerminator::Jump { target: loop_header_id, arguments: Vec::new() });
                }
                self.flush_block("loop_body");
                self.terminator = None;

                // 5. 切换到 exit 块，后续代码将追加到此块。
                self.current_block = loop_exit_id;
                self.instructions.clear();
                self.terminator = None;
                self.bindings = pre_loop_bindings;

                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Break { .. } => {
                // break：跳转到当前循环的 exit 块。
                if let Some((_, exit)) = self.loop_stack.last().copied() {
                    self.terminate(MirTerminator::Jump { target: exit, arguments: Vec::new() });
                    let label = self.current_label.clone();
                    self.flush_block(&label);
                    // 创建新块承载 break 之后的代码（不可达）。
                    self.new_block("after_break");
                }
                MirOperand::Constant(MirConstant::Unit)
            }
            HirExprKind::Continue { .. } => {
                // continue：跳转到当前循环的 header 块。
                if let Some((header, _)) = self.loop_stack.last().copied() {
                    self.terminate(MirTerminator::Jump { target: header, arguments: Vec::new() });
                    let label = self.current_label.clone();
                    self.flush_block(&label);
                    // 创建新块承载 continue 之后的代码（不可达）。
                    self.new_block("after_continue");
                }
                MirOperand::Constant(MirConstant::Unit)
            }
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
            HirExprKind::Call { callee, args } if callee_name_matches(&callee.kind, "array") => Some(args),
            HirExprKind::Call { callee, args } if callee_name_matches(&callee.kind, "tuple") => {
                Some(vec![HirExpr { kind: HirExprKind::Call { callee, args }, span: resolved.span }])
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
            HirExprKind::Path(path) if path.0.len() == 2 => {
                let receiver_name = &path.0[0];
                if !self.bindings.contains_key(receiver_name.as_str()) {
                    return None;
                }
                let receiver_operand = self.bindings.get(receiver_name.as_str()).cloned()?;
                let method_name = path.0[1].clone();
                Some((receiver_operand, method_name))
            }
            _ => None,
        }
    }
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

fn lower_callee_operand(expr: &HirExpr, builder: &mut MirBuilder) -> MirOperand {
    match &expr.kind {
        HirExprKind::Path(path) => MirOperand::Symbol(path.clone()),
        HirExprKind::Variable(identifier) => builder
            .bindings
            .get(identifier.name.as_str())
            .cloned()
            .unwrap_or_else(|| MirOperand::Symbol(NamePath::new(vec![identifier.name.clone()]))),
        HirExprKind::GenericApply { callee, .. } => lower_callee_operand(callee, builder),
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
            path.0.push(field.clone());
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

fn tuple_literal_items(expr: &HirExpr) -> Option<&[HirExpr]> {
    match &expr.kind {
        HirExprKind::Call { callee, args } if callee_name_matches(&callee.kind, "tuple") => Some(args.as_slice()),
        _ => None,
    }
}

fn lower_literal(literal: &HirLiteral) -> MirConstant {
    match literal {
        HirLiteral::Integer64(value) => MirConstant::Int(*value),
        HirLiteral::Bool(value) => MirConstant::Bool(*value),
        HirLiteral::String(value) => MirConstant::String(
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
        HirLiteral::Unit => MirConstant::Unit,
        HirLiteral::Float64(value) => MirConstant::String(value.to_string()),
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

//! `WASM/WASI` 路线 lowering。
//!
//! 将 lane-aware `LIR` 降低为可执行的 `WASM` 模块，
//! 生成 Type / Function / Export / Code 标准段，
//! 使产物可被 `node` / `wasmtime` 加载和执行。

use std::collections::{HashMap, HashSet};

use miette::{miette, Result};
use nyar::{
    abstractions::{BackendInputKind, BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily},
    lanes::{LaneLoweringResult, TargetLoweringLane, TargetLoweringLaneDescriptor},
    packaging::TargetLane,
};
use valkyrie_compiler::{
    hir::HirType,
    lir::{LirBlock, LirFunction, LirImport, LirModule, LirOperand, LirOperation, LirOperationKind, LirTargetLane, LirTerminator},
    mir::{MirBlockRef, MirConstant},
};

use crate::wasm::{WasmBinaryModule, WasmSection};

/// `WASM` 值类型常量。
const VAL_I32: u8 = 0x7F;
const VAL_I64: u8 = 0x7E;
const VAL_F32: u8 = 0x7D;
const VAL_F64: u8 = 0x7C;

/// `WASM` 指令操作码。
const OP_UNREACHABLE: u8 = 0x00;
const OP_BLOCK: u8 = 0x02;
const OP_LOOP: u8 = 0x03;
const OP_IF: u8 = 0x04;
const OP_ELSE: u8 = 0x05;
const OP_END: u8 = 0x0B;
const OP_BR: u8 = 0x0C;
const OP_BR_IF: u8 = 0x0D;
const OP_RETURN: u8 = 0x0F;
const OP_CALL: u8 = 0x10;
const OP_DROP: u8 = 0x1A;
const OP_LOCAL_GET: u8 = 0x20;
const OP_LOCAL_SET: u8 = 0x21;
const OP_I32_CONST: u8 = 0x41;
const OP_I64_CONST: u8 = 0x42;
const OP_F32_CONST: u8 = 0x43;
const OP_F64_CONST: u8 = 0x44;
const OP_I32_EQZ: u8 = 0x45;
const OP_I32_ADD: u8 = 0x6A;
const OP_I32_SUB: u8 = 0x6B;
const OP_I32_MUL: u8 = 0x6C;

/// `WASM` lane 的 `LIR -> WasmBinaryModule` 承接器。
pub struct WasmLirLoweringLane {
    descriptor: TargetLoweringLaneDescriptor,
}

impl WasmLirLoweringLane {
    /// 创建一个新的 `WASM` lane lowering。
    pub fn new() -> Self {
        Self {
            descriptor: TargetLoweringLaneDescriptor {
                name: "wasm-binary-lowering".to_string(),
                lane: TargetLane::Wasm,
                input_kind: BackendInputKind::WasmModule,
                target: BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native),
            },
        }
    }
}

impl Default for WasmLirLoweringLane {
    fn default() -> Self {
        Self::new()
    }
}

impl TargetLoweringLane for WasmLirLoweringLane {
    type PartitionInput = LirModule;
    type BackendInput = WasmBinaryModule;

    fn descriptor(&self) -> &TargetLoweringLaneDescriptor {
        &self.descriptor
    }

    fn lower_partition(&self, partition: Self::PartitionInput) -> Result<LaneLoweringResult<Self::BackendInput>> {
        if partition.lane != LirTargetLane::Wasm {
            return Err(miette!(
                code = "nyar::wasm::lowering::lane_mismatch",
                help = "请确认当前 `LIR` 分区已经选择 `WASM` 目标路线",
                "当前 lane 是 {:?}，不能进入 WASM lowering",
                partition.lane
            ));
        }

        let artifact_name = partition.name.clone();
        let input = lower_lir_to_wasm_module(&partition);
        Ok(LaneLoweringResult { input, artifact_name })
    }
}

/// 将 `LIR` 模块降低为可执行的 `WASM` 模块。
///
/// 生成 Type / Import / Function / Export / Code 标准段，
/// 使产物可被 `node` / `wasmtime` 加载和执行。
///
/// 导入函数在函数索引空间中排在本地函数之前（索引 `0..N-1`），
/// 本地函数紧随其后（索引 `N..N+M-1`）。
pub fn lower_lir_to_wasm_module(lir: &LirModule) -> WasmBinaryModule {
    let mut module = WasmBinaryModule::new();

    // 收集所有唯一的函数签名（包括导入函数和本地函数），生成 Type 段
    let type_map = build_type_section(&mut module, &lir.functions, &lir.imports);

    // 生成 Import 段（必须在 Function 段之前）
    let import_count = build_import_section(&mut module, &lir.imports, &type_map);

    // 生成 Function 段（仅本地函数）
    build_function_section(&mut module, &lir.functions, &type_map);

    // 生成 Export 段（导出入口函数，索引需偏移导入函数数量）
    build_export_section(&mut module, &lir.functions, import_count);

    // 构建统一的函数索引映射（导入函数 + 本地函数）
    let func_index_map = build_func_index_map(&lir.imports, &lir.functions);

    // 构建函数返回类型映射（导入函数 + 本地函数），用于判断 Call 结果是否需要 local.set
    let return_type_map = build_return_type_map(&lir.imports, &lir.functions);

    // 生成 Code 段（仅本地函数）
    build_code_section(&mut module, &lir.functions, &func_index_map, &return_type_map);

    // 保留元数据 custom section
    module.push_custom_section("valkyrie.module", lir.name.as_bytes().to_vec());
    module.push_custom_section("valkyrie.lane", b"wasm".to_vec());

    module
}

/// 将 `HirType` 映射为 `WASM` 值类型字节。
///
/// `Unit` / `Void` 返回 `None`（无值），
/// 其他类型映射到对应的 `WASM` 值类型。
fn hir_type_to_wasm(ty: &HirType) -> Option<u8> {
    match ty {
        HirType::Integer32 | HirType::Boolean | HirType::Utf8 | HirType::Utf16 => Some(VAL_I32),
        HirType::Integer64 => Some(VAL_I64),
        HirType::Float32 => Some(VAL_F32),
        HirType::Float64 => Some(VAL_F64),
        HirType::Unit | HirType::Void => None,
        _ => Some(VAL_I32),
    }
}

/// 构建 Type 段，收集所有唯一的函数签名。
///
/// 同时扫描本地函数和导入函数的签名，返回签名到类型索引的映射。
fn build_type_section(module: &mut WasmBinaryModule, functions: &[LirFunction], imports: &[LirImport]) -> HashMap<String, u32> {
    let mut types: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    let mut type_map: HashMap<String, u32> = HashMap::new();

    // 收集本地函数签名
    for func in functions {
        let params: Vec<u8> = func.param_types.iter().filter_map(hir_type_to_wasm).collect();
        let results: Vec<u8> = hir_type_to_wasm(&func.return_type).into_iter().collect();
        let key = signature_key(&params, &results);

        if !type_map.contains_key(&key) {
            type_map.insert(key.clone(), types.len() as u32);
            types.push((params, results));
        }
    }

    // 收集导入函数签名
    for imp in imports {
        let params: Vec<u8> = imp.param_types.iter().filter_map(hir_type_to_wasm).collect();
        let results: Vec<u8> = hir_type_to_wasm(&imp.return_type).into_iter().collect();
        let key = signature_key(&params, &results);

        if !type_map.contains_key(&key) {
            type_map.insert(key.clone(), types.len() as u32);
            types.push((params, results));
        }
    }

    let mut bytes = Vec::new();
    write_uleb128(&mut bytes, types.len() as u32);
    for (params, results) in &types {
        bytes.push(0x60); // func type
        write_uleb128(&mut bytes, params.len() as u32);
        bytes.extend_from_slice(params);
        write_uleb128(&mut bytes, results.len() as u32);
        bytes.extend_from_slice(results);
    }

    module.sections.push(WasmSection { id: 1, name: None, bytes });

    type_map
}

/// 构建 Import 段（`section id = 2`）。
///
/// 每个导入项编码为：`module_name(string) + field_name(string) + kind(1 byte) + type_index(uleb128)`。
/// 函数导入的 `kind = 0`，后跟 `Type` 段中的类型索引。
///
/// 返回导入函数数量，供后续段调整函数索引偏移。
fn build_import_section(module: &mut WasmBinaryModule, imports: &[LirImport], type_map: &HashMap<String, u32>) -> u32 {
    let mut bytes = Vec::new();
    write_uleb128(&mut bytes, imports.len() as u32);

    for imp in imports {
        // module name
        write_uleb128(&mut bytes, imp.module.len() as u32);
        bytes.extend_from_slice(imp.module.as_bytes());

        // field name
        write_uleb128(&mut bytes, imp.field.len() as u32);
        bytes.extend_from_slice(imp.field.as_bytes());

        // kind: func (0x00)
        bytes.push(0x00);

        // type index
        let params: Vec<u8> = imp.param_types.iter().filter_map(hir_type_to_wasm).collect();
        let results: Vec<u8> = hir_type_to_wasm(&imp.return_type).into_iter().collect();
        let key = signature_key(&params, &results);
        let type_idx = type_map.get(&key).copied().unwrap_or(0);
        write_uleb128(&mut bytes, type_idx);
    }

    module.sections.push(WasmSection { id: 2, name: None, bytes });

    imports.len() as u32
}

/// 构建 Function 段，为每个函数声明类型索引。
fn build_function_section(module: &mut WasmBinaryModule, functions: &[LirFunction], type_map: &HashMap<String, u32>) {
    let mut bytes = Vec::new();
    write_uleb128(&mut bytes, functions.len() as u32);

    for func in functions {
        let params: Vec<u8> = func.param_types.iter().filter_map(hir_type_to_wasm).collect();
        let results: Vec<u8> = hir_type_to_wasm(&func.return_type).into_iter().collect();
        let key = signature_key(&params, &results);
        let type_idx = type_map.get(&key).copied().unwrap_or(0);
        write_uleb128(&mut bytes, type_idx);
    }

    module.sections.push(WasmSection { id: 3, name: None, bytes });
}

/// 构建 Export 段，导出入口函数。
///
/// 导出名为 `main` 或 `_start` 的函数。
/// 函数索引需偏移 `import_count`，因为导入函数占用索引空间前缀。
fn build_export_section(module: &mut WasmBinaryModule, functions: &[LirFunction], import_count: u32) {
    let mut exports: Vec<(String, u32)> = Vec::new();

    for (index, func) in functions.iter().enumerate() {
        let symbol = &func.symbol;
        if symbol == "main" || symbol == "_start" {
            exports.push((symbol.clone(), index as u32 + import_count));
        }
    }

    if exports.is_empty() && !functions.is_empty() {
        exports.push((functions[0].symbol.clone(), import_count));
    }

    let mut bytes = Vec::new();
    write_uleb128(&mut bytes, exports.len() as u32);

    for (name, index) in &exports {
        write_uleb128(&mut bytes, name.len() as u32);
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(0x00); // kind: func
        write_uleb128(&mut bytes, *index);
    }

    module.sections.push(WasmSection { id: 7, name: None, bytes });
}

/// 构建 Code 段，为每个本地函数生成代码体。
fn build_code_section(
    module: &mut WasmBinaryModule,
    functions: &[LirFunction],
    func_index_map: &HashMap<String, u32>,
    return_type_map: &HashMap<String, HirType>,
) {
    let mut bytes = Vec::new();
    write_uleb128(&mut bytes, functions.len() as u32);

    for func in functions {
        let body = lower_function_body(func, func_index_map, return_type_map);
        write_uleb128(&mut bytes, body.len() as u32);
        bytes.extend_from_slice(&body);
    }

    module.sections.push(WasmSection { id: 10, name: None, bytes });
}

/// 构建统一的函数索引映射。
///
/// 导入函数占用索引 `0..N-1`，本地函数占用索引 `N..N+M-1`。
fn build_func_index_map(imports: &[LirImport], functions: &[LirFunction]) -> HashMap<String, u32> {
    let mut map = HashMap::new();
    let import_count = imports.len() as u32;

    for (i, imp) in imports.iter().enumerate() {
        map.insert(imp.symbol.clone(), i as u32);
    }

    for (i, func) in functions.iter().enumerate() {
        map.insert(func.symbol.clone(), import_count + i as u32);
    }

    map
}

/// 构建函数符号到返回类型的映射。
///
/// 用于在 `Call` 指令 lowering 时判断被调用函数是否返回 `unit`/`void`，
/// 若返回 `unit`/`void`，则跳过 `local.set`（WASM 栈上无返回值）。
fn build_return_type_map(imports: &[LirImport], functions: &[LirFunction]) -> HashMap<String, HirType> {
    let mut map = HashMap::new();

    for imp in imports {
        map.insert(imp.symbol.clone(), imp.return_type.clone());
    }

    for func in functions {
        map.insert(func.symbol.clone(), func.return_type.clone());
    }

    map
}

/// 函数 lowering 上下文。
struct FunctionLoweringContext {
    /// 局部变量声明字节流（不含组数前缀）。
    locals_decl: Vec<u8>,
    /// 局部变量组数。
    local_group_count: u32,
    /// 指令字节流。
    code: Vec<u8>,
    /// SSA 值到局部变量索引的映射。
    value_locals: HashMap<u32, u32>,
    /// 命名变量到局部变量索引的映射。
    named_locals: HashMap<String, u32>,
    /// 下一个可用的局部变量索引。
    next_local: u32,
}

impl FunctionLoweringContext {
    /// 创建函数 lowering 上下文。
    ///
    /// 函数参数占用局部变量 `0..param_count`。
    fn new(func: &LirFunction) -> Self {
        let param_count = func.param_types.len() as u32;
        Self {
            locals_decl: Vec::new(),
            local_group_count: 0,
            code: Vec::new(),
            value_locals: HashMap::new(),
            named_locals: HashMap::new(),
            next_local: param_count,
        }
    }

    /// 为 SSA 值分配局部变量索引。
    fn alloc_local(&mut self, value: u32, ty: Option<u8>) -> u32 {
        if let Some(idx) = self.value_locals.get(&value) {
            return *idx;
        }
        let idx = self.next_local;
        self.value_locals.insert(value, idx);
        self.next_local += 1;

        let wasm_type = ty.unwrap_or(VAL_I32);
        write_uleb128(&mut self.locals_decl, 1);
        self.locals_decl.push(wasm_type);
        self.local_group_count += 1;

        idx
    }

    /// 为命名变量分配局部变量索引。
    fn alloc_named_local(&mut self, name: &str, ty: Option<u8>) -> u32 {
        if let Some(idx) = self.named_locals.get(name) {
            return *idx;
        }
        let idx = self.next_local;
        self.named_locals.insert(name.to_string(), idx);
        self.next_local += 1;

        let wasm_type = ty.unwrap_or(VAL_I32);
        write_uleb128(&mut self.locals_decl, 1);
        self.locals_decl.push(wasm_type);
        self.local_group_count += 1;

        idx
    }

    /// 发射单字节操作码。
    fn emit(&mut self, opcode: u8) {
        self.code.push(opcode);
    }

    /// 发射 ULEB128 操作数。
    fn emit_uleb128(&mut self, value: u32) {
        write_uleb128(&mut self.code, value);
    }

    /// 发射 i32.const 指令。
    fn emit_i32_const(&mut self, value: i32) {
        self.emit(OP_I32_CONST);
        write_sleb128_i32(&mut self.code, value);
    }

    /// 发射 i64.const 指令。
    fn emit_i64_const(&mut self, value: i64) {
        self.emit(OP_I64_CONST);
        write_sleb128_i64(&mut self.code, value);
    }

    /// 发射 local.get 指令。
    fn emit_local_get(&mut self, idx: u32) {
        self.emit(OP_LOCAL_GET);
        self.emit_uleb128(idx);
    }

    /// 发射 local.set 指令。
    fn emit_local_set(&mut self, idx: u32) {
        self.emit(OP_LOCAL_SET);
        self.emit_uleb128(idx);
    }

    /// 发射 call 指令。
    fn emit_call(&mut self, func_idx: u32) {
        self.emit(OP_CALL);
        self.emit_uleb128(func_idx);
    }

    /// 求值操作数，将其压入 WASM 栈。
    fn emit_operand(&mut self, operand: &LirOperand) {
        match operand {
            LirOperand::Value(v) => {
                let idx = self.value_locals.get(&v.0).copied().unwrap_or(0);
                self.emit_local_get(idx);
            }
            LirOperand::Constant(c) => {
                self.emit_constant(c);
            }
            LirOperand::Symbol(_) => {
                // 符号引用暂不支持，用 i32.const 0 兜底
                self.emit_i32_const(0);
            }
        }
    }

    /// 发射常量值到栈。
    fn emit_constant(&mut self, constant: &MirConstant) {
        match constant {
            MirConstant::Int(i) => {
                if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                    self.emit_i32_const(*i as i32);
                }
                else {
                    self.emit_i64_const(*i);
                }
            }
            MirConstant::Bool(b) => {
                self.emit_i32_const(if *b { 1 } else { 0 });
            }
            MirConstant::String(_) => {
                // 字符串暂不支持，用 i32.const 0 兜底
                self.emit_i32_const(0);
            }
            MirConstant::Unit => {
                // Unit 无值，不压栈
            }
        }
    }

    /// 降低单条操作。
    fn lower_operation(&mut self, op: &LirOperation, func_index_map: &HashMap<String, u32>, return_type_map: &HashMap<String, HirType>) {
        match &op.kind {
            LirOperationKind::LoadConstant { constant } => {
                if let Some(output) = &op.output {
                    let ty = mir_constant_to_wasm_type(constant);
                    let local = self.alloc_local(output.0, ty);
                    self.emit_constant(constant);
                    self.emit_local_set(local);
                }
            }
            LirOperationKind::LoadSymbol { .. } => {
                if let Some(output) = &op.output {
                    let local = self.alloc_local(output.0, None);
                    self.emit_i32_const(0);
                    self.emit_local_set(local);
                }
            }
            LirOperationKind::Move { source } => {
                if let Some(output) = &op.output {
                    let local = self.alloc_local(output.0, None);
                    self.emit_operand(source);
                    self.emit_local_set(local);
                }
            }
            LirOperationKind::StoreVar { name, value, ty } => {
                let wasm_ty = ty.as_ref().and_then(hir_type_to_wasm);
                let local = self.alloc_named_local(name, wasm_ty);
                self.emit_operand(value);
                self.emit_local_set(local);
                // 将 StoreVar 的输出 SSA 值绑定到同一命名 local，
                // 使后续通过 MirOperand::Value(output) 的读取能解析到该 local。
                // 这是循环中变量重赋值能正确工作的关键：
                // MIR 不使用 phi 节点，而是依赖后端把同名 StoreVar 读写路由到同一 local。
                if let Some(output) = &op.output {
                    self.value_locals.insert(output.0, local);
                }
            }
            LirOperationKind::Call { callee, arguments, .. } => {
                // 求值参数（按顺序压栈）
                for arg in arguments {
                    self.emit_operand(arg);
                }

                // 解析被调用函数名，用于查询返回类型
                let func_name = if let LirOperand::Symbol(path) = callee { path.0.last().map(|i| i.as_str()).unwrap_or("") } else { "" };

                // 发射 call 指令
                if let Some(idx) = func_index_map.get(func_name) {
                    self.emit_call(*idx);
                }
                else {
                    self.emit(OP_UNREACHABLE);
                }

                // 存储调用结果到局部变量
                // 仅当函数返回非 unit/void 时才生成 local.set，否则 WASM 栈上无返回值
                let returns_value = return_type_map.get(func_name).map(|ty| !matches!(ty, HirType::Unit | HirType::Void)).unwrap_or(true);

                if returns_value {
                    if let Some(output) = &op.output {
                        let local = self.alloc_local(output.0, None);
                        self.emit_local_set(local);
                    }
                }
            }
            LirOperationKind::Subscript { .. } | LirOperationKind::StoreSubscript { .. } | LirOperationKind::ArrayNew { .. } => {
                // 暂不支持数组操作
                self.emit(OP_UNREACHABLE);
            }
            LirOperationKind::StructNew { .. } | LirOperationKind::FieldGet { .. } | LirOperationKind::FieldSet { .. } => {
                // 暂不支持结构体操作
                self.emit(OP_UNREACHABLE);
            }
        }
    }

    /// 降低块终结符。
    fn lower_terminator(&mut self, terminator: &LirTerminator) {
        match terminator {
            LirTerminator::Return { value } => {
                if let Some(v) = value {
                    self.emit_operand(v);
                    self.emit(OP_RETURN);
                }
                // void 返回，直接 end 即可
            }
            LirTerminator::Jump { .. } => {
                // 多基本块跳转暂不支持，单基本块函数不需要跳转
            }
            LirTerminator::Branch { .. } => {
                // 条件分支暂不支持
                self.emit(OP_UNREACHABLE);
            }
            LirTerminator::Unreachable => {
                self.emit(OP_UNREACHABLE);
            }
        }
    }

    /// 完成函数体编码，返回完整的函数体字节。
    fn finish(self) -> Vec<u8> {
        let mut body = Vec::new();
        // 局部变量声明
        write_uleb128(&mut body, self.local_group_count);
        body.extend_from_slice(&self.locals_decl);
        // 指令
        body.extend_from_slice(&self.code);
        // end 指令（函数体结束）
        body.push(OP_END);
        body
    }
}

/// 降低单个函数的代码体。
///
/// 对于单基本块函数，使用简单的顺序 lowering。
/// 对于多基本块函数，使用递归控制流 lowering，支持 `while` 循环和 `if`/`else` 分支。
fn lower_function_body(func: &LirFunction, func_index_map: &HashMap<String, u32>, return_type_map: &HashMap<String, HirType>) -> Vec<u8> {
    let mut ctx = FunctionLoweringContext::new(func);

    if func.blocks.len() <= 1 {
        // 单基本块函数：使用原有的简单 lowering
        for block in &func.blocks {
            lower_block(&mut ctx, block, func_index_map, return_type_map);
        }
    }
    else {
        // 多基本块函数：使用控制流 lowering
        let loop_headers = find_loop_headers(&func.blocks);
        let mut visited = HashSet::new();
        lower_blocks_recursive(&mut ctx, &func.blocks, 0, &mut visited, &loop_headers, func_index_map, return_type_map, 0);
    }

    ctx.finish()
}

/// 查找循环头块（回边目标）。
///
/// 如果一个 `Jump` 终结符的目标块索引小于当前块索引，
/// 说明存在回边，目标块是循环头。
fn find_loop_headers(blocks: &[LirBlock]) -> HashSet<MirBlockRef> {
    let mut headers = HashSet::new();
    for (i, block) in blocks.iter().enumerate() {
        if let LirTerminator::Jump { target, .. } = &block.terminator {
            if (target.0 as usize) < i {
                headers.insert(*target);
            }
        }
    }
    headers
}

/// 查找块 `ID` 对应的索引。
fn find_block_index(blocks: &[LirBlock], id: MirBlockRef) -> usize {
    blocks.iter().position(|b| b.id == id).unwrap_or(0)
}

/// 递归降低多基本块函数，支持结构化控制流。
///
/// `if_depth` 参数跟踪当前 `if`/`else` 嵌套深度（相对于最内层循环），
/// 用于计算回边 `br` 的目标深度。
fn lower_blocks_recursive(
    ctx: &mut FunctionLoweringContext,
    blocks: &[LirBlock],
    current: usize,
    visited: &mut HashSet<usize>,
    loop_headers: &HashSet<MirBlockRef>,
    func_index_map: &HashMap<String, u32>,
    return_type_map: &HashMap<String, HirType>,
    if_depth: u32,
) {
    if current >= blocks.len() || visited.contains(&current) {
        return;
    }
    visited.insert(current);

    let block = &blocks[current];
    let is_loop_header = loop_headers.contains(&block.id);

    if is_loop_header {
        // 循环头：发射 block (exit) + loop (header)
        ctx.emit(OP_BLOCK);
        ctx.emit(0x40); // void block type
        ctx.emit(OP_LOOP);
        ctx.emit(0x40); // void block type

        // 降低块操作（条件求值）
        for op in &block.operations {
            ctx.lower_operation(op, func_index_map, return_type_map);
        }

        // 降低 Branch 终结符：使用 eqz + br_if 模式
        match &block.terminator {
            LirTerminator::Branch { condition, then_target, else_target } => {
                // 发射条件，取反，如果为假则退出循环
                ctx.emit_operand(condition);
                ctx.emit(OP_I32_EQZ);
                ctx.emit(OP_BR_IF);
                ctx.emit_uleb128(1); // br_if 到 block (exit)，深度 1（从 loop 内部）

                // 降低 then 块（循环体），if_depth 重置为 0
                let then_idx = find_block_index(blocks, *then_target);
                lower_blocks_recursive(ctx, blocks, then_idx, visited, loop_headers, func_index_map, return_type_map, 0);

                // 循环体结束后，跳回循环头
                ctx.emit(OP_BR);
                ctx.emit_uleb128(0); // br 到 loop（回边），深度 0
            }
            _ => {
                ctx.emit(OP_UNREACHABLE);
            }
        }

        ctx.emit(OP_END); // end loop
        ctx.emit(OP_END); // end block

        // 降低退出块（else_target）
        if let LirTerminator::Branch { else_target, .. } = &block.terminator {
            let exit_idx = find_block_index(blocks, *else_target);
            lower_blocks_recursive(ctx, blocks, exit_idx, visited, loop_headers, func_index_map, return_type_map, 0);
        }
        return;
    }

    // 非循环头块：降低操作
    for op in &block.operations {
        ctx.lower_operation(op, func_index_map, return_type_map);
    }

    // 降低终结符
    match &block.terminator {
        LirTerminator::Return { value } => {
            if let Some(v) = value {
                ctx.emit_operand(v);
                ctx.emit(OP_RETURN);
            }
        }
        LirTerminator::Jump { target, .. } => {
            let target_idx = find_block_index(blocks, *target);
            if target_idx < current {
                // 回边：br 到循环头
                ctx.emit(OP_BR);
                ctx.emit_uleb128(if_depth); // br 到最内层 loop
            }
            else {
                // 前向跳转：递归降低目标块
                lower_blocks_recursive(ctx, blocks, target_idx, visited, loop_headers, func_index_map, return_type_map, if_depth);
            }
        }
        LirTerminator::Branch { condition, then_target, else_target } => {
            // 非循环 Branch：使用 if/else/end
            ctx.emit_operand(condition);
            ctx.emit(OP_IF);
            ctx.emit(0x40); // void block type

            let then_idx = find_block_index(blocks, *then_target);
            lower_blocks_recursive(ctx, blocks, then_idx, visited, loop_headers, func_index_map, return_type_map, if_depth + 1);

            ctx.emit(OP_ELSE);

            let else_idx = find_block_index(blocks, *else_target);
            lower_blocks_recursive(ctx, blocks, else_idx, visited, loop_headers, func_index_map, return_type_map, if_depth + 1);

            ctx.emit(OP_END);
        }
        LirTerminator::Unreachable => {
            ctx.emit(OP_UNREACHABLE);
        }
    }
}

/// 降低单个基本块。
fn lower_block(
    ctx: &mut FunctionLoweringContext,
    block: &LirBlock,
    func_index_map: &HashMap<String, u32>,
    return_type_map: &HashMap<String, HirType>,
) {
    // 降低非终结操作
    for op in &block.operations {
        ctx.lower_operation(op, func_index_map, return_type_map);
    }

    // 降低终结符
    ctx.lower_terminator(&block.terminator);
}

/// 获取常量的 WASM 值类型。
fn mir_constant_to_wasm_type(constant: &MirConstant) -> Option<u8> {
    match constant {
        MirConstant::Int(i) => {
            if *i >= i32::MIN as i64 && *i <= i32::MAX as i64 {
                Some(VAL_I32)
            }
            else {
                Some(VAL_I64)
            }
        }
        MirConstant::Bool(_) => Some(VAL_I32),
        MirConstant::String(_) => Some(VAL_I32),
        MirConstant::Unit => None,
    }
}

/// 生成签名的唯一键。
fn signature_key(params: &[u8], results: &[u8]) -> String {
    let mut key = String::new();
    for &p in params {
        key.push_str(&format!("{:02x}", p));
    }
    key.push('_');
    for &r in results {
        key.push_str(&format!("{:02x}", r));
    }
    key
}

/// 编码 ULEB128 无符号整数。
fn write_uleb128(output: &mut Vec<u8>, mut value: u32) {
    loop {
        let byte = (value & 0x7F) as u8;
        value >>= 7;
        if value != 0 {
            output.push(byte | 0x80);
        }
        else {
            output.push(byte);
            break;
        }
    }
}

/// 编码 SLEB128 有符号 32 位整数。
fn write_sleb128_i32(output: &mut Vec<u8>, value: i32) {
    let mut v = value;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        let done = (v == 0 && (byte & 0x40) == 0) || (v == -1 && (byte & 0x40) != 0);
        if done {
            output.push(byte);
            break;
        }
        else {
            output.push(byte | 0x80);
        }
    }
}

/// 编码 SLEB128 有符号 64 位整数。
fn write_sleb128_i64(output: &mut Vec<u8>, value: i64) {
    let mut v = value;
    loop {
        let byte = (v & 0x7F) as u8;
        v >>= 7;
        let done = (v == 0 && (byte & 0x40) == 0) || (v == -1 && (byte & 0x40) != 0);
        if done {
            output.push(byte);
            break;
        }
        else {
            output.push(byte | 0x80);
        }
    }
}

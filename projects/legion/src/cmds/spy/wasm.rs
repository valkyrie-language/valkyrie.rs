//! legion spy wasm 子模式：WASM 二进制反汇编与分析。
//!
//! 支持的能力：
//! - 解析任意 `WASM` 二进制（基于 `wasi-backend` 的 `WasmBinaryModule`）
//! - 列出所有段（`--list`）
//! - 列出所有函数（`--list --func` 或默认）
//! - 反汇编指定函数（`--func <index|name>`）
//! - 查看 `imports` / `exports`
//! - 按绝对偏移定位错误上下文（`--offset <abs>`）
//! - `JSON` 输出（`--json`）
//! - 函数体原始字节 `dump`（`--hex`）

use std::{fs, path::Path, process::ExitCode};

use miette::{miette, IntoDiagnostic, Result};
use wasi_backend::WasmBinaryModule;

use super::{SpyOptions, SpyTargetOptions};

/// WASM 段 id 常量。
const SECTION_CUSTOM: u8 = 0;
const SECTION_TYPE: u8 = 1;
const SECTION_IMPORT: u8 = 2;
const SECTION_FUNCTION: u8 = 3;
const SECTION_TABLE: u8 = 4;
const SECTION_MEMORY: u8 = 5;
const SECTION_GLOBAL: u8 = 6;
const SECTION_EXPORT: u8 = 7;
const SECTION_START: u8 = 8;
const SECTION_ELEMENT: u8 = 9;
const SECTION_CODE: u8 = 10;
const SECTION_DATA: u8 = 11;
const SECTION_DATA_COUNT: u8 = 12;

/// 执行 WASM 二进制反汇编。
///
/// 用法：`legion spy wasm <file> [--func <index|name>] [--list] [--offset <abs>] [--json] [--hex]`
pub fn run(options: &SpyOptions) -> Result<ExitCode> {
    let (_, opts) = options.split();
    let Some(target) = &opts.input
    else {
        return Err(miette!(
            "用法：legion spy wasm <file> [--func <index|name>] [--list] [--offset <abs>] [--json] [--hex]\n  file             目标 WASM 二进制文件\n  --func <i|name>  反汇编指定函数（索引或名称）\n  --list           列出所有函数 / 段\n  --offset <abs>   定位绝对偏移量处的指令\n  --json           以 JSON 格式输出\n  --hex            dump 函数体原始字节（配合 --func）"
        ));
    };

    if !Path::exists(Path::new(target)) {
        return Err(miette!("文件不存在：{}", target));
    }

    let data = fs::read(target).into_diagnostic().map_err(|error| error.wrap_err(format!("无法读取文件 {}", target)))?;

    let module = WasmBinaryModule::from_bytes(&data).map_err(|error| miette!("WASM 解析失败：{error}"))?;

    // 偏移定位模式优先级最高
    if let Some(offset) = opts.offset {
        return locate_offset(&data, offset as usize, opts);
    }

    // 函数反汇编模式
    if let Some(func_spec) = &opts.func {
        return disassemble_function(&module, &data, func_spec, opts);
    }

    // 默认 / 列表模式
    print_overview(&module, &data, opts);
    Ok(ExitCode::SUCCESS)
}

/// 打印模块概览。
fn print_overview(module: &WasmBinaryModule, data: &[u8], opts: &SpyTargetOptions) {
    if opts.json {
        print_json_overview(module, data);
        return;
    }

    println!("WASM 模块（{} 字节，版本 {}）", data.len(), module.version);
    println!("段数：{}", module.sections.len());
    println!();

    // 段摘要
    println!("=== 段列表 ===");
    for (index, section) in module.sections.iter().enumerate() {
        let section_name = section_name(section.id);
        let detail =
            if section.id == SECTION_CUSTOM { section.name.clone().unwrap_or_default() } else { format!("{} 字节", section.bytes.len()) };
        println!("  [{}] id={} {:<16} {}", index, section.id, section_name, detail);
    }
    println!();

    // imports
    let imports = parse_imports(module);
    if !imports.is_empty() {
        println!("=== Imports（{} 项）===", imports.len());
        for imp in &imports {
            println!("  {:<12} {}.{} : {}", imp.kind_str(), imp.module, imp.field, imp.type_str());
        }
        println!();
    }

    // exports
    let exports = parse_exports(module);
    if !exports.is_empty() {
        println!("=== Exports（{} 项）===", exports.len());
        for exp in &exports {
            println!("  {:<12} {} : index={}", exp.kind_str(), exp.name, exp.index);
        }
        println!();
    }

    // 函数列表
    let functions = parse_functions(module);
    if !functions.is_empty() {
        println!("=== 函数（{} 项）===", functions.len());
        for func in &functions {
            let name_hint = func.name.as_deref().unwrap_or("<unnamed>");
            println!("  [{}] {} ({} 字节, {} 局部变量组)", func.index, name_hint, func.body_len, func.local_groups);
        }
    }
    else {
        println!("（无 Code 段，模块不含函数体）");
    }
}

/// 反汇编指定函数。
fn disassemble_function(module: &WasmBinaryModule, data: &[u8], func_spec: &str, opts: &SpyTargetOptions) -> Result<ExitCode> {
    let functions = parse_functions(module);

    // 按索引或名称查找函数
    let target_func = if let Ok(index) = func_spec.parse::<usize>() {
        functions.iter().find(|f| f.index == index)
    }
    else {
        functions.iter().find(|f| f.name.as_deref() == Some(func_spec))
    };

    let Some(func) = target_func
    else {
        return Err(miette!(
            "未找到函数 '{}'，当前共有 {} 个函数（索引 0..{}）",
            func_spec,
            functions.len(),
            functions.len().saturating_sub(1)
        ));
    };

    if opts.hex {
        // 原始字节 dump
        let body = read_function_body(data, func);
        print_hex_dump(&body, func);
        return Ok(ExitCode::SUCCESS);
    }

    if opts.json {
        print_json_function(func, data);
        return Ok(ExitCode::SUCCESS);
    }

    // 反汇编
    println!("=== 函数 {} ===", func.index);
    if let Some(name) = &func.name {
        println!("名称: {}", name);
    }
    println!("代码偏移: 0x{:04X}", func.code_offset);
    println!("代码长度: {} 字节", func.body_len);
    println!("局部变量组: {}", func.local_groups);
    println!();

    let instructions = disassemble_function_body(data, func);
    print_instructions(&instructions);
    Ok(ExitCode::SUCCESS)
}

/// 按绝对偏移定位指令上下文。
fn locate_offset(data: &[u8], offset: usize, opts: &SpyTargetOptions) -> Result<ExitCode> {
    if offset >= data.len() {
        return Err(miette!("偏移 {} 超出文件长度 {}", offset, data.len()));
    }

    let module = WasmBinaryModule::from_bytes(data).map_err(|error| miette!("WASM 解析失败：{error}"))?;

    let functions = parse_functions(&module);

    // 查找包含该偏移的函数
    let containing = functions.iter().find(|f| {
        let start = f.code_offset;
        let end = f.code_offset + f.body_len;
        offset >= start && offset < end
    });

    if opts.json {
        print_json_offset(offset, containing, data);
        return Ok(ExitCode::SUCCESS);
    }

    if let Some(func) = containing {
        println!("偏移 0x{:04X} 位于函数 {} 中", offset, func.index);
        if let Some(name) = &func.name {
            println!("函数名: {}", name);
        }
        println!("函数代码范围: 0x{:04X}..0x{:04X}", func.code_offset, func.code_offset + func.body_len);
        println!();

        // 反汇编该函数并高亮目标偏移
        let instructions = disassemble_function_body(data, func);
        let context = opts.context;
        let target_pos =
            instructions.iter().position(|instr| instr.offset == offset || (instr.offset <= offset && offset < instr.offset + instr.size));

        match target_pos {
            Some(center) => {
                let start = center.saturating_sub(context);
                let end = (center + context + 1).min(instructions.len());
                for (i, instr) in instructions[start..end].iter().enumerate() {
                    let marker = if i == center - start { ">>>" } else { "   " };
                    println!("{} {:>4}:  {:<20} {}", marker, instr.offset, instr.mnemonic, instr.operand.as_deref().unwrap_or(""));
                }
            }
            None => {
                println!("（未找到精确匹配的指令，附近字节：）");
                let start = offset.saturating_sub(16);
                let end = (offset + 16).min(data.len());
                print_hex_range(data, start, end, offset);
            }
        }
    }
    else {
        println!("偏移 0x{:04X} 不在任何函数体内", offset);
        // 查找该偏移属于哪个段
        let mut cursor = 8; // 跳过 magic + version
        for section in &module.sections {
            let section_start = cursor;
            let name_overhead = if section.id == SECTION_CUSTOM {
                section.name.as_ref().map(|n| n.len() + uleb128_size(n.len() as u32)).unwrap_or(0)
            }
            else {
                0
            };
            let payload_len = name_overhead + section.bytes.len();
            let section_end = cursor + 1 + uleb128_size(payload_len as u32) + payload_len;
            if offset >= section_start && offset < section_end {
                println!("位于段 id={} ({})", section.id, section_name(section.id));
                break;
            }
            cursor = section_end;
        }
        println!();
        let start = offset.saturating_sub(16);
        let end = (offset + 16).min(data.len());
        print_hex_range(data, start, end, offset);
    }

    Ok(ExitCode::SUCCESS)
}

// ========== WASM 段解析 ==========

/// 导入项。
#[derive(Debug, Clone)]
struct WasmImport {
    module: String,
    field: String,
    kind: u8,
    /// 函数类型索引（kind=0 时有效）。
    type_index: u32,
    /// 表元素类型（kind=1 时有效）。
    table_elem_type: u8,
    /// 内存限制（kind=2 时有效）。
    memory_min: u32,
    memory_max: Option<u32>,
    /// 全局变量类型（kind=3 时有效）。
    global_value_type: u8,
    global_mutable: bool,
}

impl WasmImport {
    fn kind_str(&self) -> &'static str {
        match self.kind {
            0 => "func",
            1 => "table",
            2 => "memory",
            3 => "global",
            _ => "unknown",
        }
    }

    fn type_str(&self) -> String {
        match self.kind {
            0 => format!("type={}", self.type_index),
            1 => format!("elem={:02X}", self.table_elem_type),
            2 => match self.memory_max {
                Some(max) => format!("limits={}..{}", self.memory_min, max),
                None => format!("limits={}", self.memory_min),
            },
            3 => format!("valtype={:02X}, mut={}", self.global_value_type, self.global_mutable),
            _ => "?".to_string(),
        }
    }
}

/// 导出项。
#[derive(Debug, Clone)]
struct WasmExport {
    name: String,
    kind: u8,
    index: u32,
}

impl WasmExport {
    fn kind_str(&self) -> &'static str {
        match self.kind {
            0 => "func",
            1 => "table",
            2 => "memory",
            3 => "global",
            _ => "unknown",
        }
    }
}

/// 函数信息。
#[derive(Debug, Clone)]
struct WasmFunction {
    index: usize,
    name: Option<String>,
    code_offset: usize,
    body_len: usize,
    local_groups: usize,
}

/// 反汇编后的指令。
#[derive(Debug, Clone)]
struct WasmInstruction {
    offset: usize,
    size: usize,
    mnemonic: String,
    operand: Option<String>,
}

/// 解析 Import 段。
fn parse_imports(module: &WasmBinaryModule) -> Vec<WasmImport> {
    let Some(import_section) = module.sections.iter().find(|s| s.id == SECTION_IMPORT)
    else {
        return Vec::new();
    };

    let mut reader = ByteReader::new(&import_section.bytes);
    let count = match reader.read_uleb128() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut imports = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let module_name = match reader.read_string() {
            Ok(s) => s,
            Err(_) => break,
        };
        let field_name = match reader.read_string() {
            Ok(s) => s,
            Err(_) => break,
        };
        let kind = match reader.read_u8() {
            Ok(k) => k,
            Err(_) => break,
        };

        let mut imp = WasmImport {
            module: module_name,
            field: field_name,
            kind,
            type_index: 0,
            table_elem_type: 0,
            memory_min: 0,
            memory_max: None,
            global_value_type: 0,
            global_mutable: false,
        };

        match kind {
            0 => {
                if let Ok(idx) = reader.read_uleb128() {
                    imp.type_index = idx;
                }
            }
            1 => {
                if let Ok(elem_type) = reader.read_u8() {
                    imp.table_elem_type = elem_type;
                }
                if let Ok(flags) = reader.read_u8() {
                    if flags & 0x01 != 0 {
                        if let Ok(min) = reader.read_uleb128() {
                            imp.memory_min = min;
                        }
                        if let Ok(max) = reader.read_uleb128() {
                            imp.memory_max = Some(max);
                        }
                    }
                    else {
                        if let Ok(min) = reader.read_uleb128() {
                            imp.memory_min = min;
                        }
                    }
                }
            }
            2 => {
                if let Ok(flags) = reader.read_u8() {
                    if let Ok(min) = reader.read_uleb128() {
                        imp.memory_min = min;
                    }
                    if flags & 0x01 != 0 {
                        if let Ok(max) = reader.read_uleb128() {
                            imp.memory_max = Some(max);
                        }
                    }
                }
            }
            3 => {
                if let Ok(val_type) = reader.read_u8() {
                    imp.global_value_type = val_type;
                }
                if let Ok(mutability) = reader.read_u8() {
                    imp.global_mutable = mutability != 0;
                }
            }
            _ => break,
        }

        imports.push(imp);
    }

    imports
}

/// 解析 Export 段。
fn parse_exports(module: &WasmBinaryModule) -> Vec<WasmExport> {
    let Some(export_section) = module.sections.iter().find(|s| s.id == SECTION_EXPORT)
    else {
        return Vec::new();
    };

    let mut reader = ByteReader::new(&export_section.bytes);
    let count = match reader.read_uleb128() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let mut exports = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let name = match reader.read_string() {
            Ok(s) => s,
            Err(_) => break,
        };
        let kind = match reader.read_u8() {
            Ok(k) => k,
            Err(_) => break,
        };
        let index = match reader.read_uleb128() {
            Ok(i) => i,
            Err(_) => break,
        };
        exports.push(WasmExport { name, kind, index });
    }

    exports
}

/// 解析 Code 段中的函数列表。
fn parse_functions(module: &WasmBinaryModule) -> Vec<WasmFunction> {
    let Some(code_section) = module.sections.iter().find(|s| s.id == SECTION_CODE)
    else {
        return Vec::new();
    };

    // 计算段在文件中的绝对偏移
    let section_abs_offset = compute_section_offset(module, SECTION_CODE);

    let mut reader = ByteReader::new(&code_section.bytes);
    let count = match reader.read_uleb128() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    // 导入函数数量（用于计算函数索引基址）
    let import_func_count = parse_imports(module).iter().filter(|i| i.kind == 0).count();

    // 从 Export 段提取函数名
    let export_names = parse_exports(module);
    let mut func_names = std::collections::HashMap::new();
    for exp in &export_names {
        if exp.kind == 0 {
            func_names.insert(exp.index as usize, exp.name.clone());
        }
    }

    let mut functions = Vec::with_capacity(count as usize);

    for i in 0..count {
        let func_index = import_func_count + i as usize;

        let body_size = match reader.read_uleb128() {
            Ok(s) => s as usize,
            Err(_) => break,
        };

        // 函数体在段内的偏移
        let body_offset_in_section = reader.offset;
        // 函数体在文件中的绝对偏移（reader.offset 已指向 body 开始）
        let code_offset = section_abs_offset + reader.offset;

        // 读取局部变量组
        let local_groups = match reader.read_uleb128() {
            Ok(g) => g as usize,
            Err(_) => break,
        };

        // 跳过局部变量声明
        for _ in 0..local_groups {
            let _ = reader.read_uleb128();
            let _ = reader.read_u8();
        }

        // 跳过整个函数体
        let body_end = body_offset_in_section + body_size;
        if body_end > code_section.bytes.len() {
            break;
        }
        reader.offset = body_end;

        functions.push(WasmFunction {
            index: func_index,
            name: func_names.get(&func_index).cloned(),
            code_offset,
            body_len: body_size,
            local_groups,
        });
    }

    functions
}

/// 计算指定段在文件中的绝对偏移。
fn compute_section_offset(module: &WasmBinaryModule, target_id: u8) -> usize {
    let mut offset = 8; // magic(4) + version(4)
    for section in &module.sections {
        let id_field_size = 1;
        let name_overhead =
            if section.id == SECTION_CUSTOM { section.name.as_ref().map(|n| n.len() + uleb128_size(n.len() as u32)).unwrap_or(0) } else { 0 };
        let payload_len = if section.id == SECTION_CUSTOM { name_overhead + section.bytes.len() } else { section.bytes.len() };
        let len_field_size = uleb128_size(payload_len as u32);

        if section.id == target_id {
            return offset + id_field_size + len_field_size + name_overhead;
        }

        offset += id_field_size + len_field_size + payload_len;
    }
    offset
}

/// 读取函数体原始字节。
fn read_function_body(data: &[u8], func: &WasmFunction) -> Vec<u8> {
    let start = func.code_offset;
    let end = (start + func.body_len).min(data.len());
    data[start..end].to_vec()
}

/// 反汇编函数体指令。
fn disassemble_function_body(data: &[u8], func: &WasmFunction) -> Vec<WasmInstruction> {
    let body_start = func.code_offset;
    let body_end = (body_start + func.body_len).min(data.len());

    let mut reader = ByteReader::new(&data[body_start..body_end]);
    let mut instructions = Vec::new();

    // 跳过局部变量声明
    let local_groups = match reader.read_uleb128() {
        Ok(g) => g,
        Err(_) => return instructions,
    };
    for _ in 0..local_groups {
        let _ = reader.read_uleb128();
        let _ = reader.read_u8();
    }

    // 反汇编指令直到 end
    loop {
        if reader.is_eof() {
            break;
        }

        let instr_offset = body_start + reader.offset;
        let opcode = match reader.read_u8() {
            Ok(b) => b,
            Err(_) => break,
        };

        let (mnemonic, operand, _) = decode_instruction(opcode, &mut reader, instr_offset);
        let total_size = reader.offset - (instr_offset - body_start);

        instructions.push(WasmInstruction { offset: instr_offset, size: total_size, mnemonic, operand });

        // end (0x0B) 表示函数结束
        if opcode == 0x0B {
            break;
        }
    }

    instructions
}

/// 解码单条指令。
fn decode_instruction(opcode: u8, reader: &mut ByteReader, _offset: usize) -> (String, Option<String>, usize) {
    match opcode {
        // 控制流
        0x00 => ("unreachable".to_string(), None, 1),
        0x01 => ("nop".to_string(), None, 1),
        0x02 => {
            let bt = reader.read_block_type().ok();
            ("block".to_string(), bt.map(|b| block_type_str(b)), 1)
        }
        0x03 => {
            let bt = reader.read_block_type().ok();
            ("loop".to_string(), bt.map(|b| block_type_str(b)), 1)
        }
        0x04 => {
            let bt = reader.read_block_type().ok();
            ("if".to_string(), bt.map(|b| block_type_str(b)), 1)
        }
        0x05 => ("else".to_string(), None, 1),
        0x0B => ("end".to_string(), None, 1),
        0x0C => {
            let l = reader.read_uleb128().ok();
            ("br".to_string(), l.map(|v| v.to_string()), 1)
        }
        0x0D => {
            let l = reader.read_uleb128().ok();
            ("br_if".to_string(), l.map(|v| v.to_string()), 1)
        }
        0x0E => {
            let n = reader.read_uleb128().ok();
            let mut targets = Vec::new();
            if let Some(n) = n {
                for _ in 0..n {
                    if let Ok(t) = reader.read_uleb128() {
                        targets.push(t);
                    }
                }
            }
            let default = reader.read_uleb128().ok();
            ("br_table".to_string(), Some(format!("targets={:?}, default={:?}", targets, default)), 1)
        }
        0x0F => ("return".to_string(), None, 1),
        0x10 => {
            let f = reader.read_uleb128().ok();
            ("call".to_string(), f.map(|v| v.to_string()), 1)
        }
        0x11 => {
            let t = reader.read_uleb128().ok();
            let idx = reader.read_uleb128().ok();
            ("call_indirect".to_string(), Some(format!("type={}, table={:?}", t.unwrap_or(0), idx)), 1)
        }
        // 参数指令
        0x1A => ("drop".to_string(), None, 1),
        0x1B => ("select".to_string(), None, 1),
        0x1C => {
            let n = reader.read_uleb128().ok();
            let mut types = Vec::new();
            if let Some(n) = n {
                for _ in 0..n {
                    if let Ok(t) = reader.read_u8() {
                        types.push(t);
                    }
                }
            }
            ("select".to_string(), Some(format!("types={:?}", types)), 1)
        }
        // 变量指令
        0x20 => {
            let l = reader.read_uleb128().ok();
            ("local.get".to_string(), l.map(|v| v.to_string()), 1)
        }
        0x21 => {
            let l = reader.read_uleb128().ok();
            ("local.set".to_string(), l.map(|v| v.to_string()), 1)
        }
        0x22 => {
            let l = reader.read_uleb128().ok();
            ("local.tee".to_string(), l.map(|v| v.to_string()), 1)
        }
        0x23 => {
            let g = reader.read_uleb128().ok();
            ("global.get".to_string(), g.map(|v| v.to_string()), 1)
        }
        0x24 => {
            let g = reader.read_uleb128().ok();
            ("global.set".to_string(), g.map(|v| v.to_string()), 1)
        }
        // 内存指令
        0x28 => ("i32.load".to_string(), read_memarg(reader), 1),
        0x29 => ("i64.load".to_string(), read_memarg(reader), 1),
        0x2A => ("f32.load".to_string(), read_memarg(reader), 1),
        0x2B => ("f64.load".to_string(), read_memarg(reader), 1),
        0x2C => ("i32.load8_s".to_string(), read_memarg(reader), 1),
        0x2D => ("i32.load8_u".to_string(), read_memarg(reader), 1),
        0x2E => ("i32.load16_s".to_string(), read_memarg(reader), 1),
        0x2F => ("i32.load16_u".to_string(), read_memarg(reader), 1),
        0x36 => ("i32.store".to_string(), read_memarg(reader), 1),
        0x37 => ("i64.store".to_string(), read_memarg(reader), 1),
        0x38 => ("f32.store".to_string(), read_memarg(reader), 1),
        0x39 => ("f64.store".to_string(), read_memarg(reader), 1),
        0x3A => ("i32.store8".to_string(), read_memarg(reader), 1),
        0x3B => ("i32.store16".to_string(), read_memarg(reader), 1),
        0x3F => {
            let _ = reader.read_u8();
            ("memory.size".to_string(), None, 2)
        }
        0x40 => {
            let _ = reader.read_u8();
            ("memory.grow".to_string(), None, 2)
        }
        // 常量指令
        0x41 => {
            let v = reader.read_sleb128_i32().ok();
            ("i32.const".to_string(), v.map(|v| v.to_string()), 1)
        }
        0x42 => {
            let v = reader.read_sleb128_i64().ok();
            ("i64.const".to_string(), v.map(|v| v.to_string()), 1)
        }
        0x43 => {
            let v = reader.read_f32().ok();
            ("f32.const".to_string(), v.map(|v| v.to_string()), 1)
        }
        0x44 => {
            let v = reader.read_f64().ok();
            ("f64.const".to_string(), v.map(|v| v.to_string()), 1)
        }
        // 比较指令
        0x45 => ("i32.eqz".to_string(), None, 1),
        0x46 => ("i32.eq".to_string(), None, 1),
        0x47 => ("i32.ne".to_string(), None, 1),
        0x48 => ("i32.lt_s".to_string(), None, 1),
        0x49 => ("i32.lt_u".to_string(), None, 1),
        0x4A => ("i32.gt_s".to_string(), None, 1),
        0x4B => ("i32.gt_u".to_string(), None, 1),
        0x4C => ("i32.le_s".to_string(), None, 1),
        0x4D => ("i32.le_u".to_string(), None, 1),
        0x4E => ("i32.ge_s".to_string(), None, 1),
        0x4F => ("i32.ge_u".to_string(), None, 1),
        // 数值指令
        0x6A => ("i32.add".to_string(), None, 1),
        0x6B => ("i32.sub".to_string(), None, 1),
        0x6C => ("i32.mul".to_string(), None, 1),
        0x6D => ("i32.div_s".to_string(), None, 1),
        0x6E => ("i32.div_u".to_string(), None, 1),
        0x6F => ("i32.rem_s".to_string(), None, 1),
        0x70 => ("i32.rem_u".to_string(), None, 1),
        0x71 => ("i32.and".to_string(), None, 1),
        0x72 => ("i32.or".to_string(), None, 1),
        0x73 => ("i32.xor".to_string(), None, 1),
        0x74 => ("i32.shl".to_string(), None, 1),
        0x75 => ("i32.shr_s".to_string(), None, 1),
        0x76 => ("i32.shr_u".to_string(), None, 1),
        // 参考指令
        0xD0 => {
            let t = reader.read_uleb128().ok();
            ("ref.null".to_string(), t.map(|v| format!("0x{:02X}", v)), 1)
        }
        0xD1 => ("ref.is_null".to_string(), None, 1),
        0xD2 => {
            let x = reader.read_uleb128().ok();
            ("ref.func".to_string(), x.map(|v| v.to_string()), 1)
        }
        // 前缀指令
        0xFC => {
            let sub = reader.read_uleb128().ok();
            let name = match sub.unwrap_or(0) {
                0 => "i32.trunc_f32_s",
                1 => "i32.trunc_f32_u",
                2 => "i32.trunc_f64_s",
                3 => "i32.trunc_f64_u",
                4 => "i64.trunc_f32_s",
                5 => "i64.trunc_f32_u",
                6 => "i64.trunc_f64_s",
                7 => "i64.trunc_f64_u",
                8 => "memory.copy",
                9 => "memory.fill",
                10 => "memory.init",
                11 => "data.drop",
                _ => "fc.unknown",
            };
            (name.to_string(), sub.map(|s| s.to_string()), 1)
        }
        _ => (format!("unknown(0x{:02X})", opcode), None, 1),
    }
}

/// 读取 memarg 并格式化。
fn read_memarg(reader: &mut ByteReader) -> Option<String> {
    let align = reader.read_uleb128().ok()?;
    let offset = reader.read_uleb128().ok()?;
    Some(format!("align={}, offset={}", align, offset))
}

/// 将 block type 转为字符串。
fn block_type_str(bt: i64) -> String {
    match bt {
        -64 => "void".to_string(),
        -1 => "i32".to_string(),
        -2 => "i64".to_string(),
        -3 => "f32".to_string(),
        -4 => "f64".to_string(),
        _ => format!("type={}", bt),
    }
}

/// 段名。
fn section_name(id: u8) -> &'static str {
    match id {
        SECTION_CUSTOM => "custom",
        SECTION_TYPE => "type",
        SECTION_IMPORT => "import",
        SECTION_FUNCTION => "function",
        SECTION_TABLE => "table",
        SECTION_MEMORY => "memory",
        SECTION_GLOBAL => "global",
        SECTION_EXPORT => "export",
        SECTION_START => "start",
        SECTION_ELEMENT => "element",
        SECTION_CODE => "code",
        SECTION_DATA => "data",
        SECTION_DATA_COUNT => "data_count",
        _ => "unknown",
    }
}

// ========== 辅助输出 ==========

/// 打印指令列表。
fn print_instructions(instructions: &[WasmInstruction]) {
    println!("=== 指令 ===");
    for instr in instructions {
        println!("  {:>4}:  {:<20} {}", instr.offset, instr.mnemonic, instr.operand.as_deref().unwrap_or(""));
    }
}

/// 打印 hex dump。
fn print_hex_dump(body: &[u8], func: &WasmFunction) {
    println!("=== 函数 {} 原始字节（{} 字节）===", func.index, body.len());
    print_hex_range(body, 0, body.len(), usize::MAX);
}

/// 打印 hex 范围，高亮指定偏移。
fn print_hex_range(data: &[u8], start: usize, end: usize, highlight: usize) {
    let mut offset = start;
    while offset < end {
        let chunk_end = (offset + 16).min(end);
        let chunk = &data[offset..chunk_end];

        let marker = if offset <= highlight && highlight < chunk_end { "*" } else { " " };
        print!("{} {:04X}: ", marker, offset);

        for (i, byte) in chunk.iter().enumerate() {
            if offset + i == highlight {
                print!("[{:02X}]", byte);
            }
            else if i % 8 == 0 && i > 0 {
                print!(" {:02X} ", byte);
            }
            else {
                print!(" {:02X}", byte);
            }
        }

        // 补齐对齐
        for _ in chunk.len()..16 {
            print!("   ");
        }

        print!("  |");
        for byte in chunk {
            let ch = *byte;
            if (32..127).contains(&ch) {
                print!("{}", ch as char);
            }
            else {
                print!(".");
            }
        }
        println!("|");

        offset = chunk_end;
    }
}

// ========== JSON 输出 ==========

fn print_json_overview(module: &WasmBinaryModule, data: &[u8]) {
    let sections: Vec<String> = module
        .sections
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let name = if s.id == SECTION_CUSTOM { s.name.clone().unwrap_or_default() } else { section_name(s.id).to_string() };
            format!("    {{\"index\": {}, \"id\": {}, \"name\": {}, \"size\": {}}}", i, s.id, json_string(&name), s.bytes.len())
        })
        .collect();

    let imports = parse_imports(module);
    let imports_json: Vec<String> = imports
        .iter()
        .map(|i| {
            format!(
                "    {{\"module\": {}, \"field\": {}, \"kind\": {}, \"kind_str\": {}, \"type\": {}}}",
                json_string(&i.module),
                json_string(&i.field),
                i.kind,
                json_string(i.kind_str()),
                json_string(&i.type_str())
            )
        })
        .collect();

    let exports = parse_exports(module);
    let exports_json: Vec<String> = exports
        .iter()
        .map(|e| {
            format!(
                "    {{\"name\": {}, \"kind\": {}, \"kind_str\": {}, \"index\": {}}}",
                json_string(&e.name),
                e.kind,
                json_string(e.kind_str()),
                e.index
            )
        })
        .collect();

    let functions = parse_functions(module);
    let functions_json: Vec<String> = functions
        .iter()
        .map(|f| {
            format!(
                "    {{\"index\": {}, \"name\": {}, \"code_offset\": {}, \"body_len\": {}, \"local_groups\": {}}}",
                f.index,
                json_string(f.name.as_deref().unwrap_or("")),
                f.code_offset,
                f.body_len,
                f.local_groups
            )
        })
        .collect();

    println!(
        "{{\n  \"size\": {},\n  \"version\": {},\n  \"sections\": [\n{}\n  ],\n  \"imports\": [\n{}\n  ],\n  \"exports\": [\n{}\n  ],\n  \"functions\": [\n{}\n  ]\n}}",
        data.len(),
        module.version,
        sections.join(",\n"),
        imports_json.join(",\n"),
        exports_json.join(",\n"),
        functions_json.join(",\n")
    );
}

fn print_json_function(func: &WasmFunction, data: &[u8]) {
    let instructions = disassemble_function_body(data, func);
    let instr_json: Vec<String> = instructions
        .iter()
        .map(|i| {
            format!(
                "    {{\"offset\": {}, \"size\": {}, \"mnemonic\": {}, \"operand\": {}}}",
                i.offset,
                i.size,
                json_string(&i.mnemonic),
                json_string(i.operand.as_deref().unwrap_or(""))
            )
        })
        .collect();

    println!(
        "{{\n  \"index\": {},\n  \"name\": {},\n  \"code_offset\": {},\n  \"body_len\": {},\n  \"local_groups\": {},\n  \"instructions\": [\n{}\n  ]\n}}",
        func.index,
        json_string(func.name.as_deref().unwrap_or("")),
        func.code_offset,
        func.body_len,
        func.local_groups,
        instr_json.join(",\n")
    );
}

fn print_json_offset(offset: usize, func: Option<&WasmFunction>, _data: &[u8]) {
    match func {
        Some(f) => {
            println!(
                "{{\n  \"offset\": {},\n  \"in_function\": true,\n  \"function_index\": {},\n  \"function_name\": {},\n  \"function_range\": [{}, {}]\n}}",
                offset,
                f.index,
                json_string(f.name.as_deref().unwrap_or("")),
                f.code_offset,
                f.code_offset + f.body_len
            );
        }
        None => {
            println!("{{\n  \"offset\": {},\n  \"in_function\": false\n}}", offset);
        }
    }
}

/// JSON 字符串转义。
fn json_string(value: &str) -> String {
    let mut result = String::with_capacity(value.len() + 2);
    result.push('"');
    for ch in value.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if c.is_control() => result.push_str(&format!("\\u{:04x}", c as u32)),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

// ========== LEB128 辅助 ==========

/// 计算 ULEB128 编码占用的字节数。
fn uleb128_size(value: u32) -> usize {
    let mut size = 1;
    let mut v = value >> 7;
    while v != 0 {
        size += 1;
        v >>= 7;
    }
    size
}

/// 字节读取器。
struct ByteReader<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> ByteReader<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn is_eof(&self) -> bool {
        self.offset >= self.bytes.len()
    }

    fn read_u8(&mut self) -> Result<u8, ()> {
        let value = *self.bytes.get(self.offset).ok_or(())?;
        self.offset += 1;
        Ok(value)
    }

    fn read_uleb128(&mut self) -> Result<u32, ()> {
        let mut result = 0u32;
        let mut shift = 0u32;
        loop {
            let byte = self.read_u8()?;
            result |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift > 35 {
                return Err(());
            }
        }
    }

    fn read_sleb128_i32(&mut self) -> Result<i32, ()> {
        let mut result = 0i32;
        let mut shift = 0u32;
        let mut byte;
        loop {
            byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i32) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
            if shift > 35 {
                return Err(());
            }
        }
        if shift < 32 && (byte & 0x40) != 0 {
            result |= (!0i32) << shift;
        }
        Ok(result)
    }

    fn read_sleb128_i64(&mut self) -> Result<i64, ()> {
        let mut result = 0i64;
        let mut shift = 0u32;
        let mut byte;
        loop {
            byte = self.read_u8()?;
            result |= ((byte & 0x7F) as i64) << shift;
            shift += 7;
            if byte & 0x80 == 0 {
                break;
            }
            if shift > 70 {
                return Err(());
            }
        }
        if shift < 64 && (byte & 0x40) != 0 {
            result |= (!0i64) << shift;
        }
        Ok(result)
    }

    fn read_f32(&mut self) -> Result<f32, ()> {
        let bytes = self.bytes.get(self.offset..self.offset + 4).ok_or(())?;
        self.offset += 4;
        Ok(f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn read_f64(&mut self) -> Result<f64, ()> {
        let bytes = self.bytes.get(self.offset..self.offset + 8).ok_or(())?;
        self.offset += 8;
        Ok(f64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]]))
    }

    fn read_block_type(&mut self) -> Result<i64, ()> {
        let byte = self.read_u8()?;
        if byte & 0x80 == 0 {
            // 单字节编码
            Ok(byte as i8 as i64)
        }
        else {
            // 多字节 LEB128（简化处理）
            let mut result = (byte & 0x7F) as i64;
            let mut shift = 7u32;
            loop {
                let b = self.read_u8()?;
                result |= ((b & 0x7F) as i64) << shift;
                if b & 0x80 == 0 {
                    break;
                }
                shift += 7;
            }
            Ok(result)
        }
    }

    fn read_string(&mut self) -> Result<String, ()> {
        let len = self.read_uleb128()? as usize;
        let bytes = self.bytes.get(self.offset..self.offset + len).ok_or(())?;
        self.offset += len;
        String::from_utf8(bytes.to_vec()).map_err(|_| ())
    }
}

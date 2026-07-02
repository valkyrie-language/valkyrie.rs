//! `WASM` 段级二进制解码。
//!
//! 提供 `import` / `export` / `code` / `type` 四段的语义化解析，
//! 返回结构化的段条目模型，供上层工具（如 `spy`）直接消费。

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{
    SECTION_CODE, SECTION_EXPORT, SECTION_IMPORT, SECTION_TYPE, WasmBinaryError, WasmBinaryModule, WasmByteReader, compute_section_offset,
    skip_value_type,
};

/// `WASM` 外部实体种类，用于 `import` / `export` 段条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmExternalKind {
    /// 函数（`kind = 0`）。
    Func,
    /// 表（`kind = 1`）。
    Table,
    /// 内存（`kind = 2`）。
    Memory,
    /// 全局变量（`kind = 3`）。
    Global,
    /// 未知种类，保留原始字节。
    Unknown(u8),
}

impl WasmExternalKind {
    /// 将原始字节映射为 `WasmExternalKind`。
    pub fn from_u8(byte: u8) -> Self {
        match byte {
            0 => Self::Func,
            1 => Self::Table,
            2 => Self::Memory,
            3 => Self::Global,
            other => Self::Unknown(other),
        }
    }

    /// 编码为 import/export kind 字节。
    pub fn as_u8(self) -> u8 {
        match self {
            Self::Func => 0,
            Self::Table => 1,
            Self::Memory => 2,
            Self::Global => 3,
            Self::Unknown(byte) => byte,
        }
    }

    /// 返回种类的可读名称。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Func => "func",
            Self::Table => "table",
            Self::Memory => "memory",
            Self::Global => "global",
            Self::Unknown(_) => "unknown",
        }
    }
}

/// `WASM` 堆类型（heap type）。
///
/// 对应 `wasm-gc` proposal 中 `(ref ht)` / `(ref null ht)` 的 `ht` 部分。
/// 抽象堆类型由负的 `s33` 值表示，类型索引由非负 `s33` 值表示。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmHeapType {
    /// `func`（`s33 = -0x10`）。
    Func,
    /// `extern`（`s33 = -0x11`）。
    Extern,
    /// `any`（`s33 = -0x12`）。
    Any,
    /// `eq`（`s33 = -0x13`）。
    Eq,
    /// `i31`（`s33 = -0x14`）。
    I31,
    /// `struct`（`s33 = -0x15`）。
    Struct,
    /// `array`（`s33 = -0x16`）。
    Array,
    /// `none`（`s33 = -0x17`）。
    None,
    /// `nofunc`（`s33 = -0x18`）。
    NoFunc,
    /// `noextern`（`s33 = -0x19`）。
    NoExtern,
    /// 类型索引（非负 `s33`）。
    Index(i64),
}

impl WasmHeapType {
    /// 编码为 `s33` LEB128。
    pub fn write(&self, output: &mut Vec<u8>) {
        let value = match self {
            Self::Func => -0x10,
            Self::Extern => -0x11,
            Self::Any => -0x12,
            Self::Eq => -0x13,
            Self::I31 => -0x14,
            Self::Struct => -0x15,
            Self::Array => -0x16,
            Self::None => -0x17,
            Self::NoFunc => -0x18,
            Self::NoExtern => -0x19,
            Self::Index(index) => *index,
        };
        super::write_sleb128_i64(value, output);
    }
}

/// `WASM` 值类型（valtype）。
///
/// 覆盖单字节数值/引用类型、缩写引用类型（`anyref`）与 `wasm-gc` 多字节引用类型
///（`0x63` / `0x64` + heap type）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmValueType {
    /// `i32`（`0x7F`）。
    I32,
    /// `i64`（`0x7E`）。
    I64,
    /// `f32`（`0x7D`）。
    F32,
    /// `f64`（`0x7C`）。
    F64,
    /// `v128`（`0x7B`）。
    V128,
    /// `funcref`（`0x70`），等价于 `(ref null func)`。
    FuncRef,
    /// `externref`（`0x6F`），等价于 `(ref null extern)`。
    ExternRef,
    /// `anyref`（`0x6E`），等价于 `(ref null any)` 的缩写形式。
    AnyRef,
    /// `(ref ht)`（`0x64` + heap type）。
    Ref(WasmHeapType),
    /// `(ref null ht)`（`0x63` + heap type）。
    RefNull(WasmHeapType),
    /// 未知值类型，保留原始字节。
    Unknown(u8),
}

impl WasmValueType {
    /// 若为单字节 valtype，返回该字节；多字节引用类型返回 `None`。
    pub fn as_single_byte(&self) -> Option<u8> {
        Some(match self {
            Self::I32 => 0x7F,
            Self::I64 => 0x7E,
            Self::F32 => 0x7D,
            Self::F64 => 0x7C,
            Self::V128 => 0x7B,
            Self::FuncRef => 0x70,
            Self::ExternRef => 0x6F,
            Self::AnyRef => 0x6E,
            Self::Unknown(byte) => *byte,
            Self::Ref(_) | Self::RefNull(_) => return None,
        })
    }

    /// 从单字节缩写/数值类型解析；`0x63`/`0x64` 需走 [`read_value_type`]。
    pub fn from_single_byte(byte: u8) -> Option<Self> {
        Some(match byte {
            0x7F => Self::I32,
            0x7E => Self::I64,
            0x7D => Self::F32,
            0x7C => Self::F64,
            0x7B => Self::V128,
            0x70 => Self::FuncRef,
            0x6F => Self::ExternRef,
            0x6E => Self::AnyRef,
            _ => return None,
        })
    }

    /// 将值类型编码到输出缓冲。
    pub fn write(&self, output: &mut Vec<u8>) {
        match self {
            Self::I32 => output.push(0x7F),
            Self::I64 => output.push(0x7E),
            Self::F32 => output.push(0x7D),
            Self::F64 => output.push(0x7C),
            Self::V128 => output.push(0x7B),
            Self::FuncRef => output.push(0x70),
            Self::ExternRef => output.push(0x6F),
            Self::AnyRef => output.push(0x6E),
            Self::Ref(ht) => {
                output.push(0x64);
                ht.write(output);
            }
            Self::RefNull(ht) => {
                output.push(0x63);
                ht.write(output);
            }
            Self::Unknown(byte) => output.push(*byte),
        }
    }
}

/// 读取 `WASM` 堆类型（`s33` LEB128）。
///
/// 负值映射为抽象堆类型（`-0x10` = `func`，`-0x11` = `extern` 等），
/// 非负值视为类型索引 `Index(value)`。
pub fn read_heap_type(reader: &mut WasmByteReader) -> Result<WasmHeapType, WasmBinaryError> {
    let value = reader.read_sleb128_i64()?;
    Ok(match value {
        -0x10 => WasmHeapType::Func,
        -0x11 => WasmHeapType::Extern,
        -0x12 => WasmHeapType::Any,
        -0x13 => WasmHeapType::Eq,
        -0x14 => WasmHeapType::I31,
        -0x15 => WasmHeapType::Struct,
        -0x16 => WasmHeapType::Array,
        -0x17 => WasmHeapType::None,
        -0x18 => WasmHeapType::NoFunc,
        -0x19 => WasmHeapType::NoExtern,
        other if other >= 0 => WasmHeapType::Index(other),
        _ => WasmHeapType::Index(value),
    })
}

/// 读取 `WASM` 值类型（valtype）。
///
/// 先读单字节：`0x63` → `RefNull(read_heap_type)`，`0x64` → `Ref(read_heap_type)`，
/// 其他单字节值按数值/引用类型映射，未知字节返回 `Unknown(byte)`。
pub fn read_value_type(reader: &mut WasmByteReader) -> Result<WasmValueType, WasmBinaryError> {
    let byte = reader.read_u8()?;
    Ok(match byte {
        0x7F => WasmValueType::I32,
        0x7E => WasmValueType::I64,
        0x7D => WasmValueType::F32,
        0x7C => WasmValueType::F64,
        0x7B => WasmValueType::V128,
        0x70 => WasmValueType::FuncRef,
        0x6F => WasmValueType::ExternRef,
        0x6E => WasmValueType::AnyRef,
        0x63 => WasmValueType::RefNull(read_heap_type(reader)?),
        0x64 => WasmValueType::Ref(read_heap_type(reader)?),
        other => WasmValueType::Unknown(other),
    })
}

/// `WASM` 导入项模型。
///
/// 对应 `import` 段中的单条记录，按 `kind` 区分四种外部实体。
#[derive(Debug, Clone)]
pub struct WasmImport {
    /// 导入所在模块名。
    pub module: String,
    /// 导入字段名。
    pub field: String,
    /// 导入种类。
    pub kind: WasmExternalKind,
    /// 函数类型索引（`kind == Func` 时有效）。
    pub type_index: u32,
    /// 表元素类型（`kind == Table` 时有效）。
    pub table_elem_type: WasmValueType,
    /// 内存下限（`kind == Table / Memory` 时有效）。
    pub memory_min: u32,
    /// 内存上限（`kind == Table / Memory` 且有限制时有效）。
    pub memory_max: Option<u32>,
    /// 全局变量值类型（`kind == Global` 时有效）。
    pub global_value_type: WasmValueType,
    /// 全局变量是否可变（`kind == Global` 时有效）。
    pub global_mutable: bool,
}

/// `WASM` 导出项模型。
#[derive(Debug, Clone)]
pub struct WasmExport {
    /// 导出名称。
    pub name: String,
    /// 导出种类。
    pub kind: WasmExternalKind,
    /// 导出实体索引。
    pub index: u32,
}

/// `WASM` 代码段函数条目模型。
///
/// 对应 `code` 段中的单个函数体元信息，不含指令解码结果。
#[derive(Debug, Clone)]
pub struct WasmFunctionEntry {
    /// 函数索引（含导入函数偏移）。
    pub index: usize,
    /// 函数名称，来自 `export` 段映射，无导出名时为 `None`。
    pub name: Option<String>,
    /// 函数体在文件中的绝对偏移。
    pub code_offset: usize,
    /// 函数体长度（含局部变量声明）。
    pub body_len: usize,
    /// 局部变量组数量。
    pub local_groups: usize,
}

/// `WASM` 类型段条目种类。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WasmTypeKind {
    /// 函数类型（`form = 0x60`）。
    Func {
        /// 参数值类型列表。
        params: Vec<WasmValueType>,
        /// 返回值类型列表。
        results: Vec<WasmValueType>,
    },
    /// `wasm-gc` 结构体类型（`form = 0x5F`）。
    Struct {
        /// 字段列表，每项为 `(值类型, 是否可变)`。
        fields: Vec<(WasmValueType, bool)>,
    },
    /// `wasm-gc` 数组类型（GC MVP `form = 0x5E`；早期草稿曾用 `0x61`）。
    Array {
        /// 元素值类型。
        element: WasmValueType,
        /// 元素是否可变。
        mutable: bool,
    },
    /// 未知形式，保留原始 `form` 字节。
    Unknown {
        /// 原始 `form` 字节。
        form: u8,
    },
}

impl WasmTypeKind {
    /// 返回种类的可读名称。
    pub fn name(&self) -> &'static str {
        match self {
            Self::Func { .. } => "functype",
            Self::Struct { .. } => "structtype",
            Self::Array { .. } => "arraytype",
            Self::Unknown { .. } => "unknown",
        }
    }
}

/// `WASM` 类型段条目模型。
#[derive(Debug, Clone)]
pub struct WasmTypeEntry {
    /// 条目在 `type` 段中的索引。
    pub index: usize,
    /// 条目在文件中的绝对偏移。
    pub file_offset: usize,
    /// 条目种类。
    pub kind: WasmTypeKind,
}

/// 将堆类型映射为可读名称。
///
/// 抽象堆类型返回 `func` / `extern` / `any` / `eq` / `i31` / `struct` / `array` /
/// `none` / `nofunc` / `noextern`；类型索引返回 `typeidx:N`。
pub fn wasm_heap_type_name(ht: &WasmHeapType) -> String {
    match ht {
        WasmHeapType::Func => "func".to_string(),
        WasmHeapType::Extern => "extern".to_string(),
        WasmHeapType::Any => "any".to_string(),
        WasmHeapType::Eq => "eq".to_string(),
        WasmHeapType::I31 => "i31".to_string(),
        WasmHeapType::Struct => "struct".to_string(),
        WasmHeapType::Array => "array".to_string(),
        WasmHeapType::None => "none".to_string(),
        WasmHeapType::NoFunc => "nofunc".to_string(),
        WasmHeapType::NoExtern => "noextern".to_string(),
        WasmHeapType::Index(i) => format!("typeidx:{}", i),
    }
}

/// 将值类型映射为可读名称。
///
/// 单字节数值/引用类型返回 `i32` / `i64` / `f32` / `f64` / `v128` / `funcref` / `externref`；
/// 多字节引用类型返回 `ref <ht>` / `ref null <ht>`（`<ht>` 来自 [`wasm_heap_type_name`]）；
/// 未知字节返回 `unknown(0xXX)`。
pub fn wasm_value_type_name(vt: &WasmValueType) -> String {
    match vt {
        WasmValueType::I32 => "i32".to_string(),
        WasmValueType::I64 => "i64".to_string(),
        WasmValueType::F32 => "f32".to_string(),
        WasmValueType::F64 => "f64".to_string(),
        WasmValueType::V128 => "v128".to_string(),
        WasmValueType::FuncRef => "funcref".to_string(),
        WasmValueType::ExternRef => "externref".to_string(),
        WasmValueType::AnyRef => "anyref".to_string(),
        WasmValueType::Ref(ht) => format!("ref {}", wasm_heap_type_name(ht)),
        WasmValueType::RefNull(ht) => format!("ref null {}", wasm_heap_type_name(ht)),
        WasmValueType::Unknown(b) => format!("unknown(0x{:02X})", b),
    }
}

/// 解析 `import` 段，返回所有导入项。
///
/// 找不到 `import` 段时返回空向量。读取过程中遇错则停止并返回已收集的条目。
/// 覆盖 `kind` 0（函数）/ 1（表）/ 2（内存）/ 3（全局）四种外部实体。
pub fn parse_import_section(module: &WasmBinaryModule) -> Vec<WasmImport> {
    let Some(import_section) = module.sections.iter().find(|s| s.id == SECTION_IMPORT)
    else {
        return Vec::new();
    };

    let mut reader = WasmByteReader::new(&import_section.bytes);
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
        let kind_byte = match reader.read_u8() {
            Ok(k) => k,
            Err(_) => break,
        };

        let mut imp = WasmImport {
            module: module_name,
            field: field_name,
            kind: WasmExternalKind::from_u8(kind_byte),
            type_index: 0,
            table_elem_type: WasmValueType::Unknown(0),
            memory_min: 0,
            memory_max: None,
            global_value_type: WasmValueType::Unknown(0),
            global_mutable: false,
        };

        match kind_byte {
            0 => {
                if let Ok(idx) = reader.read_uleb128() {
                    imp.type_index = idx;
                }
            }
            1 => {
                if let Ok(elem_type) = read_value_type(&mut reader) {
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
                    else if let Ok(min) = reader.read_uleb128() {
                        imp.memory_min = min;
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
                if let Ok(val_type) = read_value_type(&mut reader) {
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

/// 解析 `export` 段，返回所有导出项。
///
/// 找不到 `export` 段时返回空向量。读取过程中遇错则停止并返回已收集的条目。
pub fn parse_export_section(module: &WasmBinaryModule) -> Vec<WasmExport> {
    let Some(export_section) = module.sections.iter().find(|s| s.id == SECTION_EXPORT)
    else {
        return Vec::new();
    };

    let mut reader = WasmByteReader::new(&export_section.bytes);
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
        let kind_byte = match reader.read_u8() {
            Ok(k) => k,
            Err(_) => break,
        };
        let index = match reader.read_uleb128() {
            Ok(i) => i,
            Err(_) => break,
        };
        exports.push(WasmExport { name, kind: WasmExternalKind::from_u8(kind_byte), index });
    }

    exports
}

/// 解析 `code` 段中的函数列表。
///
/// 找不到 `code` 段时返回空向量。函数索引包含导入函数偏移（通过 `parse_import_section` 计算），
/// 函数名从 `parse_export_section` 中 `kind == Func` 的导出项映射而来。
/// `code_offset` 为函数体在文件中的绝对偏移，`body_len` 为函数体总长度。
pub fn parse_code_section(module: &WasmBinaryModule) -> Vec<WasmFunctionEntry> {
    let Some(code_section) = module.sections.iter().find(|s| s.id == SECTION_CODE)
    else {
        return Vec::new();
    };

    let section_abs_offset = compute_section_offset(module, SECTION_CODE);

    let mut reader = WasmByteReader::new(&code_section.bytes);
    let count = match reader.read_uleb128() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let import_func_count = parse_import_section(module).iter().filter(|i| i.kind == WasmExternalKind::Func).count();

    let export_names = parse_export_section(module);
    let mut func_names: HashMap<usize, String> = HashMap::new();
    for exp in &export_names {
        if exp.kind == WasmExternalKind::Func {
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

        let body_offset_in_section = reader.offset();
        let code_offset = section_abs_offset + reader.offset();

        let local_groups = match reader.read_uleb128() {
            Ok(g) => g as usize,
            Err(_) => break,
        };

        for _ in 0..local_groups {
            let _ = reader.read_uleb128();
            let _ = skip_value_type(&mut reader);
        }

        let body_end = body_offset_in_section + body_size;
        if body_end > code_section.bytes.len() {
            break;
        }
        let remaining = body_end - reader.offset();
        if remaining > 0 {
            let _ = reader.read_bytes(remaining);
        }

        functions.push(WasmFunctionEntry {
            index: func_index,
            name: func_names.get(&func_index).cloned(),
            code_offset,
            body_len: body_size,
            local_groups,
        });
    }

    functions
}

/// 解析 `type` 段，返回所有类型条目。
///
/// 找不到 `type` 段时返回空向量。支持 `form` `0x60`（函数）/ `0x5F`（结构体）/
/// `0x5E`（GC MVP 数组）/ `0x61`（早期草稿数组），
/// 其他 `form` 归为 `WasmTypeKind::Unknown`。`file_offset` 为条目在文件中的绝对偏移。
pub fn parse_type_section(module: &WasmBinaryModule) -> Vec<WasmTypeEntry> {
    let Some(type_section) = module.sections.iter().find(|s| s.id == SECTION_TYPE)
    else {
        return Vec::new();
    };
    let section_abs = compute_section_offset(module, SECTION_TYPE);
    let mut reader = WasmByteReader::new(&type_section.bytes);
    let count = match reader.read_uleb128() {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut entries = Vec::with_capacity(count as usize);
    for i in 0..count {
        let file_offset = section_abs + reader.offset();
        let form = match reader.read_u8() {
            Ok(b) => b,
            Err(_) => break,
        };
        let kind = match form {
            0x60 => {
                let param_count = reader.read_uleb128().unwrap_or(0);
                let params = (0..param_count).filter_map(|_| read_value_type(&mut reader).ok()).collect();
                let result_count = reader.read_uleb128().unwrap_or(0);
                let results = (0..result_count).filter_map(|_| read_value_type(&mut reader).ok()).collect();
                WasmTypeKind::Func { params, results }
            }
            0x5F => {
                let field_count = reader.read_uleb128().unwrap_or(0);
                let mut fields = Vec::with_capacity(field_count as usize);
                for _ in 0..field_count {
                    let ty = read_value_type(&mut reader).unwrap_or(WasmValueType::Unknown(0));
                    let mut_ = reader.read_u8().unwrap_or(0) != 0;
                    fields.push((ty, mut_));
                }
                WasmTypeKind::Struct { fields }
            }
            0x5E | 0x61 => {
                let element = read_value_type(&mut reader).unwrap_or(WasmValueType::Unknown(0));
                let mutable = reader.read_u8().unwrap_or(0) != 0;
                WasmTypeKind::Array { element, mutable }
            }
            other => WasmTypeKind::Unknown { form: other },
        };
        entries.push(WasmTypeEntry { index: i as usize, file_offset, kind });
    }
    entries
}

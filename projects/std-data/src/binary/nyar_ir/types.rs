/// Nyar 段类型。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NyarSectionKind {
    /// 常量池段。
    Constants = 0x01,
    /// 函数表段。
    Functions = 0x02,
    /// 代码段。
    Code = 0x03,
    /// 导入表段。
    Imports = 0x04,
    /// 导出表段。
    Exports = 0x05,
    /// Witness 分派绑定段。
    WitnessEntries = 0x06,
    /// 调试信息段。
    DebugInfo = 0x10,
    /// 源码映射段。
    SourceMap = 0x11,
}

/// 常量池条目类型。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NyarConstantKind {
    /// 空值。
    Null = 0x00,
    /// 布尔值。
    Boolean = 0x01,
    /// 大整数。
    BigInt = 0x06,
    /// 字符串。
    String = 0x05,
    /// 32 位整数。
    Integer32 = 0x11,
    /// 64 位浮点数。
    Float64 = 0x22,
}

/// 导入类型。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NyarImportKind {
    /// 函数导入。
    Function = 0,
    /// 全局变量导入。
    Global = 1,
    /// 模块导入。
    Module = 2,
}

/// 导出类型。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NyarExportKind {
    /// 函数导出。
    Function = 0,
    /// 全局变量导出。
    Global = 1,
}

/// 常量池条目的值。
#[derive(Debug, Clone, PartialEq)]
pub enum NyarConstantValue {
    /// 空值。
    Null,
    /// 布尔值。
    Boolean(bool),
    /// 32 位整数。
    Integer32(i32),
    /// 大整数原始字节。
    BigInt(Vec<u8>),
    /// UTF-8 字符串。
    String(String),
    /// 64 位浮点数。
    Float64(f64),
}

/// Nyar 常量池条目。
#[derive(Debug, Clone, PartialEq)]
pub struct NyarConstant {
    /// 常量类型。
    pub kind: NyarConstantKind,
    /// 常量值。
    pub value: NyarConstantValue,
}

impl NyarConstant {
    /// 创建常量池条目。
    pub fn new(kind: NyarConstantKind, value: NyarConstantValue) -> Self {
        Self { kind, value }
    }
}

/// Nyar 函数元数据。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyarFunction {
    /// 函数名称。
    pub name: String,
    /// 参数数量。
    pub arity: i32,
    /// 局部变量数量。
    pub local_count: i32,
    /// 代码区起始字节偏移。
    pub code_offset: i32,
    /// 代码区字节长度。
    pub code_length: i32,
}

impl NyarFunction {
    /// 创建函数元数据（`code_offset` 默认为 0）。
    pub fn new(name: impl Into<String>, arity: i32, local_count: i32, code_length: i32) -> Self {
        Self {
            name: name.into(),
            arity,
            local_count,
            code_offset: 0,
            code_length,
        }
    }

    /// 创建带代码偏移的函数元数据。
    pub fn with_offset(
        name: impl Into<String>,
        arity: i32,
        local_count: i32,
        code_offset: i32,
        code_length: i32,
    ) -> Self {
        Self {
            name: name.into(),
            arity,
            local_count,
            code_offset,
            code_length,
        }
    }
}

/// Nyar 导入条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyarImport {
    /// 导入类型。
    pub kind: NyarImportKind,
    /// 模块名称。
    pub module_name: String,
    /// 符号名称。
    pub symbol_name: String,
}

impl NyarImport {
    /// 创建导入条目。
    pub fn new(
        kind: NyarImportKind,
        module_name: impl Into<String>,
        symbol_name: impl Into<String>,
    ) -> Self {
        Self {
            kind,
            module_name: module_name.into(),
            symbol_name: symbol_name.into(),
        }
    }
}

/// Nyar 导出条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyarExport {
    /// 导出类型。
    pub kind: NyarExportKind,
    /// 符号名称。
    pub symbol_name: String,
    /// 关联的函数索引（仅当 [`NyarExportKind::Function`] 时有效）。
    pub function_index: i32,
}

impl NyarExport {
    /// 创建导出条目。
    pub fn new(kind: NyarExportKind, symbol_name: impl Into<String>, function_index: i32) -> Self {
        Self {
            kind,
            symbol_name: symbol_name.into(),
            function_index,
        }
    }
}

/// Nyar Witness 分派条目。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyarWitnessDispatchEntry {
    /// 方法 ID。
    pub method_id: i32,
    /// 类型 ID。
    pub type_id: i32,
    /// 方法名称。
    pub method_name: String,
    /// 目标函数索引。
    pub function_index: i32,
    /// 接口 ID。
    pub interface_id: i32,
    /// 接口方法槽位。
    pub interface_method_index: i32,
}

/// Nyar 模块的二进制数据视图。
#[derive(Debug, Clone, PartialEq)]
pub struct NyarModuleData {
    /// 版本号。
    pub version: u32,
    /// 模块名称。
    pub name: String,
    /// 常量池。
    pub constants: Vec<NyarConstant>,
    /// 函数表。
    pub functions: Vec<NyarFunction>,
    /// 导入表。
    pub imports: Vec<NyarImport>,
    /// 导出表。
    pub exports: Vec<NyarExport>,
    /// Witness 分派绑定表。
    pub witness_entries: Vec<NyarWitnessDispatchEntry>,
    /// 模块代码字节流。
    pub code_bytes: Option<Vec<u8>>,
}

impl Default for NyarModuleData {
    fn default() -> Self {
        Self {
            version: super::constants::NyarConstants::CURRENT_VERSION,
            name: String::new(),
            constants: Vec::new(),
            functions: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            witness_entries: Vec::new(),
            code_bytes: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::NyarEncoder;

    fn minimal_module() -> NyarModuleData {
        let code_bytes = vec![
            0x10, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x01, 0x00, 0x00, 0x00,
            0x30,
            0x05,
        ];

        NyarModuleData {
            version: 1,
            name: "test".to_string(),
            constants: vec![
                NyarConstant::new(NyarConstantKind::Integer32, NyarConstantValue::Integer32(10)),
                NyarConstant::new(NyarConstantKind::Integer32, NyarConstantValue::Integer32(20)),
            ],
            functions: vec![NyarFunction::new("main", 0, 0, code_bytes.len() as i32)],
            imports: Vec::new(),
            exports: Vec::new(),
            witness_entries: Vec::new(),
            code_bytes: Some(code_bytes)
        }
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        let decoded = super::super::decode::NyarDecoder::new()
            .decode(&encoded)
            .expect("decode");

        assert_eq!(decoded.name, module.name);
        assert_eq!(decoded.functions, module.functions);
        assert_eq!(decoded.constants.len(), 2);
        assert_eq!(decoded.code_bytes, module.code_bytes);
    }
}

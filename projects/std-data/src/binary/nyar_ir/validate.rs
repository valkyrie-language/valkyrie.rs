use std::collections::HashSet;

use super::instruction::NyarInstruction;
use super::opcode::NyarHeadCode;
use super::types::{NyarExportKind, NyarFunction, NyarModuleData};

/// Nyar 代码字节流验证器。
#[derive(Debug, Default, Clone, Copy)]
pub struct NyarValidator;

/// 验证结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NyarValidationResult {
    /// 是否通过验证。
    pub valid: bool,
    /// 诊断信息。
    pub diagnostics: Vec<String>,
}

impl NyarValidator {
    /// 创建验证器。
    pub fn new() -> Self {
        Self
    }

    /// 验证模块对应的代码字节流是否安全且格式正确。
    pub fn validate(&self, module: &NyarModuleData, code_bytes: &[u8]) -> NyarValidationResult {
        let mut diagnostics = Vec::new();
        let mut valid = true;

        if module.name.is_empty() {
            diagnostics.push("Module name is empty or null.".to_string());
            valid = false;
        }

        for function in &module.functions {
            if !validate_function(function, code_bytes, &mut diagnostics) {
                valid = false;
            }
        }

        if !validate_function_overlap(&module.functions, &mut diagnostics) {
            valid = false;
        }

        for import in &module.imports {
            if import.module_name.is_empty() || import.symbol_name.is_empty() {
                diagnostics.push("Invalid import: module or symbol name is empty.".to_string());
                valid = false;
            }
        }

        for export in &module.exports {
            if export.symbol_name.is_empty() {
                diagnostics.push("Invalid export: symbol name is empty.".to_string());
                valid = false;
            }

            if export.kind == NyarExportKind::Function
                && (export.function_index < 0
                    || export.function_index as usize >= module.functions.len())
            {
                diagnostics.push(format!(
                    "Invalid export: function index {} out of range [0, {}).",
                    export.function_index,
                    module.functions.len()
                ));
                valid = false;
            }
        }

        for witness_entry in &module.witness_entries {
            if witness_entry.method_name.is_empty() {
                diagnostics.push("Invalid witness entry: method name is empty.".to_string());
                valid = false;
            }

            if witness_entry.function_index < 0
                || witness_entry.function_index as usize >= module.functions.len()
            {
                diagnostics.push(format!(
                    "Invalid witness entry: function index {} out of range [0, {}).",
                    witness_entry.function_index,
                    module.functions.len()
                ));
                valid = false;
            }
        }

        NyarValidationResult { valid, diagnostics }
    }
}

fn validate_function(function: &NyarFunction, code_bytes: &[u8], diagnostics: &mut Vec<String>) -> bool {
    let mut valid = true;

    if function.name.is_empty() {
        diagnostics.push("Function name is empty or null.".to_string());
        valid = false;
    }

    if function.code_offset < 0 || function.code_offset as usize >= code_bytes.len() {
        diagnostics.push(format!(
            "Function '{}': invalid code offset {}.",
            function.name, function.code_offset
        ));
        return false;
    }

    let end = function.code_offset + function.code_length;
    if end < 0 || end as usize > code_bytes.len() {
        diagnostics.push(format!(
            "Function '{}': code range [{}, {end}) exceeds code bytes length {}.",
            function.name, function.code_offset,
            code_bytes.len()
        ));
        return false;
    }

    if function.arity < 0 {
        diagnostics.push(format!(
            "Function '{}': negative arity {}.",
            function.name, function.arity
        ));
        valid = false;
    }

    if function.local_count < 0 {
        diagnostics.push(format!(
            "Function '{}': negative local count {}.",
            function.name, function.local_count
        ));
        valid = false;
    }

    validate_jump_targets(function, code_bytes, diagnostics) && valid
}

fn validate_jump_targets(function: &NyarFunction, code_bytes: &[u8], diagnostics: &mut Vec<String>) -> bool {
    let mut valid = true;
    let end = function.code_offset + function.code_length;
    let mut instruction_offsets = HashSet::new();

    let mut pc = function.code_offset;
    while pc < end {
        if pc < 0 {
            break;
        }
        let pc_usize = pc as usize;
        if pc_usize >= code_bytes.len() {
            break;
        }

        let Some(opcode) = NyarHeadCode::from_u8(code_bytes[pc_usize]) else {
            break;
        };

        let instruction_size = NyarInstruction::code_size(opcode);
        if instruction_size == 0 {
            break;
        }

        instruction_offsets.insert(pc);
        pc += instruction_size as i32;
    }

    let mut pc = function.code_offset;
    while pc < end {
        let pc_usize = pc as usize;
        let op = code_bytes[pc_usize];
        let Some(opcode) = NyarHeadCode::from_u8(op) else {
            diagnostics.push(format!(
                "Function '{}': undefined opcode 0x{op:02X} at offset {pc}.",
                function.name
            ));
            valid = false;
            pc += 1;
            continue;
        };

        let instruction_size = NyarInstruction::code_size(opcode);
        if instruction_size == 0 {
            diagnostics.push(format!(
                "Function '{}': invalid instruction size at offset {pc}.",
                function.name
            ));
            valid = false;
            pc += 1;
            continue;
        }

        if matches!(
            opcode,
            NyarHeadCode::Jump | NyarHeadCode::JumpIfTrue | NyarHeadCode::JumpIfFalse
        ) && pc + 1 + 4 <= end
        {
            let target = i32::from_le_bytes([
                code_bytes[pc_usize + 1],
                code_bytes[pc_usize + 2],
                code_bytes[pc_usize + 3],
                code_bytes[pc_usize + 4],
            ]);

            if target < function.code_offset || target > end {
                diagnostics.push(format!(
                    "Function '{}': jump target {target} out of range [{}, {end}] at offset {pc}.",
                    function.name, function.code_offset
                ));
                valid = false;
            } else if target != end && !instruction_offsets.contains(&target) {
                diagnostics.push(format!(
                    "Function '{}': jump target {target} not aligned with instruction start at offset {pc}.",
                    function.name
                ));
                valid = false;
            }
        }

        pc += instruction_size as i32;
    }

    valid
}

fn validate_function_overlap(functions: &[NyarFunction], diagnostics: &mut Vec<String>) -> bool {
    let mut valid = true;

    for i in 0..functions.len() {
        for j in (i + 1)..functions.len() {
            let f1 = &functions[i];
            let f2 = &functions[j];
            let f1_end = f1.code_offset + f1.code_length;
            let f2_end = f2.code_offset + f2.code_length;

            if !(f1_end <= f2.code_offset || f2_end <= f1.code_offset) {
                diagnostics.push(format!(
                    "Function overlap: '{}' [{}, {f1_end}) and '{}' [{}, {f2_end}).",
                    f1.name, f1.code_offset, f2.name, f2.code_offset
                ));
                valid = false;
            }
        }
    }

    valid
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::{
        NyarConstant, NyarConstantKind, NyarConstantValue, NyarDecoder, NyarEncoder, NyarFunction,
    };

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
    fn validates_minimal_module() {
        let module = minimal_module();
        let code_bytes = module.code_bytes.clone().expect("code");
        let result = NyarValidator::new().validate(&module, &code_bytes);
        assert!(result.valid, "{:?}", result.diagnostics);
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        let decoded = NyarDecoder::new().decode(&encoded).expect("decode");
        let code_bytes = decoded.code_bytes.clone().expect("code");
        let result = NyarValidator::new().validate(&decoded, &code_bytes);
        assert!(result.valid, "{:?}", result.diagnostics);
        assert_eq!(decoded.code_bytes, module.code_bytes);
    }
}

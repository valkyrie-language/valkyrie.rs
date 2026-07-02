//! 将 `.nyar` 字节解码为 [`NyarModuleData`]，并提供指令预解码。

use super::constants::NyarConstants;
use super::error::NyarIrError;
use super::instruction::{NyarInstruction, NyarInstructionForm};
use super::opcode::NyarHeadCode;
use super::types::{
    NyarConstant, NyarConstantKind, NyarConstantValue, NyarExport, NyarExportKind, NyarFunction,
    NyarImport, NyarImportKind, NyarModuleData, NyarSectionKind, NyarWitnessDispatchEntry,
};

/// 指令预解码器。
pub struct InstructionDecoder;

impl InstructionDecoder {
    /// 将代码字节流预解码为按字节偏移索引的指令数组。
    pub fn decode(bytecode: &[u8]) -> Vec<NyarInstruction> {
        let mut result = vec![NyarInstruction::default(); bytecode.len()];

        let mut pc = 0usize;
        while pc < bytecode.len() {
            let instruction = Self::decode_at(bytecode, pc);
            if !instruction.is_valid() {
                break;
            }

            result[pc] = instruction;
            pc += instruction.size as usize;
        }

        result
    }

    /// 解码指定字节偏移处的一条指令。
    pub fn decode_at(bytecode: &[u8], pc: usize) -> NyarInstruction {
        if pc >= bytecode.len() {
            return NyarInstruction::default();
        }

        let Some(opcode) = NyarHeadCode::from_u8(bytecode[pc]) else {
            return NyarInstruction::default();
        };

        let size = Self::try_get_instruction_size(bytecode, pc);
        if size == 0 {
            return NyarInstruction::default();
        }

        let operand1 = if size >= 5 && pc + 4 < bytecode.len() {
            read_i32_le(bytecode, pc + 1)
        } else {
            0
        };

        let operand2 = if size >= 9 && pc + 8 < bytecode.len() {
            read_i32_le(bytecode, pc + 5)
        } else {
            0
        };

        let operand3 = if size >= 13 && pc + 12 < bytecode.len() {
            read_i32_le(bytecode, pc + 9)
        } else {
            0
        };

        NyarInstruction::with_size(opcode, operand1, operand2, operand3, size)
    }

    /// 获取指令大小；`0` 表示头码无效或当前解码器不识别该编码。
    pub fn get_instruction_size(head_code: NyarHeadCode) -> i32 {
        NyarInstruction::code_size(head_code) as i32
    }

    /// 从代码字节流中尝试获取指定偏移处指令的真实编码长度。
    pub fn try_get_instruction_size(bytecode: &[u8], pc: usize) -> u8 {
        if pc >= bytecode.len() {
            return 0;
        }

        let Some(opcode) = NyarHeadCode::from_u8(bytecode[pc]) else {
            return 0;
        };

        let form = NyarInstruction::get_form(opcode);
        match form {
            NyarInstructionForm::Plain => 1,
            NyarInstructionForm::Imm1 => {
                if pc + 4 < bytecode.len() {
                    5
                } else {
                    0
                }
            }
            NyarInstructionForm::Imm2 => {
                if pc + 8 < bytecode.len() {
                    9
                } else {
                    0
                }
            }
            NyarInstructionForm::Imm3 => {
                if pc + 12 < bytecode.len() {
                    13
                } else {
                    0
                }
            }
            NyarInstructionForm::Prefixed => Self::try_get_prefixed_instruction_size(bytecode, pc, opcode),
            NyarInstructionForm::Invalid => 0,
        }
    }

    /// 重新解码指定偏移量处的指令。
    pub fn re_decode_at(instructions: &mut [NyarInstruction], bytecode: &[u8], pc: usize, old_size: usize) {
        if pc >= bytecode.len() {
            return;
        }

        instructions[pc] = Self::decode_at(bytecode, pc);

        for i in 1..old_size {
            let index = pc + i;
            if index < instructions.len() {
                instructions[index] = NyarInstruction::default();
            }
        }
    }

    fn try_get_prefixed_instruction_size(bytecode: &[u8], pc: usize, head_code: NyarHeadCode) -> u8 {
        match head_code {
            NyarHeadCode::Simd => {
                if pc + 4 < bytecode.len() {
                    5
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

/// `.nyar` 模块解码器。
#[derive(Debug, Default, Clone, Copy)]
pub struct NyarDecoder;

impl NyarDecoder {
    /// 创建解码器。
    pub fn new() -> Self {
        Self
    }

    /// 将 `.nyar` 字节解码为 [`NyarModuleData`]。
    pub fn decode(&self, data: &[u8]) -> Result<NyarModuleData, NyarIrError> {
        let mut reader = ByteReader::new(data);
        self.decode_from_reader(&mut reader)
    }

    fn decode_from_reader(&self, reader: &mut ByteReader<'_>) -> Result<NyarModuleData, NyarIrError> {
        let header = read_header(reader)?;
        if !header.is_valid() {
            return Err(NyarIrError::InvalidHeader {
                magic: header.magic,
                version: header.version,
            });
        }

        let sections = read_section_headers(reader, header.section_count)?;
        let module_name = read_module_name(reader, header.name_offset)?;

        let mut constants = Vec::new();
        let mut functions = Vec::new();
        let mut imports = Vec::new();
        let mut exports = Vec::new();
        let mut witness_entries = Vec::new();
        let mut code_bytes = None;

        for section in sections {
            reader.set_position(section.offset as usize);
            code_bytes = decode_section(
                reader,
                &section,
                &mut constants,
                &mut functions,
                &mut imports,
                &mut exports,
                &mut witness_entries,
                code_bytes,
            )?;
        }

        Ok(NyarModuleData {
            name: module_name,
            version: header.version,
            constants,
            functions,
            imports,
            exports,
            witness_entries,
            code_bytes,
        })
    }
}

#[derive(Clone, Copy)]
struct NyarFileHeader {
    magic: u32,
    version: u32,
    section_count: i32,
    name_offset: i32,
}

impl NyarFileHeader {
    fn is_valid(&self) -> bool {
        self.magic == NyarConstants::MAGIC_VALUE
    }
}

struct NyarSectionHeader {
    kind: NyarSectionKind,
    offset: i32,
    size: i32,
}

fn read_header(reader: &mut ByteReader<'_>) -> Result<NyarFileHeader, NyarIrError> {
    Ok(NyarFileHeader {
        magic: reader.read_u32_be()?,
        version: reader.read_u32_le()?,
        section_count: reader.read_i32_le()?,
        name_offset: reader.read_i32_le()?,
    })
}

fn read_section_headers(reader: &mut ByteReader<'_>, count: i32) -> Result<Vec<NyarSectionHeader>, NyarIrError> {
    let mut sections = Vec::with_capacity(count.max(0) as usize);
    for _ in 0..count {
        let kind = reader.read_u8()?;
        sections.push(NyarSectionHeader {
            kind: section_kind_from_u8(kind)?,
            offset: reader.read_i32_le()?,
            size: reader.read_i32_le()?,
        });
    }
    Ok(sections)
}

fn section_kind_from_u8(value: u8) -> Result<NyarSectionKind, NyarIrError> {
    match value {
        0x01 => Ok(NyarSectionKind::Constants),
        0x02 => Ok(NyarSectionKind::Functions),
        0x03 => Ok(NyarSectionKind::Code),
        0x04 => Ok(NyarSectionKind::Imports),
        0x05 => Ok(NyarSectionKind::Exports),
        0x06 => Ok(NyarSectionKind::WitnessEntries),
        0x10 => Ok(NyarSectionKind::DebugInfo),
        0x11 => Ok(NyarSectionKind::SourceMap),
        _ => Err(NyarIrError::Message(format!("未知的段类型：{value}"))),
    }
}

fn read_module_name(reader: &mut ByteReader<'_>, name_offset: i32) -> Result<String, NyarIrError> {
    if name_offset <= 0 {
        return Ok("<unknown>".to_string());
    }

    let saved = reader.position();
    reader.set_position(name_offset as usize);
    let name = read_length_prefixed_string(reader)?;
    reader.set_position(saved);
    Ok(name)
}

fn decode_section(
    reader: &mut ByteReader<'_>,
    section: &NyarSectionHeader,
    constants: &mut Vec<NyarConstant>,
    functions: &mut Vec<NyarFunction>,
    imports: &mut Vec<NyarImport>,
    exports: &mut Vec<NyarExport>,
    witness_entries: &mut Vec<NyarWitnessDispatchEntry>,
    current_code_bytes: Option<Vec<u8>>,
) -> Result<Option<Vec<u8>>, NyarIrError> {
    match section.kind {
        NyarSectionKind::Constants => decode_constants(reader, constants)?,
        NyarSectionKind::Functions => decode_functions(reader, functions)?,
        NyarSectionKind::Code => return Ok(Some(reader.read_bytes(section.size as usize)?)),
        NyarSectionKind::Imports => decode_imports(reader, imports)?,
        NyarSectionKind::Exports => decode_exports(reader, exports)?,
        NyarSectionKind::WitnessEntries => decode_witness_entries(reader, witness_entries)?,
        NyarSectionKind::DebugInfo | NyarSectionKind::SourceMap => {
            reader.skip(section.size as usize)?;
        }
    }

    Ok(current_code_bytes)
}

fn decode_constants(reader: &mut ByteReader<'_>, constants: &mut Vec<NyarConstant>) -> Result<(), NyarIrError> {
    let count = reader.read_i32_le()?;
    constants.reserve(count.max(0) as usize);
    for _ in 0..count {
        let kind = constant_kind_from_u8(reader.read_u8()?)?;
        constants.push(decode_constant(reader, kind)?);
    }
    Ok(())
}

fn constant_kind_from_u8(value: u8) -> Result<NyarConstantKind, NyarIrError> {
    match value {
        0x00 => Ok(NyarConstantKind::Null),
        0x01 => Ok(NyarConstantKind::Boolean),
        0x06 => Ok(NyarConstantKind::BigInt),
        0x05 => Ok(NyarConstantKind::String),
        0x11 => Ok(NyarConstantKind::Integer32),
        0x22 => Ok(NyarConstantKind::Float64),
        _ => Err(NyarIrError::UnknownConstantKind(value)),
    }
}

fn decode_constant(reader: &mut ByteReader<'_>, kind: NyarConstantKind) -> Result<NyarConstant, NyarIrError> {
    let value = match kind {
        NyarConstantKind::Integer32 => NyarConstantValue::Integer32(reader.read_i32_le()?),
        NyarConstantKind::Float64 => NyarConstantValue::Float64(reader.read_f64_le()?),
        NyarConstantKind::Boolean => NyarConstantValue::Boolean(reader.read_u8()? != 0),
        NyarConstantKind::Null => NyarConstantValue::Null,
        NyarConstantKind::String => NyarConstantValue::String(read_length_prefixed_string(reader)?),
        NyarConstantKind::BigInt => {
            let byte_count = reader.read_i32_le()?;
            NyarConstantValue::BigInt(reader.read_bytes(byte_count.max(0) as usize)?)
        }
    };
    Ok(NyarConstant::new(kind, value))
}

fn decode_functions(reader: &mut ByteReader<'_>, functions: &mut Vec<NyarFunction>) -> Result<(), NyarIrError> {
    let count = reader.read_i32_le()?;
    functions.reserve(count.max(0) as usize);
    for _ in 0..count {
        let name = read_length_prefixed_string(reader)?;
        let arity = reader.read_i32_le()?;
        let local_count = reader.read_i32_le()?;
        let code_offset = reader.read_i32_le()?;
        let code_length = reader.read_i32_le()?;
        functions.push(NyarFunction::with_offset(
            name,
            arity,
            local_count,
            code_offset,
            code_length,
        ));
    }
    Ok(())
}

fn decode_imports(reader: &mut ByteReader<'_>, imports: &mut Vec<NyarImport>) -> Result<(), NyarIrError> {
    let count = reader.read_i32_le()?;
    imports.reserve(count.max(0) as usize);
    for _ in 0..count {
        let kind = import_kind_from_u8(reader.read_u8()?)?;
        let module_name = read_length_prefixed_string(reader)?;
        let symbol_name = read_length_prefixed_string(reader)?;
        imports.push(NyarImport::new(kind, module_name, symbol_name));
    }
    Ok(())
}

fn import_kind_from_u8(value: u8) -> Result<NyarImportKind, NyarIrError> {
    match value {
        0 => Ok(NyarImportKind::Function),
        1 => Ok(NyarImportKind::Global),
        2 => Ok(NyarImportKind::Module),
        _ => Err(NyarIrError::Message(format!("未知的导入类型：{value}"))),
    }
}

fn decode_exports(reader: &mut ByteReader<'_>, exports: &mut Vec<NyarExport>) -> Result<(), NyarIrError> {
    let count = reader.read_i32_le()?;
    exports.reserve(count.max(0) as usize);
    for _ in 0..count {
        let kind = export_kind_from_u8(reader.read_u8()?)?;
        let symbol_name = read_length_prefixed_string(reader)?;
        let function_index = reader.read_i32_le()?;
        exports.push(NyarExport::new(kind, symbol_name, function_index));
    }
    Ok(())
}

fn export_kind_from_u8(value: u8) -> Result<NyarExportKind, NyarIrError> {
    match value {
        0 => Ok(NyarExportKind::Function),
        1 => Ok(NyarExportKind::Global),
        _ => Err(NyarIrError::Message(format!("未知的导出类型：{value}"))),
    }
}

fn decode_witness_entries(
    reader: &mut ByteReader<'_>,
    witness_entries: &mut Vec<NyarWitnessDispatchEntry>,
) -> Result<(), NyarIrError> {
    let count = reader.read_i32_le()?;
    witness_entries.reserve(count.max(0) as usize);
    for _ in 0..count {
        witness_entries.push(NyarWitnessDispatchEntry {
            method_id: reader.read_i32_le()?,
            type_id: reader.read_i32_le()?,
            method_name: read_length_prefixed_string(reader)?,
            function_index: reader.read_i32_le()?,
            interface_id: reader.read_i32_le()?,
            interface_method_index: reader.read_i32_le()?,
        });
    }
    Ok(())
}

fn read_length_prefixed_string(reader: &mut ByteReader<'_>) -> Result<String, NyarIrError> {
    let length = reader.read_i32_le()?;
    if length < 0 {
        return Err(NyarIrError::Message(format!("无效的字符串长度：{length}")));
    }
    let bytes = reader.read_bytes(length as usize)?;
    String::from_utf8(bytes).map_err(|error| NyarIrError::Message(error.to_string()))
}

fn read_i32_le(bytes: &[u8], offset: usize) -> i32 {
    let slice = &bytes[offset..offset + 4];
    i32::from_le_bytes([slice[0], slice[1], slice[2], slice[3]])
}

struct ByteReader<'a> {
    data: &'a [u8],
    position: usize,
}

impl<'a> ByteReader<'a> {
    fn new(data: &'a [u8]) -> Self {
        Self { data, position: 0 }
    }

    fn position(&self) -> usize {
        self.position
    }

    fn set_position(&mut self, position: usize) {
        self.position = position;
    }

    fn read_u8(&mut self) -> Result<u8, NyarIrError> {
        if self.position >= self.data.len() {
            return Err(NyarIrError::UnexpectedEof);
        }
        let value = self.data[self.position];
        self.position += 1;
        Ok(value)
    }

    fn read_u32_be(&mut self) -> Result<u32, NyarIrError> {
        self.ensure(4)?;
        let bytes: [u8; 4] = self.data[self.position..self.position + 4]
            .try_into()
            .expect("slice");
        self.position += 4;
        Ok(u32::from_be_bytes(bytes))
    }

    fn read_u32_le(&mut self) -> Result<u32, NyarIrError> {
        self.ensure(4)?;
        let bytes: [u8; 4] = self.data[self.position..self.position + 4]
            .try_into()
            .expect("slice");
        self.position += 4;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_i32_le(&mut self) -> Result<i32, NyarIrError> {
        Ok(self.read_u32_le()? as i32)
    }

    fn read_f64_le(&mut self) -> Result<f64, NyarIrError> {
        self.ensure(8)?;
        let bytes: [u8; 8] = self.data[self.position..self.position + 8]
            .try_into()
            .expect("slice");
        self.position += 8;
        Ok(f64::from_le_bytes(bytes))
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>, NyarIrError> {
        self.ensure(count)?;
        let bytes = self.data[self.position..self.position + count].to_vec();
        self.position += count;
        Ok(bytes)
    }

    fn skip(&mut self, count: usize) -> Result<(), NyarIrError> {
        self.ensure(count)?;
        self.position += count;
        Ok(())
    }

    fn ensure(&self, count: usize) -> Result<(), NyarIrError> {
        if self.position + count > self.data.len() {
            Err(NyarIrError::UnexpectedEof)
        } else {
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::{NyarEncoder, NyarHeadCode};

    fn minimal_module() -> NyarModuleData {
        let code_bytes = vec![
            NyarHeadCode::Const as u8,
            0x00,
            0x00,
            0x00,
            0x00,
            NyarHeadCode::Const as u8,
            0x01,
            0x00,
            0x00,
            0x00,
            NyarHeadCode::I32Add as u8,
            NyarHeadCode::Return as u8,
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
    fn instruction_decoder_reads_operands_as_i32_le() {
        let bytecode = vec![0x10, 0x2A, 0x00, 0x00, 0x00];
        let instruction = InstructionDecoder::decode_at(&bytecode, 0);
        assert_eq!(instruction.code, NyarHeadCode::Const);
        assert_eq!(instruction.operand1, 42);
        assert_eq!(instruction.size, 5);
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        let decoded = NyarDecoder::new().decode(&encoded).expect("decode");

        assert_eq!(decoded.name, "test");
        assert_eq!(decoded.functions.len(), 1);
        assert_eq!(decoded.constants.len(), 2);
        assert_eq!(decoded.code_bytes, module.code_bytes);

        let code = decoded.code_bytes.as_ref().expect("code");
        let instructions = InstructionDecoder::decode(code);
        assert!(instructions[0].is_valid());
        assert_eq!(instructions[0].code, NyarHeadCode::Const);
        assert_eq!(instructions[5].code, NyarHeadCode::Const);
        assert_eq!(instructions[10].code, NyarHeadCode::I32Add);
        assert_eq!(instructions[11].code, NyarHeadCode::Return);
    }
}

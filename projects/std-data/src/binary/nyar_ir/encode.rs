//! 将 [`NyarModuleData`] 编码为 `.nyar` 字节。

use super::constants::NyarConstants;
use super::error::NyarIrError;
use super::types::{
    NyarConstant, NyarConstantKind, NyarConstantValue, NyarExport, NyarFunction, NyarImport,
    NyarModuleData, NyarSectionKind, NyarWitnessDispatchEntry,
};

/// `.nyar` 模块编码器。
#[derive(Debug, Default, Clone, Copy)]
pub struct NyarEncoder;

impl NyarEncoder {
    /// 创建编码器。
    pub fn new() -> Self {
        Self
    }

    /// 将 [`NyarModuleData`] 编码为 `.nyar` 二进制格式。
    pub fn encode(&self, data: &NyarModuleData) -> Result<Vec<u8>, NyarIrError> {
        let sections = build_sections(data);
        let size = estimate_size(data, &sections);
        let mut writer = ByteWriter::with_capacity(size);

        write_header(&mut writer, data, sections.len() as i32);
        write_section_headers(&mut writer, &sections, &data.name);
        write_name_section(&mut writer, &data.name);

        for section in sections {
            writer.write(&section.data);
        }

        Ok(writer.into_bytes())
    }
}

struct NyarSection {
    kind: NyarSectionKind,
    data: Vec<u8>,
}

fn build_sections(data: &NyarModuleData) -> Vec<NyarSection> {
    let mut sections = Vec::new();

    if !data.constants.is_empty() {
        sections.push(build_constants_section(&data.constants));
    }
    if !data.functions.is_empty() {
        sections.push(build_functions_section(&data.functions));
    }
    if !data.imports.is_empty() {
        sections.push(build_imports_section(&data.imports));
    }
    if !data.exports.is_empty() {
        sections.push(build_exports_section(&data.exports));
    }
    if !data.witness_entries.is_empty() {
        sections.push(build_witness_entries_section(&data.witness_entries));
    }
    if let Some(code_bytes) = &data.code_bytes {
        if !code_bytes.is_empty() {
            sections.push(build_code_section(code_bytes));
        }
    }

    sections
}

fn build_constants_section(constants: &[NyarConstant]) -> NyarSection {
    let mut writer = ByteWriter::new();
    writer.write_i32_le(constants.len() as i32);

    for constant in constants {
        writer.write_u8(constant.kind as u8);
        match (&constant.kind, &constant.value) {
            (NyarConstantKind::Integer32, NyarConstantValue::Integer32(value)) => {
                writer.write_i32_le(*value);
            }
            (NyarConstantKind::Float64, NyarConstantValue::Float64(value)) => {
                writer.write_f64_le(*value);
            }
            (NyarConstantKind::Boolean, NyarConstantValue::Boolean(value)) => {
                writer.write_u8(u8::from(*value));
            }
            (NyarConstantKind::Null, NyarConstantValue::Null) => {}
            (NyarConstantKind::String, NyarConstantValue::String(value)) => {
                write_binary_string(&mut writer, value);
            }
            (NyarConstantKind::BigInt, NyarConstantValue::BigInt(value)) => {
                writer.write_i32_le(value.len() as i32);
                writer.write(value);
            }
            _ => writer.write_i32_le(0),
        }
    }

    NyarSection {
        kind: NyarSectionKind::Constants,
        data: writer.into_bytes(),
    }
}

fn build_functions_section(functions: &[NyarFunction]) -> NyarSection {
    let mut writer = ByteWriter::new();
    writer.write_i32_le(functions.len() as i32);

    for function in functions {
        write_binary_string(&mut writer, &function.name);
        writer.write_i32_le(function.arity);
        writer.write_i32_le(function.local_count);
        writer.write_i32_le(function.code_offset);
        writer.write_i32_le(function.code_length);
    }

    NyarSection {
        kind: NyarSectionKind::Functions,
        data: writer.into_bytes(),
    }
}

fn build_imports_section(imports: &[NyarImport]) -> NyarSection {
    let mut writer = ByteWriter::new();
    writer.write_i32_le(imports.len() as i32);

    for import in imports {
        writer.write_u8(import.kind as u8);
        write_binary_string(&mut writer, &import.module_name);
        write_binary_string(&mut writer, &import.symbol_name);
    }

    NyarSection {
        kind: NyarSectionKind::Imports,
        data: writer.into_bytes(),
    }
}

fn build_exports_section(exports: &[NyarExport]) -> NyarSection {
    let mut writer = ByteWriter::new();
    writer.write_i32_le(exports.len() as i32);

    for export in exports {
        writer.write_u8(export.kind as u8);
        write_binary_string(&mut writer, &export.symbol_name);
        writer.write_i32_le(export.function_index);
    }

    NyarSection {
        kind: NyarSectionKind::Exports,
        data: writer.into_bytes(),
    }
}

fn build_witness_entries_section(witness_entries: &[NyarWitnessDispatchEntry]) -> NyarSection {
    let mut writer = ByteWriter::new();
    writer.write_i32_le(witness_entries.len() as i32);

    for entry in witness_entries {
        writer.write_i32_le(entry.method_id);
        writer.write_i32_le(entry.type_id);
        write_binary_string(&mut writer, &entry.method_name);
        writer.write_i32_le(entry.function_index);
        writer.write_i32_le(entry.interface_id);
        writer.write_i32_le(entry.interface_method_index);
    }

    NyarSection {
        kind: NyarSectionKind::WitnessEntries,
        data: writer.into_bytes(),
    }
}

fn build_code_section(code_bytes: &[u8]) -> NyarSection {
    NyarSection {
        kind: NyarSectionKind::Code,
        data: code_bytes.to_vec(),
    }
}

fn write_header(writer: &mut ByteWriter, data: &NyarModuleData, section_count: i32) {
    writer.write_u32_be(NyarConstants::MAGIC_VALUE);
    writer.write_u32_le(data.version);
    writer.write_i32_le(section_count);

    let name_offset =
        NyarConstants::HEADER_SIZE as i32 + section_count * NyarConstants::SECTION_HEADER_SIZE as i32;
    writer.write_i32_le(name_offset);
}

fn write_section_headers(writer: &mut ByteWriter, sections: &[NyarSection], module_name: &str) {
    let name_byte_count = module_name.len();
    let data_start = NyarConstants::HEADER_SIZE
        + sections.len() * NyarConstants::SECTION_HEADER_SIZE
        + 4
        + name_byte_count;

    let mut current_offset = data_start as i32;
    for section in sections {
        writer.write_u8(section.kind as u8);
        writer.write_i32_le(current_offset);
        writer.write_i32_le(section.data.len() as i32);
        current_offset += section.data.len() as i32;
    }
}

fn write_name_section(writer: &mut ByteWriter, name: &str) {
    let name_bytes = name.as_bytes();
    writer.write_i32_le(name_bytes.len() as i32);
    writer.write(name_bytes);
}

fn write_binary_string(writer: &mut ByteWriter, value: &str) {
    let bytes = value.as_bytes();
    writer.write_i32_le(bytes.len() as i32);
    writer.write(bytes);
}

fn estimate_size(data: &NyarModuleData, sections: &[NyarSection]) -> usize {
    let mut size = NyarConstants::HEADER_SIZE;
    size += sections.len() * NyarConstants::SECTION_HEADER_SIZE;
    size += 4 + data.name.len();
    for section in sections {
        size += section.data.len();
    }
    size + 256
}

struct ByteWriter {
    buffer: Vec<u8>,
}

impl ByteWriter {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn with_capacity(capacity: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(capacity),
        }
    }

    fn write_u8(&mut self, value: u8) {
        self.buffer.push(value);
    }

    fn write_u32_be(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_be_bytes());
    }

    fn write_u32_le(&mut self, value: u32) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    fn write_i32_le(&mut self, value: i32) {
        self.write_u32_le(value as u32);
    }

    fn write_f64_le(&mut self, value: f64) {
        self.buffer.extend_from_slice(&value.to_le_bytes());
    }

    fn write(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    fn into_bytes(self) -> Vec<u8> {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::{NyarConstantKind, NyarConstantValue, NyarDecoder, NyarFunction};

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
    fn encoded_header_layout_matches_csharp() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        assert_eq!(&encoded[0..4], b"NYAR");
        assert_eq!(u32::from_le_bytes(encoded[4..8].try_into().unwrap()), 1);
        assert_eq!(i32::from_le_bytes(encoded[8..12].try_into().unwrap()), 3);
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        let decoded = NyarDecoder::new().decode(&encoded).expect("decode");
        assert_eq!(decoded, module);
    }
}

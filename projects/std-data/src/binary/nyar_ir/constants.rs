/// Nyar 字节码模块格式常量。
pub struct NyarConstants;

impl NyarConstants {
    /// 魔数值（`0x4E594152`，大端显示为 `"NYAR"`）。
    pub const MAGIC_VALUE: u32 = 0x4E59_4152;
    /// 当前版本号。
    pub const CURRENT_VERSION: u32 = 1;
    /// 头部大小（16 字节）。
    pub const HEADER_SIZE: usize = 16;
    /// 段头大小（9 字节：`Kind` 1 + `Offset` 4 + `Size` 4）。
    pub const SECTION_HEADER_SIZE: usize = 9;
    /// `.nyar` 文件魔数字节（`"NYAR"` 大端序）。
    pub const MAGIC_NUMBER: &[u8; 4] = b"NYAR";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::{
        NyarConstant, NyarConstantKind, NyarConstantValue, NyarEncoder, NyarFunction, NyarModuleData,
    };

    fn minimal_module() -> NyarModuleData {
        let code_bytes = vec![
            0x10, 0x00, 0x00, 0x00, 0x00,
            0x10, 0x01, 0x00, 0x00, 0x00,
            0x30,
            0x05,
        ];

        NyarModuleData {
            version: NyarConstants::CURRENT_VERSION,
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
    fn magic_matches_nyar_bytes() {
        let magic = NyarConstants::MAGIC_VALUE.to_be_bytes();
        assert_eq!(&magic, NyarConstants::MAGIC_NUMBER);
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        assert_eq!(&encoded[0..4], NyarConstants::MAGIC_NUMBER);

        let decoded = super::super::decode::NyarDecoder::new()
            .decode(&encoded)
            .expect("decode");
        assert_eq!(decoded.version, NyarConstants::CURRENT_VERSION);
        assert_eq!(decoded.code_bytes, module.code_bytes);
    }
}

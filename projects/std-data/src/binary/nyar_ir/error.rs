use std::fmt;

/// Nyar IR 编解码与验证错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NyarIrError {
    /// 文件头无效。
    InvalidHeader {
        /// 读取到的魔数。
        magic: u32,
        /// 读取到的版本号。
        version: u32,
    },
    /// 未知的常量类型。
    UnknownConstantKind(u8),
    /// 数据意外结束。
    UnexpectedEof,
    /// 自定义消息。
    Message(String),
}

impl fmt::Display for NyarIrError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidHeader { magic, version } => {
                write!(
                    f,
                    "无效的 .nyar 文件头：magic=0x{magic:08X}, version={version}"
                )
            }
            Self::UnknownConstantKind(kind) => write!(f, "未知的常量类型：{kind}"),
            Self::UnexpectedEof => write!(f, "数据意外结束"),
            Self::Message(message) => f.write_str(message),
        }
    }
}

impl std::error::Error for NyarIrError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binary::nyar_ir::{
        NyarConstant, NyarConstantKind, NyarConstantValue, NyarEncoder, NyarFunction, NyarModuleData,
    };

    fn minimal_module() -> NyarModuleData {
        let code_bytes = vec![
            0x10, 0x00, 0x00, 0x00, 0x00, // const 0
            0x10, 0x01, 0x00, 0x00, 0x00, // const 1
            0x30,                         // i32_add
            0x05,                         // return
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

        assert_eq!(decoded.name, "test");
        assert_eq!(decoded.functions.len(), 1);
        assert_eq!(decoded.constants.len(), 2);
        assert_eq!(decoded.code_bytes, module.code_bytes);
    }
}

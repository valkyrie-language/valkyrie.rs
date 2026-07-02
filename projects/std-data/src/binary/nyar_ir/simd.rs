/// Nyar SIMD 二级指令编码。
///
/// 它是 `simd` 前缀指令的一部分，不是独立的一级 opcode。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NyarSimdCode {
    /// v128 常量加载。
    V128Const = 0x00,
    /// v128 内存加载。
    V128Load = 0x01,
    /// v128 内存存储。
    V128Store = 0x02,
    /// i32x4 向量加法。
    I32X4Add = 0x03,
    /// i32x4 向量减法。
    I32X4Sub = 0x04,
    /// i32x4 向量乘法。
    I32X4Mul = 0x05,
    /// f32x4 向量加法。
    F32X4Add = 0x06,
    /// f32x4 向量减法。
    F32X4Sub = 0x07,
    /// f32x4 向量乘法。
    F32X4Mul = 0x08,
    /// i8x16 通道广播。
    I8X16Splat = 0x09,
    /// i16x8 通道广播。
    I16X8Splat = 0x0A,
    /// i32x4 通道广播。
    I32X4Splat = 0x0B,
    /// f32x4 通道广播。
    F32X4Splat = 0x0C,
    /// f64x2 通道广播。
    F64X2Splat = 0x0D,
    /// i8x16 提取有符号通道。
    I8X16ExtractLaneS = 0x0E,
    /// i8x16 替换通道。
    I8X16ReplaceLane = 0x0F,
    /// i32x4 提取通道。
    I32X4ExtractLane = 0x10,
    /// i32x4 替换通道。
    I32X4ReplaceLane = 0x11,
    /// v128 按位与。
    V128And = 0x12,
    /// v128 按位或。
    V128Or = 0x13,
    /// v128 按位异或。
    V128Xor = 0x14,
    /// v128 按位取反。
    V128Not = 0x15,
    /// v128 位选择。
    V128BitSelect = 0x16,
    /// i64x2 向量加法。
    I64X2Add = 0x17,
    /// i64x2 向量减法。
    I64X2Sub = 0x18,
    /// f64x2 向量加法。
    F64X2Add = 0x19,
    /// f64x2 向量减法。
    F64X2Sub = 0x1A,
    /// f64x2 向量乘法。
    F64X2Mul = 0x1B,
    /// i8x16 通道重排。
    I8X16Shuffle = 0x1C,
    /// i32x4 相等比较。
    I32X4Eq = 0x1D,
    /// f32x4 相等比较。
    F32X4Eq = 0x1E,
    /// v128 任意为真。
    V128AnyTrue = 0x1F,
}

impl NyarSimdCode {
    /// 将原始字节值解析为 SIMD 二级码。
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::V128Const),
            0x01 => Some(Self::V128Load),
            0x02 => Some(Self::V128Store),
            0x03 => Some(Self::I32X4Add),
            0x04 => Some(Self::I32X4Sub),
            0x05 => Some(Self::I32X4Mul),
            0x06 => Some(Self::F32X4Add),
            0x07 => Some(Self::F32X4Sub),
            0x08 => Some(Self::F32X4Mul),
            0x09 => Some(Self::I8X16Splat),
            0x0A => Some(Self::I16X8Splat),
            0x0B => Some(Self::I32X4Splat),
            0x0C => Some(Self::F32X4Splat),
            0x0D => Some(Self::F64X2Splat),
            0x0E => Some(Self::I8X16ExtractLaneS),
            0x0F => Some(Self::I8X16ReplaceLane),
            0x10 => Some(Self::I32X4ExtractLane),
            0x11 => Some(Self::I32X4ReplaceLane),
            0x12 => Some(Self::V128And),
            0x13 => Some(Self::V128Or),
            0x14 => Some(Self::V128Xor),
            0x15 => Some(Self::V128Not),
            0x16 => Some(Self::V128BitSelect),
            0x17 => Some(Self::I64X2Add),
            0x18 => Some(Self::I64X2Sub),
            0x19 => Some(Self::F64X2Add),
            0x1A => Some(Self::F64X2Sub),
            0x1B => Some(Self::F64X2Mul),
            0x1C => Some(Self::I8X16Shuffle),
            0x1D => Some(Self::I32X4Eq),
            0x1E => Some(Self::F32X4Eq),
            0x1F => Some(Self::V128AnyTrue),
            _ => None,
        }
    }
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
    fn simd_opcode_values_match_csharp() {
        assert_eq!(NyarSimdCode::V128Const as u8, 0x00);
        assert_eq!(NyarSimdCode::V128AnyTrue as u8, 0x1F);
    }

    #[test]
    fn round_trip_minimal_module() {
        let module = minimal_module();
        let encoded = NyarEncoder::new().encode(&module).expect("encode");
        let decoded = super::super::decode::NyarDecoder::new()
            .decode(&encoded)
            .expect("decode");
        assert_eq!(decoded.code_bytes, module.code_bytes);
    }
}

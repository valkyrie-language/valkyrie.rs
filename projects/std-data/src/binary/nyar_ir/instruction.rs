use super::opcode::NyarHeadCode;

/// 指令编码形态。
///
/// 描述编码层的取值方式，不等同于运行时语义分类。
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum NyarInstructionForm {
    /// 无效形态。
    #[default]
    Invalid = 0,
    /// 仅包含一级头码，不带立即数字段。
    Plain = 1,
    /// 头码后跟一个 `i32` 立即数字段。
    Imm1 = 2,
    /// 头码后跟两个 `i32` 立即数字段。
    Imm2 = 3,
    /// 头码后跟三个 `i32` 立即数字段。
    Imm3 = 4,
    /// 前缀形态；一级头码后还需继续解码二级子码与扩展负载。
    Prefixed = 5,
}

/// 表示一条已经从代码字节流中解出的预解码指令（16 字节固定布局）。
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct NyarInstruction {
    /// 指令头码字段。
    pub code: NyarHeadCode,
    /// 指令在原始代码字节流中的编码长度；`0` 表示无效指令。
    pub size: u8,
    /// 对齐填充。
    _padding: [u8; 2],
    /// 第一立即数缓存槽位。
    pub operand1: i32,
    /// 第二立即数缓存槽位。
    pub operand2: i32,
    /// 第三立即数缓存槽位。
    pub operand3: i32,
}

impl NyarInstruction {
    /// 构造一条预解码指令；长度由头码推导。
    pub fn new(code: NyarHeadCode, operand1: i32, operand2: i32, operand3: i32) -> Self {
        Self {
            code,
            size: Self::code_size(code),
            _padding: [0; 2],
            operand1,
            operand2,
            operand3,
        }
    }

    /// 构造一条已经完成字节级解码的指令。
    pub fn with_size(
        code: NyarHeadCode,
        operand1: i32,
        operand2: i32,
        operand3: i32,
        size: u8,
    ) -> Self {
        Self {
            code,
            size,
            _padding: [0; 2],
            operand1,
            operand2,
            operand3,
        }
    }

    /// 当前实例是否为有效指令。
    pub fn is_valid(self) -> bool {
        self.size != 0
    }

    /// 根据头码获取指令编码形态。
    pub fn get_form(head: NyarHeadCode) -> NyarInstructionForm {
        match head {
            NyarHeadCode::CallWitness
            | NyarHeadCode::CallIntrinsic => NyarInstructionForm::Imm2,
            NyarHeadCode::CallDynamic => NyarInstructionForm::Imm3,
            NyarHeadCode::AccessDynamic => NyarInstructionForm::Imm2,
            NyarHeadCode::Simd => NyarInstructionForm::Prefixed,
            NyarHeadCode::Jump
            | NyarHeadCode::JumpIfTrue
            | NyarHeadCode::JumpIfFalse
            | NyarHeadCode::Call
            | NyarHeadCode::CallStatic
            | NyarHeadCode::TailCall
            | NyarHeadCode::Catch
            | NyarHeadCode::Resume
            | NyarHeadCode::EffectHandle
            | NyarHeadCode::Const
            | NyarHeadCode::BuiltinCall
            | NyarHeadCode::LoadLocal
            | NyarHeadCode::StoreLocal
            | NyarHeadCode::LoadArg
            | NyarHeadCode::StoreArg
            | NyarHeadCode::LoadGlobal
            | NyarHeadCode::StoreGlobal
            | NyarHeadCode::Alloc
            | NyarHeadCode::I32Load
            | NyarHeadCode::I32Store
            | NyarHeadCode::I64Load
            | NyarHeadCode::I64Store
            | NyarHeadCode::NewObject
            | NyarHeadCode::FieldStore
            | NyarHeadCode::NewClosure
            | NyarHeadCode::GetUpvalue
            | NyarHeadCode::SetUpvalue
            | NyarHeadCode::AccessStatic
            | NyarHeadCode::AccessWitness
            | NyarHeadCode::InlineCacheUpdate
            | NyarHeadCode::CallNative
            | NyarHeadCode::LoadNativeLib
            | NyarHeadCode::GetNativeFunc => NyarInstructionForm::Imm1,
            _ if NyarHeadCode::is_defined(head as u8) => NyarInstructionForm::Plain,
            _ => NyarInstructionForm::Invalid,
        }
    }

    /// 根据头码获取对应指令的编码长度。
    pub fn code_size(head: NyarHeadCode) -> u8 {
        match Self::get_form(head) {
            NyarInstructionForm::Plain => 1,
            NyarInstructionForm::Imm1 | NyarInstructionForm::Prefixed => 5,
            NyarInstructionForm::Imm2 => 9,
            NyarInstructionForm::Imm3 => 13,
            NyarInstructionForm::Invalid => 0,
        }
    }
}

const _: () = assert!(core::mem::size_of::<NyarInstruction>() == 16);

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
    fn instruction_layout_is_16_bytes() {
        assert_eq!(core::mem::size_of::<NyarInstruction>(), 16);
    }

    #[test]
    fn code_sizes_match_csharp() {
        assert_eq!(NyarInstruction::code_size(NyarHeadCode::Const), 5);
        assert_eq!(NyarInstruction::code_size(NyarHeadCode::I32Add), 1);
        assert_eq!(NyarInstruction::code_size(NyarHeadCode::Return), 1);
        assert_eq!(NyarInstruction::code_size(NyarHeadCode::CallDynamic), 13);
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

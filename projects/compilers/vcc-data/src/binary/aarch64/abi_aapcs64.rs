//! AAPCS64 函数体构建器（JNI 胶水子集）。

use super::{A64Encoder, A64Instruction, EncodedModule, RegX};

/// AAPCS64 函数构建器。
#[derive(Debug, Default)]
pub struct Aapcs64FunctionBuilder {
    encoder: A64Encoder,
    stack_reserve: u32,
}

impl Aapcs64FunctionBuilder {
    /// 新建。
    pub fn new() -> Self {
        Self::default()
    }

    /// 追加指令。
    pub fn push(&mut self, instruction: A64Instruction) {
        self.encoder.push(instruction);
    }

    /// 预留栈（16 字节对齐）。
    pub fn reserve_stack(&mut self, bytes: u32) {
        self.stack_reserve = (bytes + 15) & !15;
    }

    /// 完成编码。
    pub fn finish(self) -> EncodedModule {
        self.encoder.finish()
    }

    /// 将 64 位立即数写入 `dst`（最多 4 条指令）。
    pub fn mov_imm64(&mut self, dst: RegX, value: u64) {
        self.push(A64Instruction::MovzImm16 { dst, imm: value as u16 });
        if (value >> 16) != 0 || value > u16::MAX as u64 {
            self.push(A64Instruction::MovkImm16Shift16 { dst, imm: (value >> 16) as u16 });
        }
        if (value >> 32) != 0 {
            self.push(A64Instruction::MovkImm16Shift32 { dst, imm: (value >> 32) as u16 });
        }
        if (value >> 48) != 0 {
            self.push(A64Instruction::MovkImm16Shift48 { dst, imm: (value >> 48) as u16 });
        }
    }
}

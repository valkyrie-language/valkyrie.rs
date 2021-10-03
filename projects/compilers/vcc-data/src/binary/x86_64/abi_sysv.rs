use super::{Reg64, X64Encoder, X64Instruction};

/// System V AMD64 函数体构建器（无 MSVC shadow space）。
#[derive(Debug, Default)]
pub struct SysvFunctionBuilder {
    encoder: X64Encoder,
    stack_reserve: u32,
}

impl SysvFunctionBuilder {
    /// 创建新的函数构建器。
    pub fn new() -> Self {
        Self::default()
    }

    /// 返回底层编码器。
    pub fn encoder(&self) -> &X64Encoder {
        &self.encoder
    }

    /// 返回可变底层编码器。
    pub fn encoder_mut(&mut self) -> &mut X64Encoder {
        &mut self.encoder
    }

    /// 追加一条指令。
    pub fn push(&mut self, instruction: X64Instruction) {
        self.encoder.push(instruction);
    }

    /// 返回当前函数的栈预留量，供 `Return` terminator 恢复栈后 `ret`。
    pub fn stack_reserve(&self) -> u32 {
        self.stack_reserve
    }

    /// 预留栈空间，向上对齐到 16 字节。
    ///
    /// SysV `_start` 入口由内核直接跳转（非 `call`），RSP 在入口处已 16 字节对齐，
    /// 故 `sub rsp, N` 中 `N` 应为 0 mod 16 以保持对齐。
    pub fn reserve_stack(&mut self, bytes: u32) {
        self.stack_reserve = align16(bytes);
    }

    /// 确保栈空间至少为 `bytes` 字节，取 max 不缩小已有预留。
    ///
    /// 用于在 prologue 发射前累积多个栈区需求（值类型区 + suspend 帧区等），
    /// 避免后续 lowering 覆盖 `stack_reserve` 导致 prologue/epilogue 不匹配。
    pub fn ensure_stack(&mut self, bytes: u32) {
        let aligned = align16(bytes);
        self.stack_reserve = self.stack_reserve.max(aligned);
    }

    /// 写入函数 prologue。
    pub fn emit_prologue(&mut self) {
        if self.stack_reserve > 0 {
            self.encoder.push(X64Instruction::SubRsp(self.stack_reserve));
        }
    }

    /// 写入函数 epilogue 并返回。
    pub fn emit_epilogue_and_ret(&mut self) {
        if self.stack_reserve > 0 {
            self.encoder.push(X64Instruction::AddRsp(self.stack_reserve));
        }
        self.encoder.push(X64Instruction::Ret);
    }

    /// 写入 Linux `exit(code)` 序列（不返回）。
    pub fn emit_linux_exit(&mut self, code: u32) {
        self.encoder.push(X64Instruction::MovRegImm32(Reg64::Rax, 60));
        if code == 0 {
            self.encoder.push(X64Instruction::XorReg { dst: Reg64::Rdi, src: Reg64::Rdi });
        }
        else {
            self.encoder.push(X64Instruction::MovRegImm32(Reg64::Rdi, code));
        }
        self.encoder.push(X64Instruction::Syscall);
    }

    /// 完成编码。
    pub fn finish(self) -> super::encode::EncodedModule {
        self.encoder.finish()
    }
}

/// 将栈预留量向上对齐到 16 字节。
fn align16(value: u32) -> u32 {
    (value + 15) & !15
}

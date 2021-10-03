use super::{Reg64, X64Encoder, X64Instruction};

/// Microsoft x64 调用约定 shadow space 大小。
pub const SHADOW_STACK_SIZE: u32 = 0x20;

/// 带 shadow space 的 MSVC 函数体构建器。
#[derive(Debug, Default)]
pub struct MsvcFunctionBuilder {
    encoder: X64Encoder,
    stack_reserve: u32,
}

impl MsvcFunctionBuilder {
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

    /// 预留栈空间（含 shadow space），并对齐到 8 mod 16 以保证 `call` 后 RSP 16 字节对齐。
    ///
    /// Windows x64 ABI 要求 `call` 前 RSP 16 字节对齐；`call` 压入 8 字节返回地址后
    /// RSP 为 8 mod 16。`sub rsp, N` 中 `N` 必须为 8 mod 16 才能重新对齐。
    pub fn reserve_stack(&mut self, extra: u32) {
        let needed = SHADOW_STACK_SIZE.max(extra + SHADOW_STACK_SIZE);
        self.stack_reserve = align_stack_off8(needed);
    }

    /// 确保栈空间至少为 `extra + SHADOW_STACK_SIZE` 字节，取 max 不缩小已有预留。
    ///
    /// 用于在 prologue 发射前累积多个栈区需求（值类型区 + suspend 帧区等），
    /// 避免后续 lowering 覆盖 `stack_reserve` 导致 prologue/epilogue 不匹配。
    pub fn ensure_stack(&mut self, extra: u32) {
        let needed = SHADOW_STACK_SIZE.max(extra + SHADOW_STACK_SIZE);
        let aligned = align_stack_off8(needed);
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

    /// 写入 `return 0` 序列。
    pub fn emit_return_zero(&mut self) {
        self.encoder.push(X64Instruction::XorReg { dst: Reg64::Rax, src: Reg64::Rax });
    }

    /// 完成编码。
    pub fn finish(self) -> super::encode::EncodedModule {
        self.encoder.finish()
    }
}

/// 将栈预留量对齐到 8 mod 16，保证 `call` 后 `sub rsp, N` 使 RSP 16 字节对齐。
///
/// `call` 前 RSP 为 16 字节对齐（0 mod 16），`call` 压入 8 字节返回地址后
/// RSP 变为 8 mod 16。`sub rsp, N` 中 `N` 为 8 mod 16 时 RSP 恢复为 0 mod 16。
/// `n == 0` 时返回 0（不发射 `sub rsp`）。
fn align_stack_off8(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    let aligned = (n + 7) & !7;
    if aligned % 16 == 0 { aligned + 8 } else { aligned }
}

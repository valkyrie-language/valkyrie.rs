use super::reg::Reg64;

/// x86-64 条件码，用于 `SetccRax` 与条件跳转语义映射。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConditionCode {
    /// 相等（ZF=1），`sete`。
    Equal,
    /// 不相等（ZF=0），`setne`。
    NotEqual,
    /// 有符号小于（SF!=OF），`setl`。
    Less,
    /// 有符号小于等于（ZF=1 || SF!=OF），`setle`。
    LessEqual,
    /// 有符号大于（ZF=0 && SF==OF），`setg`。
    Greater,
    /// 有符号大于等于（SF==OF），`setge`。
    GreaterEqual,
}

impl ConditionCode {
    /// 返回 `setcc` 指令的操作码字节（0x90 组，`0F 90+cc`）。
    pub fn setcc_opcode(self) -> u8 {
        match self {
            Self::Equal => 0x94,
            Self::NotEqual => 0x95,
            Self::Less => 0x9C,
            Self::LessEqual => 0x9E,
            Self::Greater => 0x9F,
            Self::GreaterEqual => 0x9D,
        }
    }
}

/// x86-64 指令 IR。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum X64Instruction {
    /// 标签锚点。
    Label(String),
    /// `sub rsp, imm`
    SubRsp(u32),
    /// `add rsp, imm`
    AddRsp(u32),
    /// `mov reg, imm32`（零扩展写入 64 位寄存器）。
    MovRegImm32(Reg64, u32),
    /// `mov reg, imm64`
    MovRegImm64(Reg64, u64),
    /// `mov dst, src`
    MovRegReg {
        /// 目标寄存器。
        dst: Reg64,
        /// 源寄存器。
        src: Reg64,
    },
    /// `xor dst, src`
    XorReg {
        /// 目标寄存器。
        dst: Reg64,
        /// 源寄存器。
        src: Reg64,
    },
    /// `lea dst, [rip + label]`
    LeaRipRelative {
        /// 目标寄存器。
        dst: Reg64,
        /// 目标标签。
        label: String,
    },
    /// `call [rip + import_slot]`
    CallImport {
        /// 导入槽索引。
        slot: usize,
    },
    /// 在 `call` 前向栈写入第 5 个参数（`[rsp+0x20]`）。
    MovStackArgQword {
        /// 立即数。
        value: u64,
    },
    /// `lea dst, [rsp + offset]`
    LeaRspOffset {
        /// 目标寄存器。
        dst: Reg64,
        /// 相对于 `RSP` 的字节偏移。
        offset: i32,
    },
    /// `mov dst, [base + disp8]`
    MovRegMemReg {
        /// 目标寄存器。
        dst: Reg64,
        /// 基址寄存器。
        base: Reg64,
        /// 字节偏移。
        offset: i8,
    },
    /// `mov dst, [rsp + offset]`（64 位读取，32 位写零扩展）。
    MovRegRspOffset {
        /// 目标寄存器。
        dst: Reg64,
        /// 相对于 `RSP` 的字节偏移。
        offset: i32,
    },
    /// `mov dword [rsp + offset], src`（32 位写入，i32 标量存储）。
    MovRspOffsetReg32 {
        /// 相对于 `RSP` 的字节偏移。
        offset: i32,
        /// 源寄存器。
        src: Reg64,
    },
    /// `mov dword [base + offset], imm32`
    MovMemRegImm32 {
        /// 基址寄存器。
        base: Reg64,
        /// 字节偏移。
        offset: i8,
        /// 立即数。
        value: u32,
    },
    /// `mov dword [rsp + offset], imm32`
    MovRspOffsetImm32 {
        /// 相对于 `RSP` 的字节偏移。
        offset: i32,
        /// 立即数。
        value: u32,
    },
    /// `add dst, src`（64 位加法）。
    AddRegReg {
        /// 目标寄存器（同时是第一源，结果写回）。
        dst: Reg64,
        /// 第二源寄存器。
        src: Reg64,
    },
    /// `sub dst, src`（64 位减法）。
    SubRegReg {
        /// 目标寄存器（同时是第一源，结果写回）。
        dst: Reg64,
        /// 第二源寄存器。
        src: Reg64,
    },
    /// `imul dst, src`（64 位有符号乘法，结果在 dst）。
    ImulRegReg {
        /// 目标寄存器（同时是第一源）。
        dst: Reg64,
        /// 第二源寄存器。
        src: Reg64,
    },
    /// `cqo; idiv src`（64 位有符号除法，商在 RAX，余数在 RDX）。
    IdivReg {
        /// 除数寄存器。
        divisor: Reg64,
    },
    /// `cmp dst, src`（64 位比较，设置 FLAGS）。
    CmpRegReg {
        /// 第一操作数寄存器。
        dst: Reg64,
        /// 第二操作数寄存器。
        src: Reg64,
    },
    /// `setcc al; movzx rax, al`（将比较结果转为 0/1 存入 RAX）。
    SetccRax {
        /// 条件码。
        cc: ConditionCode,
    },
    /// `neg dst`（64 位取负）。
    NegReg {
        /// 目标寄存器。
        dst: Reg64,
    },
    /// `cmp reg, imm32`
    CmpRegImm32(Reg64, u32),
    /// `test dst, src`
    TestRegReg {
        /// 目标寄存器。
        dst: Reg64,
        /// 源寄存器。
        src: Reg64,
    },
    /// `je label`
    Je(String),
    /// `jne label`
    Jne(String),
    /// `jmp label`
    Jmp(String),
    /// `call label`（`E8 disp32`，相对当前指令末尾的 `.text` 标签）
    CallLabel(String),
    /// `call reg`
    CallReg(Reg64),
    /// `syscall`
    Syscall,
    /// `ret`
    Ret,
}

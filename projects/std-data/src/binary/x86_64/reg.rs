/// x86-64 通用寄存器。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Reg64 {
    /// `RAX`
    Rax,
    /// `RCX`
    Rcx,
    /// `RDX`
    Rdx,
    /// `RBX`
    Rbx,
    /// `RSP`
    Rsp,
    /// `RBP`
    Rbp,
    /// `RSI`
    Rsi,
    /// `RDI`
    Rdi,
    /// `R8`
    R8,
    /// `R9`
    R9,
    /// `R10`
    R10,
    /// `R11`
    R11,
    /// `R12`
    R12,
    /// `R13`
    R13,
    /// `R14`
    R14,
    /// `R15`
    R15,
}

impl Reg64 {
    /// 返回 `true` 时该寄存器需要 REX 前缀（R8-R15）。
    pub fn needs_rex(self) -> bool {
        matches!(self, Self::R8 | Self::R9 | Self::R10 | Self::R11 | Self::R12 | Self::R13 | Self::R14 | Self::R15)
    }

    pub(crate) fn low3(self) -> u8 {
        match self {
            Self::Rax => 0,
            Self::Rcx => 1,
            Self::Rdx => 2,
            Self::Rbx => 3,
            Self::Rsp => 4,
            Self::Rbp => 5,
            Self::Rsi => 6,
            Self::Rdi => 7,
            Self::R8 => 0,
            Self::R9 => 1,
            Self::R10 => 2,
            Self::R11 => 3,
            Self::R12 => 4,
            Self::R13 => 5,
            Self::R14 => 6,
            Self::R15 => 7,
        }
    }

    /// 返回 `true` 时该寄存器是扩展寄存器（R8-R15），需要 REX 前缀。
    pub fn is_extended(self) -> bool {
        self.needs_rex()
    }
}

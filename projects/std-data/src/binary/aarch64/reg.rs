//! AArch64 通用寄存器。

/// 64 位通用寄存器 `X0`–`X30` 与栈指针。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegX {
    /// `X0`
    X0,
    /// `X1`
    X1,
    /// `X2`
    X2,
    /// `X3`
    X3,
    /// `X4`
    X4,
    /// `X5`
    X5,
    /// `X6`
    X6,
    /// `X7`
    X7,
    /// `X8`
    X8,
    /// `X9`
    X9,
    /// `X10`
    X10,
    /// `X11`
    X11,
    /// `X12`
    X12,
    /// `X13`
    X13,
    /// `X14`
    X14,
    /// `X15`
    X15,
    /// `X16`
    X16,
    /// `X17`
    X17,
    /// `X18`
    X18,
    /// `X19`
    X19,
    /// `X20`
    X20,
    /// `X21`
    X21,
    /// `X22`
    X22,
    /// `X23`
    X23,
    /// `X24`
    X24,
    /// `X25`
    X25,
    /// `X26`
    X26,
    /// `X27`
    X27,
    /// `X28`
    X28,
    /// `X29` / FP
    X29,
    /// `X30` / LR
    X30,
    /// `SP`
    Sp,
    /// 零寄存器 `XZR`
    Xzr,
}

impl RegX {
    pub(crate) fn id(self) -> u32 {
        match self {
            Self::X0 => 0,
            Self::X1 => 1,
            Self::X2 => 2,
            Self::X3 => 3,
            Self::X4 => 4,
            Self::X5 => 5,
            Self::X6 => 6,
            Self::X7 => 7,
            Self::X8 => 8,
            Self::X9 => 9,
            Self::X10 => 10,
            Self::X11 => 11,
            Self::X12 => 12,
            Self::X13 => 13,
            Self::X14 => 14,
            Self::X15 => 15,
            Self::X16 => 16,
            Self::X17 => 17,
            Self::X18 => 18,
            Self::X19 => 19,
            Self::X20 => 20,
            Self::X21 => 21,
            Self::X22 => 22,
            Self::X23 => 23,
            Self::X24 => 24,
            Self::X25 => 25,
            Self::X26 => 26,
            Self::X27 => 27,
            Self::X28 => 28,
            Self::X29 => 29,
            Self::X30 => 30,
            Self::Sp => 31,
            Self::Xzr => 31,
        }
    }
}

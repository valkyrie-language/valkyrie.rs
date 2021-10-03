//! AArch64 指令 IR（JNI 胶水子集）。

use super::RegX;

/// AArch64 指令。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum A64Instruction {
    /// 标签锚点。
    Label(String),
    /// `mov xd, xn`
    MovRegReg {
        /// 目标。
        dst: RegX,
        /// 源。
        src: RegX,
    },
    /// `movz xd, #imm16`（`hw` = 0）。
    MovzImm16 {
        /// 目标。
        dst: RegX,
        /// 16 位立即数。
        imm: u16,
    },
    /// `movk xd, #imm16, lsl #16`
    MovkImm16Shift16 {
        /// 目标。
        dst: RegX,
        /// 16 位立即数。
        imm: u16,
    },
    /// `movk xd, #imm16, lsl #32`
    MovkImm16Shift32 {
        /// 目标。
        dst: RegX,
        /// 16 位立即数。
        imm: u16,
    },
    /// `movk xd, #imm16, lsl #48`
    MovkImm16Shift48 {
        /// 目标。
        dst: RegX,
        /// 16 位立即数。
        imm: u16,
    },
    /// `ldr xt, [xn, #imm]`（64-bit，imm 字节对齐 /8）。
    LdrRegOffset {
        /// 目标。
        dst: RegX,
        /// 基址。
        base: RegX,
        /// 字节偏移（须 8 对齐）。
        offset: u32,
    },
    /// `str xt, [xn, #imm]`
    StrRegOffset {
        /// 源。
        src: RegX,
        /// 基址。
        base: RegX,
        /// 字节偏移。
        offset: u32,
    },
    /// `adr xd, label`
    Adr {
        /// 目标。
        dst: RegX,
        /// 标签。
        label: String,
    },
    /// `add xd, xn, #imm12`
    AddImm12 {
        /// 目标。
        dst: RegX,
        /// 源。
        src: RegX,
        /// 立即数。
        imm: u32,
    },
    /// `bl label`
    Bl {
        /// 目标标签。
        label: String,
    },
    /// `blr xn`
    Blr(RegX),
    /// `cbz xt, label`
    Cbz {
        /// 测试寄存器。
        reg: RegX,
        /// 目标。
        label: String,
    },
    /// `ret`
    Ret,
}

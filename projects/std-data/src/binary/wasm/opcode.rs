//! `WASM` 操作码语义枚举与助记符查表。
//!
//! 叶子操作码、`GC` 前缀（`0xFB`）与杂项前缀（`0xFC`）的二进制字面量
//! **仅**出现在本模块；发射端应通过枚举与 [`crate::binary::wasm::encode`] 助手编码。

/// `WASM` 叶子操作码（不含 `0xFB` / `0xFC` 前缀字节本身）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WasmOpcode {
    /// `unreachable`
    Unreachable = 0x00,
    /// `nop`
    Nop = 0x01,
    /// `block`
    Block = 0x02,
    /// `loop`
    Loop = 0x03,
    /// `if`
    If = 0x04,
    /// `else`
    Else = 0x05,
    /// `end`
    End = 0x0B,
    /// `br`
    Br = 0x0C,
    /// `br_if`
    BrIf = 0x0D,
    /// `br_table`
    BrTable = 0x0E,
    /// `return`
    Return = 0x0F,
    /// `call`
    Call = 0x10,
    /// `call_indirect`
    CallIndirect = 0x11,
    /// `drop`
    Drop = 0x1A,
    /// `select`（无类型立即数）
    Select = 0x1B,
    /// `select`（带结果类型立即数）
    SelectTyped = 0x1C,
    /// `local.get`
    LocalGet = 0x20,
    /// `local.set`
    LocalSet = 0x21,
    /// `local.tee`
    LocalTee = 0x22,
    /// `global.get`
    GlobalGet = 0x23,
    /// `global.set`
    GlobalSet = 0x24,
    /// `i32.load`
    I32Load = 0x28,
    /// `i64.load`
    I64Load = 0x29,
    /// `f32.load`
    F32Load = 0x2A,
    /// `f64.load`
    F64Load = 0x2B,
    /// `i32.load8_s`
    I32Load8S = 0x2C,
    /// `i32.load8_u`
    I32Load8U = 0x2D,
    /// `i32.load16_s`
    I32Load16S = 0x2E,
    /// `i32.load16_u`
    I32Load16U = 0x2F,
    /// `i32.store`
    I32Store = 0x36,
    /// `i64.store`
    I64Store = 0x37,
    /// `f32.store`
    F32Store = 0x38,
    /// `f64.store`
    F64Store = 0x39,
    /// `i32.store8`
    I32Store8 = 0x3A,
    /// `i32.store16`
    I32Store16 = 0x3B,
    /// `memory.size`
    MemorySize = 0x3F,
    /// `memory.grow`
    MemoryGrow = 0x40,
    /// `i32.const`
    I32Const = 0x41,
    /// `i64.const`
    I64Const = 0x42,
    /// `f32.const`
    F32Const = 0x43,
    /// `f64.const`
    F64Const = 0x44,
    /// `i32.eqz`
    I32Eqz = 0x45,
    /// `i32.eq`
    I32Eq = 0x46,
    /// `i32.ne`
    I32Ne = 0x47,
    /// `i32.lt_s`
    I32LtS = 0x48,
    /// `i32.lt_u`
    I32LtU = 0x49,
    /// `i32.gt_s`
    I32GtS = 0x4A,
    /// `i32.gt_u`
    I32GtU = 0x4B,
    /// `i32.le_s`
    I32LeS = 0x4C,
    /// `i32.le_u`
    I32LeU = 0x4D,
    /// `i32.ge_s`
    I32GeS = 0x4E,
    /// `i32.ge_u`
    I32GeU = 0x4F,
    /// `i32.add`
    I32Add = 0x6A,
    /// `i32.sub`
    I32Sub = 0x6B,
    /// `i32.mul`
    I32Mul = 0x6C,
    /// `i32.div_s`
    I32DivS = 0x6D,
    /// `i32.div_u`
    I32DivU = 0x6E,
    /// `i32.rem_s`
    I32RemS = 0x6F,
    /// `i32.rem_u`
    I32RemU = 0x70,
    /// `i32.and`
    I32And = 0x71,
    /// `i32.or`
    I32Or = 0x72,
    /// `i32.xor`
    I32Xor = 0x73,
    /// `i32.shl`
    I32Shl = 0x74,
    /// `i32.shr_s`
    I32ShrS = 0x75,
    /// `i32.shr_u`
    I32ShrU = 0x76,
    /// `f64.add`
    F64Add = 0xA0,
    /// `f64.sub`
    F64Sub = 0xA1,
    /// `f64.mul`
    F64Mul = 0xA2,
    /// `f64.div`
    F64Div = 0xA3,
    /// `f64.min`
    F64Min = 0xA4,
    /// `i32.wrap_i64`
    I32WrapI64 = 0xA7,
    /// `i32.trunc_f64_s`
    I32TruncF64S = 0xAA,
    /// `f64.convert_i32_s`
    F64ConvertI32S = 0xB7,
    /// `ref.null`
    RefNull = 0xD0,
    /// `ref.is_null`
    RefIsNull = 0xD1,
    /// `ref.func`
    RefFunc = 0xD2,
    /// `GC` 前缀字节（后跟 [`WasmGcOpcode`] 子操作码）
    PrefixGc = 0xFB,
    /// 杂项前缀字节（后跟 [`WasmMiscOpcode`] 子操作码）
    PrefixMisc = 0xFC,
}

impl WasmOpcode {
    /// 操作码字节。
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// 将操作码字节追加到输出缓冲。
    #[inline]
    pub fn encode(self, output: &mut Vec<u8>) {
        output.push(self.as_u8());
    }

    /// 助记符字符串。
    pub fn mnemonic(self) -> &'static str {
        opcode_mnemonic(self.as_u8())
    }

    /// 从原始字节解析；未知字节返回 `None`。
    pub fn from_u8(byte: u8) -> Option<Self> {
        Some(match byte {
            0x00 => Self::Unreachable,
            0x01 => Self::Nop,
            0x02 => Self::Block,
            0x03 => Self::Loop,
            0x04 => Self::If,
            0x05 => Self::Else,
            0x0B => Self::End,
            0x0C => Self::Br,
            0x0D => Self::BrIf,
            0x0E => Self::BrTable,
            0x0F => Self::Return,
            0x10 => Self::Call,
            0x11 => Self::CallIndirect,
            0x1A => Self::Drop,
            0x1B => Self::Select,
            0x1C => Self::SelectTyped,
            0x20 => Self::LocalGet,
            0x21 => Self::LocalSet,
            0x22 => Self::LocalTee,
            0x23 => Self::GlobalGet,
            0x24 => Self::GlobalSet,
            0x28 => Self::I32Load,
            0x29 => Self::I64Load,
            0x2A => Self::F32Load,
            0x2B => Self::F64Load,
            0x2C => Self::I32Load8S,
            0x2D => Self::I32Load8U,
            0x2E => Self::I32Load16S,
            0x2F => Self::I32Load16U,
            0x36 => Self::I32Store,
            0x37 => Self::I64Store,
            0x38 => Self::F32Store,
            0x39 => Self::F64Store,
            0x3A => Self::I32Store8,
            0x3B => Self::I32Store16,
            0x3F => Self::MemorySize,
            0x40 => Self::MemoryGrow,
            0x41 => Self::I32Const,
            0x42 => Self::I64Const,
            0x43 => Self::F32Const,
            0x44 => Self::F64Const,
            0x45 => Self::I32Eqz,
            0x46 => Self::I32Eq,
            0x47 => Self::I32Ne,
            0x48 => Self::I32LtS,
            0x49 => Self::I32LtU,
            0x4A => Self::I32GtS,
            0x4B => Self::I32GtU,
            0x4C => Self::I32LeS,
            0x4D => Self::I32LeU,
            0x4E => Self::I32GeS,
            0x4F => Self::I32GeU,
            0x6A => Self::I32Add,
            0x6B => Self::I32Sub,
            0x6C => Self::I32Mul,
            0x6D => Self::I32DivS,
            0x6E => Self::I32DivU,
            0x6F => Self::I32RemS,
            0x70 => Self::I32RemU,
            0x71 => Self::I32And,
            0x72 => Self::I32Or,
            0x73 => Self::I32Xor,
            0x74 => Self::I32Shl,
            0x75 => Self::I32ShrS,
            0x76 => Self::I32ShrU,
            0xA0 => Self::F64Add,
            0xA1 => Self::F64Sub,
            0xA2 => Self::F64Mul,
            0xA3 => Self::F64Div,
            0xA4 => Self::F64Min,
            0xA7 => Self::I32WrapI64,
            0xAA => Self::I32TruncF64S,
            0xB7 => Self::F64ConvertI32S,
            0xD0 => Self::RefNull,
            0xD1 => Self::RefIsNull,
            0xD2 => Self::RefFunc,
            0xFB => Self::PrefixGc,
            0xFC => Self::PrefixMisc,
            _ => return None,
        })
    }
}

/// `wasm-gc` 前缀（`0xFB`）子操作码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WasmGcOpcode {
    /// `struct.new`
    StructNew = 0x00,
    /// `struct.new_default`
    StructNewDefault = 0x01,
    /// `struct.get`
    StructGet = 0x02,
    /// `struct.get_s`
    StructGetS = 0x03,
    /// `struct.get_u`
    StructGetU = 0x04,
    /// `struct.set`
    StructSet = 0x05,
    /// `array.new`
    ArrayNew = 0x06,
    /// `array.new_default`
    ArrayNewDefault = 0x07,
    /// `array.new_fixed`
    ArrayNewFixed = 0x08,
    /// `array.new_data`
    ArrayNewData = 0x09,
    /// `array.new_elem`
    ArrayNewElem = 0x0A,
    /// `array.get`
    ArrayGet = 0x0B,
    /// `array.get_s`
    ArrayGetS = 0x0C,
    /// `array.get_u`
    ArrayGetU = 0x0D,
    /// `array.set`
    ArraySet = 0x0E,
    /// `array.len`
    ArrayLen = 0x0F,
    /// `array.fill`
    ArrayFill = 0x10,
    /// `array.copy`
    ArrayCopy = 0x11,
    /// `array.init_data`
    ArrayInitData = 0x12,
    /// `array.init_elem`
    ArrayInitElem = 0x13,
    /// `ref.test`（非空）
    RefTest = 0x14,
    /// `ref.test`（可空）
    RefTestNull = 0x15,
    /// `ref.cast`（非空）
    RefCast = 0x16,
    /// `ref.cast`（可空）
    RefCastNull = 0x17,
}

impl WasmGcOpcode {
    /// 子操作码字节。
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// 子操作码作为 `u32`（供 uleb 立即数路径使用）。
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u8 as u32
    }

    /// 写入 `0xFB` 前缀 + 单字节子操作码（emitter 常用的短编码形式）。
    pub fn encode(self, output: &mut Vec<u8>) {
        WasmOpcode::PrefixGc.encode(output);
        output.push(self.as_u8());
    }

    /// 助记符。
    pub fn mnemonic(self) -> Option<&'static str> {
        gc_mnemonic(self.as_u32())
    }

    /// 从子操作码解析。
    pub fn from_u8(byte: u8) -> Option<Self> {
        Some(match byte {
            0x00 => Self::StructNew,
            0x01 => Self::StructNewDefault,
            0x02 => Self::StructGet,
            0x03 => Self::StructGetS,
            0x04 => Self::StructGetU,
            0x05 => Self::StructSet,
            0x06 => Self::ArrayNew,
            0x07 => Self::ArrayNewDefault,
            0x08 => Self::ArrayNewFixed,
            0x09 => Self::ArrayNewData,
            0x0A => Self::ArrayNewElem,
            0x0B => Self::ArrayGet,
            0x0C => Self::ArrayGetS,
            0x0D => Self::ArrayGetU,
            0x0E => Self::ArraySet,
            0x0F => Self::ArrayLen,
            0x10 => Self::ArrayFill,
            0x11 => Self::ArrayCopy,
            0x12 => Self::ArrayInitData,
            0x13 => Self::ArrayInitElem,
            0x14 => Self::RefTest,
            0x15 => Self::RefTestNull,
            0x16 => Self::RefCast,
            0x17 => Self::RefCastNull,
            _ => return None,
        })
    }
}

/// 杂项前缀（`0xFC`）子操作码。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum WasmMiscOpcode {
    /// `i32.trunc_sat_f32_s`（提案编号；本仓库历史表亦标 `i32.trunc_f32_s`）
    I32TruncF32S = 0,
    /// `i32.trunc_sat_f32_u`
    I32TruncF32U = 1,
    /// `i32.trunc_sat_f64_s`
    I32TruncF64S = 2,
    /// `i32.trunc_sat_f64_u`
    I32TruncF64U = 3,
    /// `i64.trunc_sat_f32_s`
    I64TruncF32S = 4,
    /// `i64.trunc_sat_f32_u`
    I64TruncF32U = 5,
    /// `i64.trunc_sat_f64_s`
    I64TruncF64S = 6,
    /// `i64.trunc_sat_f64_u`
    I64TruncF64U = 7,
    /// `memory.init`
    MemoryInit = 8,
    /// `data.drop`
    DataDrop = 9,
    /// `memory.copy`（bulk-memory：`0xFC 0x0A`）
    MemoryCopy = 10,
    /// `memory.fill`
    MemoryFill = 11,
}

impl WasmMiscOpcode {
    /// 子操作码字节。
    #[inline]
    pub const fn as_u8(self) -> u8 {
        self as u8
    }

    /// 子操作码作为 `u32`。
    #[inline]
    pub const fn as_u32(self) -> u32 {
        self as u8 as u32
    }

    /// 写入 `0xFC` 前缀 + 单字节子操作码。
    pub fn encode(self, output: &mut Vec<u8>) {
        WasmOpcode::PrefixMisc.encode(output);
        output.push(self.as_u8());
    }

    /// 助记符。
    pub fn mnemonic(self) -> Option<&'static str> {
        fc_mnemonic(self.as_u32())
    }

    /// 从子操作码解析。
    pub fn from_u8(byte: u8) -> Option<Self> {
        Some(match byte {
            0 => Self::I32TruncF32S,
            1 => Self::I32TruncF32U,
            2 => Self::I32TruncF64S,
            3 => Self::I32TruncF64U,
            4 => Self::I64TruncF32S,
            5 => Self::I64TruncF32U,
            6 => Self::I64TruncF64S,
            7 => Self::I64TruncF64U,
            8 => Self::MemoryInit,
            9 => Self::DataDrop,
            10 => Self::MemoryCopy,
            11 => Self::MemoryFill,
            _ => return None,
        })
    }
}

/// 返回操作码对应的助记符。
///
/// 覆盖 `spy` `decode_instruction` 中全部无操作数操作码，
/// 并顺带覆盖带立即数叶子操作码，以便解码层复用。
/// 未知操作码（含 `0xFB` / `0xFC` 前缀字节）返回 `"unknown"`。
pub fn opcode_mnemonic(opcode: u8) -> &'static str {
    match WasmOpcode::from_u8(opcode) {
        Some(WasmOpcode::Unreachable) => "unreachable",
        Some(WasmOpcode::Nop) => "nop",
        Some(WasmOpcode::Block) => "block",
        Some(WasmOpcode::Loop) => "loop",
        Some(WasmOpcode::If) => "if",
        Some(WasmOpcode::Else) => "else",
        Some(WasmOpcode::End) => "end",
        Some(WasmOpcode::Br) => "br",
        Some(WasmOpcode::BrIf) => "br_if",
        Some(WasmOpcode::BrTable) => "br_table",
        Some(WasmOpcode::Return) => "return",
        Some(WasmOpcode::Call) => "call",
        Some(WasmOpcode::CallIndirect) => "call_indirect",
        Some(WasmOpcode::Drop) => "drop",
        Some(WasmOpcode::Select) | Some(WasmOpcode::SelectTyped) => "select",
        Some(WasmOpcode::LocalGet) => "local.get",
        Some(WasmOpcode::LocalSet) => "local.set",
        Some(WasmOpcode::LocalTee) => "local.tee",
        Some(WasmOpcode::GlobalGet) => "global.get",
        Some(WasmOpcode::GlobalSet) => "global.set",
        Some(WasmOpcode::I32Load) => "i32.load",
        Some(WasmOpcode::I64Load) => "i64.load",
        Some(WasmOpcode::F32Load) => "f32.load",
        Some(WasmOpcode::F64Load) => "f64.load",
        Some(WasmOpcode::I32Load8S) => "i32.load8_s",
        Some(WasmOpcode::I32Load8U) => "i32.load8_u",
        Some(WasmOpcode::I32Load16S) => "i32.load16_s",
        Some(WasmOpcode::I32Load16U) => "i32.load16_u",
        Some(WasmOpcode::I32Store) => "i32.store",
        Some(WasmOpcode::I64Store) => "i64.store",
        Some(WasmOpcode::F32Store) => "f32.store",
        Some(WasmOpcode::F64Store) => "f64.store",
        Some(WasmOpcode::I32Store8) => "i32.store8",
        Some(WasmOpcode::I32Store16) => "i32.store16",
        Some(WasmOpcode::MemorySize) => "memory.size",
        Some(WasmOpcode::MemoryGrow) => "memory.grow",
        Some(WasmOpcode::I32Const) => "i32.const",
        Some(WasmOpcode::I64Const) => "i64.const",
        Some(WasmOpcode::F32Const) => "f32.const",
        Some(WasmOpcode::F64Const) => "f64.const",
        Some(WasmOpcode::I32Eqz) => "i32.eqz",
        Some(WasmOpcode::I32Eq) => "i32.eq",
        Some(WasmOpcode::I32Ne) => "i32.ne",
        Some(WasmOpcode::I32LtS) => "i32.lt_s",
        Some(WasmOpcode::I32LtU) => "i32.lt_u",
        Some(WasmOpcode::I32GtS) => "i32.gt_s",
        Some(WasmOpcode::I32GtU) => "i32.gt_u",
        Some(WasmOpcode::I32LeS) => "i32.le_s",
        Some(WasmOpcode::I32LeU) => "i32.le_u",
        Some(WasmOpcode::I32GeS) => "i32.ge_s",
        Some(WasmOpcode::I32GeU) => "i32.ge_u",
        Some(WasmOpcode::I32Add) => "i32.add",
        Some(WasmOpcode::I32Sub) => "i32.sub",
        Some(WasmOpcode::I32Mul) => "i32.mul",
        Some(WasmOpcode::I32DivS) => "i32.div_s",
        Some(WasmOpcode::I32DivU) => "i32.div_u",
        Some(WasmOpcode::I32RemS) => "i32.rem_s",
        Some(WasmOpcode::I32RemU) => "i32.rem_u",
        Some(WasmOpcode::I32And) => "i32.and",
        Some(WasmOpcode::I32Or) => "i32.or",
        Some(WasmOpcode::I32Xor) => "i32.xor",
        Some(WasmOpcode::I32Shl) => "i32.shl",
        Some(WasmOpcode::I32ShrS) => "i32.shr_s",
        Some(WasmOpcode::I32ShrU) => "i32.shr_u",
        Some(WasmOpcode::F64Add) => "f64.add",
        Some(WasmOpcode::F64Sub) => "f64.sub",
        Some(WasmOpcode::F64Mul) => "f64.mul",
        Some(WasmOpcode::F64Div) => "f64.div",
        Some(WasmOpcode::F64Min) => "f64.min",
        Some(WasmOpcode::I32WrapI64) => "i32.wrap_i64",
        Some(WasmOpcode::I32TruncF64S) => "i32.trunc_f64_s",
        Some(WasmOpcode::F64ConvertI32S) => "f64.convert_i32_s",
        Some(WasmOpcode::RefNull) => "ref.null",
        Some(WasmOpcode::RefIsNull) => "ref.is_null",
        Some(WasmOpcode::RefFunc) => "ref.func",
        Some(WasmOpcode::PrefixGc) | Some(WasmOpcode::PrefixMisc) | None => "unknown",
    }
}

/// 返回 `GC` 前缀（`0xFB`）子操作码对应的助记符。
///
/// 覆盖 `0x00`-`0x17`（`struct.new` ... `ref.cast_null`），
/// 未知子操作码返回 `None`。
pub fn gc_mnemonic(sub: u32) -> Option<&'static str> {
    match WasmGcOpcode::from_u8(u8::try_from(sub).ok()?)? {
        WasmGcOpcode::StructNew => Some("struct.new"),
        WasmGcOpcode::StructNewDefault => Some("struct.new_default"),
        WasmGcOpcode::StructGet => Some("struct.get"),
        WasmGcOpcode::StructGetS => Some("struct.get_s"),
        WasmGcOpcode::StructGetU => Some("struct.get_u"),
        WasmGcOpcode::StructSet => Some("struct.set"),
        WasmGcOpcode::ArrayNew => Some("array.new"),
        WasmGcOpcode::ArrayNewDefault => Some("array.new_default"),
        WasmGcOpcode::ArrayNewFixed => Some("array.new_fixed"),
        WasmGcOpcode::ArrayNewData => Some("array.new_data"),
        WasmGcOpcode::ArrayNewElem => Some("array.new_elem"),
        WasmGcOpcode::ArrayGet => Some("array.get"),
        WasmGcOpcode::ArrayGetS => Some("array.get_s"),
        WasmGcOpcode::ArrayGetU => Some("array.get_u"),
        WasmGcOpcode::ArraySet => Some("array.set"),
        WasmGcOpcode::ArrayLen => Some("array.len"),
        WasmGcOpcode::ArrayFill => Some("array.fill"),
        WasmGcOpcode::ArrayCopy => Some("array.copy"),
        WasmGcOpcode::ArrayInitData => Some("array.init_data"),
        WasmGcOpcode::ArrayInitElem => Some("array.init_elem"),
        WasmGcOpcode::RefTest => Some("ref.test"),
        WasmGcOpcode::RefTestNull => Some("ref.test_null"),
        WasmGcOpcode::RefCast => Some("ref.cast"),
        WasmGcOpcode::RefCastNull => Some("ref.cast_null"),
    }
}

/// 返回杂项前缀（`0xFC`）子操作码对应的助记符。
///
/// 覆盖 `0`-`11`（`i32.trunc_f32_s` ... `data.drop`），
/// 未知子操作码返回 `None`。
pub fn fc_mnemonic(sub: u32) -> Option<&'static str> {
    match WasmMiscOpcode::from_u8(u8::try_from(sub).ok()?)? {
        WasmMiscOpcode::I32TruncF32S => Some("i32.trunc_f32_s"),
        WasmMiscOpcode::I32TruncF32U => Some("i32.trunc_f32_u"),
        WasmMiscOpcode::I32TruncF64S => Some("i32.trunc_f64_s"),
        WasmMiscOpcode::I32TruncF64U => Some("i32.trunc_f64_u"),
        WasmMiscOpcode::I64TruncF32S => Some("i64.trunc_f32_s"),
        WasmMiscOpcode::I64TruncF32U => Some("i64.trunc_f32_u"),
        WasmMiscOpcode::I64TruncF64S => Some("i64.trunc_f64_s"),
        WasmMiscOpcode::I64TruncF64U => Some("i64.trunc_f64_u"),
        WasmMiscOpcode::MemoryInit => Some("memory.init"),
        WasmMiscOpcode::DataDrop => Some("data.drop"),
        WasmMiscOpcode::MemoryCopy => Some("memory.copy"),
        WasmMiscOpcode::MemoryFill => Some("memory.fill"),
    }
}

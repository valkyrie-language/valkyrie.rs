//! ECMA-335 标准 CIL 操作码表与操作数解码。
//!
//! 本模块提供 ECMA-335 规范定义的 CIL 操作码到名称与操作数信息的映射，
//! 以及将方法体中的原始操作数字节解码为结构化 [`IlOperand`] 的工具。
//!
//! 所有操作码值均来自 ECMA-335 Partition III 标准定义。与 [`super::MethodBodyEncoder`]
//! 对称：编码侧将 `MSIL` 指令序列写入方法体，解码侧将方法体字节还原为结构化指令。

use std::{collections::HashMap, sync::LazyLock};

/// 操作码信息。
///
/// 描述单个 CIL 操作码的名称、是否携带操作数以及操作数的固定字节数。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpCodeInfo {
    /// 操作码名称（如 `"ldc.i4"`）。
    pub opcode: &'static str,
    /// 是否有操作数。
    pub has_operand: bool,
    /// 操作数大小（字节数）。`switch` (0x45) 为变长，此处记为 `0`，由解码器特殊处理。
    pub operand_size: usize,
}

/// ECMA-335 标准 CIL 操作码表。
///
/// 键为操作码字节值，单字节操作码为原始值（`0x00`-`0xA5`、`0xD0`），
/// 双字节操作码为 `0xFE00 + 第二字节`（如 `0xFE01` 即 `ceq`）。
/// 未列出的操作码视为未知，调用方应回退到原始字节输出。
pub static OP_CODES: LazyLock<HashMap<usize, OpCodeInfo>> = LazyLock::new(|| {
    HashMap::from([
        // 基础指令
        (0x00, OpCodeInfo { opcode: "nop", has_operand: false, operand_size: 0 }),
        (0x01, OpCodeInfo { opcode: "break", has_operand: false, operand_size: 0 }),
        // 参数加载（短格式）
        (0x02, OpCodeInfo { opcode: "ldarg.0", has_operand: false, operand_size: 0 }),
        (0x03, OpCodeInfo { opcode: "ldarg.1", has_operand: false, operand_size: 0 }),
        (0x04, OpCodeInfo { opcode: "ldarg.2", has_operand: false, operand_size: 0 }),
        (0x05, OpCodeInfo { opcode: "ldarg.3", has_operand: false, operand_size: 0 }),
        // 局部变量加载（短格式）
        (0x06, OpCodeInfo { opcode: "ldloc.0", has_operand: false, operand_size: 0 }),
        (0x07, OpCodeInfo { opcode: "ldloc.1", has_operand: false, operand_size: 0 }),
        (0x08, OpCodeInfo { opcode: "ldloc.2", has_operand: false, operand_size: 0 }),
        (0x09, OpCodeInfo { opcode: "ldloc.3", has_operand: false, operand_size: 0 }),
        // 局部变量存储（短格式）
        (0x0A, OpCodeInfo { opcode: "stloc.0", has_operand: false, operand_size: 0 }),
        (0x0B, OpCodeInfo { opcode: "stloc.1", has_operand: false, operand_size: 0 }),
        (0x0C, OpCodeInfo { opcode: "stloc.2", has_operand: false, operand_size: 0 }),
        (0x0D, OpCodeInfo { opcode: "stloc.3", has_operand: false, operand_size: 0 }),
        // 参数/局部变量（短格式，带操作数）
        (0x0E, OpCodeInfo { opcode: "ldarg.s", has_operand: true, operand_size: 1 }),
        (0x0F, OpCodeInfo { opcode: "ldarga.s", has_operand: true, operand_size: 1 }),
        (0x10, OpCodeInfo { opcode: "starg.s", has_operand: true, operand_size: 1 }),
        (0x11, OpCodeInfo { opcode: "ldloc.s", has_operand: true, operand_size: 1 }),
        (0x12, OpCodeInfo { opcode: "ldloca.s", has_operand: true, operand_size: 1 }),
        (0x13, OpCodeInfo { opcode: "stloc.s", has_operand: true, operand_size: 1 }),
        // 常量加载（短格式）
        (0x14, OpCodeInfo { opcode: "ldnull", has_operand: false, operand_size: 0 }),
        (0x15, OpCodeInfo { opcode: "ldc.i4.m1", has_operand: false, operand_size: 0 }),
        (0x16, OpCodeInfo { opcode: "ldc.i4.0", has_operand: false, operand_size: 0 }),
        (0x17, OpCodeInfo { opcode: "ldc.i4.1", has_operand: false, operand_size: 0 }),
        (0x18, OpCodeInfo { opcode: "ldc.i4.2", has_operand: false, operand_size: 0 }),
        (0x19, OpCodeInfo { opcode: "ldc.i4.3", has_operand: false, operand_size: 0 }),
        (0x1A, OpCodeInfo { opcode: "ldc.i4.4", has_operand: false, operand_size: 0 }),
        (0x1B, OpCodeInfo { opcode: "ldc.i4.5", has_operand: false, operand_size: 0 }),
        (0x1C, OpCodeInfo { opcode: "ldc.i4.6", has_operand: false, operand_size: 0 }),
        (0x1D, OpCodeInfo { opcode: "ldc.i4.7", has_operand: false, operand_size: 0 }),
        (0x1E, OpCodeInfo { opcode: "ldc.i4.8", has_operand: false, operand_size: 0 }),
        (0x1F, OpCodeInfo { opcode: "ldc.i4.s", has_operand: true, operand_size: 1 }),
        // 常量加载（长格式）
        (0x20, OpCodeInfo { opcode: "ldc.i4", has_operand: true, operand_size: 4 }),
        (0x21, OpCodeInfo { opcode: "ldc.i8", has_operand: true, operand_size: 8 }),
        (0x22, OpCodeInfo { opcode: "ldc.r4", has_operand: true, operand_size: 4 }),
        (0x23, OpCodeInfo { opcode: "ldc.r8", has_operand: true, operand_size: 8 }),
        // 栈操作
        (0x25, OpCodeInfo { opcode: "dup", has_operand: false, operand_size: 0 }),
        (0x26, OpCodeInfo { opcode: "pop", has_operand: false, operand_size: 0 }),
        // 控制流：调用
        (0x27, OpCodeInfo { opcode: "jmp", has_operand: true, operand_size: 4 }),
        (0x28, OpCodeInfo { opcode: "call", has_operand: true, operand_size: 4 }),
        (0x29, OpCodeInfo { opcode: "calli", has_operand: true, operand_size: 4 }),
        (0x2A, OpCodeInfo { opcode: "ret", has_operand: false, operand_size: 0 }),
        // 控制流：短跳转
        (0x2B, OpCodeInfo { opcode: "br.s", has_operand: true, operand_size: 1 }),
        (0x2C, OpCodeInfo { opcode: "brfalse.s", has_operand: true, operand_size: 1 }),
        (0x2D, OpCodeInfo { opcode: "brtrue.s", has_operand: true, operand_size: 1 }),
        (0x2E, OpCodeInfo { opcode: "beq.s", has_operand: true, operand_size: 1 }),
        (0x2F, OpCodeInfo { opcode: "bge.s", has_operand: true, operand_size: 1 }),
        (0x30, OpCodeInfo { opcode: "bgt.s", has_operand: true, operand_size: 1 }),
        (0x31, OpCodeInfo { opcode: "ble.s", has_operand: true, operand_size: 1 }),
        (0x32, OpCodeInfo { opcode: "blt.s", has_operand: true, operand_size: 1 }),
        (0x33, OpCodeInfo { opcode: "bne.un.s", has_operand: true, operand_size: 1 }),
        (0x34, OpCodeInfo { opcode: "bge.un.s", has_operand: true, operand_size: 1 }),
        (0x35, OpCodeInfo { opcode: "bgt.un.s", has_operand: true, operand_size: 1 }),
        (0x36, OpCodeInfo { opcode: "ble.un.s", has_operand: true, operand_size: 1 }),
        (0x37, OpCodeInfo { opcode: "blt.un.s", has_operand: true, operand_size: 1 }),
        // 控制流：长跳转
        (0x38, OpCodeInfo { opcode: "br", has_operand: true, operand_size: 4 }),
        (0x39, OpCodeInfo { opcode: "brfalse", has_operand: true, operand_size: 4 }),
        (0x3A, OpCodeInfo { opcode: "brtrue", has_operand: true, operand_size: 4 }),
        (0x3B, OpCodeInfo { opcode: "beq", has_operand: true, operand_size: 4 }),
        (0x3C, OpCodeInfo { opcode: "bge", has_operand: true, operand_size: 4 }),
        (0x3D, OpCodeInfo { opcode: "bgt", has_operand: true, operand_size: 4 }),
        (0x3E, OpCodeInfo { opcode: "ble", has_operand: true, operand_size: 4 }),
        (0x3F, OpCodeInfo { opcode: "blt", has_operand: true, operand_size: 4 }),
        (0x40, OpCodeInfo { opcode: "bne.un", has_operand: true, operand_size: 4 }),
        (0x41, OpCodeInfo { opcode: "bge.un", has_operand: true, operand_size: 4 }),
        (0x42, OpCodeInfo { opcode: "bgt.un", has_operand: true, operand_size: 4 }),
        (0x43, OpCodeInfo { opcode: "ble.un", has_operand: true, operand_size: 4 }),
        (0x44, OpCodeInfo { opcode: "blt.un", has_operand: true, operand_size: 4 }),
        // switch：变长，operand_size 记为 0，由解码器读取跳转表。
        (0x45, OpCodeInfo { opcode: "switch", has_operand: true, operand_size: 0 }),
        // 间接加载
        (0x46, OpCodeInfo { opcode: "ldind.i1", has_operand: false, operand_size: 0 }),
        (0x47, OpCodeInfo { opcode: "ldind.u1", has_operand: false, operand_size: 0 }),
        (0x48, OpCodeInfo { opcode: "ldind.i2", has_operand: false, operand_size: 0 }),
        (0x49, OpCodeInfo { opcode: "ldind.u2", has_operand: false, operand_size: 0 }),
        (0x4A, OpCodeInfo { opcode: "ldind.i4", has_operand: false, operand_size: 0 }),
        (0x4B, OpCodeInfo { opcode: "ldind.u4", has_operand: false, operand_size: 0 }),
        (0x4C, OpCodeInfo { opcode: "ldind.i8", has_operand: false, operand_size: 0 }),
        (0x4D, OpCodeInfo { opcode: "ldind.i", has_operand: false, operand_size: 0 }),
        (0x4E, OpCodeInfo { opcode: "ldind.r4", has_operand: false, operand_size: 0 }),
        (0x4F, OpCodeInfo { opcode: "ldind.r8", has_operand: false, operand_size: 0 }),
        (0x50, OpCodeInfo { opcode: "ldind.ref", has_operand: false, operand_size: 0 }),
        // 间接存储
        (0x51, OpCodeInfo { opcode: "stind.ref", has_operand: false, operand_size: 0 }),
        (0x52, OpCodeInfo { opcode: "stind.i1", has_operand: false, operand_size: 0 }),
        (0x53, OpCodeInfo { opcode: "stind.i2", has_operand: false, operand_size: 0 }),
        (0x54, OpCodeInfo { opcode: "stind.i4", has_operand: false, operand_size: 0 }),
        (0x55, OpCodeInfo { opcode: "stind.i8", has_operand: false, operand_size: 0 }),
        (0x56, OpCodeInfo { opcode: "stind.r4", has_operand: false, operand_size: 0 }),
        (0x57, OpCodeInfo { opcode: "stind.r8", has_operand: false, operand_size: 0 }),
        // 算术运算
        (0x58, OpCodeInfo { opcode: "add", has_operand: false, operand_size: 0 }),
        (0x59, OpCodeInfo { opcode: "sub", has_operand: false, operand_size: 0 }),
        (0x5A, OpCodeInfo { opcode: "mul", has_operand: false, operand_size: 0 }),
        (0x5B, OpCodeInfo { opcode: "div", has_operand: false, operand_size: 0 }),
        (0x5C, OpCodeInfo { opcode: "div.un", has_operand: false, operand_size: 0 }),
        (0x5D, OpCodeInfo { opcode: "rem", has_operand: false, operand_size: 0 }),
        (0x5E, OpCodeInfo { opcode: "rem.un", has_operand: false, operand_size: 0 }),
        // 逻辑运算
        (0x5F, OpCodeInfo { opcode: "and", has_operand: false, operand_size: 0 }),
        (0x60, OpCodeInfo { opcode: "or", has_operand: false, operand_size: 0 }),
        (0x61, OpCodeInfo { opcode: "xor", has_operand: false, operand_size: 0 }),
        // 移位运算
        (0x62, OpCodeInfo { opcode: "shl", has_operand: false, operand_size: 0 }),
        (0x63, OpCodeInfo { opcode: "shr", has_operand: false, operand_size: 0 }),
        (0x64, OpCodeInfo { opcode: "shr.un", has_operand: false, operand_size: 0 }),
        // 一元运算
        (0x65, OpCodeInfo { opcode: "neg", has_operand: false, operand_size: 0 }),
        (0x66, OpCodeInfo { opcode: "not", has_operand: false, operand_size: 0 }),
        // 类型转换
        (0x67, OpCodeInfo { opcode: "conv.i1", has_operand: false, operand_size: 0 }),
        (0x68, OpCodeInfo { opcode: "conv.i2", has_operand: false, operand_size: 0 }),
        (0x69, OpCodeInfo { opcode: "conv.i4", has_operand: false, operand_size: 0 }),
        (0x6A, OpCodeInfo { opcode: "conv.i8", has_operand: false, operand_size: 0 }),
        (0x6B, OpCodeInfo { opcode: "conv.r4", has_operand: false, operand_size: 0 }),
        (0x6C, OpCodeInfo { opcode: "conv.r8", has_operand: false, operand_size: 0 }),
        (0x6D, OpCodeInfo { opcode: "conv.u4", has_operand: false, operand_size: 0 }),
        (0x6E, OpCodeInfo { opcode: "conv.u8", has_operand: false, operand_size: 0 }),
        // 对象模型：调用
        (0x6F, OpCodeInfo { opcode: "callvirt", has_operand: true, operand_size: 4 }),
        (0x70, OpCodeInfo { opcode: "cpobj", has_operand: true, operand_size: 4 }),
        (0x71, OpCodeInfo { opcode: "ldobj", has_operand: true, operand_size: 4 }),
        (0x72, OpCodeInfo { opcode: "ldstr", has_operand: true, operand_size: 4 }),
        (0x73, OpCodeInfo { opcode: "newobj", has_operand: true, operand_size: 4 }),
        (0x74, OpCodeInfo { opcode: "castclass", has_operand: true, operand_size: 4 }),
        (0x75, OpCodeInfo { opcode: "isinst", has_operand: true, operand_size: 4 }),
        // 对象模型：装箱
        (0x77, OpCodeInfo { opcode: "unbox", has_operand: true, operand_size: 4 }),
        (0x78, OpCodeInfo { opcode: "throw", has_operand: false, operand_size: 0 }),
        // 对象模型：字段
        (0x79, OpCodeInfo { opcode: "ldfld", has_operand: true, operand_size: 4 }),
        (0x7A, OpCodeInfo { opcode: "ldflda", has_operand: true, operand_size: 4 }),
        (0x7B, OpCodeInfo { opcode: "stfld", has_operand: true, operand_size: 4 }),
        (0x7C, OpCodeInfo { opcode: "ldsfld", has_operand: true, operand_size: 4 }),
        (0x7D, OpCodeInfo { opcode: "ldsflda", has_operand: true, operand_size: 4 }),
        (0x7E, OpCodeInfo { opcode: "stsfld", has_operand: true, operand_size: 4 }),
        (0x7F, OpCodeInfo { opcode: "stobj", has_operand: true, operand_size: 4 }),
        // 装箱与数组
        (0x8C, OpCodeInfo { opcode: "box", has_operand: true, operand_size: 4 }),
        (0x8D, OpCodeInfo { opcode: "newarr", has_operand: true, operand_size: 4 }),
        (0xA3, OpCodeInfo { opcode: "ldelem.any", has_operand: true, operand_size: 4 }),
        (0xA4, OpCodeInfo { opcode: "stelem.any", has_operand: true, operand_size: 4 }),
        (0xA5, OpCodeInfo { opcode: "unbox.any", has_operand: true, operand_size: 4 }),
        // 元数据 token
        (0xD0, OpCodeInfo { opcode: "ldtoken", has_operand: true, operand_size: 4 }),
        // 0xFE 前缀指令（双字节操作码，键为 0xFE00 + 第二字节）
        // 比较指令 (ECMA-335 III.1.5)
        (0xFE01, OpCodeInfo { opcode: "ceq", has_operand: false, operand_size: 0 }),
        (0xFE02, OpCodeInfo { opcode: "cgt", has_operand: false, operand_size: 0 }),
        (0xFE03, OpCodeInfo { opcode: "cgt.un", has_operand: false, operand_size: 0 }),
        (0xFE04, OpCodeInfo { opcode: "clt", has_operand: false, operand_size: 0 }),
        (0xFE05, OpCodeInfo { opcode: "clt.un", has_operand: false, operand_size: 0 }),
        // 局部变量加载/存储（长格式）
        (0xFE06, OpCodeInfo { opcode: "ldftn", has_operand: true, operand_size: 4 }),
        (0xFE07, OpCodeInfo { opcode: "ldvirtftn", has_operand: true, operand_size: 4 }),
        (0xFE09, OpCodeInfo { opcode: "ldarg", has_operand: true, operand_size: 2 }),
        (0xFE0A, OpCodeInfo { opcode: "ldarga", has_operand: true, operand_size: 2 }),
        (0xFE0B, OpCodeInfo { opcode: "starg", has_operand: true, operand_size: 2 }),
        (0xFE0C, OpCodeInfo { opcode: "ldloc", has_operand: true, operand_size: 2 }),
        (0xFE0D, OpCodeInfo { opcode: "ldloca", has_operand: true, operand_size: 2 }),
        (0xFE0E, OpCodeInfo { opcode: "stloc", has_operand: true, operand_size: 2 }),
        // 其他
        (0xFE0F, OpCodeInfo { opcode: "localloc", has_operand: false, operand_size: 0 }),
        (0xFE11, OpCodeInfo { opcode: "endfilter", has_operand: false, operand_size: 0 }),
        (0xFE12, OpCodeInfo { opcode: "unaligned.", has_operand: true, operand_size: 1 }),
        (0xFE13, OpCodeInfo { opcode: "volatile.", has_operand: false, operand_size: 0 }),
        (0xFE14, OpCodeInfo { opcode: "tail.", has_operand: false, operand_size: 0 }),
        (0xFE15, OpCodeInfo { opcode: "initobj", has_operand: true, operand_size: 4 }),
        (0xFE16, OpCodeInfo { opcode: "constrained.", has_operand: true, operand_size: 4 }),
        (0xFE17, OpCodeInfo { opcode: "cpblk", has_operand: false, operand_size: 0 }),
        (0xFE18, OpCodeInfo { opcode: "initblk", has_operand: false, operand_size: 0 }),
        (0xFE1A, OpCodeInfo { opcode: "rethrow", has_operand: false, operand_size: 0 }),
        (0xFE1C, OpCodeInfo { opcode: "sizeof", has_operand: true, operand_size: 4 }),
        (0xFE1D, OpCodeInfo { opcode: "refanytype", has_operand: false, operand_size: 0 }),
        (0xFE1E, OpCodeInfo { opcode: "readonly.", has_operand: false, operand_size: 0 }),
    ])
});

/// 结构化 IL 操作数。
///
/// 由 [`read_operand`] 产生，取代旧实现中的字符串渲染。
/// 分支目标以绝对 IL 偏移表示（跳转指令下一条指令的偏移加上有符号增量）。
#[derive(Debug, Clone, PartialEq)]
pub enum IlOperand {
    /// 无操作数（指令本身不携带操作数，或操作数读取越界）。
    None,
    /// 短格式分支目标（1 字节有符号增量解析后的绝对 IL 偏移）。
    ShortBrTarget(i32),
    /// 长格式分支目标（4 字节有符号增量解析后的绝对 IL 偏移）。
    BrTarget(i32),
    /// 字段 token（`ldfld` / `stfld` / `ldsfld` 等）。
    FieldToken(u32),
    /// 方法 token（`call` / `callvirt` / `newobj` / `ldftn` 等）。
    MethodToken(u32),
    /// 类型 token（`box` / `unbox` / `castclass` / `initobj` 等）。
    TypeToken(u32),
    /// 字符串 token（`ldstr`）。
    StringToken(u32),
    /// 通用元数据 token（`ldtoken`）。
    Token(u32),
    /// 1 字节无符号索引（`ldarg.s` / `ldloc.s` 等）。
    Byte(u8),
    /// 2 字节索引（`ldarg` / `ldloc` 长格式等）。
    Short(i16),
    /// 1 字节有符号常量（`ldc.i4.s`）。
    Int8(i8),
    /// 4 字节有符号整型常量（`ldc.i4`）。
    Int32(i32),
    /// 8 字节有符号整型常量（`ldc.i8`）。
    Int64(i64),
    /// 4 字节浮点常量（`ldc.r4`）。
    Float32(f32),
    /// 8 字节浮点常量（`ldc.r8`）。
    Float64(f64),
    /// `switch` 跳转表，元素为各分支目标的绝对 IL 偏移。
    Switch(Vec<i32>),
}

/// 操作数语义类型。
///
/// 由 [`get_operand_type`] 返回，决定 [`read_operand`] 如何解释原始字节。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperandType {
    /// 无特定语义（按字节数作通用整数/索引解释）。
    None,
    /// 短格式分支目标。
    ShortBrTarget,
    /// 长格式分支目标。
    BrTarget,
    /// 字段 token。
    FieldToken,
    /// 方法 token。
    MethodToken,
    /// 类型 token。
    TypeToken,
    /// 字符串 token。
    StringToken,
    /// 通用元数据 token。
    Token,
    /// 1 字节有符号整型常量。
    Integer1,
    /// 4 字节有符号整型常量。
    Integer4,
    /// 8 字节有符号整型常量。
    Integer8,
    /// 4 字节浮点常量。
    Float4,
    /// 8 字节浮点常量。
    Float8,
}

/// 根据操作码键值返回操作数语义类型。
///
/// `op_key` 为单字节操作码原始值，或双字节操作码 `0xFE00 + 第二字节`。
/// 相较于旧实现仅按第二字节判定，本实现能正确识别 `0xFE06`(`ldftn`)、
/// `0xFE15`(`initobj`)、`0xFE16`(`constrained.`)、`0xFE1C`(`sizeof`) 等
/// 双字节 token 操作码，并将 `ldstr` (0x72) 归为 [`OperandType::StringToken`]。
pub fn get_operand_type(op_key: usize) -> OperandType {
    match op_key {
        // 短跳转（1 字节操作数）
        0x2B..=0x37 => OperandType::ShortBrTarget,
        // 长跳转（4 字节操作数）
        0x38..=0x44 => OperandType::BrTarget,
        // 方法 token（call, calli, callvirt, newobj, jmp, ldftn, ldvirtftn）
        0x27 | 0x28 | 0x29 | 0x6F | 0x73 | 0xFE06 | 0xFE07 => OperandType::MethodToken,
        // 字段 token（ldfld, ldflda, stfld, ldsfld, ldsflda, stsfld）
        0x79 | 0x7A | 0x7B | 0x7C | 0x7D | 0x7E => OperandType::FieldToken,
        // 类型 token（cpobj, ldobj, castclass, isinst, unbox, box, newarr, ldelem.any, stelem.any, unbox.any, stobj, initobj, constrained., sizeof）
        0x70 | 0x71 | 0x74 | 0x75 | 0x77 | 0x7F | 0x8C | 0x8D | 0xA3 | 0xA4 | 0xA5 | 0xFE15 | 0xFE16 | 0xFE1C => OperandType::TypeToken,
        // 字符串 token（ldstr）
        0x72 => OperandType::StringToken,
        // 通用元数据 token（ldtoken）
        0xD0 => OperandType::Token,
        // 常量加载
        0x1F => OperandType::Integer1,
        0x20 => OperandType::Integer4,
        0x21 => OperandType::Integer8,
        0x22 => OperandType::Float4,
        0x23 => OperandType::Float8,
        _ => OperandType::None,
    }
}

/// 查询操作码信息。
///
/// 已知操作码返回对应的 [`OpCodeInfo`] 引用，未知返回 `None`。
pub fn lookup_opcode(op_key: usize) -> Option<&'static OpCodeInfo> {
    OP_CODES.get(&op_key)
}

/// 解码操作码并返回 `(名称, 是否有操作数, 操作数字节数)`。
///
/// 未知操作码的名称格式化为 `0x{HEX}`，且视为无操作数。
pub fn decode_op_code(op_key: usize) -> (String, bool, usize) {
    match lookup_opcode(op_key) {
        Some(info) => (info.opcode.to_string(), info.has_operand, info.operand_size),
        None => {
            let name = if op_key > 0xFF { format!("0x{:04X}", op_key) } else { format!("0x{:02X}", op_key) };
            (name, false, 0)
        }
    }
}

/// 从方法体字节读取结构化操作数。
///
/// 统一处理 Tiny 与 Fat 两种方法体格式：`il_offset` 为操作数起始处的 IL 偏移，
/// 用于计算分支目标的绝对偏移（`下一条指令偏移 + 有符号增量`）。
/// `switch` (0x45) 为变长指令，由调用方在 [`super::MethodBodyEncoder`] 的对端
/// 解码流程中单独处理后，本函数不负责读取其跳转表。
pub fn read_operand(op_key: usize, data: &[u8], offset: usize, size: usize, il_offset: usize) -> IlOperand {
    if size == 0 || offset + size > data.len() {
        return IlOperand::None;
    }
    match get_operand_type(op_key) {
        OperandType::ShortBrTarget => {
            let delta = data[offset] as i8 as i32;
            IlOperand::ShortBrTarget(il_offset as i32 + size as i32 + delta)
        }
        OperandType::BrTarget => {
            let delta = i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]]);
            IlOperand::BrTarget(il_offset as i32 + size as i32 + delta)
        }
        OperandType::FieldToken => IlOperand::FieldToken(read_u32_le(data, offset)),
        OperandType::MethodToken => IlOperand::MethodToken(read_u32_le(data, offset)),
        OperandType::TypeToken => IlOperand::TypeToken(read_u32_le(data, offset)),
        OperandType::StringToken => IlOperand::StringToken(read_u32_le(data, offset)),
        OperandType::Token => IlOperand::Token(read_u32_le(data, offset)),
        OperandType::Integer1 => IlOperand::Int8(data[offset] as i8),
        OperandType::Integer4 => IlOperand::Int32(i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])),
        OperandType::Integer8 => IlOperand::Int64(i64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ])),
        OperandType::Float4 => IlOperand::Float32(f32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])),
        OperandType::Float8 => IlOperand::Float64(f64::from_le_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
            data[offset + 4],
            data[offset + 5],
            data[offset + 6],
            data[offset + 7],
        ])),
        OperandType::None => match size {
            1 => IlOperand::Byte(data[offset]),
            2 => IlOperand::Short(u16::from_le_bytes([data[offset], data[offset + 1]]) as i16),
            4 => IlOperand::Int32(i32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])),
            _ => IlOperand::None,
        },
    }
}

/// 读取小端 4 字节无符号整数。
fn read_u32_le(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([data[offset], data[offset + 1], data[offset + 2], data[offset + 3]])
}

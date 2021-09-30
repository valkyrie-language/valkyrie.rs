//! ECMA-335 标准 CIL 操作码表。
//!
//! 本模块提供 ECMA-335 规范定义的 CIL 操作码到名称与操作数信息的映射。
//! 所有操作码值均来自 ECMA-335 Partition III 标准定义。

use std::{collections::HashMap, sync::LazyLock};

/// 操作码信息。
#[derive(Debug, Clone, Copy)]
pub struct OpCodeInfo {
    /// 操作码名称。
    pub opcode: &'static str,
    /// 是否有操作数。
    pub has_operand: bool,
    /// 操作数大小（字节数）。
    pub operand_size: usize,
}

/// ECMA-335 标准 CIL 操作码表。
///
/// 键为操作码字节值，值为对应的操作码信息。
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

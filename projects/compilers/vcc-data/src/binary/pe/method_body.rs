//! `CLR` 方法体与 `IL` 指令解析。
//!
//! 解码 `CLR` 方法体的 Tiny / Fat 两种二进制格式（ECMA-335 II.25.4），
//! 并将方法体中的 `IL` 字节流还原为结构化 [`IlInstruction`] 列表。
//!
//! 操作数的结构化解码由 [`crate::text::msil`] 侧的 `op_codes` 模块提供，
//! 本模块仅负责方法体头部与指令边界的遍历。

use crate::text::msil::{IlOperand, lookup_opcode, read_operand};

/// `Fat` 方法体格式标记。
///
/// 表示 `Fat` 方法头中观测到的标志位掩码示例，仅用于诊断与文档化目的。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FatFormat {
    /// `Fat` 格式主标志位示例（`0x17`）。
    MajorFlag = 0x17,
}

/// 方法体标志（`Fat` 格式）。
#[derive(Debug, Clone, Copy)]
pub struct MethodBodyFlags {
    /// 格式标记：`0x00` = tiny，`0x02` = fat。
    pub format: u8,
    /// 是否启用异常处理块。
    pub has_exceptions: bool,
    /// 局部变量签名 `token`。
    pub local_var_sig_token: u32,
    /// 局部变量标志。
    pub flags: LocalVarFlags,
}

/// 局部变量标志。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalVarFlags {
    /// 安全修饰符。
    pub safe: bool,
    /// 初值置零。
    pub init_locals: bool,
    /// 签名保留。
    pub reserved: bool,
    /// 类型。
    pub type_: LocalVarType,
}

impl LocalVarFlags {
    /// 从 16 位原始值解析局部变量标志。
    pub fn from_u16(raw: u16) -> Self {
        Self {
            safe: raw & 0x01 != 0,
            init_locals: raw & 0x02 != 0,
            reserved: raw & 0x04 != 0,
            type_: match (raw >> 2) & 0x03 {
                0 => LocalVarType::Simple,
                1 => LocalVarType::SignedInt32,
                2 => LocalVarType::UnsignedInt32,
                _ => LocalVarType::Simple,
            },
        }
    }
}

/// 局部变量签名类型。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalVarType {
    /// 简单类型。
    Simple,
    /// 有符号 32 位整型。
    SignedInt32,
    /// 无符号 32 位整型。
    UnsignedInt32,
}

/// 方法体信息。
///
/// 由调用方在解析方法头后组装；[`parse_method_body_il`] 仅负责 `IL` 指令解码，
/// 返回 [`IlInstruction`] 列表，调用方可将其填入本结构的 `instructions` 字段。
#[derive(Debug, Clone)]
pub struct MethodBody {
    /// 格式标志。
    pub flags: MethodBodyFlags,
    /// 最大操作数栈深度。
    pub max_stack: u16,
    /// 局部变量 `token`（`Fat` 格式）。
    pub local_var_sig_token: u32,
    /// 代码长度。
    pub code_len: u32,
    /// 局部变量声明长度。
    pub local_var_decl_len: u32,
    /// 原始字节。
    pub raw_bytes: Vec<u8>,
    /// `IL` 指令列表。
    pub instructions: Vec<IlInstruction>,
}

/// 单条 `IL` 指令。
#[derive(Debug, Clone)]
pub struct IlInstruction {
    /// 指令在 `IL` 代码中的偏移量。
    pub offset: usize,
    /// 操作码名称（已知操作码为标准名，未知为 `0x{HEX}`）。
    pub opcode: String,
    /// 结构化操作数。
    pub operand: IlOperand,
}

/// 从方法体原始字节解析 `IL` 指令。
///
/// - `is_tiny = true`：`data` 为已剥离头部的 `IL` 代码，直接解码至末尾。
/// - `is_tiny = false`：`data` 包含 12 字节 `Fat` 头部，函数内部解析头部获取
///   `MaxStack` / `CodeLen` / `LocalVarSigTok`，随后从偏移 12 开始解码 `CodeLen` 字节。
///
/// 分支目标以绝对 `IL` 偏移返回（跳转指令下一条指令的偏移加上有符号增量）。
/// `switch` (0x45) 的变长跳转表在此处单独处理。
pub fn parse_method_body_il(data: &[u8], is_tiny: bool) -> Vec<IlInstruction> {
    let mut instructions = Vec::new();
    if data.is_empty() {
        return instructions;
    }

    let mut offset;
    let il_end;
    if is_tiny {
        offset = 0;
        il_end = data.len();
    }
    else {
        // Fat format: 12 字节头 (ECMA-335 II.25.4.3)。
        // 0-1: Flags (低 2 位 = 0x03 表示 Fat)
        // 2-3: MaxStack
        // 4-7: CodeLen
        // 8-11: LocalVarSigTok
        if data.len() < 12 {
            return instructions;
        }
        let flags = u16::from_le_bytes([data[0], data[1]]);
        if flags & 0x03 != 0x03 {
            return instructions;
        }
        let code_len = u32::from_le_bytes([data[4], data[5], data[6], data[7]]) as usize;
        offset = 12;
        il_end = code_len;
    }

    let mut il_offset = 0usize;
    while il_offset < il_end && offset < data.len() {
        let instr_start = il_offset;
        let op_byte_raw = data[offset];
        offset += 1;
        il_offset += 1;

        // 处理 0xFE 前缀（双字节操作码）。
        let op_key = if op_byte_raw == 0xFE && offset < data.len() {
            let second = data[offset];
            offset += 1;
            il_offset += 1;
            0xFE00 + second as usize
        }
        else {
            op_byte_raw as usize
        };

        let info = lookup_opcode(op_key);
        let opcode = match info {
            Some(i) => i.opcode.to_string(),
            None => {
                if op_key > 0xFF {
                    format!("0x{:04X}", op_key)
                }
                else {
                    format!("0x{:02X}", op_key)
                }
            }
        };
        let has_operand = info.map(|i| i.has_operand).unwrap_or(false);
        let operand_size = info.map(|i| i.operand_size).unwrap_or(0);

        let operand = if has_operand {
            if op_key == 0x45 {
                // switch：count(u32) + count × i32 增量。
                decode_switch_operand(data, &mut offset, &mut il_offset).unwrap_or(IlOperand::None)
            }
            else if offset + operand_size <= data.len() {
                let op = read_operand(op_key, data, offset, operand_size, il_offset);
                offset += operand_size;
                il_offset += operand_size;
                op
            }
            else {
                break;
            }
        }
        else {
            IlOperand::None
        };

        instructions.push(IlInstruction { offset: instr_start, opcode, operand });
    }

    instructions
}

/// 解码 `switch` 操作码的变长跳转表。
///
/// 读取 4 字节跳转项数量 `count`，再读取 `count` 个 4 字节有符号增量，
/// 每个目标为「跳转表之后的下一条指令偏移加上增量」。返回 `false` 表示数据越界。
fn decode_switch_operand(data: &[u8], offset: &mut usize, il_offset: &mut usize) -> Option<IlOperand> {
    if *offset + 4 > data.len() {
        return None;
    }
    let count = u32::from_le_bytes([data[*offset], data[*offset + 1], data[*offset + 2], data[*offset + 3]]) as usize;
    let targets_bytes = count.checked_mul(4)?;
    if *offset + 4 + targets_bytes > data.len() {
        return None;
    }
    let next_instr = *il_offset + 4 + targets_bytes;
    let mut targets = Vec::with_capacity(count);
    for t in 0..count {
        let base = *offset + 4 + t * 4;
        let delta = i32::from_le_bytes([data[base], data[base + 1], data[base + 2], data[base + 3]]);
        targets.push((next_instr as i32).wrapping_add(delta));
    }
    *offset += 4 + targets_bytes;
    *il_offset = next_instr;
    Some(IlOperand::Switch(targets))
}

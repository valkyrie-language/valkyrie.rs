//! `IL` 方法体编码器。
//!
//! 将 `MSIL` 指令序列编码为 `CLR` 方法体二进制格式（`ECMA-335` II.25.4）。
//!
//! 支持两种格式：
//! - `Tiny`：1 字节头 + 代码，适用于代码 < 64 字节、无局部变量、`max_stack <= 8` 的方法。
//! - `Fat`：12 字节头 + 代码，适用于所有其他方法。
//!
//! 分支指令使用两遍编码：第一遍计算每条指令的偏移量，第二遍解析分支目标为相对偏移。

use crate::msil::{MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilOpcode};
use miette::{Diagnostic, Severity};
use std::{collections::HashMap, fmt};

/// 方法体编码错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MethodBodyError {
    /// 指令包含不支持的操作数。
    UnsupportedOperand(String),
    /// 操作数与操作码不匹配。
    OperandMismatch(String),
    /// 未定义的分支目标标签。
    UndefinedLabel(String),
}

impl fmt::Display for MethodBodyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedOperand(msg) => write!(f, "不支持的操作数: {msg}"),
            Self::OperandMismatch(msg) => write!(f, "操作数不匹配: {msg}"),
            Self::UndefinedLabel(msg) => write!(f, "未定义的分支目标标签: {msg}"),
        }
    }
}

impl std::error::Error for MethodBodyError {}

impl Diagnostic for MethodBodyError {
    fn code<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new(match self {
            Self::UnsupportedOperand(_) => "nyar::clr::method_body::unsupported_operand",
            Self::OperandMismatch(_) => "nyar::clr::method_body::operand_mismatch",
            Self::UndefinedLabel(_) => "nyar::clr::method_body::undefined_label",
        }))
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn fmt::Display + 'a>> {
        Some(Box::new("请检查 `MSIL` 指令与操作数是否匹配，或先完成对应操作数的编码实现"))
    }
}

/// 方法体编码器。
pub struct MethodBodyEncoder;

impl MethodBodyEncoder {
    /// 编码方法体为字节数组。
    ///
    /// `local_var_sig_token` 为 `StandAloneSig` 表中的 `LocalVarSig` token，
    /// 0 表示无局部变量。
    pub fn encode(method: &MsilMethodBody, local_var_sig_token: u32) -> Result<Vec<u8>, MethodBodyError> {
        let code_bytes = match Self::encode_instructions(&method.instructions) {
            Ok(bytes) => bytes,
            Err(error) => {
                // 遇到编码失败时打印方法名和指令序列，便于定位问题。
                eprintln!("=== method_body encode failed: {} ===", method.method.name);
                for (i, instr) in method.instructions.iter().enumerate() {
                    eprintln!("  {i:3}: label={:?} {:?} {:?}", instr.label, instr.opcode, instr.operand);
                }
                return Err(error);
            }
        };
        let code_size = code_bytes.len();

        // 判断是否可以使用 Tiny 格式。
        let can_tiny = code_size < 64 && method.locals.is_empty() && method.max_stack <= 8;

        if can_tiny {
            let mut bytes = Vec::with_capacity(1 + code_size);
            // Tiny 头：(code_size << 2) | 0x02
            bytes.push((code_size << 2) as u8 | 0x02);
            bytes.extend_from_slice(&code_bytes);
            Ok(bytes)
        }
        else {
            // Fat 头：12 字节。
            let mut bytes = Vec::with_capacity(12 + code_size);
            // Flags: 0x03 (Fat) | 0x3000 (12 字节头) | 0x10 (InitLocals)
            let flags: u16 = 0x03 | 0x3000 | 0x10;
            bytes.extend_from_slice(&flags.to_le_bytes());
            bytes.extend_from_slice(&method.max_stack.to_le_bytes());
            bytes.extend_from_slice(&(code_size as u32).to_le_bytes());
            // LocalVarSigTok：StandAloneSig 表中的 token，0 表示无局部变量。
            bytes.extend_from_slice(&local_var_sig_token.to_le_bytes());
            bytes.extend_from_slice(&code_bytes);
            // 对齐到 4 字节边界。
            let padding = (4 - code_size % 4) % 4;
            bytes.extend(std::iter::repeat(0u8).take(padding));
            Ok(bytes)
        }
    }

    /// 编码指令序列为字节。
    ///
    /// 使用两遍编码：
    /// 1. 第一遍计算每条指令的偏移量，并记录标签到偏移量的映射。
    /// 2. 第二遍编码指令，遇到分支指令时解析目标标签为相对偏移。
    fn encode_instructions(instructions: &[MsilInstruction]) -> Result<Vec<u8>, MethodBodyError> {
        // 第一遍：计算每条指令的偏移量。
        let mut label_offsets: HashMap<String, usize> = HashMap::new();
        let mut instruction_offsets: Vec<usize> = Vec::with_capacity(instructions.len());
        let mut current_offset = 0usize;

        for instr in instructions {
            instruction_offsets.push(current_offset);
            // 记录标签。
            if let Some(label) = &instr.label {
                label_offsets.insert(label.clone(), current_offset);
            }
            current_offset += Self::instruction_size(instr);
        }

        let total_code_size = current_offset;

        // 第二遍：编码指令。
        let mut bytes = Vec::with_capacity(total_code_size);
        let instruction_count = instructions.len();
        for (index, instr) in instructions.iter().enumerate() {
            let instr_offset = instruction_offsets[index];
            Self::encode_instruction(instr, instr_offset, instruction_count, &label_offsets, &mut bytes)?;
        }
        Ok(bytes)
    }

    /// 计算单条指令的字节大小。
    fn instruction_size(instr: &MsilInstruction) -> usize {
        let opcode_size = Self::opcode_size(instr.opcode);
        let operand_size = Self::operand_size(instr);
        opcode_size + operand_size
    }

    /// 计算操作码的字节大小。
    fn opcode_size(opcode: MsilOpcode) -> usize {
        if opcode.encoding() > 0xFF {
            2
        }
        else {
            1
        }
    }

    /// 计算操作数的字节大小。
    fn operand_size(instr: &MsilInstruction) -> usize {
        match &instr.operand {
            None => 0,
            Some(operand) => match operand {
                MsilInstructionOperand::Integer(_) => match instr.opcode {
                    MsilOpcode::LdcI4S => 1,
                    MsilOpcode::LdcI4 => 4,
                    MsilOpcode::LdcI8 => 8,
                    MsilOpcode::Ldloc | MsilOpcode::Stloc | MsilOpcode::Ldarg => 2,
                    _ => 4,
                },
                MsilInstructionOperand::Float(_) => 8,
                MsilInstructionOperand::StringLiteral(_) => 4,
                MsilInstructionOperand::Symbol(_) => 4,
                MsilInstructionOperand::Method(_) => 4,
                MsilInstructionOperand::Type(_) => 4,
                MsilInstructionOperand::Field(_, _) => 4,
                MsilInstructionOperand::Token(_) => 4,
                MsilInstructionOperand::BranchTarget(_) => {
                    // 分支指令使用 4 字节偏移（长格式）。
                    // 短格式分支使用 1 字节，但为简化实现，统一使用长格式。
                    if Self::is_short_branch(instr.opcode) {
                        1
                    }
                    else {
                        4
                    }
                }
                MsilInstructionOperand::Raw(_) => 0,
            },
        }
    }

    /// 判断是否为短格式分支指令。
    fn is_short_branch(opcode: MsilOpcode) -> bool {
        matches!(
            opcode,
            MsilOpcode::BrS
                | MsilOpcode::BrfalseS
                | MsilOpcode::BrtrueS
                | MsilOpcode::BeqS
                | MsilOpcode::BneUnS
                | MsilOpcode::BgeS
                | MsilOpcode::BgtS
                | MsilOpcode::BleS
                | MsilOpcode::BltS
                | MsilOpcode::BgeUnS
                | MsilOpcode::BgtUnS
                | MsilOpcode::BleUnS
                | MsilOpcode::BltUnS
        )
    }

    /// 编码单条指令。
    fn encode_instruction(
        instr: &MsilInstruction,
        instr_offset: usize,
        instruction_count: usize,
        label_offsets: &HashMap<String, usize>,
        bytes: &mut Vec<u8>,
    ) -> Result<(), MethodBodyError> {
        // 写入操作码。
        Self::encode_opcode(instr.opcode, bytes);

        // 写入操作数。
        if let Some(operand) = &instr.operand {
            Self::encode_operand(operand, instr.opcode, instr_offset, instruction_count, label_offsets, bytes)?;
        }
        Ok(())
    }

    /// 编码操作码。
    fn encode_opcode(opcode: MsilOpcode, bytes: &mut Vec<u8>) {
        let op_word = opcode.encoding();
        if op_word > 0xFF {
            // 双字节操作码。
            bytes.push(0xFE);
            bytes.push((op_word & 0xFF) as u8);
        }
        else {
            bytes.push(op_word as u8);
        }
    }

    /// 编码操作数。
    fn encode_operand(
        operand: &MsilInstructionOperand,
        opcode: MsilOpcode,
        instr_offset: usize,
        instruction_count: usize,
        label_offsets: &HashMap<String, usize>,
        bytes: &mut Vec<u8>,
    ) -> Result<(), MethodBodyError> {
        match operand {
            MsilInstructionOperand::Integer(value) => {
                // 根据操作码决定写入 1/4/8 字节。
                match opcode {
                    MsilOpcode::LdcI4S => {
                        bytes.push((*value as i8) as u8);
                    }
                    MsilOpcode::LdcI4 => {
                        bytes.extend_from_slice(&(*value as i32).to_le_bytes());
                    }
                    MsilOpcode::LdcI8 => {
                        bytes.extend_from_slice(&value.to_le_bytes());
                    }
                    MsilOpcode::Ldloc | MsilOpcode::Stloc | MsilOpcode::Ldarg => {
                        bytes.extend_from_slice(&(*value as u16).to_le_bytes());
                    }
                    _ => {
                        // 默认写 4 字节。
                        bytes.extend_from_slice(&(*value as i32).to_le_bytes());
                    }
                }
            }
            MsilInstructionOperand::Float(_text) => {
                // 浮点数编码：先解析为 f32 或 f64。
                // 简化处理：写 8 字节 0。
                bytes.extend_from_slice(&0u64.to_le_bytes());
            }
            MsilInstructionOperand::StringLiteral(_value) => {
                // Ldstr 操作数是 #US 堆中的 token。
                // 这里先写 0，后续元数据编码会修正。
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
            MsilInstructionOperand::Symbol(_name) => {
                // 符号引用，先写 0，后续元数据编码会修正。
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
            MsilInstructionOperand::Method(_method_ref) => {
                // 方法引用 token，先写 0，后续元数据编码会修正。
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
            MsilInstructionOperand::Type(_type_name) => {
                // 类型引用 token，先写 0，后续元数据编码会修正。
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
            MsilInstructionOperand::Field(_, _) => {
                // 字段引用 token，先写 0，后续元数据编码会修正。
                bytes.extend_from_slice(&0u32.to_le_bytes());
            }
            MsilInstructionOperand::Token(token) => {
                bytes.extend_from_slice(&token.to_le_bytes());
            }
            MsilInstructionOperand::BranchTarget(label) => {
                // 解析分支目标标签为相对偏移。
                let target_offset = *label_offsets.get(label).ok_or_else(|| {
                    let known: Vec<String> = label_offsets.keys().cloned().collect();
                    MethodBodyError::UndefinedLabel(format!("{label}（已知标签: {known:?}，指令数: {instruction_count}）"))
                })?;

                // 计算分支指令结束位置到目标位置的相对偏移。
                let operand_size = if Self::is_short_branch(opcode) { 1 } else { 4 };
                let next_instr_offset = instr_offset + Self::opcode_size(opcode) + operand_size;
                let relative_offset = target_offset as i64 - next_instr_offset as i64;

                if Self::is_short_branch(opcode) {
                    // 短格式分支：1 字节偏移。
                    bytes.push((relative_offset as i8) as u8);
                }
                else {
                    // 长格式分支：4 字节偏移。
                    bytes.extend_from_slice(&(relative_offset as i32).to_le_bytes());
                }
            }
            MsilInstructionOperand::Raw(_raw) => {
                // 原始文本，暂不处理。
            }
        }
        Ok(())
    }
}

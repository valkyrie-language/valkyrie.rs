//! `WASM` 指令解码层。
//!
//! 在字节流读取器与操作码查表的基础上，提供单条指令、`GC` 前缀指令
//! 与函数体指令序列的解码能力，行为对齐 `spy` 的 `decode_instruction`。

use super::{
    instruction::{DecodedInstruction, DecodedOperand},
    opcode::{fc_mnemonic, gc_mnemonic, opcode_mnemonic},
    reader::WasmByteReader,
};

/// 解码单条 `WASM` 指令。
///
/// `opcode` 为已读取的操作码字节，`reader` 游标位于操作码之后，
/// `offset` 为该操作码在字节流中的起始偏移。读取失败时以合理默认值填充，
/// 不返回错误，与 `spy` 的容错行为一致。
pub fn decode_instruction(opcode: u8, reader: &mut WasmByteReader, offset: usize) -> DecodedInstruction {
    match opcode {
        0xFB => {
            let mut instr = decode_gc_instruction(reader);
            instr.offset = offset;
            instr
        }
        0xFC => {
            let sub = reader.read_uleb128().unwrap_or(0);
            let mnemonic = fc_mnemonic(sub).unwrap_or("fc.unknown");
            DecodedInstruction { offset, opcode, mnemonic: mnemonic.into(), operands: vec![DecodedOperand::SubOpcode(sub)], raw_size: 1 }
        }
        _ => {
            let mnemonic = opcode_mnemonic(opcode);
            let (operands, raw_size) = read_operands(opcode, reader);
            DecodedInstruction { offset, opcode, mnemonic: mnemonic.into(), operands, raw_size }
        }
    }
}

/// 解码 `GC` 前缀指令（`0xFB`）。
///
/// 读取子操作码与类型索引；对 `0x02`-`0x05` 额外读取字段索引，
/// 对 `0x08` 额外读取计数。未知子操作码仅保留子操作码而不读取类型索引，
/// 与 `spy` 的早返回行为一致。返回结构的 `offset` 字段为 `0`，
/// 由调用方（如 `decode_instruction`）按需覆盖。
pub fn decode_gc_instruction(reader: &mut WasmByteReader) -> DecodedInstruction {
    let sub = reader.read_uleb128().unwrap_or(0);
    let mnemonic = match gc_mnemonic(sub) {
        Some(name) => name,
        None => {
            return DecodedInstruction {
                offset: 0,
                opcode: 0xFB,
                mnemonic: "gc.unknown".into(),
                operands: vec![DecodedOperand::SubOpcode(sub)],
                raw_size: 1,
            };
        }
    };
    let type_index = reader.read_uleb128().unwrap_or(0);
    let mut operands = vec![DecodedOperand::TypeIndex(type_index)];
    if matches!(sub, 0x02 | 0x03 | 0x04 | 0x05) {
        let field = reader.read_uleb128().unwrap_or(0);
        operands.push(DecodedOperand::FieldIndex(field));
    }
    if sub == 0x08 {
        let count = reader.read_uleb128().unwrap_or(0);
        operands.push(DecodedOperand::Count(count));
    }
    DecodedInstruction { offset: 0, opcode: 0xFB, mnemonic: mnemonic.into(), operands, raw_size: 1 }
}

/// 解码函数体指令序列。
///
/// 输入 `bytes` 为完整的函数体字节（含局部变量声明前缀）。
/// 先跳过局部变量声明，再逐条解码指令并追踪 `block` / `loop` / `if`
/// 嵌套深度，深度归零表示函数体结束。行为对齐 `spy` 的
/// `disassemble_function_body`。
pub fn decode_code_body(bytes: &[u8]) -> Vec<DecodedInstruction> {
    let mut reader = WasmByteReader::new(bytes);
    let mut instructions = Vec::new();

    let local_groups = match reader.read_uleb128() {
        Ok(groups) => groups,
        Err(_) => return instructions,
    };
    for _ in 0..local_groups {
        let _ = reader.read_uleb128();
        let _ = skip_value_type(&mut reader);
    }

    let mut depth: u32 = 1;
    loop {
        if reader.is_eof() {
            break;
        }
        let instr_offset = reader.offset();
        let opcode = match reader.read_u8() {
            Ok(byte) => byte,
            Err(_) => break,
        };
        let instr = decode_instruction(opcode, &mut reader, instr_offset);
        instructions.push(instr);
        match opcode {
            0x02 | 0x03 | 0x04 => depth += 1,
            0x0B => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
    }

    instructions
}

/// 读取内存操作数（对齐与偏移），读取失败时以 `0` 填充。
fn read_memarg(reader: &mut WasmByteReader) -> DecodedOperand {
    let align = reader.read_uleb128().unwrap_or(0);
    let offset = reader.read_uleb128().unwrap_or(0);
    DecodedOperand::MemArg { align, offset }
}

/// 依据操作码读取立即数操作数，返回操作数列表与原始字节大小。
///
/// 原始字节大小对齐 `spy` 的尺寸语义：多数指令为 `1`（操作码字节数），
/// `memory.size` / `memory.grow` 为 `2`（含保留字节）。
fn read_operands(opcode: u8, reader: &mut WasmByteReader) -> (Vec<DecodedOperand>, usize) {
    match opcode {
        0x02 | 0x03 | 0x04 => {
            let block_type = reader.read_block_type().unwrap_or(0);
            (vec![DecodedOperand::BlockType(block_type)], 1)
        }
        0x0C | 0x0D => {
            let label = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::LabelIndex(label)], 1)
        }
        0x0E => {
            let count = reader.read_uleb128().unwrap_or(0);
            let mut targets = Vec::new();
            for _ in 0..count {
                targets.push(reader.read_uleb128().unwrap_or(0));
            }
            let default = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::BrTargets { targets, default }], 1)
        }
        0x10 => {
            let func = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::FuncIndex(func)], 1)
        }
        0x11 => {
            let type_idx = reader.read_uleb128().unwrap_or(0);
            let table_idx = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::CallIndirect { type_idx, table_idx }], 1)
        }
        0x1C => {
            let count = reader.read_uleb128().unwrap_or(0);
            let mut types = Vec::new();
            for _ in 0..count {
                types.push(reader.read_u8().unwrap_or(0));
            }
            (vec![DecodedOperand::SelectTypes(types)], 1)
        }
        0x20 | 0x21 | 0x22 => {
            let local = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::LocalIndex(local)], 1)
        }
        0x23 | 0x24 => {
            let global = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::GlobalIndex(global)], 1)
        }
        0x28..=0x2F | 0x36..=0x3B => (vec![read_memarg(reader)], 1),
        0x3F | 0x40 => {
            let _ = reader.read_u8();
            (Vec::new(), 2)
        }
        0x41 => {
            let value = reader.read_sleb128_i32().unwrap_or(0);
            (vec![DecodedOperand::ValueI32(value)], 1)
        }
        0x42 => {
            let value = reader.read_sleb128_i64().unwrap_or(0);
            (vec![DecodedOperand::ValueI64(value)], 1)
        }
        0x43 => {
            let value = reader.read_f32().unwrap_or(0.0);
            (vec![DecodedOperand::ValueF32(value)], 1)
        }
        0x44 => {
            let value = reader.read_f64().unwrap_or(0.0);
            (vec![DecodedOperand::ValueF64(value)], 1)
        }
        0xD0 => {
            let reftype = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::RefNull(reftype)], 1)
        }
        0xD2 => {
            let func = reader.read_uleb128().unwrap_or(0);
            (vec![DecodedOperand::RefFunc(func)], 1)
        }
        _ => (Vec::new(), 1),
    }
}

/// 跳过单个值类型，对齐 `spy` 的 `skip_value_type`。
///
/// 普通值类型占一字节；当读到 `0x64` 时额外消耗一个有符号 `LEB128`。
fn skip_value_type(reader: &mut WasmByteReader) -> Result<(), super::WasmBinaryError> {
    let byte = reader.read_u8()?;
    if byte == 0x64 {
        let _ = reader.read_sleb128_i32();
    }
    Ok(())
}

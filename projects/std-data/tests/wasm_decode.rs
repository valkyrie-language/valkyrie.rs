//! `WASM` 指令解码层集成测试。
//!
//! 覆盖常见操作码、`GC` 前缀（`0xFB`）全子操作码、杂项前缀（`0xFC`）
//! 子操作码，以及 `decode_code_body` 的端到端解码。

use std_data::binary::wasm::{DecodedOperand, WasmByteReader, decode_code_body, decode_instruction, write_sleb128_i64};

#[test]
fn fixed_shift_seven_contract_matches_signed_leb128_floor_step() {
    // `wasm_ashr7_i64` is a unary semantic helper: its negative values must
    // use arithmetic/floor division by 128, not truncation toward zero.
    let cases = [(-1_i64, vec![0x7F]), (-128_i64, vec![0x80, 0x7F]), (-129_i64, vec![0xFF, 0x7E]), (128_i64, vec![0x80, 0x01])];
    for (value, expected) in cases {
        let mut encoded = Vec::new();
        write_sleb128_i64(value, &mut encoded);
        assert_eq!(encoded, expected, "SLEB128 fixed-shift contract for {value}");
    }
}

/// 从字节序列解码单条指令，操作码为首个字节。
fn decode_one(bytes: &[u8]) -> std_data::binary::wasm::DecodedInstruction {
    let mut reader = WasmByteReader::new(bytes);
    let opcode = reader.read_u8().expect("读取操作码");
    decode_instruction(opcode, &mut reader, 0)
}

#[test]
fn 控制流指令解码() {
    let instr = decode_one(&[0x00]);
    assert_eq!(instr.mnemonic, "unreachable");
    assert!(instr.operands.is_empty());
    assert_eq!(instr.raw_size, 1);

    let instr = decode_one(&[0x01]);
    assert_eq!(instr.mnemonic, "nop");

    let instr = decode_one(&[0x02, 0x40]);
    assert_eq!(instr.mnemonic, "block");
    assert_eq!(instr.operands, vec![DecodedOperand::BlockType(-64)]);

    let instr = decode_one(&[0x03, 0x40]);
    assert_eq!(instr.mnemonic, "loop");
    assert_eq!(instr.operands, vec![DecodedOperand::BlockType(-64)]);

    let instr = decode_one(&[0x04, 0x7F]);
    assert_eq!(instr.mnemonic, "if");
    assert_eq!(instr.operands, vec![DecodedOperand::BlockType(-1)]);

    let instr = decode_one(&[0x05]);
    assert_eq!(instr.mnemonic, "else");

    let instr = decode_one(&[0x0B]);
    assert_eq!(instr.mnemonic, "end");

    let instr = decode_one(&[0x0C, 0x03]);
    assert_eq!(instr.mnemonic, "br");
    assert_eq!(instr.operands, vec![DecodedOperand::LabelIndex(3)]);

    let instr = decode_one(&[0x0D, 0x05]);
    assert_eq!(instr.mnemonic, "br_if");
    assert_eq!(instr.operands, vec![DecodedOperand::LabelIndex(5)]);

    let instr = decode_one(&[0x0E, 0x02, 0x00, 0x01, 0x03]);
    assert_eq!(instr.mnemonic, "br_table");
    assert_eq!(instr.operands, vec![DecodedOperand::BrTargets { targets: vec![0, 1], default: 3 }]);

    let instr = decode_one(&[0x0F]);
    assert_eq!(instr.mnemonic, "return");

    let instr = decode_one(&[0x10, 0x07]);
    assert_eq!(instr.mnemonic, "call");
    assert_eq!(instr.operands, vec![DecodedOperand::FuncIndex(7)]);

    let instr = decode_one(&[0x11, 0x05, 0x00]);
    assert_eq!(instr.mnemonic, "call_indirect");
    assert_eq!(instr.operands, vec![DecodedOperand::CallIndirect { type_idx: 5, table_idx: 0 }]);
}

#[test]
fn 变量指令解码() {
    let instr = decode_one(&[0x20, 0x05]);
    assert_eq!(instr.mnemonic, "local.get");
    assert_eq!(instr.operands, vec![DecodedOperand::LocalIndex(5)]);

    let instr = decode_one(&[0x21, 0x05]);
    assert_eq!(instr.mnemonic, "local.set");
    assert_eq!(instr.operands, vec![DecodedOperand::LocalIndex(5)]);

    let instr = decode_one(&[0x22, 0x09]);
    assert_eq!(instr.mnemonic, "local.tee");
    assert_eq!(instr.operands, vec![DecodedOperand::LocalIndex(9)]);

    let instr = decode_one(&[0x23, 0x02]);
    assert_eq!(instr.mnemonic, "global.get");
    assert_eq!(instr.operands, vec![DecodedOperand::GlobalIndex(2)]);

    let instr = decode_one(&[0x24, 0x02]);
    assert_eq!(instr.mnemonic, "global.set");
    assert_eq!(instr.operands, vec![DecodedOperand::GlobalIndex(2)]);
}

#[test]
fn 内存指令解码() {
    let instr = decode_one(&[0x28, 0x02, 0x04]);
    assert_eq!(instr.mnemonic, "i32.load");
    assert_eq!(instr.operands, vec![DecodedOperand::MemArg { align: 2, offset: 4 }]);

    let instr = decode_one(&[0x36, 0x00, 0x10]);
    assert_eq!(instr.mnemonic, "i32.store");
    assert_eq!(instr.operands, vec![DecodedOperand::MemArg { align: 0, offset: 16 }]);

    let instr = decode_one(&[0x3F, 0x00]);
    assert_eq!(instr.mnemonic, "memory.size");
    assert!(instr.operands.is_empty());
    assert_eq!(instr.raw_size, 2);

    let instr = decode_one(&[0x40, 0x00]);
    assert_eq!(instr.mnemonic, "memory.grow");
    assert!(instr.operands.is_empty());
    assert_eq!(instr.raw_size, 2);
}

#[test]
fn 常量指令解码() {
    let instr = decode_one(&[0x41, 0x0A]);
    assert_eq!(instr.mnemonic, "i32.const");
    assert_eq!(instr.operands, vec![DecodedOperand::ValueI32(10)]);

    let instr = decode_one(&[0x41, 0x7F]);
    assert_eq!(instr.operands, vec![DecodedOperand::ValueI32(-1)]);

    let instr = decode_one(&[0x42, 0x01]);
    assert_eq!(instr.mnemonic, "i64.const");
    assert_eq!(instr.operands, vec![DecodedOperand::ValueI64(1)]);

    let instr = decode_one(&[0x43, 0x00, 0x00, 0x20, 0x41]);
    assert_eq!(instr.mnemonic, "f32.const");
    assert_eq!(instr.operands, vec![DecodedOperand::ValueF32(10.0)]);

    let instr = decode_one(&[0x44, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0x40]);
    assert_eq!(instr.mnemonic, "f64.const");
    assert_eq!(instr.operands, vec![DecodedOperand::ValueF64(10.0)]);
}

#[test]
fn 数值与参考指令解码() {
    let instr = decode_one(&[0x45]);
    assert_eq!(instr.mnemonic, "i32.eqz");

    let instr = decode_one(&[0x6A]);
    assert_eq!(instr.mnemonic, "i32.add");

    let instr = decode_one(&[0x76]);
    assert_eq!(instr.mnemonic, "i32.shr_u");

    let instr = decode_one(&[0xD0, 0x70]);
    assert_eq!(instr.mnemonic, "ref.null");
    assert_eq!(instr.operands, vec![DecodedOperand::RefNull(0x70)]);

    let instr = decode_one(&[0xD1]);
    assert_eq!(instr.mnemonic, "ref.is_null");

    let instr = decode_one(&[0xD2, 0x03]);
    assert_eq!(instr.mnemonic, "ref.func");
    assert_eq!(instr.operands, vec![DecodedOperand::RefFunc(3)]);
}

#[test]
fn gc前缀全子操作码助记符() {
    let expected: &[(u32, &str)] = &[
        (0x00, "struct.new"),
        (0x01, "struct.new_default"),
        (0x02, "struct.get"),
        (0x03, "struct.get_s"),
        (0x04, "struct.get_u"),
        (0x05, "struct.set"),
        (0x06, "array.new"),
        (0x07, "array.new_default"),
        (0x08, "array.new_fixed"),
        (0x09, "array.new_data"),
        (0x0A, "array.new_elem"),
        (0x0B, "array.get"),
        (0x0C, "array.get_s"),
        (0x0D, "array.get_u"),
        (0x0E, "array.set"),
        (0x0F, "array.len"),
        (0x10, "array.fill"),
        (0x11, "array.copy"),
        (0x12, "array.init_data"),
        (0x13, "array.init_elem"),
        (0x14, "ref.test"),
        (0x15, "ref.test_null"),
        (0x16, "ref.cast"),
        (0x17, "ref.cast_null"),
    ];
    for (sub, name) in expected {
        let instr = decode_one(&[0xFB, *sub as u8, 0x01, 0x02, 0x03]);
        assert_eq!(instr.mnemonic, *name, "GC sub 0x{:02X}", sub);
        assert_eq!(instr.opcode, 0xFB);
    }
}

#[test]
fn gc字段索引操作数() {
    let instr = decode_one(&[0xFB, 0x02, 0x03, 0x04]);
    assert_eq!(instr.mnemonic, "struct.get");
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(3), DecodedOperand::FieldIndex(4)]);

    let instr = decode_one(&[0xFB, 0x03, 0x0A, 0x0B]);
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(10), DecodedOperand::FieldIndex(11)]);

    let instr = decode_one(&[0xFB, 0x04, 0x01, 0x02]);
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(1), DecodedOperand::FieldIndex(2)]);

    let instr = decode_one(&[0xFB, 0x05, 0x06, 0x07]);
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(6), DecodedOperand::FieldIndex(7)]);
}

#[test]
fn gc计数操作数() {
    let instr = decode_one(&[0xFB, 0x08, 0x01, 0x07]);
    assert_eq!(instr.mnemonic, "array.new_fixed");
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(1), DecodedOperand::Count(7)]);
}

#[test]
fn gc无字段子操作码仅有类型索引() {
    let instr = decode_one(&[0xFB, 0x00, 0x03, 0xFF]);
    assert_eq!(instr.mnemonic, "struct.new");
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(3)]);

    let instr = decode_one(&[0xFB, 0x0F, 0x05, 0xFF]);
    assert_eq!(instr.mnemonic, "array.len");
    assert_eq!(instr.operands, vec![DecodedOperand::TypeIndex(5)]);
}

#[test]
fn fc前缀全子操作码() {
    let expected: &[(u32, &str)] = &[
        (0, "i32.trunc_f32_s"),
        (1, "i32.trunc_f32_u"),
        (2, "i32.trunc_f64_s"),
        (3, "i32.trunc_f64_u"),
        (4, "i64.trunc_f32_s"),
        (5, "i64.trunc_f32_u"),
        (6, "i64.trunc_f64_s"),
        (7, "i64.trunc_f64_u"),
        (8, "memory.init"),
        (9, "data.drop"),
        (10, "memory.copy"),
        (11, "memory.fill"),
    ];
    for (sub, name) in expected {
        let instr = decode_one(&[0xFC, *sub as u8]);
        assert_eq!(instr.mnemonic, *name, "FC sub {}", sub);
        assert_eq!(instr.opcode, 0xFC);
        assert_eq!(instr.operands, vec![DecodedOperand::SubOpcode(*sub)]);
    }
}

#[test]
fn code_body端到端解码() {
    let bytes = [0x00, 0x41, 0x0A, 0x1A, 0x0B];
    let instrs = decode_code_body(&bytes);
    assert_eq!(instrs.len(), 3);
    assert_eq!(instrs[0].mnemonic, "i32.const");
    assert_eq!(instrs[0].operands, vec![DecodedOperand::ValueI32(10)]);
    assert_eq!(instrs[1].mnemonic, "drop");
    assert_eq!(instrs[2].mnemonic, "end");
}

#[test]
fn code_body追踪嵌套块深度() {
    let bytes = [0x00, 0x02, 0x40, 0x01, 0x0B, 0x0B];
    let instrs = decode_code_body(&bytes);
    assert_eq!(instrs.len(), 4);
    assert_eq!(instrs[0].mnemonic, "block");
    assert_eq!(instrs[0].operands, vec![DecodedOperand::BlockType(-64)]);
    assert_eq!(instrs[1].mnemonic, "nop");
    assert_eq!(instrs[2].mnemonic, "end");
    assert_eq!(instrs[3].mnemonic, "end");
}

#[test]
fn code_body含gc前缀指令() {
    let bytes = [0x00, 0xFB, 0x00, 0x01, 0x0B];
    let instrs = decode_code_body(&bytes);
    assert_eq!(instrs.len(), 2);
    assert_eq!(instrs[0].mnemonic, "struct.new");
    assert_eq!(instrs[0].operands, vec![DecodedOperand::TypeIndex(1)]);
    assert_eq!(instrs[1].mnemonic, "end");
}

#[test]
fn code_body跳过局部变量声明() {
    let bytes = [0x01, 0x02, 0x7F, 0x41, 0x05, 0x0B];
    let instrs = decode_code_body(&bytes);
    assert_eq!(instrs.len(), 2);
    assert_eq!(instrs[0].mnemonic, "i32.const");
    assert_eq!(instrs[0].operands, vec![DecodedOperand::ValueI32(5)]);
    assert_eq!(instrs[1].mnemonic, "end");
}

//! `JVM` 解码层集成测试。
//!
//! 覆盖 [`JvmClassFile::from_bytes`] 的字段与方法码解析、
//! [`decode_instructions`] 对常见操作码与 `tableswitch` / `lookupswitch` / `wide` 的解码，
//! 以及 [`JvmJarPackage::iter_classes`] 的归档遍历。

use std_data::binary::{
    class::{
        ConstantPool, ConstantPoolEntry, DecodedJvmOperand, JvmClassFile, JvmFieldSignature, JvmMethodDescriptor, JvmMethodSignature,
        JvmTypeDescriptor, decode_instructions,
    },
    jar::JvmJarPackage,
};

#[test]
fn long_constant_zero_one_use_category_two_opcodes() {
    let pool = ConstantPool::default();
    let instructions = decode_instructions(&[0x09, 0x0A, 0xAD], &pool);
    assert_eq!(instructions.iter().map(|instruction| instruction.mnemonic.as_str()).collect::<Vec<_>>(), ["lconst_0", "lconst_1", "lreturn"]);
}

#[test]
fn long_constant_pool_use_ldc2_w_opcode() {
    let pool = ConstantPool::default();
    let instructions = decode_instructions(&[0x14, 0x00, 0x07, 0xAD], &pool);
    assert_eq!(instructions[0].mnemonic, "ldc2_w");
    assert_eq!(instructions[0].operand, Some(DecodedJvmOperand::ConstantIndex(7)));
    assert_eq!(instructions[1].mnemonic, "lreturn");
}

/// 构造一个含字段与方法的 `class` 模型，编码后再解码，验证 `from_bytes` 完整解析。
#[test]
fn from_bytes_完整解析字段与方法码() {
    let mut class = JvmClassFile::new("demo/Sample");
    class.fields.push(JvmFieldSignature { name: "value".to_string(), descriptor: JvmTypeDescriptor::Int, access_flags: 0x0001 });
    class.push_method("compute", JvmMethodDescriptor::new(Vec::new(), JvmTypeDescriptor::Int));

    let bytes = class.to_bytes().expect("编码 class 失败");
    let decoded = JvmClassFile::from_bytes(&bytes).expect("解码 class 失败");

    assert_eq!(decoded.internal_name, "demo/Sample");
    assert_eq!(decoded.super_name, "java/lang/Object");
    assert_eq!(decoded.fields.len(), 1);
    assert_eq!(decoded.fields[0].name, "value");
    assert_eq!(decoded.fields[0].descriptor, JvmTypeDescriptor::Int);
    assert_eq!(decoded.methods.len(), 1);
    assert_eq!(decoded.methods[0].name, "compute");
    assert_eq!(decoded.raw_method_code.len(), 1);
    let method_code = decoded.raw_method_code[0].as_ref().expect("方法应含 Code 属性");
    assert!(!method_code.is_empty());
}

/// 验证 `from_bytes` 对无 `Code` 属性的抽象方法也能正确解析。
#[test]
fn from_bytes_抽象方法无code属性() {
    let mut class = JvmClassFile::new("demo/Abstract");
    class.methods.push(JvmMethodSignature {
        name: "abstractMethod".to_string(),
        descriptor: JvmMethodDescriptor::new(Vec::new(), JvmTypeDescriptor::Void),
        access_flags: 0x0401,
        code: None,
    });

    let bytes = class.to_bytes().expect("编码 class 失败");
    let decoded = JvmClassFile::from_bytes(&bytes).expect("解码 class 失败");

    assert_eq!(decoded.methods.len(), 1);
    assert_eq!(decoded.methods[0].name, "abstractMethod");
    assert!(decoded.raw_method_code[0].is_none());
}

/// 验证常见操作码（`nop` / `iconst` / `bipush` / `sipush` / `iload` / `istore` / `iadd` / `goto` / `return`）解码。
#[test]
fn decode_instructions_常用操作码() {
    let pool = ConstantPool::default();
    let code = [
        0x00, // nop
        0x03, // iconst_0
        0x04, // iconst_1
        0x10, 0x0A, // bipush 10
        0x11, 0x01, 0x00, // sipush 256
        0x15, 0x02, // iload 2
        0x36, 0x03, // istore 3
        0x60, // iadd
        0xA7, 0x00, 0x05, // goto +5
        0xB1, // return
    ];
    let instructions = decode_instructions(&code, &pool);

    assert_eq!(instructions.len(), 10);
    assert_eq!(instructions[0].mnemonic, "nop");
    assert_eq!(instructions[0].size, 1);
    assert_eq!(instructions[1].mnemonic, "iconst_0");
    assert_eq!(instructions[2].mnemonic, "iconst_1");
    assert_eq!(instructions[3].mnemonic, "bipush");
    assert_eq!(instructions[3].operand, Some(DecodedJvmOperand::Int(10)));
    assert_eq!(instructions[4].mnemonic, "sipush");
    assert_eq!(instructions[4].operand, Some(DecodedJvmOperand::Int(256)));
    assert_eq!(instructions[5].mnemonic, "iload");
    assert_eq!(instructions[5].operand, Some(DecodedJvmOperand::Local(2)));
    assert_eq!(instructions[6].mnemonic, "istore");
    assert_eq!(instructions[6].operand, Some(DecodedJvmOperand::Local(3)));
    assert_eq!(instructions[7].mnemonic, "iadd");
    assert_eq!(instructions[8].mnemonic, "goto");
    assert_eq!(instructions[8].operand, Some(DecodedJvmOperand::Branch(5)));
    assert_eq!(instructions[9].mnemonic, "return");
}

/// 验证常量池引用类操作码（`getstatic` / `invokevirtual` / `new`）的结构化解码。
#[test]
fn decode_instructions_常量池引用() {
    let pool = ConstantPool {
        entries: vec![
            ConstantPoolEntry::Padding,
            ConstantPoolEntry::Utf8("java/lang/System".to_string()),
            ConstantPoolEntry::Class(1),
            ConstantPoolEntry::Utf8("out".to_string()),
            ConstantPoolEntry::Utf8("Ljava/io/PrintStream;".to_string()),
            ConstantPoolEntry::NameAndType { name_index: 3, descriptor_index: 4 },
            ConstantPoolEntry::Fieldref { class_index: 2, name_and_type_index: 5 },
            ConstantPoolEntry::Utf8("java/io/PrintStream".to_string()),
            ConstantPoolEntry::Class(7),
            ConstantPoolEntry::Utf8("println".to_string()),
            ConstantPoolEntry::Utf8("(Ljava/lang/String;)V".to_string()),
            ConstantPoolEntry::NameAndType { name_index: 9, descriptor_index: 10 },
            ConstantPoolEntry::Methodref { class_index: 8, name_and_type_index: 11 },
            ConstantPoolEntry::Utf8("demo/Sample".to_string()),
            ConstantPoolEntry::Class(13),
        ],
    };
    let code = [
        0xB2, 0x00, 0x06, // getstatic #6
        0xB6, 0x00, 0x0C, // invokevirtual #12
        0xBB, 0x00, 0x0E, // new #14
        0xB1, // return
    ];
    let instructions = decode_instructions(&code, &pool);

    assert_eq!(instructions.len(), 4);

    let getstatic = &instructions[0];
    assert_eq!(getstatic.mnemonic, "getstatic");
    match &getstatic.operand {
        Some(DecodedJvmOperand::FieldRef(field_ref)) => {
            assert_eq!(field_ref.owner, "java/lang/System");
            assert_eq!(field_ref.name, "out");
            assert_eq!(field_ref.descriptor, JvmTypeDescriptor::Object("java/io/PrintStream".to_string()));
        }
        other => panic!("getstatic 操作数应为 FieldRef，实际：{other:?}"),
    }

    let invokevirtual = &instructions[1];
    assert_eq!(invokevirtual.mnemonic, "invokevirtual");
    match &invokevirtual.operand {
        Some(DecodedJvmOperand::MethodRef(method_ref)) => {
            assert_eq!(method_ref.owner, "java/io/PrintStream");
            assert_eq!(method_ref.name, "println");
            assert_eq!(method_ref.descriptor.parameter_types.len(), 1);
            assert_eq!(method_ref.descriptor.return_type, JvmTypeDescriptor::Void);
        }
        other => panic!("invokevirtual 操作数应为 MethodRef，实际：{other:?}"),
    }

    let new_instr = &instructions[2];
    assert_eq!(new_instr.mnemonic, "new");
    assert_eq!(new_instr.operand, Some(DecodedJvmOperand::ClassRef("demo/Sample".to_string())));
}

/// 验证 `tableswitch` 在偏移 0 处的 3 字节对齐填充与全部 `case` 偏移解码。
#[test]
fn decode_instructions_tableswitch_偏移零() {
    let pool = ConstantPool::default();
    let default: i32 = 100;
    let low: i32 = 0;
    let high: i32 = 2;
    let offsets: [i32; 3] = [10, 20, 30];
    let mut code = vec![0xAAu8];
    code.extend_from_slice(&[0u8; 3]);
    code.extend_from_slice(&default.to_be_bytes());
    code.extend_from_slice(&low.to_be_bytes());
    code.extend_from_slice(&high.to_be_bytes());
    for off in &offsets {
        code.extend_from_slice(&off.to_be_bytes());
    }

    let instructions = decode_instructions(&code, &pool);
    assert_eq!(instructions.len(), 1);
    let instr = &instructions[0];
    assert_eq!(instr.mnemonic, "tableswitch");
    assert_eq!(instr.offset, 0);
    assert_eq!(instr.size, code.len());
    match &instr.operand {
        Some(DecodedJvmOperand::TableSwitch { default, low, high, offsets }) => {
            assert_eq!(*default, 100);
            assert_eq!(*low, 0);
            assert_eq!(*high, 2);
            assert_eq!(*offsets, vec![10, 20, 30]);
        }
        other => panic!("tableswitch 操作数不匹配：{other:?}"),
    }
}

/// 验证 `tableswitch` 在偏移 3 处无需填充（`pc+1` 已 4 字节对齐）。
#[test]
fn decode_instructions_tableswitch_偏移三() {
    let pool = ConstantPool::default();
    let default: i32 = 50;
    let low: i32 = 0;
    let high: i32 = 1;
    let offsets: [i32; 2] = [5, 15];
    let mut code = vec![0x00u8; 3];
    code.push(0xAA);
    code.extend_from_slice(&default.to_be_bytes());
    code.extend_from_slice(&low.to_be_bytes());
    code.extend_from_slice(&high.to_be_bytes());
    for off in &offsets {
        code.extend_from_slice(&off.to_be_bytes());
    }

    let instructions = decode_instructions(&code, &pool);
    assert_eq!(instructions.len(), 4);
    let instr = &instructions[3];
    assert_eq!(instr.offset, 3);
    assert_eq!(instr.mnemonic, "tableswitch");
    assert_eq!(instr.size, 1 + 12 + 8);
    match &instr.operand {
        Some(DecodedJvmOperand::TableSwitch { default, low, high, offsets }) => {
            assert_eq!(*default, 50);
            assert_eq!(*low, 0);
            assert_eq!(*high, 1);
            assert_eq!(*offsets, vec![5, 15]);
        }
        other => panic!("tableswitch 操作数不匹配：{other:?}"),
    }
}

/// 验证 `lookupswitch` 在偏移 0 处的对齐填充与 `match-offset` 键值对解码。
#[test]
fn decode_instructions_lookupswitch() {
    let pool = ConstantPool::default();
    let default: i32 = 50;
    let npairs: i32 = 2;
    let pairs: [(i32, i32); 2] = [(1, 10), (5, 20)];
    let mut code = vec![0xABu8];
    code.extend_from_slice(&[0u8; 3]);
    code.extend_from_slice(&default.to_be_bytes());
    code.extend_from_slice(&npairs.to_be_bytes());
    for (match_val, offset) in &pairs {
        code.extend_from_slice(&match_val.to_be_bytes());
        code.extend_from_slice(&offset.to_be_bytes());
    }

    let instructions = decode_instructions(&code, &pool);
    assert_eq!(instructions.len(), 1);
    let instr = &instructions[0];
    assert_eq!(instr.mnemonic, "lookupswitch");
    assert_eq!(instr.offset, 0);
    assert_eq!(instr.size, code.len());
    match &instr.operand {
        Some(DecodedJvmOperand::LookupSwitch { default, pairs }) => {
            assert_eq!(*default, 50);
            assert_eq!(*pairs, vec![(1, 10), (5, 20)]);
        }
        other => panic!("lookupswitch 操作数不匹配：{other:?}"),
    }
}

/// 验证 `wide` 指令对普通操作码（4 字节）与 `iinc`（6 字节）的长度处理。
#[test]
fn decode_instructions_wide() {
    let pool = ConstantPool::default();
    let code = [
        0xC4, 0x15, 0x01, 0x02, // wide iload #258
        0xC4, 0x84, 0x01, 0x02, 0x00, 0x05, // wide iinc #258, 5
    ];
    let instructions = decode_instructions(&code, &pool);
    assert_eq!(instructions.len(), 2);

    let wide_iload = &instructions[0];
    assert_eq!(wide_iload.mnemonic, "wide");
    assert_eq!(wide_iload.size, 4);
    assert_eq!(wide_iload.operand, Some(DecodedJvmOperand::WideLocal(258)));

    let wide_iinc = &instructions[1];
    assert_eq!(wide_iinc.mnemonic, "wide");
    assert_eq!(wide_iinc.size, 6);
    assert_eq!(wide_iinc.operand, Some(DecodedJvmOperand::WideLocal(258)));
}

/// 验证 `goto_w` / `jsr_w` 的 32 位宽分支偏移解码。
#[test]
fn decode_instructions_宽分支() {
    let pool = ConstantPool::default();
    let code = [
        0xC8, 0x00, 0x00, 0x01, 0x00, // goto_w +256
        0xC9, 0xFF, 0xFF, 0xFE, 0x00, // jsr_w -512
    ];
    let instructions = decode_instructions(&code, &pool);
    assert_eq!(instructions.len(), 2);
    assert_eq!(instructions[0].mnemonic, "goto_w");
    assert_eq!(instructions[0].size, 5);
    assert_eq!(instructions[0].operand, Some(DecodedJvmOperand::BranchWide(256)));
    assert_eq!(instructions[1].mnemonic, "jsr_w");
    assert_eq!(instructions[1].operand, Some(DecodedJvmOperand::BranchWide(-512)));
}

/// 验证 `iter_classes` 仅遍历 `.class` 后缀入口，且返回路径与字节正确。
#[test]
fn iter_classes_仅遍历class入口() {
    let class_bytes = JvmClassFile::new("demo/First").to_bytes().expect("编码 class 失败");
    let mut package = JvmJarPackage::new("test.jar");
    package.push_entry("demo/First.class", class_bytes.clone());
    package.push_entry("META-INF/MANIFEST.MF", b"Manifest-Version: 1.0\r\n".to_vec());
    package.push_entry("demo/resource.txt", b"hello".to_vec());

    let collected: Vec<(String, Vec<u8>)> = package.iter_classes().map(|(path, bytes)| (path.to_string(), bytes.to_vec())).collect();
    assert_eq!(collected.len(), 1);
    assert_eq!(collected[0].0, "demo/First.class");
    assert_eq!(collected[0].1, class_bytes);
}

/// 验证 `iter_classes` 在 `JAR` 编解码往返后仍能正确遍历。
#[test]
fn iter_classes_编解码往返() {
    let mut package = JvmJarPackage::new("roundtrip.jar");
    package.push_class(&JvmClassFile::new("demo/RoundTrip")).expect("写入 class 失败");
    package.push_entry("data.txt", b"payload".to_vec());

    let bytes = package.to_bytes().expect("编码 JAR 失败");
    let decoded = JvmJarPackage::from_bytes("roundtrip.jar", &bytes).expect("解码 JAR 失败");

    let class_entries: Vec<_> = decoded.iter_classes().collect();
    assert_eq!(class_entries.len(), 1);
    assert!(class_entries[0].0.ends_with(".class"));
    let class_file = JvmClassFile::from_bytes(class_entries[0].1).expect("解析 class 失败");
    assert_eq!(class_file.internal_name, "demo/RoundTrip");
}

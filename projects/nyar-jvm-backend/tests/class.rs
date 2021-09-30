use nyar_jvm_backend::{
    encode_instructions, ConstantPoolBuilder, JvmClassFile, JvmCodeBody, JvmInstruction, JvmMethodDescriptor, JvmMethodRef, JvmMethodSignature,
    JvmTypeDescriptor,
};

const ACC_PUBLIC: u16 = 0x0001;
const ACC_STATIC: u16 = 0x0008;

#[test]
fn encodes_and_decodes_minimal_class() {
    let mut class_file = JvmClassFile::new("demo/Main");
    class_file.push_method(
        "main",
        JvmMethodDescriptor::new(
            vec![JvmTypeDescriptor::array(JvmTypeDescriptor::Object("java/lang/String".to_string()))],
            JvmTypeDescriptor::Void,
        ),
    );

    let bytes = class_file.to_bytes().unwrap();
    let decoded = JvmClassFile::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.internal_name, "demo/Main");
    assert_eq!(decoded.super_name, "java/lang/Object");
    assert_eq!(decoded.methods.len(), 1);
    assert_eq!(decoded.methods[0].name, "main");
    assert_eq!(
        decoded.methods[0].descriptor,
        JvmMethodDescriptor::new(
            vec![JvmTypeDescriptor::array(JvmTypeDescriptor::Object("java/lang/String".to_string()))],
            JvmTypeDescriptor::Void,
        )
    );
}

#[test]
fn preserves_instance_method_flags() {
    let class_file = JvmClassFile {
        internal_name: "demo/Point".to_string(),
        methods: vec![JvmMethodSignature {
            name: "x".to_string(),
            descriptor: JvmMethodDescriptor::new(Vec::new(), JvmTypeDescriptor::Int),
            access_flags: ACC_PUBLIC,
            code: None,
        }],
        ..JvmClassFile::default()
    };

    let bytes = class_file.to_bytes().unwrap();
    let decoded = JvmClassFile::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.methods[0].access_flags, ACC_PUBLIC);
}

#[test]
fn decodes_generated_static_method() {
    let mut class_file = JvmClassFile::new("demo/Tools");
    class_file.methods.push(JvmMethodSignature {
        name: "answer".to_string(),
        descriptor: JvmMethodDescriptor::new(Vec::new(), JvmTypeDescriptor::Int),
        access_flags: ACC_PUBLIC | ACC_STATIC,
        code: None,
    });

    let decoded = JvmClassFile::from_bytes(&class_file.to_bytes().unwrap()).unwrap();
    assert_eq!(decoded.methods[0].name, "answer");
}

#[test]
fn optimizes_static_self_tail_recursion_into_loop() {
    let mut class_file = JvmClassFile::new("demo/Factorial");
    class_file.methods.push(JvmMethodSignature {
        name: "countdown".to_string(),
        descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Int], JvmTypeDescriptor::Int),
        access_flags: ACC_PUBLIC | ACC_STATIC,
        code: Some(JvmCodeBody {
            max_stack: 2,
            max_locals: 1,
            instructions: vec![
                JvmInstruction::ILoad(0),
                JvmInstruction::InvokeStatic(JvmMethodRef {
                    owner: "demo/Factorial".to_string(),
                    name: "countdown".to_string(),
                    descriptor: JvmMethodDescriptor::new(vec![JvmTypeDescriptor::Int], JvmTypeDescriptor::Int),
                }),
                JvmInstruction::IReturn,
            ],
        }),
    });

    let optimized = class_file.optimize_static_self_tail_recursion().unwrap();
    assert_eq!(optimized, 1);
    let method = &class_file.methods[0];
    let code = method.code.as_ref().unwrap();
    assert_eq!(
        code.instructions,
        vec![
            JvmInstruction::Label("__tailcall_entry".to_string()),
            JvmInstruction::ILoad(0),
            JvmInstruction::IStore(0),
            JvmInstruction::Goto("__tailcall_entry".to_string()),
        ]
    );
}

/// 验证分支目标编码：JVM 规范要求分支偏移相对于 opcode 起始地址。
#[test]
fn encodes_branch_target_relative_to_opcode_start() {
    let instructions = vec![
        JvmInstruction::IConst(0),
        JvmInstruction::Goto("end".to_string()),
        JvmInstruction::IConst(1),
        JvmInstruction::Label("end".to_string()),
        JvmInstruction::IReturn,
    ];
    let mut pool = ConstantPoolBuilder::new();
    let code = encode_instructions(&instructions, &mut pool).unwrap();

    // iconst_0 (1 byte) + goto (3 bytes) + iconst_1 (1 byte) = offset 5 for "end" label
    // goto opcode at offset 1, target at offset 5
    // 正确的相对偏移 = 5 - 1 = 4
    // 错误的公式 (offset+3) 会给出 5 - (1+3) = 1
    let goto_offset = 1usize;
    assert_eq!(code[goto_offset], 0xA7, "goto opcode 应为 0xA7");
    let branch_offset = i16::from_be_bytes([code[goto_offset + 1], code[goto_offset + 2]]);
    assert_eq!(branch_offset, 4, "分支偏移应相对于 opcode 起始地址：target(5) - opcode_offset(1) = 4，实际为 {branch_offset}");
}

use nyar::backends::clr::ClrImageKind;
use std_data::{
    binary::pe::{PeWriter, PeWriterOptions},
    text::msil::*,
};

#[test]
fn legion_like_entrypoint_repro() {
    let mut global_methods = vec![MsilMethodBody {
        method: MsilMethodRef {
            owner: None,
            name: "Main".to_string(),
            signature: MsilMethodSignature::new(MsilType::Int32 { signed: true }, vec![MsilType::sz_array(MsilType::String)]),
        },
        locals: Vec::new(),
        instructions: vec![
            MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None },
            MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
        ],
        max_stack: 8,
        is_entry_point: true,
        is_async: false,
    }];
    for index in 0..439 {
        global_methods.push(MsilMethodBody {
            method: MsilMethodRef {
                owner: None,
                name: format!("stub_{index}"),
                signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
            },
            locals: Vec::new(),
            instructions: vec![MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None }],
            max_stack: 0,
            is_entry_point: false,
            is_async: false,
        });
    }

    let mut types = Vec::new();
    for index in 0..120 {
        types.push(MsilTypeDef {
            full_name: format!("Value{index}"),
            namespace: String::new(),
            fields: vec![MsilField { name: "x".to_string(), ty: MsilType::Int32 { signed: true }, is_static: false }],
            methods: Vec::new(),
            is_value_type: true,
        });
    }
    for index in 0..34 {
        types.push(MsilTypeDef {
            full_name: format!("Ref{index}"),
            namespace: String::new(),
            fields: Vec::new(),
            methods: vec![MsilMethodBody {
                method: MsilMethodRef {
                    owner: Some(format!("Ref{index}")),
                    name: ".ctor".to_string(),
                    signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
                },
                locals: Vec::new(),
                instructions: vec![
                    MsilInstruction { label: None, opcode: MsilOpcode::Ldarg0, operand: None },
                    MsilInstruction {
                        label: None,
                        opcode: MsilOpcode::Call,
                        operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                            owner: Some("[mscorlib]System.Object".to_string()),
                            name: ".ctor".to_string(),
                            signature: MsilMethodSignature::new(MsilType::Void, Vec::new()),
                        })),
                    },
                    MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
                ],
                max_stack: 8,
                is_entry_point: false,
                is_async: false,
            }],
            is_value_type: false,
        });
    }
    types.push(MsilTypeDef {
        full_name: "CliArgvState".to_string(),
        namespace: String::new(),
        fields: vec![
            MsilField { name: "__args".to_string(), ty: MsilType::sz_array(MsilType::String), is_static: true },
            MsilField { name: "__project".to_string(), ty: MsilType::String, is_static: true },
        ],
        methods: Vec::new(),
        is_value_type: false,
    });

    let module = MsilModule { assembly: MsilAssembly { name: "ReproAsm".to_string(), externs: Vec::new() }, types, global_methods };
    let options =
        PeWriterOptions { assembly_name: "ReproAsm".to_string(), module_name: "ReproAsm".to_string(), image_kind: ClrImageKind::Executable };
    let bytes = PeWriter::new(options).write_module(&module).expect("write");
    std::fs::write(env!("CARGO_MANIFEST_DIR").to_owned() + "/../../target/test-legion-repro.exe", &bytes).expect("file");
}

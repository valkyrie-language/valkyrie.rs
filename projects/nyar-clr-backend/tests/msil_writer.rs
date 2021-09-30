use nyar_clr_backend::{
    MsilAssembly, MsilInstruction, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule, MsilOpcode, MsilTextWriter, MsilType,
};

#[test]
fn writes_simple_module() {
    let module = MsilModule {
        assembly: MsilAssembly { name: "hello".to_string(), externs: vec!["mscorlib".to_string()] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "main".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![
                MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None },
                MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
            ],
            max_stack: 1,
            is_entry_point: true,
        }],
    };

    let text = MsilTextWriter::new().write_module(&module);
    assert!(text.contains(".assembly extern mscorlib {}"));
    assert!(text.contains(".assembly hello {}"));
    assert!(text.contains(".method public hidebysig static int32 main()"));
    assert!(text.contains(".entrypoint"));
    assert!(text.contains("ldc.i4.0"));
    assert!(text.contains("ret"));
}

#[test]
fn uses_custom_indent_text() {
    let module = MsilModule {
        assembly: MsilAssembly { name: "hello".to_string(), externs: vec![] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "main".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None }],
            max_stack: 1,
            is_entry_point: true,
        }],
    };

    let text = MsilTextWriter::new().with_indent_text("  ").write_module(&module);
    assert!(text.contains("\n  .entrypoint\n"));
    assert!(text.contains("\n    ret\n"));
}

#[test]
fn writes_module_file_name_and_normalizes_legacy_signature() {
    let module = MsilModule {
        assembly: MsilAssembly { name: "hello".to_string(), externs: vec![] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "main".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None }],
            max_stack: 1,
            is_entry_point: true,
        }],
    };

    let text = MsilTextWriter::new().write_module_with_file_name(&module, "hello.exe");
    assert!(text.contains(".module hello.exe"));
    assert!(text.contains(".method public hidebysig static int32 main() cil managed"));
}

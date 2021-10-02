use std::fs;

use nyar::backends::clr::ClrImageKind;
use nyar_backend_clr::{
    MsilAssembly, MsilInstruction, MsilInstructionOperand, MsilMethodBody, MsilMethodRef, MsilMethodSignature, MsilModule, MsilOpcode,
    MsilType, MsilTypeDef, PeWriter, PeWriterError, PeWriterOptions,
};
use tempfile::TempDir;

#[test]
fn generated_dll_contains_clr_metadata_root() {
    let module = build_library_module();

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.dll".to_string(),
        image_kind: ClrImageKind::DynamicLibrary,
    })
    .write_module(&module)
    .unwrap();

    assert!(bytes.windows(4).any(|window| window == b"BSJB"));
}

#[test]
fn generated_exe_contains_native_startup_stub() {
    let module = build_executable_module();

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.exe".to_string(),
        image_kind: ClrImageKind::Executable,
    })
    .write_module(&module)
    .unwrap();

    let pe_header_offset = read_u32(&bytes, 0x3C) as usize;
    let optional_header_offset = pe_header_offset + 4 + 20;
    let entry_rva = read_u32(&bytes, optional_header_offset + 16);
    let entry_file_offset = 0x200 + (entry_rva - 0x2000) as usize;
    assert_eq!(&bytes[entry_file_offset..entry_file_offset + 2], &[0xFF, 0x25]);
}

#[test]
#[ignore = "用于外部命令手动校验 CLR 运行时是否接受生成的 DLL"]
fn materialize_sample_dll_for_runtime_validation() {
    let output_dir = create_temp_dir("clr-pe-runtime-dll");
    let artifact_path = output_dir.path().join("sample.dll");
    let module = build_library_module();

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.dll".to_string(),
        image_kind: ClrImageKind::DynamicLibrary,
    })
    .write_module(&module)
    .unwrap();

    fs::write(&artifact_path, bytes).unwrap();
    println!("{}", artifact_path.display());
}

#[test]
#[ignore = "用于外部命令手动校验 CLR 运行时是否接受生成的 EXE"]
fn materialize_sample_exe_for_runtime_validation() {
    let output_dir = create_temp_dir("clr-pe-runtime-exe");
    let artifact_path = output_dir.path().join("sample.exe");
    let module = build_executable_module();

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.exe".to_string(),
        image_kind: ClrImageKind::Executable,
    })
    .write_module(&module)
    .unwrap();

    fs::write(&artifact_path, bytes).unwrap();
    println!("{}", artifact_path.display());
}

#[test]
#[ignore = "用于外部命令手动校验 `Console.WriteLine` 外部调用链"]
fn materialize_console_exe_for_runtime_validation() {
    let output_dir = create_temp_dir("clr-pe-runtime-console");
    let artifact_path = output_dir.path().join("console.exe");
    let module = build_console_executable_module();

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "console".to_string(),
        module_name: "console.exe".to_string(),
        image_kind: ClrImageKind::Executable,
    })
    .write_module(&module)
    .unwrap();

    fs::write(&artifact_path, bytes).unwrap();
    println!("{}", artifact_path.display());
}

#[test]
fn local_type_methods_require_qualified_names_for_token_lookup() {
    let module = build_ambiguous_local_type_module(MsilMethodRef {
        owner: None,
        name: "Run".to_string(),
        signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()),
    });

    let error = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.dll".to_string(),
        image_kind: ClrImageKind::DynamicLibrary,
    })
    .write_module(&module)
    .unwrap_err();

    assert!(matches!(error, PeWriterError::MissingLocalMethodToken(name) if name == "Run"));
}

#[test]
fn local_type_methods_resolve_by_qualified_owner_without_short_name_fallback() {
    let module = build_ambiguous_local_type_module(MsilMethodRef {
        owner: Some("demo.Alpha".to_string()),
        name: "Run".to_string(),
        signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()),
    });

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.dll".to_string(),
        image_kind: ClrImageKind::DynamicLibrary,
    })
    .write_module(&module)
    .unwrap();

    assert!(!bytes.is_empty());
}

fn build_library_module() -> MsilModule {
    MsilModule {
        assembly: MsilAssembly { name: "sample".to_string(), externs: vec!["mscorlib".to_string()] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "Helper".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![
                MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_1, operand: None },
                MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
            ],
            max_stack: 1,
            is_entry_point: false,
        }],
    }
}

fn build_executable_module() -> MsilModule {
    MsilModule {
        assembly: MsilAssembly { name: "sample".to_string(), externs: vec!["mscorlib".to_string()] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "Main".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![
                MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None },
                MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
            ],
            max_stack: 1,
            is_entry_point: true,
        }],
    }
}

fn build_console_executable_module() -> MsilModule {
    MsilModule {
        assembly: MsilAssembly { name: "console".to_string(), externs: vec!["mscorlib".to_string(), "System.Console".to_string()] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "Main".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![
                MsilInstruction {
                    label: None,
                    opcode: MsilOpcode::Ldstr,
                    operand: Some(MsilInstructionOperand::StringLiteral("hello from clr-backend".to_string())),
                },
                MsilInstruction {
                    label: None,
                    opcode: MsilOpcode::Call,
                    operand: Some(MsilInstructionOperand::Method(MsilMethodRef {
                        owner: Some("[System.Console]System.Console".to_string()),
                        name: "WriteLine".to_string(),
                        signature: MsilMethodSignature::new(MsilType::Void, vec![MsilType::String]),
                    })),
                },
                MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None },
                MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
            ],
            max_stack: 1,
            is_entry_point: true,
        }],
    }
}

fn build_ambiguous_local_type_module(call_target: MsilMethodRef) -> MsilModule {
    MsilModule {
        assembly: MsilAssembly { name: "sample".to_string(), externs: vec!["mscorlib".to_string()] },
        types: vec![
            MsilTypeDef {
                full_name: "Alpha".to_string(),
                namespace: "demo".to_string(),
                fields: vec![],
                methods: vec![MsilMethodBody {
                    method: MsilMethodRef {
                        owner: Some("demo.Alpha".to_string()),
                        name: "Run".to_string(),
                        signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()),
                    },
                    locals: vec![],
                    instructions: vec![
                        MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_1, operand: None },
                        MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
                    ],
                    max_stack: 1,
                    is_entry_point: false,
                }],
                is_value_type: false,
            },
            MsilTypeDef {
                full_name: "Beta".to_string(),
                namespace: "demo".to_string(),
                fields: vec![],
                methods: vec![MsilMethodBody {
                    method: MsilMethodRef {
                        owner: Some("demo.Beta".to_string()),
                        name: "Run".to_string(),
                        signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()),
                    },
                    locals: vec![],
                    instructions: vec![
                        MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_2, operand: None },
                        MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
                    ],
                    max_stack: 1,
                    is_entry_point: false,
                }],
                is_value_type: false,
            },
        ],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef {
                owner: Some("sample.dll".to_string()),
                name: "Caller".to_string(),
                signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()),
            },
            locals: vec![],
            instructions: vec![
                MsilInstruction { label: None, opcode: MsilOpcode::Call, operand: Some(MsilInstructionOperand::Method(call_target)) },
                MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
            ],
            max_stack: 1,
            is_entry_point: false,
        }],
    }
}

fn create_temp_dir(prefix: &str) -> TempDir {
    tempfile::Builder::new().prefix(prefix).tempdir().unwrap()
}

fn read_u32(bytes: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes(bytes[offset..offset + 4].try_into().unwrap())
}

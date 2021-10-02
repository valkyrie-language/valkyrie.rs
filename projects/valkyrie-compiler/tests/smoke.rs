use valkyrie_compiler::{LirOperationKind, MirInstructionKind, MirTerminator, ValkyrieCompiler};
use valkyrie_types::{hir::ValkyrieType, Identifier};

#[test]
fn compile_return_zero_lowers_to_single_return_block() {
    let source = r#"namespace test;
micro main() -> i64 {
    return 0;
}"#;
    let compiler = ValkyrieCompiler::default();
    let mir = compiler.compile_source_to_mir(source).expect("parse ok");
    assert_eq!(mir.functions.len(), 1);
    let func = &mir.functions[0];
    assert_eq!(func.symbol, "test::main");
    assert_eq!(func.blocks.len(), 1);
    let block = &func.blocks[0];
    assert_eq!(block.label, "entry");
    assert!(matches!(block.terminator, MirTerminator::Return { .. }));
}

#[test]
fn array_literal_lowers_to_builtin_array_literal_without_array_call() {
    let source = r#"micro main(): i32 {
    let mut values: [i32] = [10, 20, 30]
    return values[1]
}
"#;
    let compiler = ValkyrieCompiler::default();

    let mir = compiler.compile_source_to_mir(source).expect("mir ok");
    let mir_operations = &mir.functions[0].blocks[0].instructions;
    assert!(mir_operations.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::ArrayLiteral { .. })));
    assert!(mir_operations.iter().any(|instruction| {
        matches!(&instruction.kind, MirInstructionKind::ArrayLiteral { element_type: ValkyrieType::Integer32 { signed: true }, .. })
    }));
    assert!(!mir_operations.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: valkyrie_compiler::MirOperand::Symbol(path), .. } if path.to_string() == "array"
        )
    }));

    let lir = compiler.compile_source_to_lir(source).expect("lir ok");
    let lir_operations = &lir.functions[0].blocks[0].operations;
    assert!(lir_operations.iter().any(|operation| matches!(operation.kind, LirOperationKind::ArrayLiteral { .. })));
    assert!(lir_operations.iter().any(|operation| {
        matches!(&operation.kind, LirOperationKind::ArrayLiteral { element_type: ValkyrieType::Integer32 { signed: true }, .. })
    }));
    assert!(!lir_operations.iter().any(|operation| {
        matches!(
            &operation.kind,
            LirOperationKind::Call { callee: valkyrie_compiler::LirOperand::Symbol(path), .. } if path.to_string() == "array"
        )
    }));
}

#[test]
fn main_exit_code_shapes_compile() {
    let compiler = ValkyrieCompiler::default();

    let inferred = compiler.compile_source(
        r#"[main]
micro main() {
    return 0
}
"#,
    );
    assert!(inferred.is_err());

    let explicit = compiler.compile_source(
        r#"[main]
micro main(): ExitCode {
    return 0
}
"#,
    );
    assert!(explicit.is_err());

    let wrapped = compiler.compile_source(
        r#"[main]
micro main(): ExitCode {
    return ExitCode(0)
}
"#,
    );
    assert!(wrapped.is_ok());

    let wrapped_mir = compiler
        .compile_source_to_mir(
            r#"[main]
micro main(): ExitCode {
    return ExitCode(0)
}
"#,
        )
        .unwrap();
    assert_eq!(wrapped_mir.functions[0].return_type, ValkyrieType::Named(Identifier::new("ExitCode")));

    let wrapped_lir = compiler
        .compile_source_to_lir(
            r#"[main]
micro main(): ExitCode {
    return ExitCode(0)
}
"#,
        )
        .unwrap();
    assert_eq!(wrapped_lir.functions[0].return_type, valkyrie_types::hir::ValkyrieType::Integer32 { signed: true });
}

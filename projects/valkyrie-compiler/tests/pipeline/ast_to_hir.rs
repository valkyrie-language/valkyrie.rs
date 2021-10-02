use valkyrie_compiler::{
    AstToHir, CaptureAnalyzer, LirDispatchKind, LirEffectKind, LirOperationKind, LirTargetLane, LirTerminator, MirEffectKind,
    MirInstructionKind, MirValueOrigin, ValkyrieCompiler,
};
use valkyrie_types::{
    hir::{CaptureMode, HirBlock, HirExpr, HirExprKind, HirLiteral, HirPattern, HirStatementKind, ValkyrieType},
    SourceID,
};

#[path = "ast_to_hir/frontend_control.rs"]
mod frontend_control;
#[path = "ast_to_hir/language_surface.rs"]
mod language_surface;
#[path = "ast_to_hir/lowering_runtime.rs"]
mod lowering_runtime;

fn block_expr_or_single_statement(block: &HirBlock) -> Option<&HirExpr> {
    if let Some(expr) = block.expr.as_deref() {
        return Some(expr);
    }

    if block.statements.len() == 1 {
        if let HirStatementKind::Expr(expr) = &block.statements[0].kind {
            return Some(expr);
        }
    }

    None
}

#[test]
fn test_converter_creation() {
    let converter = AstToHir::new(SourceID::default());
    assert_eq!(converter.source_id, SourceID::default());
}

#[test]
fn test_capture_analyzer_new() {
    let analyzer = CaptureAnalyzer::new();
    let captures = analyzer.into_captures();
    assert!(captures.is_empty());
}

#[test]
fn test_capture_analyzer_add_var() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.add_var("x", ValkyrieType::Integer64 { signed: true }, false);
    analyzer.access_var("x", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].identifier.name.as_str(), "x");
    assert_eq!(captures[0].mode, CaptureMode::ByValue);
}

#[test]
fn test_capture_analyzer_mutable_var() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.add_var("y", ValkyrieType::Integer64 { signed: true }, true);
    analyzer.access_var("y", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
    assert!(captures[0].is_mutable);
}

#[test]
fn test_logical_and_lowers_to_short_circuit_if() {
    let source = r#"
micro main() -> bool {
    true && false
}
"#;

    let module = ValkyrieCompiler::new(SourceID::default()).compile_source(source).expect("compile should succeed");
    let function = &module.functions[0];
    let expr = function.body.expr.as_ref().expect("expected tail expression");

    assert!(matches!(
        &expr.kind,
        HirExprKind::If { condition, then_branch, else_branch }
            if matches!(condition.kind, HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true)))
                && matches!(then_branch.expr.as_deref(), Some(HirExpr { kind: HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(false)), .. }))
                && matches!(else_branch.as_deref().and_then(|branch| branch.expr.as_deref()),
                    Some(HirExpr { kind: HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(false)), .. }))
    ));
}

#[test]
fn test_capture_analyzer_no_capture_unknown() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.access_var("unknown", false);
    let captures = analyzer.into_captures();
    assert!(captures.is_empty());
}

#[test]
fn test_capture_analyzer_no_duplicate() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.add_var("x", ValkyrieType::Integer64 { signed: true }, false);
    analyzer.access_var("x", false);
    analyzer.access_var("x", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
}

#[test]
fn test_capture_analyzer_by_reference() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.add_var("obj", ValkyrieType::Named(valkyrie_types::Identifier::new("MyObject")), false);
    analyzer.access_var("obj", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].mode, CaptureMode::ByReference);
}

#[test]
fn test_lower_root_to_hir_module() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 17 });
    let module = compiler
        .compile_source(
            r#"namespace demo;
using std::console;
micro main(args: [utf8]) -> i64 {
    let code: i64 = 0;
    return code;
}
"#,
        )
        .unwrap();

    assert_eq!(module.name.to_string(), "demo");
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].params.len(), 1);
    assert!(matches!(module.functions[0].body.statements[0].kind, HirStatementKind::Let { .. }));
    assert!(matches!(module.functions[0].body.statements[1].kind, HirStatementKind::Expr(_)));
}

#[test]
fn test_lower_return_value_expression() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 19 });
    let module = compiler
        .compile_source(
            r#"micro main() -> i64 {
    return 42;
}
"#,
        )
        .unwrap();
    let HirStatementKind::Expr(expression) = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Return(Some(value)) => {
            assert!(matches!(value.kind, HirExprKind::Literal(_)));
        }
        _ => panic!("expected return with value"),
    }
}

#[test]
fn test_compile_source_to_mir_and_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 23 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main(input: i64) -> i64 {
    return input;
}
"#,
        )
        .unwrap();
    assert_eq!(mir.functions.len(), 1);
    assert!(mir.functions[0].values.iter().any(|value| matches!(value.origin, valkyrie_compiler::MirValueOrigin::Parameter { index: 0, .. })));

    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    std::console::write_line("hi");
}
"#,
        )
        .unwrap();
    assert_eq!(lir.functions.len(), 1);
    assert_eq!(lir.lane, LirTargetLane::Clr);
    assert_eq!(lir.functions[0].blocks[0].operations.len(), 2);
    assert!(matches!(
        lir.functions[0].blocks[0].operations.last().map(|operation| &operation.kind),
        Some(LirOperationKind::Call { dispatch: LirDispatchKind::Static, .. })
    ));
    assert!(matches!(lir.functions[0].blocks[0].terminator, LirTerminator::Return { .. }));
}
#[test]
fn lowers_root_into_hir_module_from_ast_parser() {
    let source = r#"namespace demo;
using std::console;
micro main() -> i64 {
    return 0;
}
"#;
    let root = valkyrie_parser::AstParser::parse_root(source).unwrap();
    let module = AstToHir::new(SourceID { version_id: 7 }).lower_root(&root).unwrap();
    assert_eq!(module.name.to_string(), "demo");
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].span.source, SourceID { version_id: 7 });
    assert!(matches!(module.functions[0].body.statements[0].kind, valkyrie_types::hir::HirStatementKind::Expr(_)));
}

#[test]
fn rejects_legacy_string_type_input() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 701 });
    let err = compiler
        .compile_source(
            r#"micro main(message: string) -> void {
    return;
}
"#,
        )
        .unwrap_err();
    assert!(err.to_string().contains("string"));
    assert!(err.to_string().contains("utf8"));
    assert!(err.to_string().contains("utf16"));
}

#[test]
fn does_not_treat_newarr_builtin_name_as_array_magic() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 702 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    __newarr_i32(4);
}
"#,
        )
        .unwrap();
    let statement = &module.functions[0].body.statements[0];
    let HirStatementKind::Expr(expression) = &statement.kind
    else {
        panic!("expected expression statement");
    };
    assert!(matches!(expression.kind, HirExprKind::Call { .. }));
    assert!(!matches!(expression.kind, HirExprKind::ArrayNew { .. }));
}

#[test]
fn lowers_array_literal_without_falling_back_to_call() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 703 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    let values: [i32] = [1, 2, 3];
}
"#,
        )
        .unwrap();
    let statement = &module.functions[0].body.statements[0];
    let HirStatementKind::Let { initializer: Some(expression), .. } = &statement.kind
    else {
        panic!("expected let statement");
    };
    assert!(matches!(expression.kind, HirExprKind::ArrayLiteral { .. }));
    assert!(!matches!(expression.kind, HirExprKind::Call { .. }));
}

#[test]
fn lowers_array_literal_to_builtin_array_literal_in_mir_and_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 704 });
    let source = r#"micro main(): i32 {
    let mut values: [i32] = [10, 20, 30]
    return values[1]
}
"#;

    let mir = compiler.compile_source_to_mir(source).unwrap();
    let mir_operations = &mir.functions[0].blocks[0].instructions;
    assert!(mir_operations.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::ArrayLiteral { .. })));
    assert!(!mir_operations.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: valkyrie_compiler::MirOperand::Symbol(path), .. } if path.to_string() == "array"
        )
    }));

    let lir = compiler.compile_source_to_lir(source).unwrap();
    let lir_operations = &lir.functions[0].blocks[0].operations;
    assert!(lir_operations.iter().any(|operation| matches!(operation.kind, LirOperationKind::ArrayLiteral { .. })));
    assert!(!lir_operations.iter().any(|operation| {
        matches!(
            &operation.kind,
            LirOperationKind::Call { callee: valkyrie_compiler::LirOperand::Symbol(path), .. } if path.to_string() == "array"
        )
    }));
}

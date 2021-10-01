use valkyrie_compiler::{
    AstToHir, CaptureAnalyzer, LirDispatchKind, LirEffectKind, LirOperationKind, LirTargetLane, LirTerminator, MirEffectKind,
    MirInstructionKind, MirValueOrigin, ValkyrieCompiler,
};
use valkyrie_types::{
    hir::{CaptureMode, HirExpr, HirExprKind, HirLiteral, HirPattern, HirStatementKind, ValkyrieType},
    SourceID,
};

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

#[test]
fn compiler_facade_parses_and_lowers_return_statement() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 3 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    return;
}
"#,
        )
        .unwrap();
    let statement = &module.functions[0].body.statements[0];
    let HirStatementKind::Expr(expression) = &statement.kind
    else {
        panic!("expected expression statement");
    };
    assert!(matches!(expression.kind, HirExprKind::Return(_)));
}

#[test]
fn compiler_facade_lowers_yield_statement_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 79 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    yield 1;
}
"#,
        )
        .unwrap();
    let statement = &module.functions[0].body.statements[0];
    let HirStatementKind::Expr(expression) = &statement.kind
    else {
        panic!("expected expression statement");
    };
    assert!(matches!(
        expression.kind,
        HirExprKind::Yield(Some(ref value)) if matches!(value.kind, HirExprKind::Literal(_))
    ));
}

#[test]
fn compiler_facade_lowers_yield_from_statement_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 80 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    yield from values;
}
"#,
        )
        .unwrap();
    let statement = &module.functions[0].body.statements[0];
    let HirStatementKind::Expr(expression) = &statement.kind
    else {
        panic!("expected expression statement");
    };
    assert!(matches!(
        expression.kind,
        HirExprKind::YieldFrom(ref value)
            if matches!(value.kind, HirExprKind::Variable(ref identifier) if identifier.name.as_str() == "values")
    ));
}

#[test]
fn compiler_facade_lowers_raise_statement_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 94 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    raise problem
}
"#,
        )
        .unwrap();
    let expression = module.functions[0].body.expr.as_ref().expect("expected final expression");
    assert!(matches!(
        expression.kind,
        HirExprKind::Raise(ref value)
            if matches!(value.kind, HirExprKind::Variable(ref identifier) if identifier.name.as_str() == "problem")
    ));
}

#[test]
fn compiler_facade_lowers_resume_statement_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 95 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    catch task {
        case Yielded(next_value):
            resume next_value
    }
}"#,
        )
        .unwrap();
    let expression = module.functions[0].body.expr.as_ref().expect("expected final expression");
    let HirExprKind::Catch { arms, .. } = &expression.kind
    else {
        panic!("expected catch expression");
    };
    assert!(matches!(
        arms[0].body.kind,
        HirExprKind::Block(ref body)
            if matches!(body.expr.as_deref(), Some(HirExpr { kind: HirExprKind::Resume(ref value), .. })
                if matches!(value.kind, HirExprKind::Variable(ref identifier) if identifier.name.as_str() == "next_value"))
    ));
}

#[test]
fn compiler_facade_parses_fallthrough_as_expression_statement_and_rejects_in_validation() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 97 });
    let error = compiler
        .compile_source(
            r#"micro main() {
    fallthrough
}
"#,
        )
        .unwrap_err();
    assert!(error.to_string().contains("fallthrough"));
}

#[test]
fn compiler_facade_rejects_fallthrough_in_value_match_expression() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 971 });
    let error = compiler
        .compile_source_to_mir(
            r#"micro main() -> bool {
    return match value {
        case Flag():
            fallthrough
        else:
            true
    };
}
"#,
        )
        .unwrap_err();
    assert!(error.to_string().contains("fallthrough"));
}

#[test]
fn compiler_facade_lowers_statement_match_fallthrough_into_jump_chain() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 972 });
    let hir = compiler
        .compile_source(
            r#"micro main() {
    match value {
        case Flag():
            fallthrough
        else:
            return
    };
    return
}
"#,
        )
        .unwrap();
    let HirStatementKind::Expr(case_expr) = &hir.functions[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };
    assert!(matches!(case_expr.kind, HirExprKind::Case { .. }));

    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    match value {
        case Flag():
            fallthrough
        else:
            return
    };
    return
}
"#,
        )
        .unwrap();
    let first_fallthrough_block = mir.functions[0]
        .blocks
        .iter()
        .find(|block| matches!(block.terminator, valkyrie_compiler::MirTerminator::Jump { .. }) && block.label.starts_with("case_arm_0"))
        .expect("expected first arm block");
    let target = match first_fallthrough_block.terminator {
        valkyrie_compiler::MirTerminator::Jump { target, .. } => target,
        _ => unreachable!(),
    };
    let target_block = mir.functions[0].blocks.iter().find(|block| block.id == target).expect("expected jump target block");
    assert!(target_block.label.starts_with("case_arm_1"));

    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    match value {
        case Flag():
            fallthrough
        else:
            return
    };
    return
}
"#,
        )
        .unwrap();
    let first_lir_fallthrough_block = lir.functions[0]
        .blocks
        .iter()
        .find(|block| matches!(block.terminator, LirTerminator::Jump { .. }) && block.label.starts_with("case_arm_0"))
        .expect("expected first lir arm block");
    let target = match first_lir_fallthrough_block.terminator {
        LirTerminator::Jump { target, .. } => target,
        _ => unreachable!(),
    };
    let target_block = lir.functions[0].blocks.iter().find(|block| block.id == target).expect("expected lir jump target block");
    assert!(target_block.label.starts_with("case_arm_1"));
}

#[test]
fn compiler_facade_lowers_literal_variable_and_or_match_patterns_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 973 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: i64) -> bool {
    return match value {
        case 1 | 2:
            true
        case n if n > 0:
            false
        case _:
            false
    };
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(statement) = &hir.functions[0].body.statements[0].kind
    else {
        panic!("expected return statement");
    };
    let HirExprKind::Return(Some(expression)) = &statement.kind
    else {
        panic!("expected return expression");
    };
    let HirExprKind::Match { arms, .. } = &expression.kind
    else {
        panic!("expected match expression");
    };

    assert!(matches!(
        &arms[0].pattern,
        valkyrie_types::hir::HirPattern::Or(patterns)
            if matches!(patterns.as_slice(),
                [
                    valkyrie_types::hir::HirPattern::Literal(HirLiteral::Integer64(1)),
                    valkyrie_types::hir::HirPattern::Literal(HirLiteral::Integer64(2))
                ])
    ));
    assert!(matches!(
        &arms[1].pattern,
        valkyrie_types::hir::HirPattern::Variable(identifier) if identifier.name.as_str() == "n"
    ));
    assert!(arms[1].guard.is_some());
    assert!(matches!(&arms[2].pattern, valkyrie_types::hir::HirPattern::Wildcard));
}

#[test]
fn compiler_facade_lowers_catch_expression_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 96 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    catch task {
        case Yielded(value):
            resume value
        else:
            raise fallback
    }
}
"#,
        )
        .unwrap();
    let expression = module.functions[0].body.expr.as_ref().expect("expected final expression");
    let HirExprKind::Catch { expr, arms } = &expression.kind
    else {
        panic!("expected catch expression");
    };
    assert!(matches!(expr.kind, HirExprKind::Variable(ref identifier) if identifier.name.as_str() == "task"));
    assert_eq!(arms.len(), 2);
    assert!(matches!(
        arms[0].pattern,
        HirPattern::Extractor(valkyrie_types::hir::HirExtractorPattern::Constructor { ref name, ref fields, .. })
            if name.to_string() == "Yielded"
                && fields.len() == 1
                && matches!(&fields[0], HirPattern::Variable(identifier) if identifier.name.as_str() == "value")
    ));
    assert!(matches!(
        arms[0].body.kind,
        HirExprKind::Block(ref body)
            if matches!(body.expr.as_deref(), Some(HirExpr { kind: HirExprKind::Resume(_), .. }))
    ));
    assert!(matches!(arms[1].pattern, HirPattern::Else));
    assert!(matches!(
        arms[1].body.kind,
        HirExprKind::Block(ref body)
            if matches!(body.expr.as_deref(), Some(HirExpr { kind: HirExprKind::Raise(_), .. }))
    ));
}

#[test]
fn compiler_facade_lowers_named_object_pattern_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 112 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    catch task {
        case Yielded { next_value }:
            resume next_value
        else:
            raise fallback
    }
}"#,
        )
        .unwrap();
    let expression = module.functions[0].body.expr.as_ref().expect("expected final expression");
    let HirExprKind::Catch { arms, .. } = &expression.kind
    else {
        panic!("expected catch expression");
    };
    assert!(matches!(
        &arms[0].pattern,
        HirPattern::Object { name: Some(name), fields, .. }
            if name.to_string() == "Yielded"
                && fields.len() == 1
                && matches!(&fields[0], (field, HirPattern::Variable(identifier))
                    if field.as_str() == "next_value" && identifier.name.as_str() == "next_value")
    ));
}

#[test]
fn compiler_facade_lowers_anonymous_object_pattern_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 113 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    catch task {
        case { foo, bar }:
            resume foo
        else:
            raise fallback
    }
}"#,
        )
        .unwrap();
    let expression = module.functions[0].body.expr.as_ref().expect("expected final expression");
    let HirExprKind::Catch { arms, .. } = &expression.kind
    else {
        panic!("expected catch expression");
    };
    assert!(matches!(
        &arms[0].pattern,
        HirPattern::Object { name: None, fields, .. }
            if fields.len() == 2
                && matches!(&fields[0], (field, HirPattern::Variable(identifier))
                    if field.as_str() == "foo" && identifier.name.as_str() == "foo")
                && matches!(&fields[1], (field, HirPattern::Variable(identifier))
                    if field.as_str() == "bar" && identifier.name.as_str() == "bar")
    ));
}

#[test]
fn lowers_uncaught_raise_into_mir_perform_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 104 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    raise problem
    return
}"#,
        )
        .unwrap();

    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == MirEffectKind::Raise
        )
    }));
}

#[test]
fn lowers_catch_resume_into_mir_handler_region() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 105 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let value = catch raise task {
        case Yielded(next_value):
            resume next_value
        else:
            raise fallback
    }
    return value
}"#,
        )
        .unwrap();

    let dispatch_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_dispatch").expect("expected catch dispatch block");
    assert_eq!(dispatch_block.parameters.len(), 1);

    let resume_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_resume").expect("expected catch resume block");
    assert_eq!(resume_block.parameters.len(), 1);

    let exit_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_exit").expect("expected catch exit block");
    assert_eq!(exit_block.parameters.len(), 1);

    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::Jump { target, arguments }
                if *target == resume_block.id && arguments.len() == 1
        )
    }));
}

#[test]
fn lowers_catch_guard_into_mir_arm_branch() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 107 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let value = catch raise task {
        case Yielded(next_value) if false:
            resume next_value
        else:
            raise fallback
    }
    return value
}"#,
        )
        .unwrap();

    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");
    let match_block = mir.functions[0]
        .blocks
        .iter()
        .find(|block| block.label == "catch_arm_0_match")
        .unwrap_or_else(|| panic!("expected first catch arm match block, got blocks: {:?}", mir.functions[0].blocks));
    let body_block =
        mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0_body").expect("expected guarded catch arm body block");

    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| matches!(instruction.kind, valkyrie_compiler::MirInstructionKind::PatternMatch { .. })));

    assert!(matches!(
        &guard_block.terminator,
        valkyrie_compiler::MirTerminator::Branch { then_target, .. } if *then_target == match_block.id
    ));

    assert!(matches!(
        &match_block.terminator,
        valkyrie_compiler::MirTerminator::Branch { then_target, .. } if *then_target == body_block.id
    ));
}

#[test]
fn rethrows_unmatched_catch_effect_into_outer_raise_path() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 111 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    catch raise task {
        case Yielded(next_value):
            resume next_value
    }
    return
}"#,
        )
        .unwrap();

    let no_match_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_no_match").expect("expected catch no-match block");
    assert!(matches!(
        &no_match_block.terminator,
        valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), .. }
            if *effect == MirEffectKind::Raise
    ));

    let raise_resume_block =
        mir.functions[0].blocks.iter().find(|block| block.label == "raise_resume").expect("expected propagated raise resume block");
    let catch_exit_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_exit").expect("expected catch exit block");
    assert!(matches!(
        &raise_resume_block.terminator,
        valkyrie_compiler::MirTerminator::Jump { target, arguments }
            if *target == catch_exit_block.id && arguments.len() == 1
    ));
}

#[test]
fn preserves_raise_effect_into_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 106 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    raise problem
    return
}"#,
        )
        .unwrap();

    assert!(lir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            LirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == LirEffectKind::Raise
        )
    }));
}

#[test]
fn preserves_catch_pattern_match_into_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 108 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let value = catch raise task {
        case Yielded(next_value) if false:
            resume next_value
        else:
            raise fallback
    }
    return value
}"#,
        )
        .unwrap();

    let guard_block = lir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block.operations.iter().any(|operation| matches!(operation.kind, valkyrie_compiler::LirOperationKind::PatternMatch { .. })));
}

#[test]
fn compiler_facade_lowers_labeled_loop_control_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 99 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    'outer: while true {
        while true {
            continue 'outer
        }
    }
}"#,
        )
        .unwrap();
    let HirStatementKind::Expr(expression) = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected outer loop expression");
    };
    let HirExprKind::Loop { label: Some(label), body, .. } = &expression.kind
    else {
        panic!("expected labeled loop");
    };
    assert_eq!(label.as_str(), "outer");
    let HirStatementKind::Expr(inner_loop) = &body.statements[0].kind
    else {
        panic!("expected inner loop expression");
    };
    let HirExprKind::Loop { body: inner_body, .. } = &inner_loop.kind
    else {
        panic!("expected inner loop");
    };
    let HirStatementKind::Expr(continue_expr) = &inner_body.statements[0].kind
    else {
        panic!("expected continue expression");
    };
    assert!(matches!(
        continue_expr.kind,
        HirExprKind::Continue { label: Some(ref label) } if label.as_str() == "outer"
    ));
}

#[test]
fn rejects_continue_with_unknown_label() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 100 });
    let err = compiler
        .compile_source(
            r#"micro main() {
    while true {
        continue 'missing
    }
}"#,
        )
        .unwrap_err();
    assert!(err.to_string().contains("missing"));
    assert!(err.to_string().contains("label"));
}

#[test]
fn rejects_resume_outside_catch_arm() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 101 });
    let err = compiler
        .compile_source(
            r#"micro main() {
    resume value
}"#,
        )
        .unwrap_err();
    assert!(err.to_string().contains("resume"));
    assert!(err.to_string().contains("catch"));
}

#[test]
fn rejects_break_expr_in_statement_loop() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 110 });
    let err = compiler
        .compile_source(
            r#"micro main() {
    loop {
        break 1
    }
}"#,
        )
        .unwrap_err();
    assert!(err.to_string().contains("break expr"));
    assert!(err.to_string().contains("不接受值"));
}

#[test]
fn lowers_let_binding_and_final_expression() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 9 });
    let module = compiler
        .compile_source(
            r#"micro main() -> i64 {
    let value: i64 = 42;
    value
}
"#,
        )
        .unwrap();
    assert_eq!(module.functions[0].body.statements.len(), 1);
    assert!(matches!(module.functions[0].body.statements[0].kind, HirStatementKind::Let { .. }));
    assert!(matches!(module.functions[0].body.expr.as_ref().map(|expr| &expr.kind), Some(HirExprKind::Variable(_))));
}

#[test]
fn lowers_tuple_pattern_let_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 71 });
    let module = compiler
        .compile_source(
            r#"micro main() -> i64 {
    let (x, y) = (1, 2);
    return x + y;
}
"#,
        )
        .unwrap();

    let HirStatementKind::Let { pattern, initializer, .. } = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected let statement");
    };

    assert!(matches!(
        pattern,
        HirPattern::Tuple(items)
            if items.len() == 2
                && matches!(&items[0], HirPattern::Variable(identifier) if identifier.name.as_str() == "x")
                && matches!(&items[1], HirPattern::Variable(identifier) if identifier.name.as_str() == "y")
    ));
    assert!(matches!(
        initializer.as_deref().map(|expr| &expr.kind),
        Some(HirExprKind::Call { callee, args, .. })
            if matches!(callee.kind, HirExprKind::Path(ref path) if path.to_string() == "tuple") && args.len() == 2
    ));
}

#[test]
fn lowers_loop_in_tuple_pattern_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 73 });
    let module = compiler
        .compile_source(
            r#"micro main() -> i64 {
    loop (x, y) in [(1, 2)] {
        return x + y;
    }
    return 0;
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected loop expression statement");
    };

    assert!(matches!(
        expression.kind,
        HirExprKind::Loop { ref pattern, ref iterator, ref condition, .. }
            if condition.is_none()
                && iterator.is_some()
                && matches!(pattern, Some(HirPattern::Tuple(items))
                    if items.len() == 2
                        && matches!(&items[0], HirPattern::Variable(identifier) if identifier.name.as_str() == "x")
                        && matches!(&items[1], HirPattern::Variable(identifier) if identifier.name.as_str() == "y"))
    ));
}

#[test]
fn lowers_only_explicit_builtin_type_names() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 81 });
    let module = compiler
        .compile_source(
            r#"micro main(a: utf8, b: utf16) -> void {
    let value: int32 = 0;
    return;
}
"#,
        )
        .unwrap();
    let function = &module.functions[0];
    assert_eq!(function.params[0].ty, ValkyrieType::Utf8);
    assert_eq!(function.params[1].ty, ValkyrieType::Utf16);
    assert_eq!(function.return_type, ValkyrieType::Void);

    let HirStatementKind::Let { ty: Some(local_ty), .. } = &function.body.statements[0].kind
    else {
        panic!("expected typed let statement");
    };
    assert_eq!(local_ty, &ValkyrieType::Named(valkyrie_types::Identifier::new("int32")));
}

#[test]
fn lowers_tuple_pattern_bindings_into_mir_values() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 75 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() -> i64 {
    let (x, y) = (1, 2);
    return x + y;
}
"#,
        )
        .unwrap();

    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "x")));
    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "y")));
}

#[test]
fn lowers_non_literal_nested_tuple_pattern_bindings_into_mir_values() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 76 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() -> i64 {
    let pair = ((1, 2), 3);
    let ((x, _), y) = pair;
    return x + y;
}
"#,
        )
        .unwrap();

    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "x")));
    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "y")));
    assert!(!mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "_")));
    assert!(mir.functions[0]
        .blocks[0]
        .instructions
        .iter()
        .any(|instruction| matches!(&instruction.kind, MirInstructionKind::Call { callee: valkyrie_compiler::MirOperand::Symbol(path), .. } if path.to_string() == "tuple_get_0")));
}

#[test]
fn statically_unrolls_loop_in_tuple_pattern_for_literal_iterables() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 77 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() -> i64 {
    loop (x, y) in [(1, 2)] {
        return x + y;
    }
    return 0;
}
"#,
        )
        .unwrap();

    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "x")));
    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "y")));
    assert!(mir.functions[0].blocks[0].instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call {
                callee: valkyrie_compiler::MirOperand::Symbol(path),
                ..
            } if path.to_string() == "infix +"
        )
    }));
}

#[test]
fn statically_unrolls_loop_in_tuple_pattern_for_named_literal_iterables() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 78 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() -> i64 {
    let pairs = [((1, 2), 3)];
    loop ((x, _), y) in pairs {
        return x + y;
    }
    return 0;
}
"#,
        )
        .unwrap();

    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "x")));
    assert!(mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "y")));
    assert!(!mir.functions[0]
        .values
        .iter()
        .any(|value| matches!(&value.origin, valkyrie_compiler::MirValueOrigin::LetBinding { name } if name == "_")));
}

#[test]
fn lowers_break_expr_into_loop_exit_block_parameter_in_mir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 84 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() -> i64 {
    let value: i64 = loop {
        break 7
    }
    return value
}
"#,
        )
        .unwrap();

    let exit_block = mir.functions[0].blocks.iter().find(|block| !block.parameters.is_empty()).expect("expected loop exit block parameter");
    assert_eq!(exit_block.parameters.len(), 1);
    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::Jump { target, arguments }
                if *target == exit_block.id && arguments.len() == 1
        )
    }));
}

#[test]
fn preserves_break_expr_jump_arguments_into_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 85 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() -> i64 {
    let value: i64 = loop {
        break 7
    }
    return value
}
"#,
        )
        .unwrap();

    let exit_block = lir.functions[0].blocks.iter().find(|block| !block.parameters.is_empty()).expect("expected loop exit block parameter");
    assert_eq!(exit_block.parameters.len(), 1);
    assert!(lir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            LirTerminator::Jump { target, arguments }
                if *target == exit_block.id && arguments.len() == 1
        )
    }));
}

#[test]
fn lowers_continue_into_loop_header_block_parameter_in_mir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 97 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let seed: i64 = 0
    while true {
        let seed: i64 = 1
        continue
    }
}
"#,
        )
        .unwrap();

    let header_block = mir.functions[0].blocks.iter().find(|block| block.label == "loop_header").expect("expected loop header");
    assert_eq!(header_block.parameters.len(), 1);
    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::Jump { target, arguments }
                if *target == header_block.id && arguments.len() == 1
        )
    }));
}

#[test]
fn preserves_continue_jump_arguments_into_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 98 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let seed: i64 = 0
    while true {
        let seed: i64 = 1
        continue
    }
}
"#,
        )
        .unwrap();

    let header_block = lir.functions[0].blocks.iter().find(|block| block.label == "loop_header").expect("expected loop header");
    assert_eq!(header_block.parameters.len(), 1);
    assert!(lir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            LirTerminator::Jump { target, arguments }
                if *target == header_block.id && arguments.len() == 1
        )
    }));
}

#[test]
fn lowers_continue_label_into_outer_loop_header_in_mir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 102 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let seed: i64 = 0
    'outer: while true {
        let seed: i64 = 1
        while true {
            continue 'outer
        }
    }
}"#,
        )
        .unwrap();

    let outer_header = mir.functions[0]
        .blocks
        .iter()
        .find(|block| block.label == "loop_header" && block.parameters.len() == 1)
        .expect("expected outer loop header");
    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::Jump { target, arguments }
                if *target == outer_header.id && arguments.len() == 1
        )
    }));
}

#[test]
fn lowers_break_label_expr_into_outer_loop_exit_in_mir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 103 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() -> i64 {
    let value: i64 = 'outer: loop {
        while true {
            break 'outer 7
        }
    }
    return value
}"#,
        )
        .unwrap();

    let outer_exit = mir.functions[0]
        .blocks
        .iter()
        .find(|block| block.label == "loop_exit" && block.parameters.len() == 1)
        .unwrap_or_else(|| panic!("expected outer loop exit, got blocks: {:?}", mir.functions[0].blocks));
    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::Jump { target, arguments }
                if *target == outer_exit.id && arguments.len() == 1
        )
    }));
}

#[test]
fn lowers_yield_statement_into_mir_perform_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 86 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    yield 1
    return
}
"#,
        )
        .unwrap();

    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), resume_target } if *effect == MirEffectKind::Yield => {
                Some(*resume_target)
            }
            _ => None,
        })
        .expect("expected yield perform effect");
    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == MirEffectKind::Yield
        )
    }));
    let resume_block = mir.functions[0].blocks.iter().find(|block| block.id == resume_target).expect("expected yield resume block");
    assert_eq!(resume_block.parameters.len(), 1);
}

#[test]
fn lowers_yield_from_statement_into_lir_perform_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 87 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    yield from values
    return
}
"#,
        )
        .unwrap();

    assert!(lir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            LirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == LirEffectKind::DelegateYield
        )
    }));
}

#[test]
fn lowers_yield_from_statement_into_mir_resume_block_parameter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 93 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    yield from values
    return
}
"#,
        )
        .unwrap();
    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), resume_target }
                if *effect == MirEffectKind::DelegateYield =>
            {
                Some(*resume_target)
            }
            _ => None,
        })
        .expect("expected yield from perform effect");
    let resume_block = mir.functions[0].blocks.iter().find(|block| block.id == resume_target).expect("expected yield from resume block");
    assert_eq!(resume_block.parameters.len(), 1);
}

#[test]
fn lowers_await_member_access_into_hir_control_flow() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 88 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    future.await
}
"#,
        )
        .unwrap();
    let expression = module.functions[0].body.expr.as_ref().expect("expected final expression");
    assert!(matches!(
        expression.kind,
        HirExprKind::Await(ref value)
            if matches!(value.kind, HirExprKind::Variable(ref identifier) if identifier.name.as_str() == "future")
    ));
}

#[test]
fn lowers_await_member_access_into_mir_perform_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 89 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    future.await
    return
}
"#,
        )
        .unwrap();

    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), resume_target } if *effect == MirEffectKind::Await => {
                Some(*resume_target)
            }
            _ => None,
        })
        .expect("expected await perform effect");
    assert!(mir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == MirEffectKind::Await
        )
    }));
    let resume_block = mir.functions[0].blocks.iter().find(|block| block.id == resume_target).expect("expected await resume block");
    assert_eq!(resume_block.parameters.len(), 1);
    let resume_value = resume_block.parameters[0];
    assert!(mir.functions[0].values.iter().any(
        |value| matches!(value.origin, MirValueOrigin::BlockParameter { block, .. } if block == resume_block.id && value.id == resume_value)
    ));
}

#[test]
fn lowers_block_member_access_into_lir_perform_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 90 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    future.block
    return
}
"#,
        )
        .unwrap();

    assert!(lir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            LirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == LirEffectKind::AsyncBlock
        )
    }));
}

#[test]
fn lowers_awake_member_access_into_lir_perform_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 91 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    future.awake
    return
}
"#,
        )
        .unwrap();

    assert!(lir.functions[0].blocks.iter().any(|block| {
        matches!(
            &block.terminator,
            LirTerminator::PerformEffect { effect, payload: Some(_), .. }
                if *effect == LirEffectKind::AsyncSpawn
        )
    }));
}

#[test]
fn lowers_frame_layouts_into_clr_runtime_frames() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 190 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#,
        )
        .unwrap();

    let runtime_frame = lir.functions[0].runtime_frames.first().expect("expected runtime frame");
    assert!(runtime_frame.carrier.contains("$clr_state_"));
    assert_eq!(runtime_frame.state_id, lir.functions[0].frame_layouts[0].state_id);
    assert_eq!(runtime_frame.resume_target, lir.functions[0].frame_layouts[0].resume_target);
    assert_eq!(runtime_frame.slots.len(), lir.functions[0].frame_layouts[0].slots.len());
    assert_eq!(runtime_frame.slots[0].field_name, "slot_0");
}

#[test]
fn lowers_catch_resume_into_clr_runtime_continuation() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 191 });
    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    catch raise true {
        else:
            resume true
    }
    return
}
"#,
        )
        .unwrap();

    let runtime_continuation = lir.functions[0].runtime_continuations.first().expect("expected runtime continuation");
    assert!(runtime_continuation.carrier.contains("$clr_continuation_"));
    assert_eq!(runtime_continuation.resume_parameter_field, "resume_value");
    assert_eq!(runtime_continuation.resume_target, lir.functions[0].continuations[0].resume_target);
    assert_eq!(runtime_continuation.resume_parameter, lir.functions[0].continuations[0].resume_parameter);
    assert_eq!(runtime_continuation.resume_parameter_type, lir.functions[0].continuations[0].resume_parameter_type);
}

#[test]
fn lowers_block_member_access_into_mir_resume_block_parameter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 92 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let value = future.block
    return
}
"#,
        )
        .unwrap();
    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            valkyrie_compiler::MirTerminator::PerformEffect { effect, payload: Some(_), resume_target }
                if *effect == MirEffectKind::AsyncBlock =>
            {
                Some(*resume_target)
            }
            _ => None,
        })
        .expect("expected block perform effect");
    let resume_block = mir.functions[0].blocks.iter().find(|block| block.id == resume_target).expect("expected block resume block");
    assert_eq!(resume_block.parameters.len(), 1);
}

#[test]
fn compiler_facade_lowers_into_mir_and_lir_from_moved_tests() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 11 });
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
    assert_eq!(lir.functions[0].blocks.len(), 1);
}

#[test]
fn lowers_structured_attribute_arguments_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 13 });
    let module = compiler
        .compile_source(
            r#"
[clr("System.Console", "System.Console", "WriteLine")]
micro helper(message: utf16) {
    return;
}
"#,
        )
        .unwrap();

    assert_eq!(module.functions[0].annotations.len(), 1);
    assert_eq!(module.functions[0].annotations[0].name.to_string(), "clr");
    assert_eq!(module.functions[0].annotations[0].arguments.len(), 3);
    assert!(matches!(module.functions[0].annotations[0].arguments[0].value.kind, HirExprKind::Literal(_)));
}

#[test]
fn lowers_term_turbofish_into_generic_apply() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 29 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    T::<i64>();
}
"#,
        )
        .unwrap();
    let HirStatementKind::Expr(expression) = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert!(args.is_empty());
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref arguments, .. }
                    if arguments.len() == 1 && arguments[0] == ValkyrieType::Integer64 { signed: true }
            ));
        }
        _ => panic!("expected call expression"),
    }
}

#[test]
fn lowers_instance_method_with_implicit_self_param() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 31 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro heal(amount: i64) -> i64 {
        self.health;
        return amount;
    }
}
"#,
        )
        .unwrap();

    let method = &module.structs[0].methods[0];
    assert_eq!(method.params.len(), 2);
    assert_eq!(method.params[0].name.name.as_str(), "self");
    assert!(matches!(method.params[0].ty, ValkyrieType::r#SelfType));
    assert_eq!(method.params[1].name.name.as_str(), "amount");
}

#[test]
fn keeps_void_alias_and_self_name_as_user_types_until_hir_lowering() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 67 });
    let module = compiler
        .compile_source(
            r#"
type void = c_void;
micro convert(value: Self) -> void {
}
micro make() -> () {
}
"#,
        )
        .unwrap();

    let convert = &module.functions[0];
    assert!(matches!(
        convert.params[0].ty,
        ValkyrieType::Named(ref name) if name.as_str() == "Self"
    ));
    assert!(matches!(
        convert.return_type,
        ValkyrieType::Named(ref name) if name.as_str() == "void"
    ));

    let make = &module.functions[1];
    assert_eq!(make.return_type, ValkyrieType::Unit);
}

#[test]
fn lowers_getter_and_setter_into_one_hir_property() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 61 });
    let module = compiler
        .compile_source(
            r#"class Rectangle {
    get area(self) -> i64 {
        return self.width;
    }

    set area(mut self, value: i64) {
        self.width = value;
    }
}"#,
        )
        .unwrap();

    let class = &module.structs[0];
    assert_eq!(class.properties.len(), 1);
    let property = &class.properties[0];
    assert_eq!(property.name.as_str(), "area");
    assert_eq!(property.ty, ValkyrieType::Integer64 { signed: true });
    assert!(!property.is_readonly);
    assert!(property.getter.is_some());
    assert!(property.setter.is_some());

    let getter = property.getter.as_ref().unwrap();
    assert_eq!(getter.name.as_str(), "area");
    assert_eq!(getter.params.len(), 1);
    assert_eq!(getter.return_type, ValkyrieType::Integer64 { signed: true });

    let setter = property.setter.as_ref().unwrap();
    assert_eq!(setter.name.as_str(), "set_area");
    assert_eq!(setter.params.len(), 2);
    assert_eq!(setter.return_type, ValkyrieType::Unit);
}

#[test]
fn lowers_property_modifiers_into_hir_flags() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 63 });
    let module = compiler
        .compile_source(
            r#"class Shape {
    virtual get area(self) -> i64;
}

class MathConstants {
    static final get pi() -> i64 {
        return 3;
    }
}"#,
        )
        .unwrap();

    let shape = &module.structs[0];
    assert_eq!(shape.properties.len(), 1);
    let area = &shape.properties[0];
    assert!(area.is_abstract);
    assert!(area.is_virtual);
    assert!(!area.is_static);
    assert!(area.getter.as_ref().unwrap().is_abstract);

    let math = &module.structs[1];
    assert_eq!(math.properties.len(), 1);
    let pi = &math.properties[0];
    assert!(pi.is_static);
    assert!(pi.is_final);
    assert!(pi.is_readonly);
    assert!(pi.getter.is_some());
    assert_eq!(pi.getter.as_ref().unwrap().params.len(), 0);
}

#[test]
fn lowers_static_method_without_implicit_self_param() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 37 });
    let module = compiler
        .compile_source(
            r#"
class Math {
    static micro abs(value: i64) -> i64 {
        return value;
    }
}
"#,
        )
        .unwrap();

    let method = &module.structs[0].methods[0];
    assert_eq!(method.params.len(), 1);
    assert_eq!(method.params[0].name.name.as_str(), "value");
}

#[test]
fn lowers_member_field_access_and_assignment_into_getter_setter_calls() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 41 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro heal(amount: i64) {
        self.health = amount;
        self.health;
    }
}
"#,
        )
        .unwrap();

    let statements = &module.structs[0].methods[0].body.statements;
    let HirStatementKind::Expr(setter_expr) = &statements[0].kind
    else {
        panic!("expected setter expression");
    };
    let HirStatementKind::Expr(getter_expr) = &statements[1].kind
    else {
        panic!("expected getter expression");
    };

    match &setter_expr.kind {
        HirExprKind::StoreField { object, field, value } => {
            assert_eq!(field.as_str(), "health");
            assert!(matches!(object.kind, HirExprKind::Variable(_)));
            assert!(matches!(value.kind, HirExprKind::Variable(_)));
        }
        _ => panic!("expected store field"),
    }

    match &getter_expr.kind {
        HirExprKind::FieldAccess { object, field } => {
            assert_eq!(field.as_str(), "health");
            assert!(matches!(object.kind, HirExprKind::Variable(_)));
        }
        _ => panic!("expected field access"),
    }
}

#[test]
fn preserves_instance_method_call_without_rewriting_to_getter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 43 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro tick() {
        self.refresh();
    }

    micro refresh() {
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[0].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0].kind, HirExprKind::Variable(_)));
            assert!(matches!(
                callee.kind,
                HirExprKind::Path(ref path) if path.to_string() == "refresh"
            ));
        }
        _ => panic!("expected method call"),
    }
}

#[test]
fn lowers_member_turbofish_call_with_receiver_as_first_argument() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 47 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro tick(value: i64) {
        self.refresh::<i64>(value);
    }

    micro refresh(value: i64) {
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[0].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0].kind, HirExprKind::Variable(_)));
            assert!(matches!(args[1].kind, HirExprKind::Variable(_)));
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref callee, ref arguments }
                    if matches!(callee.kind, HirExprKind::Path(ref path) if path.to_string() == "refresh")
                        && arguments.len() == 1
            ));
        }
        _ => panic!("expected turbofish method call"),
    }
}

#[test]
fn lowers_parent_slot_method_call_as_slot_access_plus_method_call() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 53 });
    let module = compiler
        .compile_source(
            r#"
class Display {
    micro show() {
    }
}

class Document(rename: Display) {
    micro render() {
        self.rename.show();
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[1].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(
                callee.kind,
                HirExprKind::Path(ref path) if path.to_string() == "show"
            ));
            assert!(matches!(
                args[0].kind,
                HirExprKind::FieldAccess { ref object, ref field }
                    if field.as_str() == "rename"
                        && matches!(object.kind, HirExprKind::Variable(_))
            ));
        }
        _ => panic!("expected renamed parent method call"),
    }
}

#[test]
fn lowers_parent_slot_turbofish_call_as_slot_access_plus_method_call() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 59 });
    let module = compiler
        .compile_source(
            r#"
class Reader {
    micro read(value: i64) {
    }
}

class Hybrid(reader: Reader) {
    micro consume(value: i64) {
        self.reader.read::<i64>(value);
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[1].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[1].kind, HirExprKind::Variable(_)));
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref callee, ref arguments }
                    if matches!(callee.kind, HirExprKind::Path(ref path) if path.to_string() == "read")
                        && arguments.len() == 1
            ));
            assert!(matches!(
                args[0].kind,
                HirExprKind::FieldAccess { ref object, ref field }
                    if field.as_str() == "reader"
                        && matches!(object.kind, HirExprKind::Variable(_))
            ));
        }
        _ => panic!("expected renamed parent turbofish method call"),
    }
}

#[test]
fn lowers_parent_slot_name_from_alias_or_type_name() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 61 });
    let module = compiler
        .compile_source(
            r#"
class Mixed(primary: Teacher, BaseWidget) {
}
"#,
        )
        .unwrap();

    assert_eq!(module.structs[0].parents.len(), 2);
    assert_eq!(module.structs[0].parents[0].slot_name().as_str(), "primary");
    assert_eq!(module.structs[0].parents[1].slot_name().as_str(), "base_widget");
}

#[test]
fn lowers_unite_declaration_into_hir_enum_family() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 67 });
    let module = compiler
        .compile_source(
            r#"
unite Option {
    Some {
        value: i64,
    }
    None
}
"#,
        )
        .unwrap();

    assert_eq!(module.enums.len(), 1);
    let option = &module.enums[0];
    assert!(option.is_unity());
    assert_eq!(option.name.as_str(), "Option");
    assert_eq!(option.variants.len(), 2);
    assert_eq!(option.variants[0].name.as_str(), "Some");
    assert_eq!(option.variants[0].fields.len(), 1);
    assert_eq!(option.variants[1].name.as_str(), "None");
    assert!(option.variants[1].fields.is_empty());
}

#[test]
fn lowers_trait_associated_types_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 81 });
    let module = compiler
        .compile_source(
            r#"
trait Iterator<T>: Display + Clone {
    type Item
    type Output = T
    const Limit: i64 = 42

    micro next(self) -> Self::Item
    micro collect(self) -> T {
        return self;
    }
}
"#,
        )
        .unwrap();

    assert_eq!(module.traits.len(), 1);
    let trait_def = &module.traits[0];
    assert_eq!(trait_def.name.as_str(), "Iterator");
    assert_eq!(trait_def.super_traits.len(), 2);
    assert_eq!(trait_def.associated_types.len(), 2);
    assert_eq!(trait_def.associated_constants.len(), 1);
    assert_eq!(trait_def.associated_types[0].name.as_str(), "Item");
    assert!(trait_def.associated_types[0].default.is_none());
    assert_eq!(trait_def.associated_types[1].name.as_str(), "Output");
    assert!(matches!(trait_def.associated_types[1].default, Some(ValkyrieType::Named(ref name)) if name.as_str() == "T"));
    assert_eq!(trait_def.associated_constants[0].name.as_str(), "Limit");
    assert_eq!(trait_def.associated_constants[0].const_type, ValkyrieType::Integer64 { signed: true });
    assert!(matches!(
        trait_def.associated_constants[0].default_value.as_ref(),
        Some(expr) if matches!(expr.kind, valkyrie_types::hir::HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(42)))
    ));
    assert_eq!(trait_def.methods.len(), 1);
    assert_eq!(trait_def.default_methods.len(), 1);
}

#[test]
fn lowers_imply_blocks_into_hir_impls() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 83 });
    let module = compiler
        .compile_source(
            r#"
imply<T: Clone> Buffer<T>: Iterator
where T: Display {
    type Item = T
    const SIZE: i64 = 1

    micro next(self) -> T {
        return self.value;
    }
}

imply Point {
    micro length(self) -> i64 {
        return self.x;
    }
}
"#,
        )
        .unwrap();

    assert_eq!(module.impls.len(), 2);

    let trait_impl = &module.impls[0];
    assert!(matches!(trait_impl.target, ValkyrieType::Apply(_, _)));
    assert!(matches!(trait_impl.trait_path.as_ref(), Some(path) if path.to_string() == "Iterator"));
    assert_eq!(trait_impl.generics.len(), 1);
    assert_eq!(trait_impl.generics[0].name.as_str(), "T");
    assert_eq!(trait_impl.generics[0].bounds.len(), 1);
    assert_eq!(trait_impl.where_constraints.len(), 1);
    assert!(matches!(trait_impl.where_constraints[0].target, ValkyrieType::Named(ref name) if name.as_str() == "T"));
    assert_eq!(trait_impl.where_constraints[0].bounds.len(), 1);
    assert_eq!(trait_impl.where_constraints[0].bounds[0].to_string(), "Display");
    assert_eq!(trait_impl.methods.len(), 1);
    assert_eq!(trait_impl.associated_type_impls.len(), 1);
    assert_eq!(trait_impl.associated_const_impls.len(), 1);
    assert_eq!(trait_impl.associated_type_impls[0].name.as_str(), "Item");
    assert!(matches!(trait_impl.associated_type_impls[0].concrete_type, ValkyrieType::Named(ref name) if name.as_str() == "T"));
    assert_eq!(trait_impl.associated_const_impls[0].name.as_str(), "SIZE");
    assert_eq!(trait_impl.associated_const_impls[0].const_type, Some(ValkyrieType::Integer64 { signed: true }));

    let inherent_impl = &module.impls[1];
    assert!(matches!(inherent_impl.target, ValkyrieType::Named(ref name) if name.as_str() == "Point"));
    assert!(inherent_impl.trait_path.is_none());
    assert!(inherent_impl.where_constraints.is_empty());
    assert_eq!(inherent_impl.methods.len(), 1);
    assert!(inherent_impl.associated_type_impls.is_empty());
    assert!(inherent_impl.associated_const_impls.is_empty());
}

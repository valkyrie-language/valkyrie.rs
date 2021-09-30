use valkyrie_compiler::{pipeline::ast_to_hir::*, LirDispatchKind, LirOperationKind, LirTargetLane, LirTerminator, MirInstructionKind};
use valkyrie_types::{
    hir::{CaptureMode, HirExpr, HirExprKind, HirPattern, HirStatementKind, HirType},
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
    analyzer.add_var("x", HirType::Integer64, false);
    analyzer.access_var("x", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].identifier.name.as_str(), "x");
    assert_eq!(captures[0].mode, CaptureMode::ByValue);
}

#[test]
fn test_capture_analyzer_mutable_var() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.add_var("y", HirType::Integer64, true);
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
    analyzer.add_var("x", HirType::Integer64, false);
    analyzer.access_var("x", false);
    analyzer.access_var("x", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
}

#[test]
fn test_capture_analyzer_by_reference() {
    let mut analyzer = CaptureAnalyzer::new();
    analyzer.add_var("obj", HirType::Named(valkyrie_types::Identifier::new("MyObject")), false);
    analyzer.access_var("obj", false);
    let captures = analyzer.into_captures();
    assert_eq!(captures.len(), 1);
    assert_eq!(captures[0].mode, CaptureMode::ByReference);
}

#[test]
fn test_lower_root_to_hir_module() {
    let compiler = ValkyrieCompiler::new(SourceID(17));
    let module = compiler
        .compile_source(
            "namespace demo;\nusing std::console;\nmicro main(args: [utf8]) -> i64 {\n    let code: i64 = 0;\n    return code;\n}\n",
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
    let compiler = ValkyrieCompiler::new(SourceID(19));
    let module = compiler.compile_source("micro main() -> i64 {\n    return 42;\n}\n").unwrap();
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
    let compiler = ValkyrieCompiler::new(SourceID(23));
    let mir = compiler.compile_source_to_mir("micro main(input: i64) -> i64 {\n    return input;\n}\n").unwrap();
    assert_eq!(mir.functions.len(), 1);
    assert!(mir.functions[0].values.iter().any(|value| matches!(value.origin, valkyrie_compiler::MirValueOrigin::Parameter { index: 0, .. })));

    let lir = compiler.compile_source_to_lir("micro main() {\n    std::console::write_line(\"hi\");\n}\n").unwrap();
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
    let source = "namespace demo;\nusing std::console;\nmicro main() -> i64 {\n    return 0;\n}\n";
    let root = valkyrie_parser::AstParser::parse_root(source).unwrap();
    let module = AstToHir::new(SourceID(7)).lower_root(&root).unwrap();
    assert_eq!(module.name.to_string(), "demo");
    assert_eq!(module.imports.len(), 1);
    assert_eq!(module.functions.len(), 1);
    assert_eq!(module.functions[0].span.source, SourceID(7));
    assert!(matches!(module.functions[0].body.statements[0].kind, valkyrie_types::hir::HirStatementKind::Expr(_)));
}

#[test]
fn rejects_legacy_string_type_input() {
    let compiler = ValkyrieCompiler::new(SourceID(701));
    let err = compiler.compile_source("micro main(message: string) -> void {\n    return;\n}\n").unwrap_err();
    assert!(err.to_string().contains("string"));
    assert!(err.to_string().contains("utf8"));
    assert!(err.to_string().contains("utf16"));
}

#[test]
fn compiler_facade_parses_and_lowers_return_statement() {
    let compiler = ValkyrieCompiler::new(SourceID(3));
    let module = compiler.compile_source("micro main() {\n    return;\n}\n").unwrap();
    let statement = &module.functions[0].body.statements[0];
    let HirStatementKind::Expr(expression) = &statement.kind
    else {
        panic!("expected expression statement");
    };
    assert!(matches!(expression.kind, HirExprKind::Return(_)));
}

#[test]
fn lowers_let_binding_and_final_expression() {
    let compiler = ValkyrieCompiler::new(SourceID(9));
    let module = compiler.compile_source("micro main() -> i64 {\n    let value: i64 = 42;\n    value\n}\n").unwrap();
    assert_eq!(module.functions[0].body.statements.len(), 1);
    assert!(matches!(module.functions[0].body.statements[0].kind, HirStatementKind::Let { .. }));
    assert!(matches!(module.functions[0].body.expr.as_ref().map(|expr| &expr.kind), Some(HirExprKind::Variable(_))));
}

#[test]
fn lowers_tuple_pattern_let_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID(71));
    let module = compiler.compile_source("micro main() -> i64 {\n    let (x, y) = (1, 2);\n    return x + y;\n}\n").unwrap();

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
        Some(HirExprKind::Call { callee, args })
            if matches!(callee.kind, HirExprKind::Path(ref path) if path.to_string() == "tuple") && args.len() == 2
    ));
}

#[test]
fn lowers_loop_in_tuple_pattern_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID(73));
    let module = compiler
        .compile_source("micro main() -> i64 {\n    loop (x, y) in [(1, 2)] {\n        return x + y;\n    }\n    return 0;\n}\n")
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
    let compiler = ValkyrieCompiler::new(SourceID(81));
    let module = compiler.compile_source("micro main(a: utf8, b: utf16) -> void {\n    let value: int32 = 0;\n    return;\n}\n").unwrap();

    let function = &module.functions[0];
    assert_eq!(function.params[0].ty, HirType::Utf8);
    assert_eq!(function.params[1].ty, HirType::Utf16);
    assert_eq!(function.return_type, HirType::Void);

    let HirStatementKind::Let { ty: Some(local_ty), .. } = &function.body.statements[0].kind
    else {
        panic!("expected typed let statement");
    };
    assert_eq!(local_ty, &HirType::Named(valkyrie_types::Identifier::new("int32")));
}

#[test]
fn lowers_tuple_pattern_bindings_into_mir_values() {
    let compiler = ValkyrieCompiler::new(SourceID(75));
    let mir = compiler.compile_source_to_mir("micro main() -> i64 {\n    let (x, y) = (1, 2);\n    return x + y;\n}\n").unwrap();

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
    let compiler = ValkyrieCompiler::new(SourceID(76));
    let mir = compiler
        .compile_source_to_mir("micro main() -> i64 {\n    let pair = ((1, 2), 3);\n    let ((x, _), y) = pair;\n    return x + y;\n}\n")
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
    let compiler = ValkyrieCompiler::new(SourceID(77));
    let mir = compiler
        .compile_source_to_mir("micro main() -> i64 {\n    loop (x, y) in [(1, 2)] {\n        return x + y;\n    }\n    return 0;\n}\n")
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
    let compiler = ValkyrieCompiler::new(SourceID(78));
    let mir = compiler
        .compile_source_to_mir(
            "micro main() -> i64 {\n    let pairs = [((1, 2), 3)];\n    loop ((x, _), y) in pairs {\n        return x + y;\n    }\n    return 0;\n}\n",
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
fn compiler_facade_lowers_into_mir_and_lir_from_moved_tests() {
    let compiler = ValkyrieCompiler::new(SourceID(11));
    let mir = compiler.compile_source_to_mir("micro main(input: i64) -> i64 {\n    return input;\n}\n").unwrap();
    assert_eq!(mir.functions.len(), 1);
    assert!(mir.functions[0].values.iter().any(|value| matches!(value.origin, valkyrie_compiler::MirValueOrigin::Parameter { index: 0, .. })));

    let lir = compiler.compile_source_to_lir("micro main() {\n    std::console::write_line(\"hi\");\n}\n").unwrap();
    assert_eq!(lir.functions.len(), 1);
    assert_eq!(lir.functions[0].blocks.len(), 1);
}

#[test]
fn lowers_structured_attribute_arguments_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID(13));
    let module = compiler
        .compile_source(
            "[clr(\"System.Console\", \"System.Console\", \"WriteLine\")]\n\
micro helper(message: utf16) {\n\
    return;\n\
}\n",
        )
        .unwrap();

    assert_eq!(module.functions[0].annotations.len(), 1);
    assert_eq!(module.functions[0].annotations[0].name.to_string(), "clr");
    assert_eq!(module.functions[0].annotations[0].arguments.len(), 3);
    assert!(matches!(module.functions[0].annotations[0].arguments[0].value.kind, HirExprKind::Literal(_)));
}

#[test]
fn lowers_term_turbofish_into_generic_apply() {
    let compiler = ValkyrieCompiler::new(SourceID(29));
    let module = compiler.compile_source("micro main() {\n    T::<i64>();\n}\n").unwrap();
    let HirStatementKind::Expr(expression) = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args } => {
            assert!(args.is_empty());
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref arguments, .. } if arguments.len() == 1 && arguments[0] == HirType::Integer64
            ));
        }
        _ => panic!("expected call expression"),
    }
}

#[test]
fn lowers_instance_method_with_implicit_self_param() {
    let compiler = ValkyrieCompiler::new(SourceID(31));
    let module = compiler
        .compile_source(
            "class Player {\n\
    micro heal(amount: i64) -> i64 {\n\
        self.health;\n\
        return amount;\n\
    }\n\
}\n",
        )
        .unwrap();

    let method = &module.structs[0].methods[0];
    assert_eq!(method.params.len(), 2);
    assert_eq!(method.params[0].name.name.as_str(), "self");
    assert!(matches!(method.params[0].ty, HirType::SelfType));
    assert_eq!(method.params[1].name.name.as_str(), "amount");
}

#[test]
fn lowers_getter_and_setter_into_one_hir_property() {
    let compiler = ValkyrieCompiler::new(SourceID(61));
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
    assert_eq!(property.ty, HirType::Integer64);
    assert!(!property.is_readonly);
    assert!(property.getter.is_some());
    assert!(property.setter.is_some());

    let getter = property.getter.as_ref().unwrap();
    assert_eq!(getter.name.as_str(), "area");
    assert_eq!(getter.params.len(), 1);
    assert_eq!(getter.return_type, HirType::Integer64);

    let setter = property.setter.as_ref().unwrap();
    assert_eq!(setter.name.as_str(), "set_area");
    assert_eq!(setter.params.len(), 2);
    assert_eq!(setter.return_type, HirType::Unit);
}

#[test]
fn lowers_property_modifiers_into_hir_flags() {
    let compiler = ValkyrieCompiler::new(SourceID(63));
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
    let compiler = ValkyrieCompiler::new(SourceID(37));
    let module = compiler
        .compile_source(
            "class Math {\n\
    static micro abs(value: i64) -> i64 {\n\
        return value;\n\
    }\n\
}\n",
        )
        .unwrap();

    let method = &module.structs[0].methods[0];
    assert_eq!(method.params.len(), 1);
    assert_eq!(method.params[0].name.name.as_str(), "value");
}

#[test]
fn lowers_member_field_access_and_assignment_into_getter_setter_calls() {
    let compiler = ValkyrieCompiler::new(SourceID(41));
    let module = compiler
        .compile_source(
            "class Player {\n\
    micro heal(amount: i64) {\n\
        self.health = amount;\n\
        self.health;\n\
    }\n\
}\n",
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
    let compiler = ValkyrieCompiler::new(SourceID(43));
    let module = compiler
        .compile_source(
            "class Player {\n\
    micro tick() {\n\
        self.refresh();\n\
    }\n\
\n\
    micro refresh() {\n\
    }\n\
}\n",
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[0].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args } => {
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
    let compiler = ValkyrieCompiler::new(SourceID(47));
    let module = compiler
        .compile_source(
            "class Player {\n\
    micro tick(value: i64) {\n\
        self.refresh::<i64>(value);\n\
    }\n\
\n\
    micro refresh(value: i64) {\n\
    }\n\
}\n",
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[0].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args } => {
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
    let compiler = ValkyrieCompiler::new(SourceID(53));
    let module = compiler
        .compile_source(
            "class Display {\n\
    micro show() {\n\
    }\n\
}\n\
\n\
class Document(rename: Display) {\n\
    micro render() {\n\
        self.rename.show();\n\
    }\n\
}\n",
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[1].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args } => {
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
    let compiler = ValkyrieCompiler::new(SourceID(59));
    let module = compiler
        .compile_source(
            "class Reader {\n\
    micro read(value: i64) {\n\
    }\n\
}\n\
\n\
class Hybrid(reader: Reader) {\n\
    micro consume(value: i64) {\n\
        self.reader.read::<i64>(value);\n\
    }\n\
}\n",
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[1].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args } => {
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
    let compiler = ValkyrieCompiler::new(SourceID(61));
    let module = compiler
        .compile_source(
            "class Mixed(primary: Teacher, BaseWidget) {\n\
}\n",
        )
        .unwrap();

    assert_eq!(module.structs[0].parents.len(), 2);
    assert_eq!(module.structs[0].parents[0].slot_name().as_str(), "primary");
    assert_eq!(module.structs[0].parents[1].slot_name().as_str(), "base_widget");
}

#[test]
fn lowers_unite_declaration_into_hir_enum_family() {
    let compiler = ValkyrieCompiler::new(SourceID(67));
    let module = compiler
        .compile_source(
            "unite Option {\n\
    Some {\n\
        value: i64,\n\
    }\n\
    None\n\
}\n",
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
    let compiler = ValkyrieCompiler::new(SourceID(81));
    let module = compiler
        .compile_source(
            "trait Iterator<T>: Display + Clone {\n\
    type Item\n\
    type Output = T\n\
    const Limit: i64 = 42\n\
\n\
    micro next(self) -> Self::Item\n\
    micro collect(self) -> T {\n\
        return self;\n\
    }\n\
}\n",
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
    assert!(matches!(trait_def.associated_types[1].default, Some(HirType::Named(ref name)) if name.as_str() == "T"));
    assert_eq!(trait_def.associated_constants[0].name.as_str(), "Limit");
    assert_eq!(trait_def.associated_constants[0].const_type, HirType::Integer64);
    assert!(matches!(
        trait_def.associated_constants[0].default_value.as_ref(),
        Some(expr) if matches!(expr.kind, valkyrie_types::hir::HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(42)))
    ));
    assert_eq!(trait_def.methods.len(), 1);
    assert_eq!(trait_def.default_methods.len(), 1);
}

#[test]
fn lowers_imply_blocks_into_hir_impls() {
    let compiler = ValkyrieCompiler::new(SourceID(83));
    let module = compiler
        .compile_source(
            "imply<T: Clone> Buffer<T>: Iterator\n\
where T: Display {\n\
    type Item = T\n\
    const SIZE: i64 = 1\n\
\n\
    micro next(self) -> T {\n\
        return self.value;\n\
    }\n\
}\n\
\n\
imply Point {\n\
    micro length(self) -> i64 {\n\
        return self.x;\n\
    }\n\
}\n",
        )
        .unwrap();

    assert_eq!(module.impls.len(), 2);

    let trait_impl = &module.impls[0];
    assert!(matches!(trait_impl.target, HirType::Apply(_, _)));
    assert!(matches!(trait_impl.trait_path.as_ref(), Some(path) if path.to_string() == "Iterator"));
    assert_eq!(trait_impl.generics.len(), 1);
    assert_eq!(trait_impl.generics[0].name.as_str(), "T");
    assert_eq!(trait_impl.generics[0].bounds.len(), 1);
    assert_eq!(trait_impl.where_constraints.len(), 1);
    assert!(matches!(trait_impl.where_constraints[0].target, HirType::Named(ref name) if name.as_str() == "T"));
    assert_eq!(trait_impl.where_constraints[0].bounds.len(), 1);
    assert_eq!(trait_impl.where_constraints[0].bounds[0].to_string(), "Display");
    assert_eq!(trait_impl.methods.len(), 1);
    assert_eq!(trait_impl.associated_type_impls.len(), 1);
    assert_eq!(trait_impl.associated_const_impls.len(), 1);
    assert_eq!(trait_impl.associated_type_impls[0].name.as_str(), "Item");
    assert!(matches!(trait_impl.associated_type_impls[0].concrete_type, HirType::Named(ref name) if name.as_str() == "T"));
    assert_eq!(trait_impl.associated_const_impls[0].name.as_str(), "SIZE");
    assert_eq!(trait_impl.associated_const_impls[0].const_type, Some(HirType::Integer64));

    let inherent_impl = &module.impls[1];
    assert!(matches!(inherent_impl.target, HirType::Named(ref name) if name.as_str() == "Point"));
    assert!(inherent_impl.trait_path.is_none());
    assert!(inherent_impl.where_constraints.is_empty());
    assert_eq!(inherent_impl.methods.len(), 1);
    assert!(inherent_impl.associated_type_impls.is_empty());
    assert!(inherent_impl.associated_const_impls.is_empty());
}

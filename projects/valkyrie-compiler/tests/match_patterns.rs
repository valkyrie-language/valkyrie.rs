use valkyrie_compiler::ValkyrieCompiler;
use valkyrie_types::{
    hir::{HirExprKind, HirExtractorPattern, HirLiteral, HirPattern, HirStatementKind},
    SourceID,
};

#[test]
fn lowers_literal_variable_and_or_match_patterns_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2973 });
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
        HirPattern::Or(patterns)
            if matches!(patterns.as_slice(), [HirPattern::Literal(HirLiteral::Integer64(1)), HirPattern::Literal(HirLiteral::Integer64(2))])
    ));
    assert!(matches!(&arms[1].pattern, HirPattern::Variable(identifier) if identifier.name.as_str() == "n"));
    assert!(arms[1].guard.is_some());
    assert!(matches!(&arms[2].pattern, HirPattern::Wildcard));
}

#[test]
fn lowers_tuple_match_patterns_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2974 });
    let hir = compiler
        .compile_source(
            r#"micro main(pair: ((i64, i64), i64)) -> bool {
    return match pair {
        case ((x, y), z) if z > 0:
            true
        case (_, 0):
            false
        else:
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
        HirPattern::Tuple(items)
            if matches!(
                items.as_slice(),
                [
                    HirPattern::Tuple(inner),
                    HirPattern::Variable(last)
                ]
                if matches!(
                    inner.as_slice(),
                    [
                        HirPattern::Variable(first),
                        HirPattern::Variable(second)
                    ]
                    if first.name.as_str() == "x" && second.name.as_str() == "y"
                ) && last.name.as_str() == "z"
            )
    ));
    assert!(arms[0].guard.is_some());
    assert!(matches!(
        &arms[1].pattern,
        HirPattern::Tuple(items)
            if matches!(
                items.as_slice(),
                [
                    HirPattern::Wildcard,
                    HirPattern::Literal(HirLiteral::Integer64(0))
                ]
            )
    ));
}

#[test]
fn lowers_nested_constructor_and_object_match_patterns_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2975 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: Wrapper) -> bool {
    return match value {
        case Wrapper(inner, Pair(left, right)):
            true
        case Wrapper { inner: Some(result), fallback }:
            false
        else:
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
        HirPattern::Extractor(HirExtractorPattern::Constructor { name, fields, .. })
            if name.to_string() == "Wrapper"
                && matches!(
                    fields.as_slice(),
                    [
                        HirPattern::Variable(inner),
                        HirPattern::Extractor(HirExtractorPattern::Constructor { name: nested_name, fields: nested_fields, .. })
                    ]
                    if inner.name.as_str() == "inner"
                        && nested_name.to_string() == "Pair"
                        && matches!(
                            nested_fields.as_slice(),
                            [
                                HirPattern::Variable(left),
                                HirPattern::Variable(right)
                            ] if left.name.as_str() == "left" && right.name.as_str() == "right"
                        )
                )
    ));
    assert!(matches!(
        &arms[1].pattern,
        HirPattern::Object { name: Some(name), fields, rest: None }
            if name.to_string() == "Wrapper"
                && fields.len() == 2
                && matches!(
                    &fields[0],
                    (field_name, HirPattern::Extractor(HirExtractorPattern::Constructor { name, fields, .. }))
                        if field_name.as_str() == "inner"
                            && name.to_string() == "Some"
                            && matches!(fields.as_slice(), [HirPattern::Variable(result)] if result.name.as_str() == "result")
                )
                && matches!(
                    &fields[1],
                    (field_name, HirPattern::Variable(fallback))
                        if field_name.as_str() == "fallback" && fallback.name.as_str() == "fallback"
                )
    ));
}

#[test]
fn lowers_nested_constructor_and_object_match_patterns_into_mir_and_lir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2976 });
    let source = r#"micro main(value: Wrapper) -> bool {
    return match value {
        case Wrapper(inner, Pair(left, right)):
            true
        case Wrapper { inner: Some(result), fallback }:
            false
        else:
            false
    };
}
"#;

    let mir = compiler.compile_source_to_mir(source).unwrap();
    assert!(mir.functions[0].blocks.iter().any(|block| {
        block.instructions.iter().any(|instruction| matches!(instruction.kind, valkyrie_compiler::MirInstructionKind::PatternMatch { .. }))
    }));

    let lir = compiler.compile_source_to_lir(source).unwrap();
    assert!(lir.functions[0].blocks.iter().any(|block| {
        block.operations.iter().any(|operation| matches!(operation.kind, valkyrie_compiler::LirOperationKind::PatternMatch { .. }))
    }));
}

#[test]
fn lowers_range_array_rest_and_typed_bind_patterns_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2977 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: Vector) -> bool {
    return match value {
        case [head, ..tail, last]:
            true
        case numbers as Vector:
            false
        case 1..=10 if true:
            false
        else:
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
        HirPattern::Extractor(HirExtractorPattern::Array { prefix, rest: Some(rest), suffix, .. })
            if matches!(prefix.as_slice(), [HirPattern::Variable(head)] if head.name.as_str() == "head")
                && rest.name.as_str() == "tail"
                && matches!(suffix.as_slice(), [HirPattern::Variable(last)] if last.name.as_str() == "last")
    ));
    assert!(matches!(
        &arms[1].pattern,
        HirPattern::TypedBind { identifier, ty }
            if identifier.name.as_str() == "numbers" && ty.to_string() == "Vector"
    ));
    assert!(matches!(
        &arms[2].pattern,
        HirPattern::Range { start: Some(HirLiteral::Integer64(1)), end: Some(HirLiteral::Integer64(10)), inclusive_end: true }
    ));
    assert!(arms[2].guard.is_some());
}

#[test]
fn lowers_qualified_and_bare_name_patterns_without_confusing_variable_binding() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2978 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: Payload) -> bool {
    return match value {
        case package::module::Unite::Variant:
            true
        case Variant:
            false
        case var:
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
        HirPattern::Name(path)
            if path.parts().iter().map(|part| part.as_str()).eq(["package", "module", "Unite", "Variant"].into_iter())
    ));
    assert!(matches!(
        &arms[1].pattern,
        HirPattern::Name(path) if path.parts().iter().map(|part| part.as_str()).eq(["Variant"].into_iter())
    ));
    assert!(matches!(
        &arms[2].pattern,
        HirPattern::Variable(identifier) if identifier.name.as_str() == "var"
    ));
}

#[test]
fn resolves_single_segment_name_pattern_into_type_when_scrutinee_type_matches() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2979 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: Payload) -> bool {
    return match value {
        case Payload:
            true
        case Variant:
            false
        else:
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
        HirPattern::Type(path) if path.parts().iter().map(|part| part.as_str()).eq(["Payload"].into_iter())
    ));
    assert!(matches!(
        &arms[1].pattern,
        HirPattern::Name(path) if path.parts().iter().map(|part| part.as_str()).eq(["Variant"].into_iter())
    ));
}

#[test]
fn accepts_qualified_lowercase_name_patterns_at_hir_validation() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 2980 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: Payload) -> bool {
    return match value {
        case package::module::value:
            true
        else:
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
        HirPattern::Name(path) if path.parts().iter().map(|part| part.as_str()).eq(["package", "module", "value"].into_iter())
    ));
}

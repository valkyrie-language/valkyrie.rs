use valkyrie_parser::{
    ast::{DeclarationStatement, LiteralExpression, PatternExpression, Statement, TermExpression},
    AstParser,
};

#[test]
fn parses_match_literal_variable_and_or_patterns() {
    let source = r#"micro main(value: i64) -> bool {
    match value {
        case 1 | 2:
            true
        case n if n > 0:
            false
        case _:
            false
    };
    return false;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected body");
    let Statement::Expr { expression: TermExpression::Match { arms, .. }, .. } = &body.statements[0]
    else {
        panic!("expected match statement");
    };

    assert!(matches!(
        &arms[0].pattern,
        Some(PatternExpression::Or(pattern))
            if matches!(
                pattern.patterns.as_slice(),
                [
                    PatternExpression::Literal { literal: LiteralExpression::Integer(first), .. },
                    PatternExpression::Literal { literal: LiteralExpression::Integer(second), .. }
                ] if first == "1" && second == "2"
            )
    ));
    assert!(matches!(
        &arms[1].pattern,
        Some(PatternExpression::Variable { name, .. }) if name == "n"
    ));
    assert!(arms[1].guard.is_some());
    assert!(matches!(&arms[2].pattern, Some(PatternExpression::Wildcard { .. })));
}

#[test]
fn parses_match_tuple_patterns() {
    let source = r#"micro main(pair: (i64, i64)) -> bool {
    match pair {
        case (left, 1) if left > 0:
            true
        case ((x, y), z):
            false
        else:
            false
    };
    return false;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected body");
    let Statement::Expr { expression: TermExpression::Match { arms, .. }, .. } = &body.statements[0]
    else {
        panic!("expected match statement");
    };

    assert!(matches!(
        &arms[0].pattern,
        Some(PatternExpression::Tuple(pattern))
            if matches!(
                pattern.items.as_slice(),
                [
                    PatternExpression::Variable { name, .. },
                    PatternExpression::Literal { literal: LiteralExpression::Integer(value), .. }
                ] if name == "left" && value == "1"
            )
    ));
    assert!(arms[0].guard.is_some());
    assert!(matches!(
        &arms[1].pattern,
        Some(PatternExpression::Tuple(pattern))
            if matches!(
                pattern.items.as_slice(),
                [
                    PatternExpression::Tuple(inner),
                    PatternExpression::Variable { name, .. }
                ]
                if matches!(
                    inner.items.as_slice(),
                    [
                        PatternExpression::Variable { name: first, .. },
                        PatternExpression::Variable { name: second, .. }
                    ] if first == "x" && second == "y"
                ) && name == "z"
            )
    ));
}

#[test]
fn parses_case_statement_source_entry() {
    let source = r#"micro main() -> bool {
    case value {
        case 1:
            fallthrough
        else:
            false
    };
    return false;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected body");
    let Statement::Expr { expression: TermExpression::Case { arms, .. }, .. } = &body.statements[0]
    else {
        panic!("expected case statement");
    };
    assert_eq!(arms.len(), 2);
    assert!(matches!(
        &arms[0].pattern,
        Some(PatternExpression::Literal { literal: LiteralExpression::Integer(value), .. }) if value == "1"
    ));
}

#[test]
fn parses_nested_constructor_and_object_match_patterns() {
    let source = r#"micro main(value: Wrapper) -> bool {
    match value {
        case Wrapper(inner, Pair(left, right)):
            true
        case Wrapper { inner: Some(result), fallback }:
            false
        else:
            false
    };
    return false;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected body");
    let Statement::Expr { expression: TermExpression::Match { arms, .. }, .. } = &body.statements[0]
    else {
        panic!("expected match statement");
    };

    assert!(matches!(
        &arms[0].pattern,
        Some(PatternExpression::Extract(pattern))
            if pattern.name.parts == vec!["Wrapper".to_string()]
                && matches!(
                    pattern.fields.as_slice(),
                    [
                        PatternExpression::Variable { name: first, .. },
                        PatternExpression::Extract(nested)
                    ]
                    if first == "inner"
                        && nested.name.parts == vec!["Pair".to_string()]
                        && matches!(
                            nested.fields.as_slice(),
                            [
                                PatternExpression::Variable { name: left, .. },
                                PatternExpression::Variable { name: right, .. }
                            ] if left == "left" && right == "right"
                        )
                )
    ));
    assert!(matches!(
        &arms[1].pattern,
        Some(PatternExpression::Object(pattern))
            if pattern.name.as_ref().is_some_and(|name| name.parts == vec!["Wrapper".to_string()])
                && pattern.fields.len() == 2
                && matches!(
                    &pattern.fields[0],
                    valkyrie_parser::ast::MatchObjectField { name: field_name, pattern: PatternExpression::Extract(extract), .. }
                        if field_name == "inner"
                            && extract.name.parts == vec!["Some".to_string()]
                            && matches!(extract.fields.as_slice(), [PatternExpression::Variable { name, .. }] if name == "result")
                )
                && matches!(
                    &pattern.fields[1],
                    valkyrie_parser::ast::MatchObjectField { name: field_name, pattern: PatternExpression::Variable { name, .. }, .. }
                        if field_name == "fallback" && name == "fallback"
                )
    ));
}

#[test]
fn parses_range_array_rest_and_typed_bind_patterns() {
    let source = r#"micro main(value: Vector) -> bool {
    match value {
        case [head, ..tail, last]:
            true
        case numbers as Vector:
            false
        case _:
            false
    };
    return false;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected body");
    let Statement::Expr { expression: TermExpression::Match { arms, .. }, .. } = &body.statements[0]
    else {
        panic!("expected match statement");
    };

    assert!(matches!(
        &arms[0].pattern,
        Some(PatternExpression::Array(pattern))
            if matches!(pattern.prefix.as_slice(), [PatternExpression::Variable { name, .. }] if name == "head")
                && pattern.rest.as_ref().is_some_and(|rest| rest.as_str() == "tail")
                && matches!(pattern.suffix.as_slice(), [PatternExpression::Variable { name, .. }] if name == "last")
    ));
    assert!(matches!(
        &arms[1].pattern,
        Some(PatternExpression::TypedBind { name, ty, .. })
            if name == "numbers" && ty.parts == vec!["Vector".to_string()]
    ));
}

#[test]
fn parses_range_and_object_rest_patterns_with_guards() {
    let source = r#"micro main(value: Sample) -> bool {
    match value {
        case 1..=10 if true:
            true
        case Wrapper { inner, ...rest }:
            false
        case ..<100:
            false
        else:
            false
    };
    return false;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected body");
    let Statement::Expr { expression: TermExpression::Match { arms, .. }, .. } = &body.statements[0]
    else {
        panic!("expected match statement");
    };

    assert!(matches!(
        &arms[0].pattern,
        Some(PatternExpression::Range { start: Some(LiteralExpression::Integer(start)), end: Some(LiteralExpression::Integer(end)), inclusive_end: true, .. })
            if start == "1" && end == "10"
    ));
    assert!(arms[0].guard.is_some());
    assert!(matches!(
        &arms[1].pattern,
        Some(PatternExpression::Object(pattern))
            if pattern.name.as_ref().is_some_and(|name| name.parts == vec!["Wrapper".to_string()])
                && pattern.rest.as_ref().is_some_and(|rest| rest.as_str() == "rest")
                && matches!(pattern.fields.as_slice(), [valkyrie_parser::ast::MatchObjectField { name: field_name, pattern: PatternExpression::Variable { name, .. }, .. }] if field_name == "inner" && name == "inner")
    ));
    assert!(matches!(
        &arms[2].pattern,
        Some(PatternExpression::Range { start: None, end: Some(LiteralExpression::Integer(end)), inclusive_end: false, .. }) if end == "100"
    ));
}

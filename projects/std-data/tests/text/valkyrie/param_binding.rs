use std_data::text::valkyrie::{AstParser, FunctionStatement, ParameterBindingKind, RootStatement, TermExpression};

fn first_micro_params(source: &str) -> Vec<std_data::text::valkyrie::FunctionParameter> {
    let root = AstParser::parse_root(source).expect("parse");
    match &root.statements[0] {
        RootStatement::Function(function) => function.params.clone(),
        other => panic!("expected micro function, got {other:?}"),
    }
}

fn first_call_arguments(source: &str) -> Vec<std_data::text::valkyrie::TermCallArgument> {
    let root = AstParser::parse_root(source).expect("parse");
    let RootStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("body");
    let expression = body
        .tail_expression
        .as_ref()
        .or_else(|| match body.statements.first()? {
            FunctionStatement::Term { expression, .. } => Some(expression),
            _ => None,
        })
        .expect("expression");
    let TermExpression::Call(call) = expression
    else {
        panic!("expected call expression");
    };
    call.args.arguments.clone()
}

#[test]
fn assigns_binding_kinds_for_lt_gt_markers() {
    let params = first_micro_params(
        r#"
micro f(a, <, b, >, c) {
    c
}
"#,
    );
    assert_eq!(params.len(), 3);
    assert_eq!(params[0].binding_kind, ParameterBindingKind::PositionalOnly);
    assert_eq!(params[1].binding_kind, ParameterBindingKind::PositionalOrKeyword);
    assert_eq!(params[2].binding_kind, ParameterBindingKind::KeywordOnly);
}

#[test]
fn assigns_binding_kinds_for_lt_only() {
    let params = first_micro_params(
        r#"
micro f(a, <, b, c) {
    b
}
"#,
    );
    assert_eq!(params[0].binding_kind, ParameterBindingKind::PositionalOnly);
    assert_eq!(params[1].binding_kind, ParameterBindingKind::PositionalOrKeyword);
    assert_eq!(params[2].binding_kind, ParameterBindingKind::PositionalOrKeyword);
}

#[test]
fn defaults_to_positional_or_keyword_without_markers() {
    let params = first_micro_params(
        r#"
micro f(a, b) {
    a
}
"#,
    );
    assert!(params.iter().all(|param| param.binding_kind == ParameterBindingKind::PositionalOrKeyword));
}

#[test]
fn rejects_duplicate_lt_marker() {
    let err = AstParser::parse_root(
        r#"
micro f(<, <, a) {
    a
}
"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("at most one '<'"));
}

#[test]
fn rejects_gt_without_lt() {
    let err = AstParser::parse_root(
        r#"
micro f(>, a) {
    a
}
"#,
    )
    .unwrap_err();
    assert!(err.to_string().contains("preceding '<'"));
}

#[test]
fn parses_keyword_call_arguments() {
    let args = first_call_arguments(
        r#"
micro main() {
    f(1, b = 2, c = 3)
}
"#,
    );
    assert_eq!(args.len(), 3);
    assert!(args[0].key.is_none());
    assert_eq!(args[1].key.as_deref(), Some("b"));
    assert_eq!(args[2].key.as_deref(), Some("c"));
}

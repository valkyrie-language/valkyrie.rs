use std_data::text::valkyrie::{
    AstParser, ParseError,
    ast::{LiteralExpression, StringSegment, TermExpression},
};

#[test]
fn parses_triple_quoted_interpolation() {
    let root = AstParser::parse_root(r#"micro main(name: utf8) { let s = """Hello, {name}!"""; }"#).expect("parse");
    let function = match &root.statements[0] {
        std_data::text::valkyrie::RootStatement::Function(function) => function,
        _ => panic!("expected function"),
    };
    let body = function.body.as_ref().expect("body");
    let expr = match &body.statements[0] {
        std_data::text::valkyrie::FunctionStatement::Let(let_stmt) => let_stmt.initializer.as_ref().expect("initializer"),
        _ => panic!("expected let"),
    };
    let TermExpression::Literal { literal: LiteralExpression::String(string), .. } = expr
    else {
        panic!("expected string literal");
    };
    assert_eq!(string.quote_count, 3);
    assert!(matches!(
        string.segments.as_slice(),
        [
            StringSegment::Text(prefix),
            StringSegment::Interpolation { .. },
            StringSegment::Text(suffix)
        ] if prefix == "Hello, " && suffix == "!"
    ));
}

#[test]
fn rejects_quote_escape_in_cooked_string() {
    let err = AstParser::parse_root(r#"micro main() { let s = "\q"; }"#).expect_err("invalid escape");
    assert!(matches!(err, ParseError::Invalid { .. }));
}

#[test]
fn parses_escaped_braces_as_literal_text() {
    let root = AstParser::parse_root(r#"micro main() { let s = "\{name\}"; }"#).expect("parse");
    let function = match &root.statements[0] {
        std_data::text::valkyrie::RootStatement::Function(function) => function,
        _ => panic!("expected function"),
    };
    let body = function.body.as_ref().expect("body");
    let expr = match &body.statements[0] {
        std_data::text::valkyrie::FunctionStatement::Let(let_stmt) => let_stmt.initializer.as_ref().expect("initializer"),
        _ => panic!("expected let"),
    };
    let TermExpression::Literal { literal: LiteralExpression::String(string), .. } = expr
    else {
        panic!("expected string literal");
    };
    assert!(matches!(string.segments.as_slice(), [StringSegment::Text(text)] if text == "{name}"));
}

#[test]
fn parses_empty_double_quoted_string() {
    let root = AstParser::parse_root(r#"micro main() { let s = ""; }"#).expect("parse");
    let function = match &root.statements[0] {
        std_data::text::valkyrie::RootStatement::Function(function) => function,
        _ => panic!("expected function"),
    };
    let body = function.body.as_ref().expect("body");
    let expr = match &body.statements[0] {
        std_data::text::valkyrie::FunctionStatement::Let(let_stmt) => let_stmt.initializer.as_ref().expect("initializer"),
        _ => panic!("expected let"),
    };
    let TermExpression::Literal { literal: LiteralExpression::String(string), .. } = expr
    else {
        panic!("expected string literal");
    };
    assert_eq!(string.quote_count, 2);
    assert!(matches!(string.segments.as_slice(), [StringSegment::Text(text)] if text.is_empty()));
}

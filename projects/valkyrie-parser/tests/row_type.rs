use valkyrie_parser::{
    ast::{RootStatement, TypeExpression},
    AstParser,
};

#[test]
fn parses_anonymous_row_parameter_type() {
    let source = r#"
micro invoke_g(value: { g() -> unit, clone() -> Self }) {
    value.g();
}
"#;
    let root = AstParser::parse_root(source).expect("解析匿名 row 参数失败");

    let RootStatement::Function(function) = &root.statements[0]
    else {
        panic!("期望解析出函数声明");
    };

    let Some(TypeExpression::Row { methods, .. }) = function.params[0].parameter_type.as_ref()
    else {
        panic!("期望参数类型为匿名 row");
    };

    assert_eq!(methods.len(), 2);
    assert_eq!(methods[0].name.as_str(), "g");
    assert_eq!(methods[1].name.as_str(), "clone");
}

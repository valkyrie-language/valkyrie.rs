use valkyrie_parser::AstParser;

/// 验证解析器能处理 `Iterator<Item = T>` 形式的关联类型绑定。
#[test]
fn parses_associated_type_binding() {
    let source = "micro test<I>(x: I) -> i32 where I: Iterator<Item = T> { return 0; }";
    let result = AstParser::parse_root(source);
    assert!(result.is_ok(), "解析关联类型绑定失败：{:?}", result.err());
}

/// 验证解析器能处理 `Iterator<Item=K>` 形式（无空格）的关联类型绑定。
#[test]
fn parses_associated_type_binding_no_space() {
    let source = "micro keys(self) -> Iterator<Item=K> { return self; }";
    let result = AstParser::parse_root(source);
    assert!(result.is_ok(), "解析无空格关联类型绑定失败：{:?}", result.err());
}

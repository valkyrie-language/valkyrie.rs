use vcc_data::text::valkyrie::AstParser;

#[test]
fn parses_export_case_colon_named_attribute_argument() {
    let source = r#"
[export(case: "camelCase")]
micro two_sum(): i64 {
    return 0
}
"#;
    AstParser::parse_root(source).expect("parse export case attribute");
}

#[test]
fn parses_export_case_equal_named_attribute_argument() {
    let source = r#"
[export(case = "camelCase")]
micro two_sum(): i64 {
    return 0
}
"#;
    AstParser::parse_root(source).expect("parse export case equal attribute");
}

#[test]
fn parses_export_partition_path_without_named_argument_confusion() {
    let source = r#"
namespace demo.export;

[export(unity.runtime)]
micro run_host(): i64 {
    return 0
}
"#;
    AstParser::parse_root(source).expect("parse export partition path");
}

use nyar_backend_wasi::WatDocument;

#[test]
fn parses_named_module_fields() {
    let source = r#"
    (module $demo
      (func $main)
      (export "main" (func $main))
    )
    "#;

    let document = WatDocument::parse(source).unwrap();
    assert_eq!(document.module_name.as_deref(), Some("$demo"));
    assert_eq!(document.fields.len(), 2);
}

#[test]
fn formats_module_back_to_text() {
    let mut document = WatDocument::new();
    document.module_name = Some("$demo".to_string());
    document.push_field("(func $main)");

    let text = document.to_text();
    assert!(text.contains("(module $demo"));
    assert!(text.contains("(func $main)"));
}

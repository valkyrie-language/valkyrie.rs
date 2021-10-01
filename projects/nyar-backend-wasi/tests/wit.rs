use nyar_wasi_backend::WitPackage;

#[test]
fn parses_wit_package_with_interfaces() {
    let source = r#"
    package demo:math@0.1.0;

    interface calculator {
      add: func(lhs: s32, rhs: s32) -> s32;
      sub: func(lhs: s32, rhs: s32) -> s32;
    }
    "#;

    let package = WitPackage::parse(source).unwrap();
    assert_eq!(package.package_name, "demo:math@0.1.0");
    assert_eq!(package.interfaces.len(), 1);
    assert_eq!(package.interfaces[0].functions.len(), 2);
}

#[test]
fn formats_wit_package_back_to_text() {
    let mut package = WitPackage::new("demo:math");
    package.push_interface("calculator", vec!["add: func(lhs: s32, rhs: s32) -> s32".to_string()]);

    let text = package.to_text();
    assert!(text.contains("package demo:math;"));
    assert!(text.contains("interface calculator {"));
    assert!(text.contains("add: func(lhs: s32, rhs: s32) -> s32;"));
}

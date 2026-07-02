use std_data::text::valkyrie::{AstParser, RootStatement, SumTypeKind};

#[test]
fn parses_enums_with_explicit_discriminators() {
    let source = "enums Color { RED = 2, GREEN = 4, BLUE = 6 }";
    let root = AstParser::parse_root(source).expect("parse enums with discriminators");
    let RootStatement::Unite(unite) = &root.statements[0]
    else {
        panic!("expected Unite declaration");
    };
    assert_eq!(unite.kind, SumTypeKind::Enum);
    assert_eq!(unite.variants.len(), 3);
    assert_eq!(unite.variants[0].name.as_str(), "RED");
    assert!(unite.variants[0].value.is_some());
    assert_eq!(unite.variants[1].name.as_str(), "GREEN");
    assert!(unite.variants[1].value.is_some());
    assert_eq!(unite.variants[2].name.as_str(), "BLUE");
    assert!(unite.variants[2].value.is_some());
}

#[test]
fn parses_enums_with_mixed_discriminators() {
    let source = "enums Status { Active = 0, Inactive }";
    let root = AstParser::parse_root(source).expect("parse enums with mixed discriminators");
    let RootStatement::Unite(unite) = &root.statements[0]
    else {
        panic!("expected Unite declaration");
    };
    assert_eq!(unite.variants.len(), 2);
    assert!(unite.variants[0].value.is_some());
    assert!(unite.variants[1].value.is_none());
}

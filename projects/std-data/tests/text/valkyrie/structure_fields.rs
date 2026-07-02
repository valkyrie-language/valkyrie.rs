use std_data::text::valkyrie::{AstParser, ClassLikeKind, RootStatement};

#[test]
fn parses_soft_keyword_structure_fields() {
    let source = r#"
structure BootstrapMirBlock {
    id: i32
    predecessors: [i32]
    sealed: bool
    static: utf8
    open: bool
}
"#;
    let root = AstParser::parse_root(source).expect("parse structure with soft-keyword fields");
    let RootStatement::Class(class) = &root.statements[0]
    else {
        panic!("expected Class/structure declaration");
    };
    assert_eq!(class.kind, ClassLikeKind::Structure);
    let field_names: Vec<_> = class.body.fields.iter().map(|field| field.name.as_str()).collect();
    assert_eq!(field_names, vec!["id", "predecessors", "sealed", "static", "open"]);
}

#[test]
fn still_parses_sealed_class_modifier() {
    let source = "sealed class Shape { }";
    let root = AstParser::parse_root(source).expect("parse sealed class modifier");
    let RootStatement::Class(class) = &root.statements[0]
    else {
        panic!("expected Class declaration");
    };
    assert!(class.annotations.modifiers.iter().any(|modifier| modifier.as_str() == "sealed"));
}

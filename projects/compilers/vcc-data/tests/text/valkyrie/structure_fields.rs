use vcc_data::text::valkyrie::{AstParser, ClassLikeKind, RootStatement, TypeExpression};

#[test]
fn preserves_method_generic_parameters_and_where_constraints() {
    let root = AstParser::parse_root(r#"
class Collector {
    micro collect<I, T>(iter: I) -> T where I: Iterator<Item = T> { }
}
"#).expect("generic method");
    let RootStatement::Class(class) = &root.statements[0] else {
        panic!("expected class");
    };
    let method = &class.body.methods[0];
    assert_eq!(method.generic_parameters.len(), 2);
    assert_eq!(method.generic_parameters[0].name.as_str(), "I");
    assert_eq!(method.generic_parameters[1].name.as_str(), "T");
    assert_eq!(method.where_constraints.len(), 1);
    assert_eq!(method.where_constraints[0].bounds.len(), 1);
    let TypeExpression::Path(bound) = &method.where_constraints[0].bounds[0] else {
        panic!("expected structured trait bound");
    };
    assert_eq!(bound.name.parts, vec!["Iterator"]);
    assert_eq!(bound.arguments.len(), 1);
    let TypeExpression::Associated { name, ty, .. } = &bound.arguments[0] else {
        panic!("expected associated type equation");
    };
    assert_eq!(name.as_str(), "Item");
    let TypeExpression::Path(binding) = ty.as_ref() else {
        panic!("expected associated type binding");
    };
    assert_eq!(binding.name.parts, vec!["T"]);
}

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

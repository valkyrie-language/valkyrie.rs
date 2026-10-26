use oak_core::{Builder, parser::ParseSession, source::SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage, ast::StatementNode};

fn parse_items(source: &str) -> Vec<StatementNode> {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let text = SourceText::new(source);
    let mut session = ParseSession::default();
    let output = builder.build(&text, &[], &mut session);
    output.result.expect("fixture must parse").items
}

#[test]
fn dotted_namespace_declaration_parses() {
    let items = parse_items("namespace leetcode.two_sum;\n");
    let namespace = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Namespace(ns) => Some(ns),
            _ => None,
        })
        .expect("namespace item");
    assert_eq!(namespace.name.parts.len(), 2);
    assert_eq!(namespace.name.parts[0].name, "leetcode");
    assert_eq!(namespace.name.parts[1].name, "two_sum");
}

#[test]
fn wasm_attribute_with_two_string_arguments_parses() {
    let items = parse_items(
        r#"[wasm("env", "utf8_concat")]
private micro __host_utf8_concat(a: Utf8Text, b: Utf8Text): Utf8Text
"#,
    );
    let micro = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) => Some(micro),
            _ => None,
        })
        .expect("micro item");
    let wasm = micro.annotations.iter().find(|annotation| annotation.name.name == "wasm").expect("wasm attribute");
    assert_eq!(wasm.args.len(), 2);
}

#[test]
fn keyword_micro_name_yield_parses() {
    let items = parse_items("micro yield(): unit { delay(0) }\n");
    let micro = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) => Some(micro),
            _ => None,
        })
        .expect("micro item");
    assert_eq!(micro.name.name, "yield");
}

#[test]
fn signature_only_micro_declaration_parses() {
    let items = parse_items("private micro __host_utf8_length(s: Utf8Text): i32\n");
    let micro = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) => Some(micro),
            _ => None,
        })
        .expect("micro item");
    assert_eq!(micro.name.name, "__host_utf8_length");
    assert!(micro.body.statements.is_empty());
}

#[test]
fn imply_associated_type_default_parses() {
    let items = parse_items(
        r#"imply Promise<T>: Future {
    type Output = T
    micro poll(mut self): bool { return self._done }
}"#,
    );
    let imply = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Imply(imply) => Some(imply),
            _ => None,
        })
        .expect("imply item");
    assert_eq!(imply.methods.len(), 1);
}

#[test]
fn trait_associated_type_bound_parses() {
    let items = parse_items(
        r#"trait IntoIterator {
    type Item;
    type Iter: Iterator<Item=Self::Item>;
}"#,
    );
    let trait_node = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Trait(trait_node) => Some(trait_node),
            _ => None,
        })
        .expect("trait item");
    assert_eq!(trait_node.body.associated_types.len(), 2);
}

#[test]
fn bracket_export_attribute_with_string_argument_parses() {
    let items = parse_items(
        r#"namespace leetcode.two_sum;

[export(case: "camelCase")]
micro two_sum() {}"#,
    );
    assert_eq!(items.len(), 2);
    let micro = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) => Some(micro),
            _ => None,
        })
        .expect("micro item");
    assert_eq!(micro.name.name, "two_sum");
    assert_eq!(micro.annotations.len(), 1);
    assert_eq!(micro.annotations[0].name.name, "export");
    assert_eq!(micro.annotations[0].args.len(), 1);
    assert_eq!(micro.annotations[0].args[0].key.as_ref().map(|k| k.name.as_str()), Some("case"));
}

#[test]
fn selective_using_import_list_parses() {
    let items = parse_items("using core::types::{Option, Some, None, option_none};\n");
    let using = items
        .iter()
        .find_map(|item| match item {
            StatementNode::Using(using) => Some(using.as_ref()),
            _ => None,
        })
        .expect("using item");
    assert_eq!(using.path.parts.len(), 2);
    assert_eq!(using.path.parts[0].name, "core");
    assert_eq!(using.path.parts[1].name, "types");
    assert_eq!(using.imports.len(), 4);
    assert_eq!(using.imports[0].name, "Option");
    assert_eq!(using.imports[3].name, "option_none");
}

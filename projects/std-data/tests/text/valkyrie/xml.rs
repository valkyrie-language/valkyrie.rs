use std_data::text::valkyrie::{
    AstParser, TermExpression,
    tgrammar::TgNode,
    xml::{XgAttrValue, XgNode, XgTextPart, parse_xgrammar_markup, parse_xgrammar_template, parse_xgrammar_with_meta},
};

#[test]
fn parses_inline_xgrammar_markup() {
    let (nodes, consumed) = parse_xgrammar_markup(r#"<div class="box">{count}</div>"#).expect("parse");
    assert_eq!(consumed, r#"<div class="box">{count}</div>"#.len());
    assert_eq!(nodes.len(), 1);
    let XgNode::Element(element) = &nodes[0]
    else {
        panic!("expected element");
    };
    assert_eq!(element.tag, "div");
    assert_eq!(element.attrs[0], ("class".to_string(), XgAttrValue::Literal("box".to_string())));
    assert!(matches!(
        element.children[0],
        XgNode::Text { ref parts, .. } if matches!(&parts[0], XgTextPart::Expression(expr) if expr == "count")
    ));
}

#[test]
fn parses_self_closing_element() {
    let nodes = parse_xgrammar_template("<br/>").expect("parse");
    assert_eq!(nodes.len(), 1);
    let XgNode::Element(element) = &nodes[0]
    else {
        panic!("expected element");
    };
    assert!(element.self_closing);
}

#[test]
fn parses_nested_elements() {
    let (nodes, _) = parse_xgrammar_markup("<div><button>{count}</button></div>").expect("parse nested");
    assert_eq!(nodes.len(), 1);
    let XgNode::Element(div) = &nodes[0]
    else {
        panic!("expected div");
    };
    assert_eq!(div.tag, "div");
    assert_eq!(div.children.len(), 1);
    let XgNode::Element(button) = &div.children[0]
    else {
        panic!("expected button child");
    };
    assert_eq!(button.tag, "button");
}

#[test]
fn valkyrie_parser_recognizes_xml_markup_expression() {
    let root = AstParser::parse_root(
        r#"
widget Counter {
    micro view() {
        <button :on_click="increment">{count}</button>
    }
}
"#,
    )
    .expect("parse vx widget");
    let widget = root
        .statements
        .iter()
        .find_map(|stmt| match stmt {
            std_data::text::valkyrie::RootStatement::Class(class) if class.kind == std_data::text::valkyrie::ClassLikeKind::Widget => {
                Some(class)
            }
            _ => None,
        })
        .expect("widget");
    let view = widget.body.methods.iter().find(|method| method.name.name.as_str() == "view").expect("view method");
    let body = view.body.as_ref().expect("view body");
    let tail = body.tail_expression.as_ref().expect("xml tail");
    assert!(matches!(tail, TermExpression::XmlMarkup { .. }));
}

#[test]
fn parses_xgrammar_with_meta_directive() {
    let (nodes, consumed) = parse_xgrammar_with_meta("<% if s %><div>{x}</div><% end %>").expect("parse");
    assert_eq!(consumed, "<% if s %><div>{x}</div><% end %>".len());
    assert_eq!(nodes.len(), 1);
    let XgNode::Meta { nodes: meta_nodes, .. } = &nodes[0]
    else {
        panic!("expected meta");
    };
    assert!(matches!(&meta_nodes[0], TgNode::If(_)));
}

#[test]
fn parse_vx_root_restores_meta_in_view_body() {
    let source = r#"
widget Counter {
    micro view() {
        <% if show %><div>{count}</div><% end %>
    }
}
"#;
    let root = AstParser::parse_vx_root(source).expect("parse vx");
    let widget = root
        .statements
        .iter()
        .find_map(|stmt| match stmt {
            std_data::text::valkyrie::RootStatement::Class(class) if class.kind == std_data::text::valkyrie::ClassLikeKind::Widget => {
                Some(class)
            }
            _ => None,
        })
        .expect("widget");
    let view = widget.body.methods.iter().find(|method| method.name.name.as_str() == "view").expect("view");
    let tail = view.body.as_ref().expect("body").tail_expression.as_ref().expect("tail");
    let TermExpression::XmlMarkup { nodes, .. } = tail
    else {
        panic!("expected xml markup");
    };
    assert!(matches!(&nodes[0], XgNode::Meta { .. }));
}

#[test]
fn parses_colon_binding_attribute_as_expression() {
    let (nodes, _) = parse_xgrammar_markup(r#"<button :on_click="increment">{count}</button>"#).expect("parse binding");
    let XgNode::Element(button) = &nodes[0]
    else {
        panic!("expected button");
    };
    assert_eq!(button.attrs[0], ("on_click".to_string(), XgAttrValue::Expression("increment".to_string())));
}

#[test]
fn parses_colon_binding_with_spaces_around_equals() {
    let (nodes, _) = parse_xgrammar_markup(r#"<button :on_click = "increment">{count}</button>"#).expect("parse spaced binding");
    let XgNode::Element(button) = &nodes[0]
    else {
        panic!("expected button");
    };
    assert_eq!(button.attrs[0], ("on_click".to_string(), XgAttrValue::Expression("increment".to_string())));
}

#[test]
fn rejects_bare_colon_binding_without_quotes() {
    let error = parse_xgrammar_markup(r#"<button :on_click=increment />"#).expect_err("bare binding");
    assert!(error.message.contains("quoted"));
}

#[test]
fn rejects_braced_attribute_values() {
    let error = parse_xgrammar_markup(r#"<button on_click={increment} />"#).expect_err("braced attr");
    assert!(error.message.contains(":attr"));
}

#[test]
fn rejects_bare_static_attribute_values() {
    let error = parse_xgrammar_markup(r#"<div class=box />"#).expect_err("bare class");
    assert!(error.message.contains("quoted"));
}

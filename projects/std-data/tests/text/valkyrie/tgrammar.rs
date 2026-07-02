use std_data::text::valkyrie::tgrammar::{TgNode, TgTextPart, parse_tgrammar_fragment, parse_tgrammar_template};

#[test]
fn parses_stmt_directive() {
    let nodes = parse_tgrammar_template("<% let x = 1 %>").expect("parse");
    assert_eq!(nodes.len(), 1);
    assert!(matches!(
        &nodes[0],
        TgNode::Stmt { body, .. } if body == "let x = 1"
    ));
}

#[test]
fn parses_text_and_expression() {
    let nodes = parse_tgrammar_template("Hello {name}!").expect("parse");
    assert_eq!(nodes.len(), 1);
    let TgNode::Text { parts, .. } = &nodes[0]
    else {
        panic!("expected text");
    };
    assert_eq!(parts.len(), 3);
    assert!(matches!(&parts[0], TgTextPart::Static(s) if s == "Hello "));
    assert!(matches!(&parts[1], TgTextPart::Expression(s) if s == "name"));
    assert!(matches!(&parts[2], TgTextPart::Static(s) if s == "!"));
}

#[test]
fn parses_if_block_with_end() {
    let nodes = parse_tgrammar_template("<% if show %>yes<% end %>").expect("parse");
    assert_eq!(nodes.len(), 1);
    let TgNode::If(if_block) = &nodes[0]
    else {
        panic!("expected if");
    };
    assert_eq!(if_block.arms.len(), 1);
    assert_eq!(if_block.arms[0].condition.as_deref(), Some("show"));
    assert_eq!(if_block.arms[0].body.len(), 1);
    assert!(matches!(
        &if_block.arms[0].body[0],
        TgNode::Text { parts, .. }
            if matches!(&parts[0], TgTextPart::Static(s) if s == "yes")
    ));
}

#[test]
fn parses_if_else_if_else() {
    let source = "<% if a %>A<% else if b %>B<% else %>C<% end %>";
    let nodes = parse_tgrammar_template(source).expect("parse");
    let TgNode::If(if_block) = &nodes[0]
    else {
        panic!("expected if");
    };
    assert_eq!(if_block.arms.len(), 3);
    assert_eq!(if_block.arms[0].condition.as_deref(), Some("a"));
    assert_eq!(if_block.arms[1].condition.as_deref(), Some("b"));
    assert!(if_block.arms[2].condition.is_none());
}

#[test]
fn parses_loop_block() {
    let nodes = parse_tgrammar_template("<% loop i in items %>item {i}<% end %>").expect("parse");
    let TgNode::Loop(loop_block) = &nodes[0]
    else {
        panic!("expected loop");
    };
    assert_eq!(loop_block.header, "i in items");
    assert_eq!(loop_block.body.len(), 1);
}

#[test]
fn parses_match_with_case_and_else() {
    let source = "<% match v %><% case 1 %>one<% case 2 %>two<% else %>other<% end %>";
    let nodes = parse_tgrammar_template(source).expect("parse");
    let TgNode::Match(match_block) = &nodes[0]
    else {
        panic!("expected match");
    };
    assert_eq!(match_block.scrutinee, "v");
    assert_eq!(match_block.arms.len(), 3);
    assert_eq!(match_block.arms[0].pattern.as_deref(), Some("1"));
    assert_eq!(match_block.arms[2].pattern, None);
}

#[test]
fn parses_nested_if_inside_loop() {
    let source = "<% loop x in xs %><% if x %>ok<% end %><% end %>";
    let nodes = parse_tgrammar_template(source).expect("parse");
    let TgNode::Loop(loop_block) = &nodes[0]
    else {
        panic!("expected loop");
    };
    assert_eq!(loop_block.body.len(), 1);
    assert!(matches!(&loop_block.body[0], TgNode::If(_)));
}

#[test]
fn parses_template_comment() {
    let nodes = parse_tgrammar_template("a<# note #>b").expect("parse");
    assert_eq!(nodes.len(), 3);
    assert!(matches!(&nodes[0], TgNode::Text { .. }));
    assert!(matches!(&nodes[1], TgNode::Comment { .. }));
    assert!(matches!(&nodes[2], TgNode::Text { .. }));
}

#[test]
fn parses_fragment_for_xml_meta() {
    let (nodes, consumed) = parse_tgrammar_fragment("<% if show %>x<% end %><span>").expect("parse");
    assert_eq!(consumed, "<% if show %>x<% end %>".len());
    assert_eq!(nodes.len(), 1);
    assert!(matches!(&nodes[0], TgNode::If(_)));
}

#[test]
fn rejects_stray_end_at_root() {
    let err = parse_tgrammar_template("<% end %>").expect_err("should fail");
    assert!(err.message.contains("unexpected"));
}

#[test]
fn valkyrie_parser_recognizes_t_string_as_template_expression() {
    use std_data::text::valkyrie::{AstParser, RootStatement, TermExpression};

    let root = AstParser::parse_root(
        r#"
micro main() {
    t"<% if show %>yes<% end %>"
}
"#,
    )
    .expect("parse");
    let func = root
        .statements
        .iter()
        .find_map(|stmt| match stmt {
            RootStatement::Function(function) if function.name.name.as_str() == "main" => Some(function),
            _ => None,
        })
        .expect("main");
    let tail = func.body.as_ref().expect("body").tail_expression.as_ref().expect("tail");
    let TermExpression::Template { nodes, .. } = tail
    else {
        panic!("expected template");
    };
    assert_eq!(nodes.len(), 1);
    assert!(matches!(&nodes[0], TgNode::If(_)));
}

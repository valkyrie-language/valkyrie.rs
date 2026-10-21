use oak_core::{Builder, parser::ParseSession, source::SourceText};
use oak_valkyrie::{
    ValkyrieBuilder, ValkyrieLanguage,
    ast::{Statement, StatementNode},
};

fn build_source(source: &str) -> Result<oak_valkyrie::ast::ValkyrieRoot, oak_core::OakError> {
    let language = ValkyrieLanguage::default();
    ValkyrieBuilder::new(&language).build(&SourceText::new(source), &[], &mut ParseSession::default()).result
}

#[test]
fn assignment_keeps_precedence_and_right_associativity() {
    use oak_valkyrie::{ast::TermExpression, lexer::token_type::ValkyrieTokenType};
    let root = build_source("micro update() { left = right = 3 + 4 * 5 }").expect("赋值语法");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    assert_eq!(micro.body.statements.len(), 1);
    let Statement::Expression(expr) = &micro.body.statements[0]
    else {
        panic!("缺少表达式")
    };
    let TermExpression::Binary(outer) = &expr.expr
    else {
        panic!("缺少赋值")
    };
    assert_eq!(outer.operator, ValkyrieTokenType::Eq);
    let TermExpression::Binary(inner) = &outer.rhs
    else {
        panic!("赋值必须右结合")
    };
    assert_eq!(inner.operator, ValkyrieTokenType::Eq);
    let TermExpression::Binary(sum) = &inner.rhs
    else {
        panic!("缺少加法")
    };
    assert_eq!(sum.operator, ValkyrieTokenType::Plus);
    assert!(matches!(&sum.rhs, TermExpression::Binary(product) if product.operator == ValkyrieTokenType::Star));
}

#[test]
fn contextual_members_and_cardinal_indices_keep_ast() {
    use oak_valkyrie::ast::TermExpression;
    let root = build_source("micro read() { object.get(1); object.set(1, 2); values⁅0⁆; values::[0]; values[1] }").expect("成员与下标语法");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    assert_eq!(micro.body.statements.len(), 5);
    for index in [2, 3] {
        assert!(matches!(&micro.body.statements[index], Statement::Expression(expr) if matches!(expr.expr, TermExpression::Offset { .. })));
    }
    assert!(matches!(&micro.body.statements[4], Statement::Expression(expr) if matches!(expr.expr, TermExpression::Index { .. })));
}

#[test]
fn malformed_expressions_and_arguments_are_rejected() {
    for source in [
        "micro bad() { value = }",
        "micro bad() { call(1, =) }",
        "micro bad() { 3 = 4 }",
        "micro bad() { value⁅0 }",
        "micro bad() { value.() }",
    ] {
        assert!(build_source(source).is_err(), "不能吞掉错误: {source}");
    }
}

#[test]
fn array_declarations_keep_empty_initializer_and_ordered_elements() {
    use oak_valkyrie::ast::TermExpression;
    let root = build_source("micro arrays() { let empty = []; let values = [3, 7,]; }").expect("数组声明");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    assert_eq!(micro.body.statements.len(), 2);
    let Statement::Let(empty) = &micro.body.statements[0]
    else {
        panic!("空数组声明丢失")
    };
    assert!(matches!(&empty.expr, TermExpression::ArrayLiteral { items, .. } if items.is_empty()));
    let Statement::Let(values) = &micro.body.statements[1]
    else {
        panic!("数组声明丢失")
    };
    let TermExpression::ArrayLiteral { items, .. } = &values.expr
    else {
        panic!("数组未结构化")
    };
    assert_eq!(items.len(), 2);
    assert!(matches!(&items[0], TermExpression::IntegerLiteral { value, .. } if value == "3"));
    assert!(matches!(&items[1], TermExpression::IntegerLiteral { value, .. } if value == "7"));
    assert!(build_source("micro bad() { let items = [1,; }").is_err());
}

#[test]
fn iteration_loop_keeps_binding_source_and_body() {
    use oak_valkyrie::ast::TermExpression;
    let root = build_source("micro visit() { loop item in values { consume(item) } loop { break } }").expect("循环语法");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    assert_eq!(micro.body.statements.len(), 2);
    let Statement::Expression(first) = &micro.body.statements[0]
    else {
        panic!("循环丢失")
    };
    assert!(matches!(&first.expr, TermExpression::Loop { pattern: Some(_), condition: Some(_), body, .. } if body.statements.len() == 1));
    let Statement::Expression(second) = &micro.body.statements[1]
    else {
        panic!("循环丢失")
    };
    assert!(matches!(&second.expr, TermExpression::Loop { pattern: None, condition: None, .. }));
    assert!(build_source("micro bad() { loop item in { consume(item) } }").is_err());
}

#[test]
fn bitwise_operators_preserve_precedence_and_shift_operands() {
    use oak_valkyrie::{ast::TermExpression, lexer::token_type::ValkyrieTokenType};
    let root =
        build_source("micro bits() { left | middle & right == expected; word | (1 << bit); word ^ (word & (1 << bit)); }").expect("按位表达式");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    let Statement::Expression(first) = &micro.body.statements[0]
    else {
        panic!("缺少表达式")
    };
    let TermExpression::Binary(equal) = &first.expr
    else {
        panic!("缺少比较")
    };
    assert_eq!(equal.operator, ValkyrieTokenType::EqEq);
    let TermExpression::Binary(or) = &equal.lhs
    else {
        panic!("缺少位或")
    };
    assert_eq!(or.operator, ValkyrieTokenType::Pipe);
    assert!(matches!(&or.rhs, TermExpression::Binary(and) if and.operator == ValkyrieTokenType::Ampersand));
    assert!(build_source("micro bad() { value | } ").is_err());
    assert!(build_source("micro bad() { value << } ").is_err());
    assert!(build_source("micro bad() { value |> function } ").is_err());
}

#[test]
fn range_loop_and_anonymous_micro_keep_structured_children() {
    use oak_valkyrie::{ast::TermExpression, lexer::token_type::ValkyrieTokenType};
    let root = build_source("micro visit() { loop item in 0..count { consume(item) } apply(micro(value: i32) -> i32 { return value }) }")
        .expect("范围与匿名函数");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    let Statement::Expression(first) = &micro.body.statements[0]
    else {
        panic!("缺少循环")
    };
    let TermExpression::Loop { condition: Some(iterable), .. } = &first.expr
    else {
        panic!("缺少迭代源")
    };
    assert!(matches!(iterable.as_ref(), TermExpression::Binary(range) if range.operator == ValkyrieTokenType::DotDot));
    let Statement::Expression(second) = &micro.body.statements[1]
    else {
        panic!("缺少调用")
    };
    let TermExpression::ApplyCall { args, .. } = &second.expr
    else {
        panic!("缺少实参")
    };
    assert!(matches!(&args[0], TermExpression::Micro(lambda) if lambda.params.len() == 1 && lambda.return_type.is_some()));
    for source in ["micro bad() { values = 0.. }", "micro bad() { apply(micro(value: i32)) }"] {
        assert!(build_source(source).is_err(), "不能吞错: {source}");
    }
}

#[test]
fn number_tokens_do_not_consume_range_or_member_punctuation() {
    use oak_valkyrie::ast::TermExpression;
    let root = build_source("micro numbers() { 0xFF; 0b101; 0o17; 1.25; 1..3; }").expect("数值词法");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    assert_eq!(micro.body.statements.len(), 5);
    for (index, expected) in [(0, "0xFF"), (1, "0b101"), (2, "0o17")] {
        assert!(
            matches!(&micro.body.statements[index], Statement::Expression(expr) if matches!(&expr.expr, TermExpression::IntegerLiteral { value, .. } if value == expected))
        );
    }
    assert!(
        matches!(&micro.body.statements[3], Statement::Expression(expr) if matches!(&expr.expr, TermExpression::FloatLiteral { value, .. } if value == "1.25"))
    );
    for source in ["micro bad() { 0x; }", "micro bad() { 0b; }", "micro bad() { 1.2.3; }"] {
        assert!(build_source(source).is_err(), "非法字面量不能成功: {source}");
    }
}

#[test]
fn attribute_binding_paths_keep_all_segments() {
    let root = build_source("[host_provider(demo::write)] micro write(value: i32) -> i32 { return value }").expect("属性路径");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("缺少函数")
    };
    let oak_valkyrie::ast::TermExpression::NamePath(path) = &micro.annotations[0].args[0].value
    else {
        panic!("缺少结构路径")
    };
    assert_eq!(path.parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>(), ["demo", "write"]);
}

#[test]
fn micro_body_keeps_if_with_return_turbofish_call() {
    let source = r#"micro take(flag: bool) -> Option<i64> {
    if flag {
        return option_none::<i64>()
    }
    let value: i64 = 1
    return Some(value)
}"#;
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let text = SourceText::new(source);
    let mut session = ParseSession::default();
    let output = builder.build(&text, &[], &mut session);
    let root = output.result.expect("fixture must parse");
    let take = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) if micro.name.name == "take" => Some(micro),
            _ => None,
        })
        .expect("take micro");
    let has_if = take.body.statements.iter().any(|statement| {
        matches!(
            statement,
            Statement::Expression(expr) if matches!(expr.expr, oak_valkyrie::ast::TermExpression::If { .. })
        )
    });
    assert!(has_if, "take body must include if before let, got {} statements", take.body.statements.len());
}

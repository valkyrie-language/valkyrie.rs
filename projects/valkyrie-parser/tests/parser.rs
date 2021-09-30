use std::path::PathBuf;

use valkyrie_parser::{
    ast::PatternExpression, AstParser, DeclarationStatement, LiteralExpression, Statement, StringSegment, TermExpression, TypeExpression,
};

#[test]
fn parses_namespace_and_functions() {
    let source = r#"
namespace hello_world;

[main]
micro main(args: [utf8]) -> ExitCode {
    std::console::write_line("Hello");
    return ExitCode(0 as i32)
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    assert_eq!(root.statements.len(), 2);

    let DeclarationStatement::Namespace(namespace) = &root.statements[0]
    else {
        panic!("expected namespace");
    };
    assert_eq!(namespace.name.parts, vec!["hello_world".to_string()]);

    let DeclarationStatement::Function(function) = &root.statements[1]
    else {
        panic!("expected function");
    };
    assert_eq!(function.name, "main");
    let attributes: Vec<_> = function.annotations.attributes().filter_map(|attribute| attribute.name.parts.last()).cloned().collect();
    assert_eq!(attributes, vec!["main".to_string()]);
    assert!(matches!(
        function.return_type.as_ref(),
        Some(TypeExpression::Path(path)) if path.name.parts == vec!["ExitCode".to_string()]
    ));
    let body = function.body.as_ref().expect("expected function body");
    assert_eq!(body.statements.len(), 2);
    assert!(body.tail_expression.is_none());
    assert!(function.span.end >= function.span.start);
}

#[test]
fn parses_from_path() {
    let path = PathBuf::from("e:\\RiderProjects\\.tmp\\parser-smoke.v");
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(&path, "namespace demo;\n").unwrap();
    let root = AstParser::parse_path(&path).unwrap();
    assert_eq!(root.statements.len(), 1);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn parses_structured_attribute_arguments() {
    let source = r#"
[clr("System.Console", "System.Console", "WriteLine"), entry(kind = "cli")]
micro helper(message: utf16) {
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let attributes: Vec<_> = function.annotations.attributes().collect();
    assert_eq!(attributes.len(), 2);
    assert_eq!(attributes[0].name.parts, vec!["clr".to_string()]);
    assert!(matches!(
        attributes[0].arguments[0].value,
        TermExpression::Literal { literal: LiteralExpression::String(ref value), .. }
            if matches!(value.segments.as_slice(), [StringSegment::Text(text)] if text == "System.Console")
    ));
    assert!(matches!(
        attributes[0].arguments[1].value,
        TermExpression::Literal { literal: LiteralExpression::String(ref value), .. }
            if matches!(value.segments.as_slice(), [StringSegment::Text(text)] if text == "System.Console")
    ));
    assert!(matches!(
        attributes[0].arguments[2].value,
        TermExpression::Literal { literal: LiteralExpression::String(ref value), .. }
            if matches!(value.segments.as_slice(), [StringSegment::Text(text)] if text == "WriteLine")
    ));
    assert_eq!(attributes[1].name.parts, vec!["entry".to_string()]);
    assert_eq!(attributes[1].arguments[0].key.as_deref(), Some("kind"));
    assert!(matches!(
        attributes[1].arguments[0].value,
        TermExpression::Literal { literal: LiteralExpression::String(ref value), .. }
            if matches!(value.segments.as_slice(), [StringSegment::Text(text)] if text == "cli")
    ));
}

#[test]
fn parses_raw_and_interpolated_strings() {
    let source = r#"
micro main(name: utf8) {
    let raw = r"C:\Users\Name\Documents";
    let message = "Hello, {name}!\n";
    let literal = "Template: \{name\}";
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };
    let body = function.body.as_ref().expect("expected function body");

    let Statement::Let { statement: raw_stmt, .. } = &body.statements[0]
    else {
        panic!("expected raw let statement");
    };
    assert!(matches!(
        raw_stmt.initializer.as_ref(),
        Some(TermExpression::Literal { literal: LiteralExpression::String(value), .. })
            if value.prefix.as_deref() == Some("r")
                && value.quote_count == 1
                && matches!(value.segments.as_slice(), [StringSegment::Text(text)] if text == r"C:\Users\Name\Documents")
    ));

    let Statement::Let { statement: message_stmt, .. } = &body.statements[1]
    else {
        panic!("expected message let statement");
    };
    assert!(matches!(
        message_stmt.initializer.as_ref(),
        Some(TermExpression::Literal { literal: LiteralExpression::String(value), .. })
            if value.prefix.is_none()
                && value.quote_count == 1
                && matches!(value.segments.as_slice(),
                    [
                        StringSegment::Text(prefix),
                        StringSegment::Interpolation { expression, is_fluent },
                        StringSegment::Text(suffix)
                    ]
                    if prefix == "Hello, "
                        && !is_fluent
                        && matches!(expression.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["name".to_string()])
                        && suffix == "!\n")
    ));

    let Statement::Let { statement: literal_stmt, .. } = &body.statements[2]
    else {
        panic!("expected literal let statement");
    };
    assert!(matches!(
        literal_stmt.initializer.as_ref(),
        Some(TermExpression::Literal { literal: LiteralExpression::String(value), .. })
            if matches!(value.segments.as_slice(), [StringSegment::Text(text)] if text == "Template: {name}")
    ));
}

#[test]
fn parses_class_with_fields_and_methods() {
    let source = r#"
[derive(Debug)]
public open class Player(Entity, Damageable) {
    public name: utf8;
    readonly health: i64 = 100;

    public micro heal(amount: i64) -> i64 {
        return amount;
    }
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Class(class_decl) = &root.statements[0]
    else {
        panic!("expected class");
    };

    assert_eq!(class_decl.name, "Player");
    assert_eq!(class_decl.inheritance.len(), 2);
    assert_eq!(class_decl.body.fields.len(), 2);
    assert_eq!(class_decl.body.methods.len(), 1);
    assert_eq!(class_decl.annotations.modifiers, vec!["public".to_string(), "open".to_string()]);
    assert_eq!(class_decl.body.fields[1].annotations.modifiers, vec!["readonly".to_string()]);
    assert_eq!(class_decl.body.methods[0].annotations.modifiers, vec!["public".to_string()]);
}

#[test]
fn parses_trait_with_super_traits_and_methods() {
    let source = r#"
public trait Renderable: Drawable, Sized {
    micro render(surface: utf8);
    micro label() -> utf8 {
        return "ui";
    }
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Trait(trait_decl) = &root.statements[0]
    else {
        panic!("expected trait");
    };

    assert_eq!(trait_decl.name, "Renderable");
    assert_eq!(trait_decl.inheritance.len(), 2);
    assert_eq!(trait_decl.body.fields.len(), 0);
    assert_eq!(trait_decl.body.methods.len(), 2);
    assert_eq!(trait_decl.annotations.modifiers, vec!["public".to_string()]);
}

#[test]
fn parses_operator_methods_in_trait_and_imply() {
    let source = r#"
trait Add {
    micro infix +(self, rhs: Self) -> Self
    micro prefix -(self) -> Self {
        return self;
    }
}

imply Vec2: Add {
    micro infix +(self, rhs: Self) -> Self {
        return self;
    }
}
"#;

    let root = AstParser::parse_root(source).unwrap();

    let DeclarationStatement::Trait(trait_decl) = &root.statements[0]
    else {
        panic!("expected trait");
    };
    let DeclarationStatement::Imply(imply_decl) = &root.statements[1]
    else {
        panic!("expected imply");
    };

    assert_eq!(trait_decl.body.methods.len(), 2);
    assert_eq!(trait_decl.body.methods[0].name, "infix +");
    assert!(trait_decl.body.methods[0].body.is_none());
    assert_eq!(trait_decl.body.methods[1].name, "prefix -");
    assert!(trait_decl.body.methods[1].body.is_some());

    assert_eq!(imply_decl.methods.len(), 1);
    assert_eq!(imply_decl.methods[0].name, "infix +");
}

#[test]
fn parses_trait_document_model_items() {
    let source = r#"
trait Iterator<T>: Display + Clone {
    type Item
    const SIZE: i64 = 1

    micro next(self) -> Self::Item
    micro collect(self) -> T {
        return self;
    }
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Trait(trait_decl) = &root.statements[0]
    else {
        panic!("expected trait");
    };

    assert_eq!(trait_decl.name, "Iterator");
    assert_eq!(trait_decl.generic_parameters, vec!["T".to_string()]);
    assert_eq!(trait_decl.inheritance.len(), 2);
    assert!(!trait_decl.is_alias);
    assert_eq!(trait_decl.body.associated_types.len(), 1);
    assert_eq!(trait_decl.body.associated_types[0].name, "Item");
    assert_eq!(trait_decl.body.associated_constants.len(), 1);
    assert_eq!(trait_decl.body.associated_constants[0].name, "SIZE");
    assert!(matches!(
        trait_decl.body.associated_constants[0].default_value.as_ref(),
        Some(TermExpression::Literal { literal: LiteralExpression::Integer(value), .. }) if value == "1"
    ));
    assert_eq!(trait_decl.body.methods.len(), 2);
    assert!(trait_decl.body.methods[0].body.is_none());
    assert!(trait_decl.body.methods[1].body.is_some());
}

#[test]
fn parses_trait_alias_with_plus_targets() {
    let source = r#"
trait Printable = Display + Debug + Clone
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Trait(trait_decl) = &root.statements[0]
    else {
        panic!("expected trait");
    };

    assert!(trait_decl.is_alias);
    assert_eq!(trait_decl.alias_targets.len(), 3);
    assert_eq!(trait_decl.body.methods.len(), 0);
    assert_eq!(trait_decl.body.associated_types.len(), 0);
}

#[test]
fn parses_inherent_imply_with_methods() {
    let source = r#"
imply Point {
    micro length(self) -> f64 {
        return self.x;
    }
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Imply(imply_decl) = &root.statements[0]
    else {
        panic!("expected imply");
    };

    assert!(imply_decl.trait_type.is_none());
    assert_eq!(imply_decl.methods.len(), 1);
    assert_eq!(imply_decl.associated_type_bindings.len(), 0);
    assert_eq!(imply_decl.associated_const_bindings.len(), 0);
    assert!(matches!(
        &imply_decl.target_type,
        TypeExpression::Path(path) if path.name.parts == vec!["Point".to_string()]
    ));
}

#[test]
fn parses_trait_imply_with_generics_where_and_associated_types() {
    let source = r#"
imply<T: Clone = Item> Buffer<T>: Iterator
where T: Display {
    type Item = T
    const SIZE: i64 = 1

    micro next(self) -> T {
        return self.value;
    }
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Imply(imply_decl) = &root.statements[0]
    else {
        panic!("expected imply");
    };

    assert_eq!(imply_decl.generic_parameters.len(), 1);
    assert_eq!(imply_decl.generic_parameters[0].name, "T");
    assert_eq!(imply_decl.generic_parameters[0].bounds.len(), 1);
    assert!(matches!(
        imply_decl.generic_parameters[0].bounds[0],
        TypeExpression::Path(ref path) if path.name.parts == vec!["Clone".to_string()]
    ));
    assert!(matches!(
        imply_decl.generic_parameters[0].default_type.as_ref(),
        Some(TypeExpression::Path(path)) if path.name.parts == vec!["Item".to_string()]
    ));
    assert_eq!(imply_decl.where_constraints.len(), 1);
    assert!(matches!(
        imply_decl.where_constraints[0].target_type,
        TypeExpression::Path(ref path) if path.name.parts == vec!["T".to_string()]
    ));
    assert_eq!(imply_decl.where_constraints[0].bounds.len(), 1);
    assert!(matches!(
        imply_decl.where_constraints[0].bounds[0],
        TypeExpression::Path(ref path) if path.name.parts == vec!["Display".to_string()]
    ));
    assert!(matches!(
        imply_decl.trait_type.as_ref(),
        Some(TypeExpression::Path(path)) if path.name.parts == vec!["Iterator".to_string()]
    ));
    assert_eq!(imply_decl.associated_type_bindings.len(), 1);
    assert_eq!(imply_decl.associated_type_bindings[0].name, "Item");
    assert!(matches!(
        imply_decl.associated_type_bindings[0].concrete_type,
        TypeExpression::Path(ref path) if path.name.parts == vec!["T".to_string()]
    ));
    assert_eq!(imply_decl.associated_const_bindings.len(), 1);
    assert_eq!(imply_decl.associated_const_bindings[0].name, "SIZE");
    assert!(matches!(
        imply_decl.associated_const_bindings[0].const_type.as_ref(),
        Some(TypeExpression::Path(path)) if path.name.parts == vec!["i64".to_string()]
    ));
    assert_eq!(imply_decl.methods.len(), 1);
}

#[test]
fn separates_type_arguments_from_term_turbofish() {
    let source = r#"
micro build(value: T<X>) -> Result<Y> {
    T::<X>(value);
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    assert!(matches!(
        function.params[0].parameter_type.as_ref(),
        Some(TypeExpression::Path(path))
            if path.name.parts == vec!["T".to_string()]
                && matches!(path.arguments.as_slice(), [TypeExpression::Path(argument)] if argument.name.parts == vec!["X".to_string()])
    ));
    assert!(matches!(
        function.return_type.as_ref(),
        Some(TypeExpression::Path(path))
            if path.name.parts == vec!["Result".to_string()]
                && matches!(path.arguments.as_slice(), [TypeExpression::Path(argument)] if argument.name.parts == vec!["Y".to_string()])
    ));

    let body = function.body.as_ref().expect("expected function body");
    let Statement::Expr { expression, .. } = &body.statements[0]
    else {
        panic!("expected expression statement");
    };

    assert!(matches!(
        expression,
        TermExpression::Call { callee, args, .. }
            if args.len() == 1
                && matches!(callee.as_ref(),
                    TermExpression::Turbofish { expr, arguments, .. }
                        if arguments.len() == 1
                            && matches!(expr.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["T".to_string()])
                            && matches!(arguments.as_slice(), [TypeExpression::Path(argument)] if argument.name.parts == vec!["X".to_string()])
                )
    ));
}

#[test]
fn parses_member_assignment_expression() {
    let source = r#"
micro heal(amount: i64) {
    self.health = amount;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let Statement::Expr { expression, .. } = &body.statements[0]
    else {
        panic!("expected expression statement");
    };

    assert!(matches!(
        expression,
        TermExpression::Assign { target, value, .. }
            if matches!(target.as_ref(),
                TermExpression::MemberAccess { object, member, .. }
                    if member == "health"
                        && matches!(object.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["self".to_string()])
            )
            && matches!(value.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["amount".to_string()])
    ));
}

#[test]
fn parses_property_accessors_as_annotated_object_methods() {
    let source = r#"
abstract class Rectangle {
    get area(self) -> i64 {
        return self.width;
    }

    set area(mut self, value: i64) {
        self.width = value;
    }

    abstract get perimeter(self) -> i64;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Class(class_decl) = &root.statements[0]
    else {
        panic!("expected class");
    };

    assert_eq!(class_decl.body.methods.len(), 3);
    assert_eq!(class_decl.body.methods[0].name, "area");
    assert_eq!(class_decl.body.methods[0].annotations.modifiers, vec!["get".to_string()]);
    assert!(class_decl.body.methods[0].body.is_some());
    assert!(matches!(
        class_decl.body.methods[0].return_type.as_ref(),
        Some(TypeExpression::Path(path)) if path.name.parts == vec!["i64".to_string()]
    ));

    assert_eq!(class_decl.body.methods[1].name, "area");
    assert_eq!(class_decl.body.methods[1].annotations.modifiers, vec!["set".to_string()]);
    assert_eq!(class_decl.body.methods[1].params.len(), 2);
    assert!(class_decl.body.methods[1].body.is_some());

    assert_eq!(class_decl.body.methods[2].name, "perimeter");
    assert!(class_decl.body.methods[2].body.is_none());
    assert_eq!(class_decl.body.methods[2].annotations.modifiers, vec!["abstract".to_string(), "get".to_string()]);
}

#[test]
fn respects_pratt_precedence_and_assignment_associativity() {
    let source = r#"
micro compute() {
    alpha = beta = gamma + delta * epsilon;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let Statement::Expr { expression, .. } = &body.statements[0]
    else {
        panic!("expected expression statement");
    };

    assert!(matches!(
        expression,
        TermExpression::Assign { target, value, .. }
            if matches!(target.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["alpha".to_string()])
                && matches!(value.as_ref(),
                    TermExpression::Assign { target, value, .. }
                        if matches!(target.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["beta".to_string()])
                            && matches!(value.as_ref(),
                                TermExpression::Binary { op, lhs, rhs, .. }
                                    if matches!(op, valkyrie_parser::BinaryOperator::Add)
                                        && matches!(lhs.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["gamma".to_string()])
                                        && matches!(rhs.as_ref(),
                                            TermExpression::Binary { op, lhs, rhs, .. }
                                                if matches!(op, valkyrie_parser::BinaryOperator::Mul)
                                                    && matches!(lhs.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["delta".to_string()])
                                                    && matches!(rhs.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["epsilon".to_string()])
                                        )
                            )
                )
    ));
}

#[test]
fn unary_operator_binds_after_postfix_receiver_chain() {
    let source = r#"
micro check() {
    !player.inventory.get(0).ready
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let expression = body.tail_expression.as_ref().expect("expected tail expression");

    assert!(matches!(
        expression.as_ref(),
        TermExpression::Unary { op, expr, .. }
            if matches!(op, valkyrie_parser::UnaryOperator::Not)
                && matches!(expr.as_ref(),
                    TermExpression::MemberAccess { object, member, .. }
                        if member == "ready"
                            && matches!(object.as_ref(),
                                TermExpression::Call { callee, args, .. }
                                    if args.len() == 1
                                        && matches!(args[0], TermExpression::Literal { literal: LiteralExpression::Integer(ref value), .. } if value == "0")
                                        && matches!(callee.as_ref(),
                                            TermExpression::MemberAccess { object, member, .. }
                                                if member == "get"
                                                    && matches!(object.as_ref(),
                                                        TermExpression::MemberAccess { object, member, .. }
                                                            if member == "inventory"
                                                                && matches!(object.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["player".to_string()])
                                                    )
                                        )
                            )
                )
    ));
}

#[test]
fn cast_operator_binds_looser_than_additive_expression() {
    let source = r#"
micro cast_sum() {
    alpha + beta as Total
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let expression = body.tail_expression.as_ref().expect("expected tail expression");

    assert!(matches!(
        expression.as_ref(),
        TermExpression::As { expr, ty, .. }
            if matches!(expr.as_ref(),
                TermExpression::Binary { op, lhs, rhs, .. }
                    if matches!(op, valkyrie_parser::BinaryOperator::Add)
                        && matches!(lhs.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["alpha".to_string()])
                        && matches!(rhs.as_ref(), TermExpression::Name { path, .. } if path.parts == vec!["beta".to_string()])
            )
            && matches!(ty, TypeExpression::Path(path) if path.name.parts == vec!["Total".to_string()])
    ));
}

#[test]
fn parses_tuple_pattern_let_binding() {
    let source = r#"
micro main() {
    let (x, y) = (1, 2);
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let Statement::Let { statement, .. } = &body.statements[0]
    else {
        panic!("expected let statement");
    };

    assert!(matches!(
        &statement.pattern,
        PatternExpression::Tuple { items, .. }
            if items.len() == 2
                && matches!(&items[0], PatternExpression::Variable { name, .. } if name == "x")
                && matches!(&items[1], PatternExpression::Variable { name, .. } if name == "y")
    ));
    assert!(matches!(
        statement.initializer.as_ref(),
        Some(TermExpression::Tuple { items, .. })
            if items.len() == 2
                && matches!(&items[0], TermExpression::Literal { literal: LiteralExpression::Integer(value), .. } if value == "1")
                && matches!(&items[1], TermExpression::Literal { literal: LiteralExpression::Integer(value), .. } if value == "2")
    ));
}

#[test]
fn parses_loop_in_with_tuple_pattern() {
    let source = r#"
micro main() -> i64 {
    loop (x, y) in [(1, 2)] {
        return x + y;
    }
    return 0;
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let Statement::Expr { expression, .. } = &body.statements[0]
    else {
        panic!("expected loop expression statement");
    };

    assert!(matches!(
        expression,
        TermExpression::Loop { pattern, iterator, condition, .. }
            if condition.is_none()
                && matches!(pattern.as_ref(), Some(PatternExpression::Tuple { items, .. })
                    if items.len() == 2
                        && matches!(&items[0], PatternExpression::Variable { name, .. } if name == "x")
                        && matches!(&items[1], PatternExpression::Variable { name, .. } if name == "y"))
                && matches!(iterator.as_deref(), Some(TermExpression::Array { items, .. })
                    if items.len() == 1
                        && matches!(&items[0], TermExpression::Tuple { items, .. } if items.len() == 2))
    ));
}

#[test]
fn parses_nested_tuple_pattern_with_wildcard() {
    let source = r#"
micro main() {
    let ((x, _), y) = ((1, 2), 3);
}
"#;

    let root = AstParser::parse_root(source).unwrap();
    let DeclarationStatement::Function(function) = &root.statements[0]
    else {
        panic!("expected function");
    };

    let body = function.body.as_ref().expect("expected function body");
    let Statement::Let { statement, .. } = &body.statements[0]
    else {
        panic!("expected let statement");
    };

    assert!(matches!(
        &statement.pattern,
        PatternExpression::Tuple { items, .. }
            if items.len() == 2
                && matches!(&items[0], PatternExpression::Tuple { items, .. }
                    if items.len() == 2
                        && matches!(&items[0], PatternExpression::Variable { name, .. } if name == "x")
                        && matches!(&items[1], PatternExpression::Wildcard { .. }))
                && matches!(&items[1], PatternExpression::Variable { name, .. } if name == "y")
    ));
}

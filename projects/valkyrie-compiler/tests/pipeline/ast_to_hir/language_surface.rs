use super::*;

#[test]
fn compiler_facade_lowers_into_mir_and_lir_from_moved_tests() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 11 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main(input: i64) -> i64 {
    return input;
}
"#,
        )
        .unwrap();
    assert_eq!(mir.functions.len(), 1);
    assert!(mir.functions[0].values.iter().any(|value| matches!(value.origin, valkyrie_compiler::MirValueOrigin::Parameter { index: 0, .. })));

    let lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    std::console::write_line("hi");
}
"#,
        )
        .unwrap();
    assert_eq!(lir.functions.len(), 1);
    assert_eq!(lir.functions[0].blocks.len(), 1);
}

#[test]
fn lowers_structured_attribute_arguments_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 13 });
    let module = compiler
        .compile_source(
            r#"
[clr("System.Console", "System.Console", "WriteLine")]
micro helper(message: utf16) {
    return;
}
"#,
        )
        .unwrap();

    assert_eq!(module.functions[0].annotations.len(), 1);
    assert_eq!(module.functions[0].annotations[0].name.to_string(), "clr");
    assert_eq!(module.functions[0].annotations[0].arguments.len(), 3);
    assert!(matches!(module.functions[0].annotations[0].arguments[0].value.kind, HirExprKind::Literal(_)));
}

#[test]
fn lowers_term_turbofish_into_generic_apply() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 29 });
    let module = compiler
        .compile_source(
            r#"micro main() {
    T::<i64>();
}
"#,
        )
        .unwrap();
    let HirStatementKind::Expr(expression) = &module.functions[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert!(args.is_empty());
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref arguments, .. }
                    if arguments.len() == 1 && arguments[0] == ValkyrieType::Integer64 { signed: true }
            ));
        }
        _ => panic!("expected call expression"),
    }
}

#[test]
fn lowers_instance_method_with_implicit_self_param() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 31 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro heal(amount: i64) -> i64 {
        self.health;
        return amount;
    }
}
"#,
        )
        .unwrap();

    let method = &module.structs[0].methods[0];
    assert_eq!(method.params.len(), 2);
    assert_eq!(method.params[0].name.name.as_str(), "self");
    assert!(matches!(method.params[0].ty, ValkyrieType::r#SelfType));
    assert_eq!(method.params[1].name.name.as_str(), "amount");
}

#[test]
fn keeps_void_alias_and_self_name_as_user_types_until_hir_lowering() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 67 });
    let module = compiler
        .compile_source(
            r#"
type void = c_void;
micro convert(value: Self) -> void {
}
micro make() -> () {
}
"#,
        )
        .unwrap();

    let convert = &module.functions[0];
    assert!(matches!(
        convert.params[0].ty,
        ValkyrieType::Named(ref name) if name.as_str() == "Self"
    ));
    assert!(matches!(
        convert.return_type,
        ValkyrieType::Named(ref name) if name.as_str() == "void"
    ));

    let make = &module.functions[1];
    assert_eq!(make.return_type, ValkyrieType::Unit);
}

#[test]
fn lowers_getter_and_setter_into_one_hir_property() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 61 });
    let module = compiler
        .compile_source(
            r#"class Rectangle {
    get area(self) -> i64 {
        return self.width;
    }

    set area(mut self, value: i64) {
        self.width = value;
    }
}"#,
        )
        .unwrap();

    let class = &module.structs[0];
    assert_eq!(class.properties.len(), 1);
    let property = &class.properties[0];
    assert_eq!(property.name.as_str(), "area");
    assert_eq!(property.ty, ValkyrieType::Integer64 { signed: true });
    assert!(!property.is_readonly);
    assert!(property.getter.is_some());
    assert!(property.setter.is_some());

    let getter = property.getter.as_ref().unwrap();
    assert_eq!(getter.name.as_str(), "area");
    assert_eq!(getter.params.len(), 1);
    assert_eq!(getter.return_type, ValkyrieType::Integer64 { signed: true });

    let setter = property.setter.as_ref().unwrap();
    assert_eq!(setter.name.as_str(), "set_area");
    assert_eq!(setter.params.len(), 2);
    assert_eq!(setter.return_type, ValkyrieType::Unit);
}

#[test]
fn lowers_property_modifiers_into_hir_flags() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 63 });
    let module = compiler
        .compile_source(
            r#"class Shape {
    virtual get area(self) -> i64;
}

class MathConstants {
    static final get pi() -> i64 {
        return 3;
    }
}"#,
        )
        .unwrap();

    let shape = &module.structs[0];
    assert_eq!(shape.properties.len(), 1);
    let area = &shape.properties[0];
    assert!(area.is_abstract);
    assert!(area.is_virtual);
    assert!(!area.is_static);
    assert!(area.getter.as_ref().unwrap().is_abstract);

    let math = &module.structs[1];
    assert_eq!(math.properties.len(), 1);
    let pi = &math.properties[0];
    assert!(pi.is_static);
    assert!(pi.is_final);
    assert!(pi.is_readonly);
    assert!(pi.getter.is_some());
    assert_eq!(pi.getter.as_ref().unwrap().params.len(), 0);
}

#[test]
fn lowers_static_method_without_implicit_self_param() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 37 });
    let module = compiler
        .compile_source(
            r#"
class Math {
    static micro abs(value: i64) -> i64 {
        return value;
    }
}
"#,
        )
        .unwrap();

    let method = &module.structs[0].methods[0];
    assert_eq!(method.params.len(), 1);
    assert_eq!(method.params[0].name.name.as_str(), "value");
}

#[test]
fn lowers_member_field_access_and_assignment_into_getter_setter_calls() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 41 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro heal(amount: i64) {
        self.health = amount;
        self.health;
    }
}
"#,
        )
        .unwrap();

    let statements = &module.structs[0].methods[0].body.statements;
    let HirStatementKind::Expr(setter_expr) = &statements[0].kind
    else {
        panic!("expected setter expression");
    };
    let HirStatementKind::Expr(getter_expr) = &statements[1].kind
    else {
        panic!("expected getter expression");
    };

    match &setter_expr.kind {
        HirExprKind::StoreField { object, field, value } => {
            assert_eq!(field.as_str(), "health");
            assert!(matches!(object.kind, HirExprKind::Variable(_)));
            assert!(matches!(value.kind, HirExprKind::Variable(_)));
        }
        _ => panic!("expected store field"),
    }

    match &getter_expr.kind {
        HirExprKind::FieldAccess { object, field } => {
            assert_eq!(field.as_str(), "health");
            assert!(matches!(object.kind, HirExprKind::Variable(_)));
        }
        _ => panic!("expected field access"),
    }
}

#[test]
fn preserves_instance_method_call_without_rewriting_to_getter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 43 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro tick() {
        self.refresh();
    }

    micro refresh() {
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[0].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(args[0].kind, HirExprKind::Variable(_)));
            assert!(matches!(
                callee.kind,
                HirExprKind::Path(ref path) if path.to_string() == "refresh"
            ));
        }
        _ => panic!("expected method call"),
    }
}

#[test]
fn lowers_member_turbofish_call_with_receiver_as_first_argument() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 47 });
    let module = compiler
        .compile_source(
            r#"
class Player {
    micro tick(value: i64) {
        self.refresh::<i64>(value);
    }

    micro refresh(value: i64) {
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[0].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[0].kind, HirExprKind::Variable(_)));
            assert!(matches!(args[1].kind, HirExprKind::Variable(_)));
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref callee, ref arguments }
                    if matches!(callee.kind, HirExprKind::Path(ref path) if path.to_string() == "refresh")
                        && arguments.len() == 1
            ));
        }
        _ => panic!("expected turbofish method call"),
    }
}

#[test]
fn lowers_parent_slot_method_call_as_slot_access_plus_method_call() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 53 });
    let module = compiler
        .compile_source(
            r#"
class Display {
    micro show() {
    }
}

class Document(rename: Display) {
    micro render() {
        self.rename.show();
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[1].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 1);
            assert!(matches!(
                callee.kind,
                HirExprKind::Path(ref path) if path.to_string() == "show"
            ));
            assert!(matches!(
                args[0].kind,
                HirExprKind::FieldAccess { ref object, ref field }
                    if field.as_str() == "rename"
                        && matches!(object.kind, HirExprKind::Variable(_))
            ));
        }
        _ => panic!("expected renamed parent method call"),
    }
}

#[test]
fn lowers_parent_slot_turbofish_call_as_slot_access_plus_method_call() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 59 });
    let module = compiler
        .compile_source(
            r#"
class Reader {
    micro read(value: i64) {
    }
}

class Hybrid(reader: Reader) {
    micro consume(value: i64) {
        self.reader.read::<i64>(value);
    }
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(expression) = &module.structs[1].methods[0].body.statements[0].kind
    else {
        panic!("expected expression statement");
    };

    match &expression.kind {
        HirExprKind::Call { callee, args, .. } => {
            assert_eq!(args.len(), 2);
            assert!(matches!(args[1].kind, HirExprKind::Variable(_)));
            assert!(matches!(
                callee.kind,
                HirExprKind::GenericApply { ref callee, ref arguments }
                    if matches!(callee.kind, HirExprKind::Path(ref path) if path.to_string() == "read")
                        && arguments.len() == 1
            ));
            assert!(matches!(
                args[0].kind,
                HirExprKind::FieldAccess { ref object, ref field }
                    if field.as_str() == "reader"
                        && matches!(object.kind, HirExprKind::Variable(_))
            ));
        }
        _ => panic!("expected renamed parent turbofish method call"),
    }
}

#[test]
fn lowers_parent_slot_name_from_alias_or_type_name() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 61 });
    let module = compiler
        .compile_source(
            r#"
class Mixed(primary: Teacher, BaseWidget) {
}
"#,
        )
        .unwrap();

    assert_eq!(module.structs[0].parents.len(), 2);
    assert_eq!(module.structs[0].parents[0].slot_name().as_str(), "primary");
    assert_eq!(module.structs[0].parents[1].slot_name().as_str(), "base_widget");
}

#[test]
fn lowers_unite_declaration_into_hir_enum_family() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 67 });
    let module = compiler
        .compile_source(
            r#"
unite Option {
    Some {
        value: i64,
    }
    None
}
"#,
        )
        .unwrap();

    assert_eq!(module.enums.len(), 1);
    let option = &module.enums[0];
    assert!(option.is_unity());
    assert_eq!(option.name.as_str(), "Option");
    assert_eq!(option.variants.len(), 2);
    assert_eq!(option.variants[0].name.as_str(), "Some");
    assert_eq!(option.variants[0].fields.len(), 1);
    assert_eq!(option.variants[1].name.as_str(), "None");
    assert!(option.variants[1].fields.is_empty());
}

#[test]
fn lowers_trait_associated_types_into_hir() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 81 });
    let module = compiler
        .compile_source(
            r#"
trait Iterator<T>: Display + Clone {
    type Item
    type Output = T
    const Limit: i64 = 42

    micro next(self) -> Self::Item
    micro collect(self) -> T {
        return self;
    }
}
"#,
        )
        .unwrap();

    assert_eq!(module.traits.len(), 1);
    let trait_def = &module.traits[0];
    assert_eq!(trait_def.name.as_str(), "Iterator");
    assert_eq!(trait_def.super_traits.len(), 2);
    assert_eq!(trait_def.associated_types.len(), 2);
    assert_eq!(trait_def.associated_constants.len(), 1);
    assert_eq!(trait_def.associated_types[0].name.as_str(), "Item");
    assert!(trait_def.associated_types[0].default.is_none());
    assert_eq!(trait_def.associated_types[1].name.as_str(), "Output");
    assert!(matches!(trait_def.associated_types[1].default, Some(ValkyrieType::Named(ref name)) if name.as_str() == "T"));
    assert_eq!(trait_def.associated_constants[0].name.as_str(), "Limit");
    assert_eq!(trait_def.associated_constants[0].const_type, ValkyrieType::Integer64 { signed: true });
    assert!(matches!(
        trait_def.associated_constants[0].default_value.as_ref(),
        Some(expr) if matches!(expr.kind, valkyrie_types::hir::HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(42)))
    ));
    assert_eq!(trait_def.methods.len(), 1);
    assert_eq!(trait_def.default_methods.len(), 1);
}

#[test]
fn lowers_imply_blocks_into_hir_impls() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 83 });
    let module = compiler
        .compile_source(
            r#"
imply<T: Clone> Buffer<T>: Iterator
where T: Display {
    type Item = T
    const SIZE: i64 = 1

    micro next(self) -> T {
        return self.value;
    }
}

imply Point {
    micro length(self) -> i64 {
        return self.x;
    }
}
"#,
        )
        .unwrap();

    assert_eq!(module.impls.len(), 2);

    let trait_impl = &module.impls[0];
    assert!(matches!(trait_impl.target, ValkyrieType::Apply(_, _)));
    assert!(matches!(trait_impl.trait_path.as_ref(), Some(path) if path.to_string() == "Iterator"));
    assert_eq!(trait_impl.generics.len(), 1);
    assert_eq!(trait_impl.generics[0].name.as_str(), "T");
    assert_eq!(trait_impl.generics[0].bounds.len(), 1);
    assert_eq!(trait_impl.where_constraints.len(), 1);
    assert!(matches!(trait_impl.where_constraints[0].target, ValkyrieType::Named(ref name) if name.as_str() == "T"));
    assert_eq!(trait_impl.where_constraints[0].bounds.len(), 1);
    assert_eq!(trait_impl.where_constraints[0].bounds[0].to_string(), "Display");
    assert_eq!(trait_impl.methods.len(), 1);
    assert_eq!(trait_impl.associated_type_impls.len(), 1);
    assert_eq!(trait_impl.associated_const_impls.len(), 1);
    assert_eq!(trait_impl.associated_type_impls[0].name.as_str(), "Item");
    assert!(matches!(trait_impl.associated_type_impls[0].concrete_type, ValkyrieType::Named(ref name) if name.as_str() == "T"));
    assert_eq!(trait_impl.associated_const_impls[0].name.as_str(), "SIZE");
    assert_eq!(trait_impl.associated_const_impls[0].const_type, Some(ValkyrieType::Integer64 { signed: true }));

    let inherent_impl = &module.impls[1];
    assert!(matches!(inherent_impl.target, ValkyrieType::Named(ref name) if name.as_str() == "Point"));
    assert!(inherent_impl.trait_path.is_none());
    assert!(inherent_impl.where_constraints.is_empty());
    assert_eq!(inherent_impl.methods.len(), 1);
    assert!(inherent_impl.associated_type_impls.is_empty());
    assert!(inherent_impl.associated_const_impls.is_empty());
}

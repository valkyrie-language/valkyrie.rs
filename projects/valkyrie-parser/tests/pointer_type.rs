use valkyrie_parser::{
    ast::{ParameterPassingKind, PointerKind, RootStatement, TypeExpression},
    AstParser,
};

#[test]
fn parses_parameter_passing_modifiers() {
    let source = "micro test(ref x: ◆u8, mut y: ◇i32, own z: T?) {}";
    let root = AstParser::parse_root(source).expect("解析带参数 modifier 的函数失败");

    let RootStatement::Function(function) = &root.statements[0]
    else {
        panic!("期望解析出函数声明");
    };

    assert_eq!(function.params.len(), 3);

    assert_eq!(function.params[0].passing, ParameterPassingKind::Ref);
    assert!(!function.params[0].is_mutable);
    assert!(matches!(function.params[0].parameter_type.as_ref(), Some(TypeExpression::Pointer { kind: PointerKind::Mutable, .. })));

    assert_eq!(function.params[1].passing, ParameterPassingKind::Mut);
    assert!(function.params[1].is_mutable);
    assert!(matches!(function.params[1].parameter_type.as_ref(), Some(TypeExpression::Pointer { kind: PointerKind::ReadOnly, .. })));

    assert_eq!(function.params[2].passing, ParameterPassingKind::Own);
    assert!(!function.params[2].is_mutable);
    assert!(matches!(function.params[2].parameter_type.as_ref(), Some(TypeExpression::Nullable { .. })));
}

#[test]
fn rejects_ref_modifier_in_call_arguments() {
    let source = "micro test(x: ◆u8) { foo(ref x); }";
    let result = AstParser::parse_root(source);
    assert!(result.is_err(), "调用参数不应接受 `ref` modifier");
}

#[test]
fn rejects_mut_modifier_in_call_arguments() {
    let source = "micro test(x: ◆u8) { foo(mut x); }";
    let result = AstParser::parse_root(source);
    assert!(result.is_err(), "调用参数不应接受 `mut` modifier");
}

#[test]
fn rejects_own_modifier_in_call_arguments() {
    let source = "micro test(x: ◆u8) { foo(own x); }";
    let result = AstParser::parse_root(source);
    assert!(result.is_err(), "调用参数不应接受 `own` modifier");
}

#[test]
fn parses_pointer_suffix_null_and_unsafe_block() {
    let source = "micro test(ref ptr: ◆u8) { let fallback: ◆u8? = null; unsafe { ptr.◇ }; ptr.◆; }";
    let root = AstParser::parse_root(source).expect("解析指针相关语法失败");

    let RootStatement::Function(function) = &root.statements[0]
    else {
        panic!("期望解析出函数声明");
    };
    assert!(function.body.is_some(), "期望函数体存在");
}

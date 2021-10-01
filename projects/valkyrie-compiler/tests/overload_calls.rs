use valkyrie_compiler::ValkyrieCompiler;
use valkyrie_types::{
    hir::{HirCallableDomain, HirExprKind, HirExtractorPattern, HirPattern, HirStatementKind, ValkyrieType},
    SourceID,
};

#[test]
fn resolves_plain_call_into_hir_metadata() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 4101 });
    let hir = compiler
        .compile_source(
            r#"
micro choose(value: i64) -> i64 {
    return value;
}

micro main(value: i64) -> i64 {
    return choose(value);
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(statement) = &hir.functions[1].body.statements[0].kind
    else {
        panic!("expected return statement");
    };
    let HirExprKind::Return(Some(expression)) = &statement.kind
    else {
        panic!("expected return expression");
    };
    let HirExprKind::Call { resolved: Some(resolved), .. } = &expression.kind
    else {
        panic!("expected resolved call");
    };

    assert_eq!(resolved.domain, HirCallableDomain::Function);
    assert_eq!(resolved.symbol.to_string(), "choose");
    assert_eq!(resolved.return_type, ValkyrieType::Integer64 { signed: true });
}

#[test]
fn keeps_plain_function_call_as_function_domain() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 4104 });
    let hir = compiler
        .compile_source(
            r#"
micro ready() -> bool {
    return true;
}

micro main() -> bool {
    return ready();
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(statement) = &hir.functions[1].body.statements[0].kind
    else {
        panic!("expected return statement");
    };
    let HirExprKind::Return(Some(expression)) = &statement.kind
    else {
        panic!("expected return expression");
    };
    let HirExprKind::Call { callee, resolved: Some(resolved), .. } = &expression.kind
    else {
        panic!("expected resolved plain call");
    };

    assert!(matches!(&callee.kind, HirExprKind::Path(path) if path.to_string() == "ready"));
    assert_eq!(resolved.domain, HirCallableDomain::Function);
    assert_eq!(resolved.symbol.to_string(), "ready");
    assert_eq!(resolved.return_type, ValkyrieType::Boolean);
}

#[test]
fn resolves_point_call_as_constructor_in_hir_metadata() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 4105 });
    let hir = compiler
        .compile_source(
            r#"
class Point {
    x: i64;
    y: i64;
}

micro main(x: i64, y: i64) -> Point {
    return Point(x, y);
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(statement) = &hir.functions[0].body.statements[0].kind
    else {
        panic!("expected return statement");
    };
    let HirExprKind::Return(Some(expression)) = &statement.kind
    else {
        panic!("expected return expression");
    };
    let HirExprKind::Call { resolved: Some(resolved), .. } = &expression.kind
    else {
        panic!("expected resolved constructor");
    };

    assert_eq!(resolved.domain, HirCallableDomain::Constructor);
    assert_eq!(resolved.symbol.to_string(), "Point");
    assert_eq!(resolved.return_type, ValkyrieType::Named(valkyrie_types::Identifier::new("Point")));
}

#[test]
fn resolves_operator_sugar_into_hir_metadata() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 4102 });
    let hir = compiler
        .compile_source(
            r#"
class Vec {
    infix `+`(other: Vec) -> Vec {
        return self;
    }

    prefix `-`() -> Vec {
        return self;
    }

    suffix `[]`(ordinal: i64) -> i64 {
        return ordinal;
    }
}

micro add(left: Vec, right: Vec) -> Vec {
    return left + right;
}

micro negate(value: Vec) -> Vec {
    return -value;
}

micro get(buffer: Vec) -> i64 {
    return buffer[1];
}
"#,
        )
        .unwrap();

    let add_call = match &hir.functions[0].body.statements[0].kind {
        HirStatementKind::Expr(statement) => match &statement.kind {
            HirExprKind::Return(Some(expression)) => expression,
            _ => panic!("expected return expression"),
        },
        _ => panic!("expected expression statement"),
    };
    let HirExprKind::Call { resolved: Some(add_resolved), .. } = &add_call.kind
    else {
        panic!("expected resolved infix call");
    };
    assert_eq!(add_resolved.domain, HirCallableDomain::Operator);
    assert_eq!(add_resolved.symbol.to_string(), "infix +");

    let negate_call = match &hir.functions[1].body.statements[0].kind {
        HirStatementKind::Expr(statement) => match &statement.kind {
            HirExprKind::Return(Some(expression)) => expression,
            _ => panic!("expected return expression"),
        },
        _ => panic!("expected expression statement"),
    };
    let HirExprKind::Call { resolved: Some(negate_resolved), .. } = &negate_call.kind
    else {
        panic!("expected resolved prefix call");
    };
    assert_eq!(negate_resolved.domain, HirCallableDomain::Operator);
    assert_eq!(negate_resolved.symbol.to_string(), "prefix -");

    let get_call = match &hir.functions[2].body.statements[0].kind {
        HirStatementKind::Expr(statement) => match &statement.kind {
            HirExprKind::Return(Some(expression)) => expression,
            _ => panic!("expected return expression"),
        },
        _ => panic!("expected expression statement"),
    };
    let HirExprKind::Call { resolved: Some(get_resolved), .. } = &get_call.kind
    else {
        panic!("expected resolved subscript call");
    };
    assert_eq!(get_resolved.domain, HirCallableDomain::Operator);
    assert_eq!(get_resolved.symbol.to_string(), "suffix []");
}

#[test]
fn lowers_constructor_pattern_into_canonical_extractor_callee() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 4103 });
    let hir = compiler
        .compile_source(
            r#"micro main(value: Point) -> bool {
    return match value {
        case Point(flag, [1, ..rest]):
            flag
        else:
            false
    };
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(statement) = &hir.functions[0].body.statements[0].kind
    else {
        panic!("expected return statement");
    };
    let HirExprKind::Return(Some(expression)) = &statement.kind
    else {
        panic!("expected return expression");
    };
    let HirExprKind::Match { arms, .. } = &expression.kind
    else {
        panic!("expected match expression");
    };

    let HirPattern::Extractor(HirExtractorPattern::Constructor { name, canonical_callee, fields, resolved: None }) = &arms[0].pattern
    else {
        panic!("expected constructor extractor pattern");
    };
    assert_eq!(name.to_string(), "Point");
    assert_eq!(canonical_callee.parts().len(), 2);
    assert_eq!(canonical_callee.parts()[0].as_str(), "Point");
    assert_eq!(canonical_callee.parts()[1].as_str(), "extractor");
    assert!(matches!(fields.first(), Some(HirPattern::Variable(flag)) if flag.name.as_str() == "flag"));
}

#[test]
fn resolves_constructor_pattern_into_hir_extractor_metadata() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 4106 });
    let hir = compiler
        .compile_source(
            r#"
class Point {
    micro extractor() -> [bool, bool] {
        return (true, true);
    }
}

micro main(value: Point) -> bool {
    return match value {
        case Point(flag):
            flag
        else:
            false
    };
}
"#,
        )
        .unwrap();

    let HirStatementKind::Expr(statement) = &hir.functions[0].body.statements[0].kind
    else {
        panic!("expected return statement");
    };
    let HirExprKind::Return(Some(expression)) = &statement.kind
    else {
        panic!("expected return expression");
    };
    let HirExprKind::Match { arms, .. } = &expression.kind
    else {
        panic!("expected match expression");
    };
    let HirPattern::Extractor(HirExtractorPattern::Constructor { resolved: Some(resolved), .. }) = &arms[0].pattern
    else {
        panic!("expected resolved extractor pattern");
    };

    assert_eq!(resolved.domain, HirCallableDomain::Extractor);
    assert_eq!(resolved.symbol.to_string(), "extractor");
    assert_eq!(resolved.return_type, ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Boolean]));
}

use valkyrie_compiler::type_checker::*;
use valkyrie_types::{
    hir::{HirBlock, HirExpr, HirExprKind, HirLiteral, HirType},
    Identifier, NamePath, SourceID, SourceSpan,
};

fn test_span() -> SourceSpan {
    SourceSpan::new(SourceID::default(), 0, 0)
}

fn operator_call(name: &str, args: Vec<HirExpr>) -> HirExpr {
    HirExpr {
        kind: HirExprKind::Call {
            callee: Box::new(HirExpr { kind: HirExprKind::Path(NamePath::new(vec![Identifier::new(name)])), span: test_span() }),
            args,
        },
        span: test_span(),
    }
}

#[test]
fn test_int_literal() {
    let mut inf = TypeInference::new();
    let expr = HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(42)), span: test_span() };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Integer64);
}

#[test]
fn test_float_literal() {
    let mut inf = TypeInference::new();
    let expr = HirExpr { kind: HirExprKind::Literal(HirLiteral::Float64(ordered_float::OrderedFloat(3.14))), span: test_span() };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Float64);
}

#[test]
fn test_bool_literal() {
    let mut inf = TypeInference::new();
    let expr = HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(true)), span: test_span() };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Boolean);
}

#[test]
fn test_string_literal_defaults_to_utf8() {
    let mut inf = TypeInference::new();
    let expr = HirExpr {
        kind: HirExprKind::Literal(HirLiteral::String(valkyrie_types::hir::HirStringLiteral {
            prefix: None,
            quote_count: 1,
            segments: vec![valkyrie_types::hir::HirStringSegment::Text("hello".to_string())],
        })),
        span: test_span(),
    };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Utf8);
}

#[test]
fn test_unit_literal() {
    let mut inf = TypeInference::new();
    let expr = HirExpr { kind: HirExprKind::Literal(HirLiteral::Unit), span: test_span() };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Unit);
}

#[test]
fn test_variable_propagation() {
    let mut inf = TypeInference::new();
    inf.bind_variable(Identifier::new("x"), HirType::Integer64);

    let expr = HirExpr {
        kind: HirExprKind::Variable(valkyrie_types::hir::HirIdentifier { name: Identifier::new("x"), shadow_index: 0, span: test_span() }),
        span: test_span(),
    };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Integer64);
}

#[test]
fn test_unbound_variable() {
    let mut inf = TypeInference::new();
    let expr = HirExpr {
        kind: HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("unknown"),
            shadow_index: 0,
            span: test_span(),
        }),
        span: test_span(),
    };
    let result = inf.infer(&expr);
    assert!(matches!(result, Err(TypeError::UnboundVariable { .. })));
}

#[test]
fn test_binary_add() {
    let mut inf = TypeInference::new();
    let left = HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(1)), span: test_span() };
    let right = HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(2)), span: test_span() };
    let expr = operator_call("infix +", vec![left, right]);
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Integer64);
}

#[test]
fn test_binary_comparison() {
    let mut inf = TypeInference::new();
    let left = HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(1)), span: test_span() };
    let right = HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(2)), span: test_span() };
    let expr = operator_call("infix <", vec![left, right]);
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Boolean);
}

#[test]
fn test_bool_equality_call() {
    let mut inf = TypeInference::new();
    let left = HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(true)), span: test_span() };
    let right = HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(false)), span: test_span() };
    let expr = operator_call("infix ==", vec![left, right]);
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Boolean);
}

#[test]
fn test_unary_neg() {
    let mut inf = TypeInference::new();
    let inner = HirExpr { kind: HirExprKind::Literal(HirLiteral::Integer64(42)), span: test_span() };
    let expr = operator_call("prefix -", vec![inner]);
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Integer64);
}

#[test]
fn test_unary_not() {
    let mut inf = TypeInference::new();
    let inner = HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(true)), span: test_span() };
    let expr = operator_call("prefix !", vec![inner]);
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Boolean);
}

#[test]
fn test_if_expression_bool_result() {
    let mut inf = TypeInference::new();
    let expr = HirExpr {
        kind: HirExprKind::If {
            condition: Box::new(HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(true)), span: test_span() }),
            then_branch: Box::new(HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(false)), span: test_span() })),
                span: test_span(),
            }),
            else_branch: Some(Box::new(HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(HirExpr { kind: HirExprKind::Literal(HirLiteral::Bool(true)), span: test_span() })),
                span: test_span(),
            })),
        },
        span: test_span(),
    };
    let t = inf.infer(&expr).unwrap();
    assert_eq!(t, HirType::Boolean);
}

#[test]
fn test_fresh_var() {
    let mut inf = TypeInference::new();
    let v1 = inf.fresh_var();
    let v2 = inf.fresh_var();
    let v3 = inf.fresh_var();

    assert_eq!(v1, InferenceTypeVar(0));
    assert_eq!(v2, InferenceTypeVar(1));
    assert_eq!(v3, InferenceTypeVar(2));
}

#[test]
fn test_unify_same_types() {
    let mut inf = TypeInference::new();
    let result = inf.unify(&HirType::Integer64, &HirType::Integer64);
    assert!(result.is_ok());
}

#[test]
fn test_unify_different_types() {
    let mut inf = TypeInference::new();
    let result = inf.unify(&HirType::Integer64, &HirType::Boolean);
    assert!(matches!(result, Err(TypeError::Mismatch { .. })));
}

#[test]
fn test_unify_with_infer() {
    let mut inf = TypeInference::new();
    let result = inf.unify(&HirType::Infer, &HirType::Integer64);
    assert!(result.is_ok());

    let result = inf.unify(&HirType::Integer64, &HirType::Infer);
    assert!(result.is_ok());
}

#[test]
fn test_unify_array_types() {
    let mut inf = TypeInference::new();
    let t1 = HirType::Array(Box::new(HirType::Integer64));
    let t2 = HirType::Array(Box::new(HirType::Integer64));
    let result = inf.unify(&t1, &t2);
    assert!(result.is_ok());
}

#[test]
fn test_unify_function_types() {
    let mut inf = TypeInference::new();
    let t1 = HirType::Function { params: vec![HirType::Integer64], return_type: Box::new(HirType::Boolean) };
    let t2 = HirType::Function { params: vec![HirType::Integer64], return_type: Box::new(HirType::Boolean) };
    let result = inf.unify(&t1, &t2);
    assert!(result.is_ok());
}

#[test]
fn test_apply_subst() {
    let inf = TypeInference::new();
    let t = HirType::Array(Box::new(HirType::Integer64));
    let result = inf.apply_subst(&t);
    assert_eq!(result, t);
}

#[test]
fn test_is_numeric() {
    let inf = TypeInference::new();

    assert!(inf.is_numeric(&HirType::Integer32));
    assert!(inf.is_numeric(&HirType::Integer64));
    assert!(inf.is_numeric(&HirType::Float32));
    assert!(inf.is_numeric(&HirType::Float64));

    assert!(!inf.is_numeric(&HirType::Boolean));
    assert!(!inf.is_numeric(&HirType::Utf8));
    assert!(!inf.is_numeric(&HirType::Unit));
}

#[test]
fn test_is_integer() {
    let inf = TypeInference::new();

    assert!(inf.is_integer(&HirType::Integer32));
    assert!(inf.is_integer(&HirType::Integer64));

    assert!(!inf.is_integer(&HirType::Float32));
    assert!(!inf.is_integer(&HirType::Float64));
    assert!(!inf.is_integer(&HirType::Boolean));
}

#[test]
fn test_clear() {
    let mut inf = TypeInference::new();
    inf.fresh_var();
    inf.fresh_var();
    inf.bind_variable(Identifier::new("x"), HirType::Integer64);

    inf.clear();

    let v = inf.fresh_var();
    assert_eq!(v, InferenceTypeVar(0));
    assert!(inf.get_variable_type(&Identifier::new("x")).is_none());
}

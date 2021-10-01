use ordered_float::OrderedFloat;
use valkyrie_compiler::{
    mir::{
        ssa::test_support::{block, expr, lower_test_function, lower_test_literal, lower_test_module, span, TestMirBuilder},
        MirBuiltinCall, MirBuiltinCompareOp,
    },
    MirConstant, MirDispatchKind, MirEffectKind, MirInstructionKind, MirOperand, MirTerminator, MirValueOrigin, ValkyrieCompiler,
};
use valkyrie_types::{
    hir::{
        HirDocumentation, HirExprKind, HirExtractorPattern, HirField, HirFunction, HirLiteral, HirMatchArm, HirPattern, HirResolvedCall,
        HirStatement, HirStatementKind, HirStringLiteral, HirStringSegment, HirStruct, HirVisibility, ValkyrieType,
    },
    Identifier, NamePath,
};

fn extractor_resolved(symbol: NamePath, return_type: ValkyrieType) -> HirResolvedCall {
    HirResolvedCall { symbol, domain: valkyrie_types::hir::HirCallableDomain::Extractor, return_type }
}

fn constructor_extractor_pattern(name: NamePath, fields: Vec<HirPattern>, symbol: NamePath, return_type: ValkyrieType) -> HirPattern {
    HirPattern::Extractor(HirExtractorPattern::Constructor {
        canonical_callee: {
            let mut parts = name.parts().to_vec();
            parts.push(Identifier::new("extractor"));
            NamePath::new(parts)
        },
        name,
        fields,
        resolved: Some(extractor_resolved(symbol, return_type)),
    })
}

fn array_extractor_pattern(
    prefix: Vec<HirPattern>,
    rest: Option<valkyrie_types::hir::HirIdentifier>,
    suffix: Vec<HirPattern>,
    symbol: NamePath,
    return_type: ValkyrieType,
) -> HirPattern {
    HirPattern::Extractor(HirExtractorPattern::Array {
        canonical_callee: NamePath::new(vec![Identifier::new("array"), Identifier::new("extractor")]),
        prefix,
        rest,
        suffix,
        resolved: Some(extractor_resolved(symbol, return_type)),
    })
}

#[test]
fn lowers_float64_literal_as_numeric_constant() {
    let literal = HirLiteral::Float64(OrderedFloat(3.5));

    let (constant, ty) = lower_test_literal(&literal, None);

    assert_eq!(constant, MirConstant::Float64(OrderedFloat(3.5)));
    assert_eq!(ty, Some(ValkyrieType::Float64));
}

#[test]
fn lowers_literal_catch_pattern_into_builtin_compare() {
    let mir = lower_test_function(expr(HirExprKind::Catch {
        expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true))))))),
        arms: vec![
            HirMatchArm {
                pattern: HirPattern::Literal(HirLiteral::Bool(true)),
                guard: None,
                body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
            },
            HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
        ],
    }));

    let guard_block = mir.blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call {
                dispatch: MirDispatchKind::Static,
                callee: MirOperand::Symbol(path),
                builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)),
                ..
            } if path == &NamePath::new(vec![Identifier::new("infix ==")])
        )
    }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn returns_constant_false_for_mismatched_literal_pattern_types() {
    let mut builder = TestMirBuilder::new();

    let matched = builder.lower_pattern_match_operand(&HirPattern::Literal(HirLiteral::Bool(true)), MirOperand::Constant(MirConstant::Int(1)));

    assert_eq!(matched, MirOperand::Constant(MirConstant::Bool(false)));
    assert!(builder.instructions().is_empty());
}

#[test]
fn returns_constant_bool_for_unit_literal_pattern() {
    let mut builder = TestMirBuilder::new();

    let matched = builder.lower_pattern_match_operand(&HirPattern::Literal(HirLiteral::Unit), MirOperand::Constant(MirConstant::Unit));

    assert_eq!(matched, MirOperand::Constant(MirConstant::Bool(true)));
    assert!(builder.instructions().is_empty());
    assert!(builder.values().iter().all(|value| !matches!(value.origin, MirValueOrigin::Temporary)));
}

#[test]
fn lowers_constant_string_literal_pattern_without_fallback() {
    let mut builder = TestMirBuilder::new();
    let literal =
        HirLiteral::String(HirStringLiteral { prefix: None, quote_count: 1, segments: vec![HirStringSegment::Text("hello".to_string())] });

    let matched =
        builder.lower_pattern_match_operand(&HirPattern::Literal(literal), MirOperand::Constant(MirConstant::String("hello".to_string())));

    assert_eq!(matched, MirOperand::Constant(MirConstant::Bool(true)));
    assert!(builder.instructions().is_empty());
}

#[test]
fn returns_constant_false_for_anonymous_object_pattern_on_known_scalar_without_fallback() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
            ty: ValkyrieType::Boolean,
        }],
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Match {
                scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("input"),
                    shadow_index: 0,
                    span: span(),
                }))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Object {
                            name: None,
                            fields: vec![(Identifier::new("flag"), HirPattern::Literal(HirLiteral::Bool(true)))],
                            rest: None,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0").expect("expected first match arm block");

    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
    assert!(matches!(&guard_block.terminator, MirTerminator::Branch { condition: MirOperand::Constant(MirConstant::Bool(false)), .. }));
}

#[test]
fn lowers_type_pattern_into_static_bool_when_operand_type_is_known() {
    let mut builder = TestMirBuilder::new();

    let matched = builder.lower_pattern_match_operand(
        &HirPattern::Type(NamePath::new(vec![Identifier::new("bool")])),
        MirOperand::Constant(MirConstant::Bool(true)),
    );

    assert_eq!(matched, MirOperand::Constant(MirConstant::Bool(true)));
    assert!(builder.instructions().is_empty());
}

#[test]
fn lowers_object_pattern_into_field_get_and_compare_for_single_field() {
    let point_struct = HirStruct {
        name: Identifier::new("Point"),
        fields: vec![HirField {
            name: Identifier::new("x"),
            doc: HirDocumentation::default(),
            ty: ValkyrieType::Boolean,
            visibility: HirVisibility::default(),
            is_readonly: false,
        }],
        ..HirStruct::new(Identifier::new("Point"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Point"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("x"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Object {
                            name: Some(NamePath::new(vec![Identifier::new("Point")])),
                            fields: vec![(Identifier::new("x"), HirPattern::Literal(HirLiteral::Bool(true)))],
                            rest: None,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![point_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { ref field, .. } if field == "x")));
    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)), .. })
    }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_named_object_pattern_for_subtype_into_field_get_and_compare() {
    let base_struct = HirStruct {
        name: Identifier::new("Base"),
        fields: vec![HirField {
            name: Identifier::new("flag"),
            doc: HirDocumentation::default(),
            ty: ValkyrieType::Boolean,
            visibility: HirVisibility::default(),
            is_readonly: false,
        }],
        ..HirStruct::new(Identifier::new("Base"))
    };
    let child_struct = HirStruct {
        name: Identifier::new("Child"),
        parents: vec![valkyrie_types::hir::HirParent::new(NamePath::new(vec![Identifier::new("Base")]))],
        ..HirStruct::new(Identifier::new("Child"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Child"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("flag"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Object {
                            name: Some(NamePath::new(vec![Identifier::new("Base")])),
                            fields: vec![(Identifier::new("flag"), HirPattern::Literal(HirLiteral::Bool(true)))],
                            rest: None,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![base_struct, child_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { ref field, .. } if field == "flag")));
    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)), .. })
    }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_anonymous_object_pattern_for_inherited_field_into_field_get_and_compare() {
    let base_struct = HirStruct {
        name: Identifier::new("Base"),
        fields: vec![HirField {
            name: Identifier::new("flag"),
            doc: HirDocumentation::default(),
            ty: ValkyrieType::Boolean,
            visibility: HirVisibility::default(),
            is_readonly: false,
        }],
        ..HirStruct::new(Identifier::new("Base"))
    };
    let child_struct = HirStruct {
        name: Identifier::new("Child"),
        parents: vec![valkyrie_types::hir::HirParent::new(NamePath::new(vec![Identifier::new("Base")]))],
        ..HirStruct::new(Identifier::new("Child"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Child"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("flag"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Object {
                            name: None,
                            fields: vec![(Identifier::new("flag"), HirPattern::Literal(HirLiteral::Bool(true)))],
                            rest: None,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![base_struct, child_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { ref field, .. } if field == "flag")));
    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)), .. })
    }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_constructor_pattern_into_extractor_call_and_payload_compare() {
    let point_struct = HirStruct {
        name: Identifier::new("Point"),
        fields: vec![HirField {
            name: Identifier::new("x"),
            doc: HirDocumentation::default(),
            ty: ValkyrieType::Boolean,
            visibility: HirVisibility::default(),
            is_readonly: false,
        }],
        ..HirStruct::new(Identifier::new("Point"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Point"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("x"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![
                    HirMatchArm {
                        pattern: constructor_extractor_pattern(
                            NamePath::new(vec![Identifier::new("Point")]),
                            vec![HirPattern::Literal(HirLiteral::Bool(true))],
                            NamePath::new(vec![Identifier::new("demo"), Identifier::new("point_extract")]),
                            ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Boolean]),
                        ),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![point_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                if path == &NamePath::new(vec![Identifier::new("demo"), Identifier::new("point_extract")])
        )
    }));
    assert!(
        guard_block
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(
                    &instruction.kind,
                    MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                        if path == &NamePath::new(vec![Identifier::new("tuple_get_0")])
                            || path == &NamePath::new(vec![Identifier::new("tuple_get_1")])
                )
            })
            .count()
            >= 2
    );
    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)), .. })
    }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { .. })));
}

#[test]
fn binds_constructor_pattern_field_from_extractor_payload_before_resume() {
    let point_struct = HirStruct {
        name: Identifier::new("Point"),
        fields: vec![HirField {
            name: Identifier::new("x"),
            doc: HirDocumentation::default(),
            ty: ValkyrieType::Boolean,
            visibility: HirVisibility::default(),
            is_readonly: false,
        }],
        ..HirStruct::new(Identifier::new("Point"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Unit,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Point"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("x"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![HirMatchArm {
                    pattern: constructor_extractor_pattern(
                        NamePath::new(vec![Identifier::new("Point")]),
                        vec![HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("x"),
                            shadow_index: 0,
                            span: span(),
                        })],
                        NamePath::new(vec![Identifier::new("demo"), Identifier::new("point_extract")]),
                        ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Boolean]),
                    ),
                    guard: None,
                    body: Box::new(expr(HirExprKind::Resume(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("x"),
                        shadow_index: 0,
                        span: span(),
                    })))))),
                }],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![point_struct]);
    let body_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0_match").expect("expected constructor match block");

    assert!(body_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                if path == &NamePath::new(vec![Identifier::new("tuple_get_1")])
        )
    }));
    assert!(body_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::StoreVar { name, .. } if name == "x") }));
    assert!(!body_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { .. })));
}

#[test]
fn does_not_bind_unknown_layout_constructor_field_as_whole_payload() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Unit,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Point"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("x"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![HirMatchArm {
                    pattern: constructor_extractor_pattern(
                        NamePath::new(vec![Identifier::new("Point")]),
                        vec![HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("x"),
                            shadow_index: 0,
                            span: span(),
                        })],
                        NamePath::new(vec![Identifier::new("demo"), Identifier::new("point_extract")]),
                        ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Boolean]),
                    ),
                    guard: None,
                    body: Box::new(expr(HirExprKind::Resume(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("x"),
                        shadow_index: 0,
                        span: span(),
                    })))))),
                }],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let match_block =
        mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected constructor extractor match block");
    let body_block = mir.functions[0]
        .blocks
        .iter()
        .find(|block| block.label == "catch_arm_0_match")
        .expect("expected constructor body block after fallback match");

    assert!(match_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                if path == &NamePath::new(vec![Identifier::new("demo"), Identifier::new("point_extract")])
        )
    }));
    assert!(body_block
        .instructions
        .iter()
        .any(|instruction| { matches!(instruction.kind, MirInstructionKind::StoreVar { ref name, .. } if name == "x") }));
    assert!(!match_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn does_not_bind_unknown_layout_object_field_as_generic_field_get() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Unit,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Point"),
                    args: vec![expr(HirExprKind::FieldInit {
                        name: Identifier::new("x"),
                        value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    })],
                    resolved: None,
                }))))),
                arms: vec![HirMatchArm {
                    pattern: HirPattern::Object {
                        name: Some(NamePath::new(vec![Identifier::new("Point")])),
                        fields: vec![(
                            Identifier::new("x"),
                            HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                                name: Identifier::new("x"),
                                shadow_index: 0,
                                span: span(),
                            }),
                        )],
                        rest: None,
                    },
                    guard: None,
                    body: Box::new(expr(HirExprKind::Resume(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("x"),
                        shadow_index: 0,
                        span: span(),
                    })))))),
                }],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let match_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected object fallback match block");
    let body_block = mir.functions[0]
        .blocks
        .iter()
        .find(|block| block.label == "catch_arm_0_match")
        .expect("expected object body block after fallback match");

    assert!(match_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
    assert!(!body_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { .. })));
    assert!(matches!(
        &body_block.terminator,
        MirTerminator::Jump { arguments, .. }
            if matches!(
                arguments.first(),
                Some(MirOperand::Symbol(path)) if path == &NamePath::new(vec![Identifier::new("unsupported_pattern")])
            )
    ));
}

#[test]
fn propagates_resume_value_type_into_catch_resume_parameter() {
    let mut builder = TestMirBuilder::new();
    let _ = builder.lower_expr_to_operand(&expr(HirExprKind::Catch {
        expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true))))))),
        arms: vec![HirMatchArm {
            pattern: HirPattern::Else,
            guard: None,
            body: Box::new(expr(HirExprKind::Resume(Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true))))))),
        }],
    }));

    let resume_block = builder.blocks().iter().find(|block| block.label == "catch_resume").expect("expected catch resume block");
    let resume_parameter = *resume_block.parameters.first().expect("expected catch resume parameter");
    assert_eq!(builder.value_types().get(&resume_parameter), Some(&ValkyrieType::Boolean));
}

#[test]
fn preseeds_catch_resume_parameter_type_from_raised_payload() {
    let mut builder = TestMirBuilder::new();
    let _ = builder.lower_expr_to_operand(&expr(HirExprKind::Catch {
        expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true))))))),
        arms: vec![HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Unit))) }],
    }));

    let resume_block = builder.blocks().iter().find(|block| block.label == "catch_resume").expect("expected catch resume block");
    let resume_parameter = *resume_block.parameters.first().expect("expected catch resume parameter");
    assert_eq!(builder.value_types().get(&resume_parameter), Some(&ValkyrieType::Boolean));
}

#[test]
fn records_catch_resume_continuation_metadata() {
    let mir = lower_test_function(expr(HirExprKind::Catch {
        expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true))))))),
        arms: vec![HirMatchArm {
            pattern: HirPattern::Else,
            guard: None,
            body: Box::new(expr(HirExprKind::Resume(Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true))))))),
        }],
    }));

    let continuation = mir.continuations.first().expect("expected continuation metadata");
    let resume_block = mir.blocks.iter().find(|block| block.id == continuation.resume_target).expect("expected catch resume block");
    assert_eq!(resume_block.label, "catch_resume");
    assert!(resume_block.parameters.contains(&continuation.resume_parameter));
    assert_eq!(continuation.resume_parameter_type, Some(ValkyrieType::Boolean));
}

#[test]
fn records_await_suspend_point_metadata() {
    let future_type = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let mut builder = TestMirBuilder::new();
    builder.lower_statement(&HirStatement {
        kind: HirStatementKind::Let {
            is_mutable: false,
            pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                name: Identifier::new("future"),
                shadow_index: 0,
                span: span(),
            }),
            initializer: Some(Box::new(expr(HirExprKind::Literal(HirLiteral::Unit)))),
            ty: Some(future_type.clone()),
        },
        span: span(),
    });
    let _ =
        builder.lower_expr_to_operand(&expr(HirExprKind::Await(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("future"),
            shadow_index: 0,
            span: span(),
        }))))));

    let suspend_point = builder.suspend_points().first().expect("expected suspend point");
    assert_eq!(suspend_point.effect, MirEffectKind::Await);
    assert_eq!(suspend_point.resume_parameter_count, 1);
    assert_eq!(suspend_point.payload_type, Some(future_type));
    assert!(!suspend_point.spill_candidates.is_empty());
}

#[test]
fn keeps_only_live_values_in_await_spill_candidates() {
    let compiler = ValkyrieCompiler::default();
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    let dropped: bool = false
    future.await
    let sink: bool = kept
    return
}
"#,
        )
        .expect("mir ok");

    let suspend_point = mir.functions[0].suspend_points.first().expect("expected suspend point");
    let spilled_names = suspend_point
        .spill_candidates
        .iter()
        .filter_map(|value| mir.functions[0].values.iter().find(|candidate| candidate.id == *value))
        .filter_map(|value| match &value.origin {
            MirValueOrigin::LetBinding { name } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(spilled_names, vec!["kept"]);
}

#[test]
fn builds_frame_layout_from_suspend_spill_candidates() {
    let compiler = ValkyrieCompiler::default();
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#,
        )
        .expect("mir ok");

    let suspend_point = mir.functions[0].suspend_points.first().expect("expected suspend point");
    let frame_layout = mir.functions[0].frame_layouts.first().expect("expected frame layout");
    let spilled_names = frame_layout
        .slots
        .iter()
        .filter_map(|slot| mir.functions[0].values.iter().find(|candidate| candidate.id == slot.value))
        .filter_map(|value| match &value.origin {
            MirValueOrigin::LetBinding { name } => Some(name.as_str()),
            _ => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(frame_layout.state_id, suspend_point.state_id);
    assert_eq!(frame_layout.resume_target, suspend_point.resume_target);
    assert_eq!(spilled_names, vec!["kept"]);
}

#[test]
fn infers_await_resume_parameter_type_from_future_payload() {
    let future_type = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let mut builder = TestMirBuilder::new();
    builder.lower_statement(&HirStatement {
        kind: HirStatementKind::Let {
            is_mutable: false,
            pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                name: Identifier::new("future"),
                shadow_index: 0,
                span: span(),
            }),
            initializer: Some(Box::new(expr(HirExprKind::Literal(HirLiteral::Unit)))),
            ty: Some(future_type),
        },
        span: span(),
    });
    let _ =
        builder.lower_expr_to_operand(&expr(HirExprKind::Await(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("future"),
            shadow_index: 0,
            span: span(),
        }))))));

    let resume_block = builder.blocks().iter().find(|block| block.label == "await_resume").expect("expected await resume block");
    let resume_parameter = *resume_block.parameters.first().expect("expected await resume parameter");
    assert_eq!(builder.value_types().get(&resume_parameter), Some(&ValkyrieType::Boolean));
}

#[test]
fn infers_block_resume_parameter_type_from_future_payload() {
    let promise_type = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Promise"))), vec![ValkyrieType::Utf8]);
    let mut builder = TestMirBuilder::new();
    builder.lower_statement(&HirStatement {
        kind: HirStatementKind::Let {
            is_mutable: false,
            pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                name: Identifier::new("future"),
                shadow_index: 0,
                span: span(),
            }),
            initializer: Some(Box::new(expr(HirExprKind::Literal(HirLiteral::Unit)))),
            ty: Some(promise_type),
        },
        span: span(),
    });
    let _ =
        builder.lower_expr_to_operand(&expr(HirExprKind::BlockOn(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("future"),
            shadow_index: 0,
            span: span(),
        }))))));

    let resume_block = builder.blocks().iter().find(|block| block.label == "block_resume").expect("expected block resume block");
    let resume_parameter = *resume_block.parameters.first().expect("expected block resume parameter");
    assert_eq!(builder.value_types().get(&resume_parameter), Some(&ValkyrieType::Utf8));
}

#[test]
fn lowers_awake_into_async_spawn_with_empty_resume_parameters() {
    let compiler = ValkyrieCompiler::default();
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    future.awake
    return
}
"#,
        )
        .expect("mir ok");

    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            MirTerminator::PerformEffect { effect: MirEffectKind::AsyncSpawn, resume_target, .. } => Some(*resume_target),
            _ => None,
        })
        .expect("expected awake perform effect");
    let resume_block = mir.functions[0].blocks.iter().find(|block| block.id == resume_target).expect("expected awake resume block");

    assert!(resume_block.parameters.is_empty());
}

#[test]
fn lowers_awake_as_unit_without_creating_resume_parameter() {
    let future_type = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let mut builder = TestMirBuilder::new();
    builder.lower_statement(&HirStatement {
        kind: HirStatementKind::Let {
            is_mutable: false,
            pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                name: Identifier::new("future"),
                shadow_index: 0,
                span: span(),
            }),
            initializer: Some(Box::new(expr(HirExprKind::Literal(HirLiteral::Unit)))),
            ty: Some(future_type),
        },
        span: span(),
    });

    let awake_result =
        builder.lower_expr_to_operand(&expr(HirExprKind::Awake(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("future"),
            shadow_index: 0,
            span: span(),
        }))))));

    assert_eq!(awake_result, MirOperand::Constant(MirConstant::Unit));
    let resume_block = builder.blocks().iter().find(|block| block.label == "awake_resume").expect("expected awake resume block");
    assert!(resume_block.parameters.is_empty());
}

#[test]
fn lowers_multi_field_object_pattern_into_logical_and() {
    let pair_struct = HirStruct {
        name: Identifier::new("Pair"),
        fields: vec![
            HirField {
                name: Identifier::new("left"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Boolean,
                visibility: HirVisibility::default(),
                is_readonly: false,
            },
            HirField {
                name: Identifier::new("right"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Boolean,
                visibility: HirVisibility::default(),
                is_readonly: false,
            },
        ],
        ..HirStruct::new(Identifier::new("Pair"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Pair"),
                    args: vec![
                        expr(HirExprKind::FieldInit {
                            name: Identifier::new("left"),
                            value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                        }),
                        expr(HirExprKind::FieldInit {
                            name: Identifier::new("right"),
                            value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                        }),
                    ],
                    resolved: None,
                }))))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Object {
                            name: Some(NamePath::new(vec![Identifier::new("Pair")])),
                            fields: vec![
                                (Identifier::new("left"), HirPattern::Literal(HirLiteral::Bool(true))),
                                (Identifier::new("right"), HirPattern::Literal(HirLiteral::Bool(true))),
                            ],
                            rest: None,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![pair_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert_eq!(
        guard_block.instructions.iter().filter(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { .. })).count(),
        2
    );
    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::LogicalAnd), .. }) }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_multi_field_constructor_pattern_into_extractor_call_and_payload_compare() {
    let pair_struct = HirStruct {
        name: Identifier::new("Pair"),
        fields: vec![
            HirField {
                name: Identifier::new("left"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Boolean,
                visibility: HirVisibility::default(),
                is_readonly: false,
            },
            HirField {
                name: Identifier::new("right"),
                doc: HirDocumentation::default(),
                ty: ValkyrieType::Boolean,
                visibility: HirVisibility::default(),
                is_readonly: false,
            },
        ],
        ..HirStruct::new(Identifier::new("Pair"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Catch {
                expr: Box::new(expr(HirExprKind::Raise(Box::new(expr(HirExprKind::Construct {
                    name: Identifier::new("Pair"),
                    args: vec![
                        expr(HirExprKind::FieldInit {
                            name: Identifier::new("left"),
                            value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                        }),
                        expr(HirExprKind::FieldInit {
                            name: Identifier::new("right"),
                            value: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                        }),
                    ],
                    resolved: None,
                }))))),
                arms: vec![
                    HirMatchArm {
                        pattern: constructor_extractor_pattern(
                            NamePath::new(vec![Identifier::new("Pair")]),
                            vec![HirPattern::Literal(HirLiteral::Bool(true)), HirPattern::Literal(HirLiteral::Bool(true))],
                            NamePath::new(vec![Identifier::new("demo"), Identifier::new("pair_extract")]),
                            ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Boolean, ValkyrieType::Boolean]),
                        ),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![pair_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "catch_arm_0").expect("expected first catch arm block");

    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                if path == &NamePath::new(vec![Identifier::new("demo"), Identifier::new("pair_extract")])
        )
    }));
    assert!(
        guard_block
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(
                    &instruction.kind,
                    MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                        if path == &NamePath::new(vec![Identifier::new("tuple_get_1")])
                            || path == &NamePath::new(vec![Identifier::new("tuple_get_2")])
                )
            })
            .count()
            >= 2
    );
    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::LogicalAnd), .. }) }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { .. })));
}

#[test]
fn lowers_tuple_pattern_into_tuple_get_and_compare_without_fallback() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
            ty: ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Boolean]),
        }],
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Match {
                scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("input"),
                    shadow_index: 0,
                    span: span(),
                }))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Tuple(vec![
                            HirPattern::Literal(HirLiteral::Bool(true)),
                            HirPattern::Literal(HirLiteral::Bool(true)),
                        ]),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0").expect("expected first match arm block");

    assert!(
        guard_block
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(
                    &instruction.kind,
                    MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                        if path == &NamePath::new(vec![Identifier::new("tuple_get_0")])
                            || path == &NamePath::new(vec![Identifier::new("tuple_get_1")])
                )
            })
            .count()
            >= 2
    );
    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)), .. })
    }));
    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::LogicalAnd), .. }) }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_or_pattern_into_logical_or_without_fallback() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
            ty: ValkyrieType::Boolean,
        }],
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Match {
                scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("input"),
                    shadow_index: 0,
                    span: span(),
                }))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Or(vec![
                            HirPattern::Literal(HirLiteral::Bool(false)),
                            HirPattern::Literal(HirLiteral::Bool(true)),
                        ]),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0").expect("expected first match arm block");

    assert!(
        guard_block
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Eq)), .. })
            })
            .count()
            >= 2
    );
    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::LogicalOr), .. }) }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_range_pattern_into_compare_chain_without_fallback() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
            ty: ValkyrieType::Integer32 { signed: true },
        }],
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Match {
                scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("input"),
                    shadow_index: 0,
                    span: span(),
                }))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Range {
                            start: Some(HirLiteral::Integer64(1)),
                            end: Some(HirLiteral::Integer64(10)),
                            inclusive_end: true,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0").expect("expected first match arm block");

    assert!(
        guard_block
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(
                    &instruction.kind,
                    MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Ge)), .. }
                        | MirInstructionKind::Call { builtin: Some(MirBuiltinCall::Compare(MirBuiltinCompareOp::Le)), .. }
                )
            })
            .count()
            >= 2
    );
    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::LogicalAnd), .. }) }));
    assert!(!guard_block.instructions.iter().any(|instruction| matches!(instruction.kind, MirInstructionKind::PatternMatch { .. })));
}

#[test]
fn lowers_array_rest_pattern_into_extractor_call_and_payload_bindings() {
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
            ty: ValkyrieType::Array(Box::new(ValkyrieType::Boolean)),
        }],
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Match {
                scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("input"),
                    shadow_index: 0,
                    span: span(),
                }))),
                arms: vec![
                    HirMatchArm {
                        pattern: array_extractor_pattern(
                            vec![HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                                name: Identifier::new("head"),
                                shadow_index: 0,
                                span: span(),
                            })],
                            Some(valkyrie_types::hir::HirIdentifier { name: Identifier::new("tail"), shadow_index: 0, span: span() }),
                            vec![HirPattern::Literal(HirLiteral::Bool(true))],
                            NamePath::new(vec![Identifier::new("demo"), Identifier::new("array_extract_bool")]),
                            ValkyrieType::Tuple(vec![
                                ValkyrieType::Boolean,
                                ValkyrieType::Boolean,
                                ValkyrieType::Array(Box::new(ValkyrieType::Boolean)),
                                ValkyrieType::Boolean,
                            ]),
                        ),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("head"),
                            shadow_index: 0,
                            span: span(),
                        }))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], Vec::new());
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0").expect("expected first match arm block");
    let body_block =
        mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0_check").expect("expected first match arm body block");

    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                if path == &NamePath::new(vec![Identifier::new("demo"), Identifier::new("array_extract_bool")])
        )
    }));
    assert!(
        guard_block
            .instructions
            .iter()
            .filter(|instruction| {
                matches!(
                    &instruction.kind,
                    MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                        if path == &NamePath::new(vec![Identifier::new("tuple_get_0")])
                            || path == &NamePath::new(vec![Identifier::new("tuple_get_1")])
                            || path == &NamePath::new(vec![Identifier::new("tuple_get_3")])
                )
            })
            .count()
            >= 3
    );
    assert!(!guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::ArrayLength), .. }) }));
    assert!(!guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::ArrayGet), .. }) }));
    assert!(body_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::StoreVar { name, .. } if name == "head" || name == "tail") }));
}

#[test]
fn lowers_nested_object_pattern_with_array_extractor_call() {
    let container_struct = HirStruct {
        name: Identifier::new("Container"),
        fields: vec![HirField {
            name: Identifier::new("items"),
            doc: HirDocumentation::default(),
            ty: ValkyrieType::Array(Box::new(ValkyrieType::Integer32 { signed: true })),
            visibility: HirVisibility::default(),
            is_readonly: false,
        }],
        ..HirStruct::new(Identifier::new("Container"))
    };
    let function = HirFunction {
        name: Identifier::new("main"),
        doc: HirDocumentation::default(),
        annotations: Vec::new(),
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
            ty: ValkyrieType::Named(Identifier::new("Container")),
        }],
        return_type: ValkyrieType::Boolean,
        body: block(
            Vec::new(),
            Some(expr(HirExprKind::Match {
                scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("input"),
                    shadow_index: 0,
                    span: span(),
                }))),
                arms: vec![
                    HirMatchArm {
                        pattern: HirPattern::Object {
                            name: Some(NamePath::new(vec![Identifier::new("Container")])),
                            fields: vec![(
                                Identifier::new("items"),
                                array_extractor_pattern(
                                    vec![HirPattern::Range {
                                        start: Some(HirLiteral::Integer64(1)),
                                        end: Some(HirLiteral::Integer64(10)),
                                        inclusive_end: true,
                                    }],
                                    Some(valkyrie_types::hir::HirIdentifier { name: Identifier::new("tail"), shadow_index: 0, span: span() }),
                                    vec![HirPattern::Literal(HirLiteral::Integer64(20))],
                                    NamePath::new(vec![Identifier::new("demo"), Identifier::new("array_extract_i32")]),
                                    ValkyrieType::Tuple(vec![
                                        ValkyrieType::Boolean,
                                        ValkyrieType::Integer32 { signed: true },
                                        ValkyrieType::Array(Box::new(ValkyrieType::Integer32 { signed: true })),
                                        ValkyrieType::Integer32 { signed: true },
                                    ]),
                                ),
                            )],
                            rest: None,
                        },
                        guard: None,
                        body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(true)))),
                    },
                    HirMatchArm { pattern: HirPattern::Else, guard: None, body: Box::new(expr(HirExprKind::Literal(HirLiteral::Bool(false)))) },
                ],
            })),
        ),
        span: span(),
        visibility: HirVisibility::default(),
        is_abstract: false,
        is_final: false,
    };

    let mir = lower_test_module(vec![function], vec![container_struct]);
    let guard_block = mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0").expect("expected first match arm block");
    let body_block =
        mir.functions[0].blocks.iter().find(|block| block.label == "match_arm_0_check").expect("expected first match arm body block");

    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { ref field, .. } if field == "items")));
    assert!(guard_block
        .instructions
        .iter()
        .any(|instruction| matches!(instruction.kind, MirInstructionKind::FieldGet { ref field, .. } if field == "items")));
    assert!(guard_block.instructions.iter().any(|instruction| {
        matches!(
            &instruction.kind,
            MirInstructionKind::Call { callee: MirOperand::Symbol(path), builtin: None, .. }
                if path == &NamePath::new(vec![Identifier::new("demo"), Identifier::new("array_extract_i32")])
        )
    }));
    assert!(!guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::ArrayLength), .. }) }));
    assert!(!guard_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::Call { builtin: Some(MirBuiltinCall::ArrayGet), .. }) }));
    assert!(body_block
        .instructions
        .iter()
        .any(|instruction| { matches!(&instruction.kind, MirInstructionKind::StoreVar { name, .. } if name == "tail") }));
}

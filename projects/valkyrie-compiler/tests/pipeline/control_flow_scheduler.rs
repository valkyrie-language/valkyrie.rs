use ordered_float::OrderedFloat;
use valkyrie_compiler::{
    lir::{LirEffectKind, LirOperand, LirTerminator},
    mir::{MirConstant, MirEffectKind, MirOperand, MirTerminator, MirValueRef},
    ControlFlowScheduler, MirLowerer, ValkyrieCompiler,
};
use valkyrie_types::{
    hir::{
        HirBlock, HirDocumentation, HirExpr, HirExprKind, HirFunction, HirMatchArm, HirModule, HirPattern, HirStatement, HirStatementKind,
        HirVisibility, ValkyrieType,
    },
    Identifier, NamePath, SourceID, SourceSpan,
};

fn span() -> SourceSpan {
    SourceSpan::new(SourceID::default(), 0, 0)
}

fn expr(kind: HirExprKind) -> HirExpr {
    HirExpr { kind, span: span() }
}

fn demo_module(body_expr: HirExpr) -> HirModule {
    HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Unit,
            body: HirBlock { statements: Vec::new(), expr: Some(Box::new(body_expr)), span: span() },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    }
}

#[test]
fn rejects_resume_inside_catch_guard_at_hir_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Unit,
            body: HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(expr(HirExprKind::Catch {
                    expr: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("task"),
                        shadow_index: 0,
                        span: span(),
                    }))),
                    arms: vec![HirMatchArm {
                        pattern: HirPattern::Else,
                        guard: Some(Box::new(expr(HirExprKind::Resume(Box::new(expr(HirExprKind::Variable(
                            valkyrie_types::hir::HirIdentifier { name: Identifier::new("value"), shadow_index: 0, span: span() },
                        ))))))),
                        body: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("value"),
                            shadow_index: 0,
                            span: span(),
                        }))),
                    }],
                }))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("catch arm body"));
    assert!(error.to_string().contains("resume"));
}

#[test]
fn rejects_block_inside_catch_guard_at_hir_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Unit,
            body: HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(expr(HirExprKind::Catch {
                    expr: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("task"),
                        shadow_index: 0,
                        span: span(),
                    }))),
                    arms: vec![HirMatchArm {
                        pattern: HirPattern::Else,
                        guard: Some(Box::new(expr(HirExprKind::BlockOn(Box::new(expr(HirExprKind::Variable(
                            valkyrie_types::hir::HirIdentifier { name: Identifier::new("future"), shadow_index: 0, span: span() },
                        ))))))),
                        body: Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Unit))),
                    }],
                }))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("guard"));
    assert!(error.to_string().contains("block"));
}

#[test]
fn rejects_break_expr_without_value_loop_context_at_hir_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Unit,
            body: HirBlock {
                statements: vec![HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Loop {
                        label: None,
                        pattern: None,
                        iterator: None,
                        condition: None,
                        body: Box::new(HirBlock {
                            statements: vec![HirStatement {
                                kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                                    label: None,
                                    expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                                }))),
                                span: span(),
                            }],
                            expr: None,
                            span: span(),
                        }),
                    }))),
                    span: span(),
                }],
                expr: None,
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("break expr"));
    assert!(error.to_string().contains("不接受值"));
}

#[test]
fn rejects_incompatible_break_expr_types_inside_same_value_loop() {
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("break expr"));
    assert!(error.to_string().contains("i64"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn infers_block_and_if_types_for_break_expr_validation() {
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Block(Box::new(HirBlock {
                            statements: Vec::new(),
                            expr: Some(Box::new(expr(HirExprKind::If {
                                condition: Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true)))),
                                then_branch: Box::new(HirBlock {
                                    statements: Vec::new(),
                                    expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                                    span: span(),
                                }),
                                else_branch: Some(Box::new(HirBlock {
                                    statements: Vec::new(),
                                    expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(2))))),
                                    span: span(),
                                })),
                            }))),
                            span: span(),
                        }))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("i64"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn infers_match_types_for_break_expr_validation() {
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Match {
                            scrutinee: Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true)))),
                            arms: vec![
                                HirMatchArm {
                                    pattern: HirPattern::Literal(valkyrie_types::hir::HirLiteral::Bool(true)),
                                    guard: None,
                                    body: Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1)))),
                                },
                                HirMatchArm {
                                    pattern: HirPattern::Else,
                                    guard: None,
                                    body: Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(2)))),
                                },
                            ],
                        }))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Float64(OrderedFloat(1.0)))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("i64"));
    assert!(error.to_string().contains("f64"));
}

#[test]
fn infers_let_bound_variable_type_for_break_expr_validation() {
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Let {
                        is_mutable: false,
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("value"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                        ty: Some(ValkyrieType::Integer64 { signed: true }),
                    },
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("value"),
                            shadow_index: 0,
                            span: span(),
                        })))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("i64"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn infers_await_and_block_result_types_for_break_expr_validation() {
    let future_bool = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let promise_bool = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Promise"))), vec![ValkyrieType::Boolean]);
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Let {
                        is_mutable: false,
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("future"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Unit)))),
                        ty: Some(future_bool),
                    },
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Let {
                        is_mutable: false,
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("promise"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Unit)))),
                        ty: Some(promise_bool),
                    },
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Await(Box::new(expr(HirExprKind::Variable(
                            valkyrie_types::hir::HirIdentifier { name: Identifier::new("future"), shadow_index: 0, span: span() },
                        ))))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::BlockOn(Box::new(expr(HirExprKind::Variable(
                            valkyrie_types::hir::HirIdentifier { name: Identifier::new("promise"), shadow_index: 0, span: span() },
                        ))))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn rejects_await_on_non_future_operand_at_hir_validation() {
    let module = demo_module(expr(HirExprKind::Block(Box::new(HirBlock {
        statements: vec![HirStatement {
            kind: HirStatementKind::Let {
                is_mutable: false,
                pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("value"),
                    shadow_index: 0,
                    span: span(),
                }),
                initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                ty: Some(ValkyrieType::Integer64 { signed: true }),
            },
            span: span(),
        }],
        expr: Some(Box::new(expr(HirExprKind::Await(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("value"),
            shadow_index: 0,
            span: span(),
        }))))))),
        span: span(),
    }))));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("`await`"));
    assert!(error.to_string().contains("Future<T>"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn rejects_awake_on_non_future_operand_at_hir_validation() {
    let module = demo_module(expr(HirExprKind::Block(Box::new(HirBlock {
        statements: vec![HirStatement {
            kind: HirStatementKind::Let {
                is_mutable: false,
                pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("flag"),
                    shadow_index: 0,
                    span: span(),
                }),
                initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true))))),
                ty: Some(ValkyrieType::Boolean),
            },
            span: span(),
        }],
        expr: Some(Box::new(expr(HirExprKind::Awake(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("flag"),
            shadow_index: 0,
            span: span(),
        }))))))),
        span: span(),
    }))));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("`awake`"));
    assert!(error.to_string().contains("Future<T>"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn rejects_block_on_non_future_operand_at_hir_validation() {
    let module = demo_module(expr(HirExprKind::Block(Box::new(HirBlock {
        statements: vec![HirStatement {
            kind: HirStatementKind::Let {
                is_mutable: false,
                pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                    name: Identifier::new("text"),
                    shadow_index: 0,
                    span: span(),
                }),
                initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::String(
                    valkyrie_types::hir::HirStringLiteral {
                        prefix: None,
                        quote_count: 1,
                        segments: vec![valkyrie_types::hir::HirStringSegment::Text("hello".to_string())],
                    },
                ))))),
                ty: Some(ValkyrieType::Utf8),
            },
            span: span(),
        }],
        expr: Some(Box::new(expr(HirExprKind::BlockOn(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
            name: Identifier::new("text"),
            shadow_index: 0,
            span: span(),
        }))))))),
        span: span(),
    }))));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("`block`"));
    assert!(error.to_string().contains("Future<T>"));
    assert!(error.to_string().contains("utf8"));
}

#[test]
fn rejects_block_inside_lambda_body_without_blocking_context() {
    let future_bool = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let module = demo_module(expr(HirExprKind::Lambda {
        generics: Vec::new(),
        params: vec![valkyrie_types::hir::HirParam {
            name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("future"), shadow_index: 0, span: span() },
            ty: future_bool,
        }],
        return_type: ValkyrieType::Boolean,
        body: Box::new(HirBlock {
            statements: Vec::new(),
            expr: Some(Box::new(expr(HirExprKind::BlockOn(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                name: Identifier::new("future"),
                shadow_index: 0,
                span: span(),
            }))))))),
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("`block`"));
    assert!(error.to_string().contains("允许阻塞"));
}

#[test]
fn rejects_yield_inside_lambda_body_without_generator_context() {
    let module = demo_module(expr(HirExprKind::Lambda {
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Unit,
        body: Box::new(HirBlock {
            statements: Vec::new(),
            expr: Some(Box::new(expr(HirExprKind::Yield(Some(Box::new(expr(HirExprKind::Literal(
                valkyrie_types::hir::HirLiteral::Integer64(1),
            )))))))),
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("`yield`"));
    assert!(error.to_string().contains("生成器函数"));
}

#[test]
fn accepts_bare_yield_as_unit_payload_sugar() {
    let module = demo_module(expr(HirExprKind::Yield(None)));
    ControlFlowScheduler::validate_hir_module(&module).expect("bare yield should lower as yield unit");
}

#[test]
fn lowers_bare_yield_into_unit_payload_effect() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 207 });
    let mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    yield
    return
}
"#,
        )
        .unwrap();
    let effect_block = mir.functions[0]
        .blocks
        .iter()
        .find(|block| matches!(block.terminator, MirTerminator::PerformEffect { effect, .. } if effect == MirEffectKind::Yield))
        .expect("expected yield perform effect");
    assert!(matches!(effect_block.terminator, MirTerminator::PerformEffect { payload: Some(MirOperand::Constant(MirConstant::Unit)), .. }));
}

#[test]
fn rejects_yield_from_inside_lambda_body_without_generator_context() {
    let module = demo_module(expr(HirExprKind::Lambda {
        generics: Vec::new(),
        params: Vec::new(),
        return_type: ValkyrieType::Unit,
        body: Box::new(HirBlock {
            statements: Vec::new(),
            expr: Some(Box::new(expr(HirExprKind::YieldFrom(Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                name: Identifier::new("values"),
                shadow_index: 0,
                span: span(),
            }))))))),
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("`yield from`"));
    assert!(error.to_string().contains("生成器函数"));
}

#[test]
fn infers_awake_as_unit_for_break_expr_validation() {
    let future_bool = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Let {
                        is_mutable: false,
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("future"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Unit)))),
                        ty: Some(future_bool),
                    },
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Awake(Box::new(expr(HirExprKind::Variable(
                            valkyrie_types::hir::HirIdentifier { name: Identifier::new("future"), shadow_index: 0, span: span() },
                        ))))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("unit"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn infers_yield_as_unit_for_break_expr_validation() {
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Yield(Some(Box::new(expr(HirExprKind::Literal(
                            valkyrie_types::hir::HirLiteral::Integer64(1),
                        )))))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Bool(true))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("unit"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn infers_yield_from_as_unit_for_break_expr_validation() {
    let module = demo_module(expr(HirExprKind::Loop {
        label: None,
        pattern: None,
        iterator: None,
        condition: None,
        body: Box::new(HirBlock {
            statements: vec![
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::YieldFrom(Box::new(expr(HirExprKind::Variable(
                            valkyrie_types::hir::HirIdentifier { name: Identifier::new("values"), shadow_index: 0, span: span() },
                        ))))))),
                    }))),
                    span: span(),
                },
                HirStatement {
                    kind: HirStatementKind::Expr(Box::new(expr(HirExprKind::Break {
                        label: None,
                        expr: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                    }))),
                    span: span(),
                },
            ],
            expr: None,
            span: span(),
        }),
    }));

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("unit"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn rejects_incompatible_return_expr_type_at_hir_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Integer64 { signed: true },
            body: HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(expr(HirExprKind::Return(Some(Box::new(expr(HirExprKind::Literal(
                    valkyrie_types::hir::HirLiteral::Bool(true),
                )))))))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("return expr"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn infers_typed_variable_for_return_expr_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Boolean,
            body: HirBlock {
                statements: vec![HirStatement {
                    kind: HirStatementKind::Let {
                        is_mutable: false,
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("value"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Integer64(1))))),
                        ty: Some(ValkyrieType::Integer64 { signed: true }),
                    },
                    span: span(),
                }],
                expr: Some(Box::new(expr(HirExprKind::Return(Some(Box::new(expr(HirExprKind::Variable(
                    valkyrie_types::hir::HirIdentifier { name: Identifier::new("value"), shadow_index: 0, span: span() },
                )))))))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("return expr"));
    assert!(error.to_string().contains("i64"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn infers_await_result_type_for_return_expr_validation() {
    let future_bool = ValkyrieType::Apply(Box::new(ValkyrieType::Named(Identifier::new("Future"))), vec![ValkyrieType::Boolean]);
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Integer64 { signed: true },
            body: HirBlock {
                statements: vec![HirStatement {
                    kind: HirStatementKind::Let {
                        is_mutable: false,
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("future"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        initializer: Some(Box::new(expr(HirExprKind::Literal(valkyrie_types::hir::HirLiteral::Unit)))),
                        ty: Some(future_bool),
                    },
                    span: span(),
                }],
                expr: Some(Box::new(expr(HirExprKind::Return(Some(Box::new(expr(HirExprKind::Await(Box::new(expr(
                    HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("future"),
                        shadow_index: 0,
                        span: span(),
                    }),
                )))))))))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("return expr"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn infers_match_arm_pattern_variable_for_return_expr_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: vec![valkyrie_types::hir::HirParam {
                name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
                ty: ValkyrieType::Integer64 { signed: true },
            }],
            return_type: ValkyrieType::Boolean,
            body: HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(expr(HirExprKind::Return(Some(Box::new(expr(HirExprKind::Match {
                    scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("input"),
                        shadow_index: 0,
                        span: span(),
                    }))),
                    arms: vec![HirMatchArm {
                        pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("captured"),
                            shadow_index: 0,
                            span: span(),
                        }),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("captured"),
                            shadow_index: 0,
                            span: span(),
                        }))),
                    }],
                }))))))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("return expr"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn infers_nested_tuple_match_arm_pattern_variable_for_return_expr_validation() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: vec![valkyrie_types::hir::HirParam {
                name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
                ty: ValkyrieType::Tuple(vec![
                    ValkyrieType::Integer64 { signed: true },
                    ValkyrieType::Tuple(vec![ValkyrieType::Boolean, ValkyrieType::Integer64 { signed: true }]),
                ]),
            }],
            return_type: ValkyrieType::Integer64 { signed: true },
            body: HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(expr(HirExprKind::Return(Some(Box::new(expr(HirExprKind::Match {
                    scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("input"),
                        shadow_index: 0,
                        span: span(),
                    }))),
                    arms: vec![HirMatchArm {
                        pattern: HirPattern::Tuple(vec![
                            HirPattern::Wildcard,
                            HirPattern::Tuple(vec![
                                HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                                    name: Identifier::new("captured"),
                                    shadow_index: 0,
                                    span: span(),
                                }),
                                HirPattern::Wildcard,
                            ]),
                        ]),
                        guard: None,
                        body: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                            name: Identifier::new("captured"),
                            shadow_index: 0,
                            span: span(),
                        }))),
                    }],
                }))))))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("return expr"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("i64"));
}

#[test]
fn rejects_case_fallthrough_when_next_arm_tries_to_reuse_previous_pattern_binding() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: vec![valkyrie_types::hir::HirParam {
                name: valkyrie_types::hir::HirIdentifier { name: Identifier::new("input"), shadow_index: 0, span: span() },
                ty: ValkyrieType::Integer64 { signed: true },
            }],
            return_type: ValkyrieType::Unit,
            body: HirBlock {
                statements: Vec::new(),
                expr: Some(Box::new(expr(HirExprKind::Case {
                    scrutinee: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                        name: Identifier::new("input"),
                        shadow_index: 0,
                        span: span(),
                    }))),
                    arms: vec![
                        HirMatchArm {
                            pattern: HirPattern::Variable(valkyrie_types::hir::HirIdentifier {
                                name: Identifier::new("captured"),
                                shadow_index: 0,
                                span: span(),
                            }),
                            guard: None,
                            body: Box::new(expr(HirExprKind::Fallthrough)),
                        },
                        HirMatchArm {
                            pattern: HirPattern::Else,
                            guard: None,
                            body: Box::new(expr(HirExprKind::Variable(valkyrie_types::hir::HirIdentifier {
                                name: Identifier::new("captured"),
                                shadow_index: 0,
                                span: span(),
                            }))),
                        },
                    ],
                }))),
                span: span(),
            },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("fallthrough"));
    assert!(error.to_string().contains("captured"));
}

#[test]
fn rejects_empty_return_for_non_unit_function() {
    let module = HirModule {
        name: NamePath::new(vec![Identifier::new("demo")]),
        doc: HirDocumentation::default(),
        imports: Vec::new(),
        submodules: Vec::new(),
        functions: vec![HirFunction {
            name: Identifier::new("main"),
            doc: HirDocumentation::default(),
            annotations: Vec::new(),
            generics: Vec::new(),
            params: Vec::new(),
            return_type: ValkyrieType::Boolean,
            body: HirBlock { statements: Vec::new(), expr: Some(Box::new(expr(HirExprKind::Return(None)))), span: span() },
            span: span(),
            visibility: HirVisibility::default(),
            is_abstract: false,
            is_final: false,
        }],
        structs: Vec::new(),
        enums: Vec::new(),
        flags: Vec::new(),
        traits: Vec::new(),
        impls: Vec::new(),
        type_functions: Vec::new(),
        type_families: Vec::new(),
        widgets: Vec::new(),
        singletons: Vec::new(),
        statements: Vec::new(),
    };

    let error = ControlFlowScheduler::validate_hir_module(&module).unwrap_err();
    assert!(error.to_string().contains("无值 `return`"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn rejects_mir_yield_resume_block_without_parameter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 201 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    yield 1
    return
}
"#,
        )
        .unwrap();
    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            MirTerminator::PerformEffect { effect, resume_target, .. } if *effect == MirEffectKind::Yield => Some(*resume_target),
            _ => None,
        })
        .expect("expected yield perform effect");
    let resume_block = mir.functions[0].blocks.iter_mut().find(|block| block.id == resume_target).expect("expected yield resume block");
    resume_block.parameters.clear();

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("`MIR`"));
    assert!(error.to_string().contains("effect 恢复点参数个数"));
}

#[test]
fn rejects_lir_awake_resume_block_with_unexpected_parameter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 202 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    future.awake
    return
}
"#,
        )
        .unwrap();
    let resume_target = lir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            LirTerminator::PerformEffect { effect, resume_target, .. } if matches!(effect, LirEffectKind::AsyncSpawn) => Some(*resume_target),
            _ => None,
        })
        .expect("expected awake perform effect");
    let resume_block = lir.functions[0].blocks.iter_mut().find(|block| block.id == resume_target).expect("expected awake resume block");
    resume_block.parameters.push(MirValueRef(999));

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("`LIR`"));
    assert!(error.to_string().contains("effect 恢复点参数个数"));
}

#[test]
fn rejects_pipeline_when_effect_resume_parameter_shape_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 203 });
    let source = r#"micro main() {
    future.await
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            MirTerminator::PerformEffect { effect, resume_target, .. } if *effect == MirEffectKind::Await => Some(*resume_target),
            _ => None,
        })
        .expect("expected await perform effect");
    let resume_block = lir.functions[0].blocks.iter_mut().find(|block| block.id == resume_target).expect("expected await resume block");
    resume_block.parameters.clear();

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("effect 恢复点参数形状不一致"));
}

#[test]
fn rejects_mir_yield_resume_block_with_non_unit_parameter_type() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 203 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    yield 1
    return
}
"#,
        )
        .unwrap();
    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            MirTerminator::PerformEffect { effect: MirEffectKind::Yield, resume_target, .. } => Some(*resume_target),
            _ => None,
        })
        .expect("expected yield perform effect");
    let resume_parameter = *mir.functions[0]
        .blocks
        .iter()
        .find(|block| block.id == resume_target)
        .and_then(|block| block.parameters.first())
        .expect("expected yield resume parameter");
    mir.functions[0].value_types.insert(resume_parameter, ValkyrieType::Boolean);

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("恢复点参数类型"));
    assert!(error.to_string().contains("unit"));
}

#[test]
fn rejects_mir_await_perform_effect_without_payload() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 204 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    future.await
    return
}
"#,
        )
        .unwrap();
    let effect_block = mir.functions[0]
        .blocks
        .iter_mut()
        .find(|block| matches!(block.terminator, MirTerminator::PerformEffect { effect, .. } if effect == MirEffectKind::Await))
        .expect("expected await perform effect");
    if let MirTerminator::PerformEffect { payload, .. } = &mut effect_block.terminator {
        *payload = None;
    }

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("`MIR`"));
    assert!(error.to_string().contains("effect payload"));
}

#[test]
fn rejects_lir_block_perform_effect_without_payload() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 205 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    future.block
    return
}
"#,
        )
        .unwrap();
    let effect_block = lir.functions[0]
        .blocks
        .iter_mut()
        .find(|block| matches!(block.terminator, LirTerminator::PerformEffect { effect, .. } if matches!(effect, LirEffectKind::AsyncBlock)))
        .expect("expected block perform effect");
    if let LirTerminator::PerformEffect { payload, .. } = &mut effect_block.terminator {
        *payload = None;
    }

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("`LIR`"));
    assert!(error.to_string().contains("effect payload"));
}

#[test]
fn rejects_lir_block_resume_block_with_wrong_parameter_type() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 211 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    yield 1
    return
}
"#,
        )
        .unwrap();
    let resume_target = lir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            LirTerminator::PerformEffect { effect: LirEffectKind::Yield, resume_target, .. } => Some(*resume_target),
            _ => None,
        })
        .expect("expected yield perform effect");
    let resume_parameter = *lir.functions[0]
        .blocks
        .iter()
        .find(|block| block.id == resume_target)
        .and_then(|block| block.parameters.first())
        .expect("expected yield resume parameter");
    lir.functions[0].value_types.insert(resume_parameter, ValkyrieType::Boolean);

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("`LIR`"));
    assert!(error.to_string().contains("恢复点参数类型"));
    assert!(error.to_string().contains("unit"));
}

#[test]
fn rejects_pipeline_when_effect_payload_shape_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 206 });
    let source = r#"micro main() {
    future.await
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    let effect_block = lir.functions[0]
        .blocks
        .iter_mut()
        .find(|block| matches!(block.terminator, LirTerminator::PerformEffect { effect, .. } if matches!(effect, LirEffectKind::Await)))
        .expect("expected await perform effect");
    if let LirTerminator::PerformEffect { payload, .. } = &mut effect_block.terminator {
        *payload = None;
    }

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("effect payload"));
}

#[test]
fn rejects_mir_await_perform_effect_with_non_future_payload_type() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 208 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    future.await
    return
}
"#,
        )
        .unwrap();
    let effect_block = mir.functions[0]
        .blocks
        .iter_mut()
        .find(|block| matches!(block.terminator, MirTerminator::PerformEffect { effect, .. } if effect == MirEffectKind::Await))
        .expect("expected await perform effect");
    if let MirTerminator::PerformEffect { payload, .. } = &mut effect_block.terminator {
        *payload = Some(MirOperand::Constant(MirConstant::Bool(true)));
    }

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("`MIR`"));
    assert!(error.to_string().contains("`await`"));
    assert!(error.to_string().contains("Future<T>` / `Promise<T>`"));
}

#[test]
fn rejects_lir_awake_perform_effect_with_non_future_payload_type() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 209 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    future.awake
    return
}
"#,
        )
        .unwrap();
    let effect_block = lir.functions[0]
        .blocks
        .iter_mut()
        .find(|block| matches!(block.terminator, LirTerminator::PerformEffect { effect, .. } if matches!(effect, LirEffectKind::AsyncSpawn)))
        .expect("expected awake perform effect");
    if let LirTerminator::PerformEffect { payload, .. } = &mut effect_block.terminator {
        *payload = Some(LirOperand::Constant(MirConstant::Bool(true)));
    }

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("`LIR`"));
    assert!(error.to_string().contains("`awake`"));
    assert!(error.to_string().contains("Future<T>` / `Promise<T>`"));
}

#[test]
fn rejects_pipeline_when_effect_payload_static_type_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 210 });
    let source = r#"micro main() {
    future.await
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    let effect_block = lir.functions[0]
        .blocks
        .iter_mut()
        .find(|block| matches!(block.terminator, LirTerminator::PerformEffect { effect, .. } if matches!(effect, LirEffectKind::Await)))
        .expect("expected await perform effect");
    if let LirTerminator::PerformEffect { payload, .. } = &mut effect_block.terminator {
        *payload = Some(LirOperand::Constant(MirConstant::Bool(true)));
    }

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("`LIR`"));
    assert!(error.to_string().contains("`await`"));
    assert!(error.to_string().contains("Future<T>` / `Promise<T>`"));
}

#[test]
fn rejects_pipeline_when_effect_resume_static_type_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 212 });
    let source = r#"micro main() {
    yield 1
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    let resume_target = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            MirTerminator::PerformEffect { effect: MirEffectKind::Yield, resume_target, .. } => Some(*resume_target),
            _ => None,
        })
        .expect("expected yield perform effect");
    let resume_parameter = *lir.functions[0]
        .blocks
        .iter()
        .find(|block| block.id == resume_target)
        .and_then(|block| block.parameters.first())
        .expect("expected yield resume parameter");
    lir.functions[0].value_types.insert(resume_parameter, ValkyrieType::Boolean);

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("恢复点参数类型"));
    assert!(error.to_string().contains("unit"));
}

#[test]
fn rejects_pipeline_when_catch_resume_parameter_type_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 213 });
    let source = r#"micro main() {
    catch raise true {
        default:
            resume true
    }
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    let resume_parameter = *lir.functions[0]
        .blocks
        .iter()
        .find(|block| block.label == "catch_resume")
        .and_then(|block| block.parameters.first())
        .expect("expected catch resume parameter");
    lir.functions[0].value_types.insert(resume_parameter, ValkyrieType::Unit);

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("catch_resume"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("unit"));
}

#[test]
fn rejects_mir_jump_argument_type_drift_to_catch_resume_parameter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 214 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let flag: bool = true
    catch raise true {
        else:
            resume flag
    }
    return
}
"#,
        )
        .unwrap();
    let resume_target =
        mir.functions[0].blocks.iter().find(|block| block.label == "catch_resume").map(|block| block.id).expect("expected catch resume block");
    let jump_argument = mir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            MirTerminator::Jump { target, arguments } if *target == resume_target => match arguments.first() {
                Some(MirOperand::Value(value)) => Some(*value),
                _ => None,
            },
            _ => None,
        })
        .expect("expected jump into catch resume block");
    mir.functions[0].value_types.insert(jump_argument, ValkyrieType::Unit);

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("Jump 参数类型"));
    assert!(error.to_string().contains("catch_resume"));
    assert!(error.to_string().contains("unit"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn rejects_lir_jump_argument_type_drift_to_catch_resume_parameter() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 215 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let flag: bool = true
    catch raise true {
        else:
            resume flag
    }
    return
}
"#,
        )
        .unwrap();
    let resume_target =
        lir.functions[0].blocks.iter().find(|block| block.label == "catch_resume").map(|block| block.id).expect("expected catch resume block");
    let jump_argument = lir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            LirTerminator::Jump { target, arguments } if *target == resume_target => match arguments.first() {
                Some(LirOperand::Value(value)) => Some(*value),
                _ => None,
            },
            _ => None,
        })
        .expect("expected jump into catch resume block");
    lir.functions[0].value_types.insert(jump_argument, ValkyrieType::Unit);

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("Jump 参数类型"));
    assert!(error.to_string().contains("catch_resume"));
    assert!(error.to_string().contains("unit"));
    assert!(error.to_string().contains("bool"));
}

#[test]
fn rejects_pipeline_when_jump_argument_type_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 216 });
    let source = r#"micro main() {
    let flag: bool = true
    catch raise true {
        else:
            resume flag
    }
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    let resume_target =
        lir.functions[0].blocks.iter().find(|block| block.label == "catch_resume").map(|block| block.id).expect("expected catch resume block");
    let jump_argument = lir.functions[0]
        .blocks
        .iter()
        .find_map(|block| match &block.terminator {
            LirTerminator::Jump { target, arguments } if *target == resume_target => match arguments.first() {
                Some(LirOperand::Value(value)) => Some(*value),
                _ => None,
            },
            _ => None,
        })
        .expect("expected jump into catch resume block");
    lir.functions[0].value_types.insert(jump_argument, ValkyrieType::Unit);

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("Jump 参数类型"));
    assert!(error.to_string().contains("catch_arm_0"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("unit"));
}

#[test]
fn rejects_mir_continuation_when_resume_parameter_leaves_target_block() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 217 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    catch raise true {
        else:
            resume true
    }
    return
}
"#,
        )
        .unwrap();
    mir.functions[0].continuations[0].resume_parameter = MirValueRef(999);

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("continuation"));
    assert!(error.to_string().contains("恢复参数"));
}

#[test]
fn rejects_pipeline_when_continuation_resume_type_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 218 });
    let source = r#"micro main() {
    catch raise true {
        else:
            resume true
    }
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    lir.functions[0].continuations[0].resume_parameter_type = Some(ValkyrieType::Unit);

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("continuation"));
    assert!(error.to_string().contains("恢复类型"));
    assert!(error.to_string().contains("bool"));
    assert!(error.to_string().contains("unit"));
}

#[test]
fn rejects_mir_suspend_point_when_resume_parameter_count_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 219 });
    let mut mir = compiler
        .compile_source_to_mir(
            r#"micro main() {
    let future: Future<bool> = ()
    future.await
    return
}
"#,
        )
        .unwrap();
    mir.functions[0].suspend_points[0].resume_parameter_count = 0;

    let error = ControlFlowScheduler::validate_mir_module(&mir).unwrap_err();
    assert!(error.to_string().contains("suspend"));
    assert!(error.to_string().contains("恢复参数个数"));
}

#[test]
fn rejects_pipeline_when_suspend_point_payload_type_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 220 });
    let source = r#"micro main() {
    let future: Future<bool> = ()
    future.await
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    lir.functions[0].suspend_points[0].payload_type = Some(ValkyrieType::Unit);

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("suspend"));
    assert!(error.to_string().contains("元数据"));
}

#[test]
fn rejects_pipeline_when_suspend_point_spill_candidates_drift() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 221 });
    let source = r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    lir.functions[0].suspend_points[0].spill_candidates.clear();

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("suspend"));
    assert!(error.to_string().contains("元数据"));
}

#[test]
fn rejects_pipeline_when_frame_layout_slots_drift() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 222 });
    let source = r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#;
    let hir = compiler.compile_source(source).unwrap();
    let mir = MirLowerer::lower_module(&hir);
    let mut lir = compiler.compile_source_to_lir(source).unwrap();
    lir.functions[0].frame_layouts[0].slots.clear();

    let error = ControlFlowScheduler::validate_pipeline(&hir, &mir, &lir).unwrap_err();
    assert!(error.to_string().contains("frame layout"));
}

#[test]
fn rejects_lir_runtime_frame_when_slots_drift_from_frame_layout() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 223 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    let future: Future<bool> = ()
    let kept: bool = true
    future.await
    let sink: bool = kept
    return
}
"#,
        )
        .unwrap();
    lir.functions[0].runtime_frames[0].slots.clear();

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("runtime frame"));
}

#[test]
fn rejects_lir_runtime_continuation_when_resume_binding_drifts() {
    let compiler = ValkyrieCompiler::new(SourceID { version_id: 224 });
    let mut lir = compiler
        .compile_source_to_lir(
            r#"micro main() {
    catch raise true {
        default:
            resume true
    }
    return
}
"#,
        )
        .unwrap();
    lir.functions[0].runtime_continuations[0].resume_parameter = MirValueRef(999);

    let error = ControlFlowScheduler::validate_lir_module(&lir).unwrap_err();
    assert!(error.to_string().contains("runtime continuation"));
}

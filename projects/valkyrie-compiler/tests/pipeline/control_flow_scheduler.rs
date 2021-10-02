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

#[path = "control_flow_scheduler/hir_validation.rs"]
mod hir_validation;
#[path = "control_flow_scheduler/pipeline_consistency.rs"]
mod pipeline_consistency;

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

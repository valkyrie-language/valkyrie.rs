use nyar_language::{MirLowerer, ValkyrieCompiler, mir_function_to_executable};
use nyar_language::MirOperation;
use nyar_language::MirTerminator;

fn main() {
    let src = r#"
unite Result<T, E> {
    Fine { value: T }
    Fail { error: E }
}
class VonDiagnostic {
    message: utf8
}
class Plan {
    name: utf8
}
type VonParseResult<T> = Result<T, VonDiagnostic>
micro build_compile_plan(ok: bool) -> VonParseResult<Plan> {
    if ok {
        return Fine(Plan { name: "x" })
    }
    return Fail(VonDiagnostic { message: "e" })
}
"#;
    let hir = ValkyrieCompiler::default().compile_source(src).expect("hir");
    let f = hir.functions.iter().find(|f| f.name.as_str() == "build_compile_plan").expect("fn");
    println!("HIR return_type = {:?}", f.return_type);
    // dump Fine/Fail resolved calls in body
    dump_resolved(&f.body);
    let mir = MirLowerer::lower_module(&hir);
    let mf = mir.functions.iter().find(|f| f.symbol.to_string().contains("build_compile_plan")).expect("mir fn");
    println!("MIR return_type = {:?}", mf.return_type);
    for block in &mf.blocks {
        println!("block {} term={:?}", block.id.0, block.terminator);
        if let MirTerminator::Return { value: Some(op) } = &block.terminator {
            if let nyar_language::MirOperand::Value(v) = op {
                println!("  return value {:?} type={:?}", v, mf.value_types.get(v));
            }
        }
        for insn in &block.instructions {
            if matches!(insn.kind, MirOperation::SumNew { .. } | MirOperation::StructNew { .. } | MirOperation::Call { .. }) {
                    println!("    ty={:?}", mf.value_types.get(&out));
                }
            }
        }
    }
    let exec = mir_function_to_executable(mf);
    println!("EXEC return_type = {:?}", exec.return_type);
    for (k, v) in &exec.value_types {
        if format!("{:?}", k).contains("") {
            // print all that matter - last few
        }
    }
    println!("EXEC value_types count={}", exec.value_types.len());
    for block in &exec.blocks {
        if let nyar_types::executable::Terminator::Return { value: Some(op) } = &block.terminator {
            println!("exec block {} return {:?}", block.id.0, op);
            if let nyar_types::executable::Operand::Value(v) = op {
                println!("  ty={:?}", exec.value_types.get(v));
                println!("  match fn ret? {}", exec.value_types.get(v) == Some(&exec.return_type));
            }
        }
    }
}

fn dump_resolved(block: &nyar_language::types::hir::HirBlock) {
    use nyar_language::types::hir::*;
    for st in &block.statements {
        if let HirStatementKind::Expr(e) = &st.kind {
            walk(e);
        }
    }
    if let Some(e) = &block.expr { walk(e); }
}
fn walk(expr: &nyar_language::types::hir::HirExpr) {
    use nyar_language::types::hir::*;
    match &expr.kind {
        HirExprKind::Call { callee, args, resolved } => {
            println!("CALL resolved={:?} callee_kind={:?}", resolved.as_ref().map(|r| (&r.symbol, &r.domain, &r.return_type, &r.parameter_types)), callee.kind);
            walk(callee);
            for a in args { walk(&a.value); }
        }
        HirExprKind::Return(Some(v)) | HirExprKind::Await(v) => walk(v),
        HirExprKind::If { condition, then_branch, else_branch } => {
            walk(condition); walk(then_branch); if let Some(e) = else_branch { walk(e); }
        }
        HirExprKind::Block(b) => dump_resolved(b),
        HirExprKind::Construct { args, resolved, .. } => {
            println!("CONSTRUCT resolved={:?}", resolved.as_ref().map(|r| (&r.symbol, &r.return_type)));
            for a in args { walk(a); }
        }
        HirExprKind::FieldInit { value, .. } => walk(value),
        HirExprKind::Match { scrutinee, arms } => {
            walk(scrutinee);
            for arm in arms {
                for s in &arm.body.statements {
                    if let HirStatementKind::Expr(e) = &s.kind { walk(e); }
                }
                if let Some(e) = &arm.body.expr { walk(e); }
            }
        }
        _ => {}
    }
}

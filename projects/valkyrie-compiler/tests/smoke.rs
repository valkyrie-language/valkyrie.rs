use valkyrie_compiler::{MirTerminator, ValkyrieCompiler};

#[test]
fn compile_return_zero_lowers_to_single_return_block() {
    let source = r#"namespace test;
micro main() -> i32 {
    return 0;
}"#;
    let compiler = ValkyrieCompiler::default();
    let mir = compiler.compile_source_to_mir(source).expect("parse ok");
    assert_eq!(mir.functions.len(), 1);
    let func = &mir.functions[0];
    assert_eq!(func.symbol, "main");
    assert_eq!(func.blocks.len(), 1);
    let block = &func.blocks[0];
    assert_eq!(block.label, "entry");
    assert!(matches!(block.terminator, MirTerminator::Return { .. }));
}

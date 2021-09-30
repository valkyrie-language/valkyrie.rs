use nyar_clr_backend::{build_clr_method_signature, resolve_clr_import_ref, MsilMethodSignature, MsilType};
use valkyrie_compiler::ValkyrieCompiler;

#[test]
fn derives_entry_signature_from_hir_without_reparsing_signature_text() {
    let compiler = ValkyrieCompiler::default();
    let hir = compiler.compile_source("micro main() -> i64 {\n    return 0;\n}\n").unwrap();

    assert_eq!(build_clr_method_signature(&hir.functions[0], true).unwrap(), MsilMethodSignature::new(MsilType::Int32, Vec::new()));
}

#[test]
fn resolves_clr_import_reference_from_frontend_attribute() {
    let source = r#"
[clr("System.Console", "System.Console", "WriteLine")]
micro helper(message: utf16) -> void {
}
"#;
    let compiler = ValkyrieCompiler::default();
    let hir = compiler.compile_source(source).unwrap();

    let method_ref = resolve_clr_import_ref(&hir.functions[0]).unwrap().unwrap();
    assert_eq!(method_ref.owner.as_deref(), Some("[System.Console]System.Console"));
    assert_eq!(method_ref.name, "WriteLine");
    assert_eq!(method_ref.signature, MsilMethodSignature::new(MsilType::Void, vec![MsilType::String]));
}

#[test]
fn maps_utf8_to_string_when_projecting_clr_signatures() {
    let source = r#"
[clr("System.Console", "System.Console", "WriteLine")]
micro helper(message: utf8) -> void {
}
"#;
    let compiler = ValkyrieCompiler::default();
    let hir = compiler.compile_source(source).unwrap();

    let method_ref = resolve_clr_import_ref(&hir.functions[0]).unwrap().unwrap();
    assert_eq!(method_ref.signature, MsilMethodSignature::new(MsilType::Void, vec![MsilType::String]));
}

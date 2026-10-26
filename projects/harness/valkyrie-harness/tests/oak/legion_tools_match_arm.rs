use oak_core::{Builder, parser::ParseSession, source::SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

fn build_source(source: &str) -> Result<oak_valkyrie::ast::ValkyrieRoot, oak_core::OakError> {
    let language = ValkyrieLanguage::default();
    ValkyrieBuilder::new(&language).build(&SourceText::new(source), &[], &mut ParseSession::default()).result
}

#[test]
fn match_case_with_single_return_arm() {
    build_source(
        "micro demo() {
    match read() {
        case Fine(manifest):
            return
        case Fail(error):
            return
    }
}",
    )
    .expect("single-return match arms");
}

#[test]
fn match_case_with_multi_statement_arm_body() {
    build_source(
        "micro demo() {
    match read() {
        case Fine(manifest):
            let x: utf8 = \"a\"
            return
        case Fail(error):
            return
    }
}",
    )
    .expect("multi-statement match arms");
}

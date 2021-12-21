use oak_core::{Builder, parser::ParseSession, source::SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};
use std::fs;

fn build_source(source: &str) -> Result<oak_valkyrie::ast::ValkyrieRoot, oak_core::OakError> {
    let language = ValkyrieLanguage::default();
    ValkyrieBuilder::new(&language).build(&SourceText::new(source), &[], &mut ParseSession::default()).result
}

#[test]
fn execute_pack_match_case_fine_parses() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../valkyrie.v/projects/legion._/projects/legion.tools/source/_.v"
    );
    let source = fs::read_to_string(path).expect("read legion.tools _.v");
    build_source(&source).expect("parse legion.tools _.v");
}

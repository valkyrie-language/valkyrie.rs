use std::fs;

use oak_core::{Builder, parser::ParseSession, source::SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

#[test]
fn legion_tools_root_source_parses() {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../../projects/valkyrie.v/projects/legion._/projects/legion.tools/source/_.v");
    let source = fs::read_to_string(path).expect("read legion.tools source");
    let language = ValkyrieLanguage::default();
    let result = ValkyrieBuilder::new(&language).build(&SourceText::new(source.as_str()), &[], &mut ParseSession::default()).result;
    if let Err(error) = result {
        panic!("legion.tools _.v parse failed: {error:?}");
    }
}

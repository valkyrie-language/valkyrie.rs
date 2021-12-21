use oak_core::{Builder, parser::ParseSession, source::SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

fn build_source(source: &str) -> Result<oak_valkyrie::ast::ValkyrieRoot, oak_core::OakError> {
    let language = ValkyrieLanguage::default();
    ValkyrieBuilder::new(&language).build(&SourceText::new(source), &[], &mut ParseSession::default()).result
}

#[test]
fn named_tuple_associated_type() {
    build_source(
        "imply EnumerateIterator<T, I>: Iterator {
    type Item = (ordinal: usize, value: T);
}",
    )
    .expect("named tuple associated type");
}

#[test]
fn tuple_return_in_some() {
    build_source(
        "imply EnumerateIterator<T, I>: Iterator {
    type Item = (ordinal: usize, value: T);
    micro next(mut self): Option<(ordinal: usize, value: T)> {
        return Some((ordinal, value))
    }
}",
    )
    .expect("named tuple type and tuple return expression");
}

use data_text_core::{Builder, parser::ParseSession, source::SourceText};
use data_text_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

fn build_source(source: &str) -> Result<data_text_valkyrie::ast::ValkyrieRoot, data_text_core::TextError> {
    let language = ValkyrieLanguage::default();
    ValkyrieBuilder::new(&language).build(&SourceText::new(source), &[], &mut ParseSession::default()).result
}

#[test]
fn if_expression_with_field_access_branches() {
    build_source(
        "structure BuildRequest { target: utf8 }
micro demo(request: BuildRequest) {
    let requested_target: utf8 = if request.target.length() > 0 { request.target } else { \"clr\" }
}",
    )
    .expect("if-expression with field-access branches");
}

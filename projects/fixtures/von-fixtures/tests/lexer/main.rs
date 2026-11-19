use language_testing::lexing::LexerTester;
use data_text_von::{VonLanguage, VonLexer};
use std::{path::Path, time::Duration};

#[test]
fn test_von_lexer() -> Result<(), data_text_core::TextError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = VonLanguage::default();
    let lexer = VonLexer::new(&language);
    let test_runner = LexerTester::new(here.join("tests/lexer")).with_extension("von").with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&lexer)
}

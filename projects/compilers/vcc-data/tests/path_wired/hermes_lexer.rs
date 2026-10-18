//! Migrated from `src/hermes/lexer.rs` (`tests`).

use super::*;

#[test]
fn tokenize_primary_unique_markers() {
    let source = "@@id @email name";
    let tokens = tokenize(source);
    assert_eq!(tokens[0].kind, TokenKind::AtAtIdent);
    assert_eq!(tokens[0].lexeme(source), "id");
    assert_eq!(tokens[1].kind, TokenKind::AtIdent);
    assert_eq!(tokens[1].lexeme(source), "email");
    assert_eq!(tokens[2].kind, TokenKind::Ident);
    assert_eq!(tokens[2].lexeme(source), "name");
}

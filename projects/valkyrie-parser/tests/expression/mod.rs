use super::*;
use valkyrie_ast::{NamespaceDeclarationNode, NumberLiteralNode, PrettyPrint};

#[test]
fn lex_expression() {
    repl_debug(include_str!("infix.vk"), "expression/infix_debug.rkt").expect("infix");
    repl_debug(include_str!("unary.vk"), "expression/unary_debug.rkt").expect("unary");
    repl_debug(include_str!("table.vk"), "expression/table_debug.rkt").expect("table");
    repl_debug(include_str!("apply.vk"), "expression/apply_debug.rkt").expect("apply");
    repl_debug(include_str!("slice.vk"), "expression/slice_debug.rkt").expect("slice");
    repl_debug(include_str!("generic.vk"), "expression/generic_debug.rkt").expect("generic");
}

#[test]
fn test_apply2() {
    repl_debug(include_str!("new.vk"), "expression/new_debug.rkt").expect("new");
}

#[test]
fn test_apply() {
    let raw = "namespace! std.io.print";
    let apply = NamespaceDeclarationNode::parse_text(raw).unwrap();
    apply.pretty_print(42)
}

#[test]
fn main2() {
    let raw = "⁅:, ::, : :, 1, :index0:, ::-1, i::j, i: :j⁆";
    let slice = SubscriptNode::parse_text(raw).unwrap();
    slice.pretty_print(42)
}

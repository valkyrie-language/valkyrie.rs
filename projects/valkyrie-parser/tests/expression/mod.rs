use valkyrie_error::third_party::Url;
use std::path::PathBuf;
use valkyrie_parser::{NewStatementNode, ProgramContext, RangeLiteralNode};

use super::*;

fn take_expression(input: StatementNode) -> Option<MainExpressionNode> {
    todo!()
}

fn debug_expression(input: &str) -> std::io::Result<()> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").canonicalize()?;
    let cst = ValkyrieParser::parse_cst(input, ValkyrieRule::GENERIC_HIDE).unwrap();
    println!("Short Form:\n{}", cst);
    // let ast = NewStatementNode::from_str(input).unwrap();
    // println!("Long Form:\n{:#?}", ast);
    Ok(())
}

#[test]
fn debug() {
    let raw = "<A>B";
    debug_expression(raw).unwrap();
}

// #[test]
// fn main2() {
//     let raw = "⁅:, ::, : :, 1, :index0:, ::-1, i::j, i: :j⁆";
//     let slice = SubscriptCallNode::parse_text(raw).unwrap();
//     pretty_print(&slice)
// }

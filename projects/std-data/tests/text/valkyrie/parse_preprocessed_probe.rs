use std_data::text::valkyrie::{AstParser, RootStatement};

#[test]
fn namespace_body_none() {
    let source = r#"
namespace core::text;
structure TextSpan { offset: usize, length: usize }

namespace std.data.text.msil;
structure TextSpan { start: usize, stop: usize }
"#;
    let root = AstParser::parse_root(source).expect("parse");
    for (i, st) in root.statements.iter().enumerate() {
        match st {
            RootStatement::Namespace(ns) => {
                println!("{i}: Namespace parts={:?} body_is_none={}", ns.name.parts, ns.body.is_none());
            }
            RootStatement::Class(c) => {
                println!("{i}: Class {}", c.name.name);
            }
            other => println!("{i}: other {:?}", std::mem::discriminant(other)),
        }
    }
    // Also run validation path via nyar-language if available
}

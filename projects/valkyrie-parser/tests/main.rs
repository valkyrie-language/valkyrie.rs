use lispify::{
    helpers::{colored_lisp, display_lisp},
    Lispify,
};
use std::{
    fs::File,
    io::Write,
    path::{Path, PathBuf},
};
use valkyrie_parser::repl::parse_repl;
// use std::io::stdout;
//
// use valkyrie_errors::{FileID, TextManager, ValkyrieResult};
// use valkyrie_parser::ValkyrieParser;
//
// mod declaration;
mod expression;
mod literal;
// mod statement;
//
// #[test]
// fn ready() {
//     println!("it works!")
// }
//
// pub fn debug_lexer(files: &[&str]) -> ValkyrieResult {
//     let mut parser = ValkyrieParser::default();
//     let mut problems = vec![];
//     let mut text = TextManager::new(env!("CARGO_MANIFEST_DIR"));
//     for file in files {
//         // println!("Parsing file: {}", Url::from_file_path(text.resolve_file(file))?);
//         let basic = text.add_file(file)?;
//         if let Err(e) = parser.parse_file(basic, &text.get_text(basic)) {
//             problems.push(e.as_report())
//         }
//     }
//     for e in parser.take_errors() {
//         problems.push(e.as_report())
//     }
//     for problem in problems {
//         problem.write(&mut text, stdout().lock())?;
//     }
//     Ok(())
// }

fn here() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests").canonicalize().expect("failed to get manifest dir")
}

fn lisp_debug(text: &str, output: &str) -> std::io::Result<()> {
    let mut file = File::create(here().join(output))?;
    let apply = parse_repl(text);
    for expr in &apply {
        writeln!(file, "{}", display_lisp(expr.lispify(), 144).unwrap())?;
        println!("{}", colored_lisp(expr.lispify(), 144).unwrap());
    }
    Ok(())
}

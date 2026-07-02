fn main() {
    use nyar_language::valkyrie::AstParser;
    let path = std::env::args().nth(1).expect("path");
    let src = std::fs::read_to_string(&path).expect("read");
    match AstParser::parse_root(&src) {
        Ok(root) => eprintln!("PARSE_OK statements={}", root.statements.len()),
        Err(e) => {
            eprintln!("PARSE_ERR");
            eprintln!("{e}");
        }
    }
}

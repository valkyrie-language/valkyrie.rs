//! Parse every nyar.emitter `.v` via sibling checkout layout:
//!   <workspace>/valkyrie.rs/projects/vcc-data
//!   <workspace>/valkyrie.v/projects/nyar._/projects/nyar.emitter/source
//! Never hardcode machine-local absolute paths.
use std::{fs, path::PathBuf};
use vcc_data::text::valkyrie::{AstParser, parser::ParseError};

#[test]
fn parse_nyar_emitter_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..").join("valkyrie.v/projects/nyar._/projects/nyar.emitter/source");
    let root = root.canonicalize().unwrap_or(root);
    assert!(root.is_dir(), "missing sibling emitter sources at {} (expect workspace layout valkyrie.rs + valkyrie.v)", root.display());

    let mut files = Vec::new();
    fn walk(dir: &std::path::Path, out: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            }
            else if path.extension().and_then(|s| s.to_str()) == Some("v") {
                out.push(path);
            }
        }
    }
    walk(&root, &mut files);
    files.sort();

    let mut failures = 0usize;
    for path in &files {
        let source = fs::read_to_string(path).unwrap();
        let display = path.strip_prefix(&root).unwrap_or(path);
        match AstParser::parse_root(&source) {
            Ok(_) => println!("OK {}", display.display()),
            Err(err) => {
                failures += 1;
                println!("ERR {}", display.display());
                println!("  {err}");
                if let ParseError::Invalid { span: Some(span), .. } = &err {
                    let start = span.start.min(source.len());
                    let end = span.end.min(source.len()).max(start);
                    let before = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
                    let after = source[end..].find('\n').map(|i| end + i).unwrap_or(source.len());
                    let line_no = source[..start].bytes().filter(|b| *b == b'\n').count() + 1;
                    println!("  line {line_no}: {}", &source[before..after]);
                    println!("  mark: {}{}", " ".repeat(start.saturating_sub(before)), "^".repeat((end - start).max(1)));
                }
            }
        }
    }
    assert_eq!(failures, 0, "{failures} emitter source(s) failed to parse");
}

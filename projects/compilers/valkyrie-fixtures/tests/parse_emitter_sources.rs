//! Parse every nyar.emitter `.v` via `projects/valkyrie.v` submodule:
//!   projects/valkyrie.v/projects/nyar._/projects/nyar.emitter/source
use std::{fs, path::PathBuf};

use oak_valkyrie::printer::parse_source;

#[test]
fn parse_nyar_emitter_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../valkyrie.v/projects/nyar._/projects/nyar.emitter/source");
    let root = root.canonicalize().unwrap_or(root);
    assert!(root.is_dir(), "missing valkyrie.v emitter sources at {} (init `projects/valkyrie.v` submodule)", root.display());

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
        match parse_source(&source) {
            Ok(_) => println!("OK {}", display.display()),
            Err(err) => {
                failures += 1;
                println!("ERR {}", display.display());
                println!("  {err}");
            }
        }
    }
    assert_eq!(failures, 0, "{failures} emitter source(s) failed to parse");
}

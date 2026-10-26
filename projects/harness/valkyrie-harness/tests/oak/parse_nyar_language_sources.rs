//! Parse every nyar.language `.v` via `projects/examples/valkyrie.v` submodule:
//!   projects/examples/valkyrie.v/projects/nyar._/projects/nyar.language/source
use std::{fs, path::PathBuf};

use oak_valkyrie::printer::parse_source;

#[test]
fn parse_nyar_language_sources() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/valkyrie.v/projects/nyar._/projects/nyar.language/source");
    let root = root.canonicalize().unwrap_or(root);
    assert!(root.is_dir(), "missing valkyrie.v nyar.language sources at {} (init `projects/examples/valkyrie.v` submodule)", root.display());

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

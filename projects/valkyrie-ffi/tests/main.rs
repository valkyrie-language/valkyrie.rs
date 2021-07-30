use std::path::Path;
use valkyrie_wit::ValkyrieFFI;
use wit_parser::Resolve;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn load_all() {
    let dir = r#"C:\Users\Dell\Downloads\wasi-cloud-core-main\wit"#;
    let mut resolved = ValkyrieFFI::new(dir).unwrap();
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    resolved.generate(here).unwrap();
}

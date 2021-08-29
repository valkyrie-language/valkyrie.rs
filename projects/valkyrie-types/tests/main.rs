use std::{fs::File, io::Write, path::Path};
use valkyrie_types::ResolveState;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test_hello_world() -> nyar_error::Result<()> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let mut context = ResolveState::new("std");
    context.resolve_package(r#"C:\Users\Dell\CLionProjects\valkyrie-std\packages\valkyrie-standard\source"#)?;
    context.resolve_file(here.join("main.vk"))?;
    context.show_errors();
    let mut wat = File::create(here.join("component.wat"))?;
    let source = context.resolve()?;
    let wast = source.encode();
    wat.write_all(wast.as_bytes())?;
    Ok(())
}

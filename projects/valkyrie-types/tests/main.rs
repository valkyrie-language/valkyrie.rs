use nyar_error::FileCache;
use std::{path::Path, process::Command};
use valkyrie_types::ModuleResolver;

#[test]
fn ready() {
    println!("it works!")
}

#[test]
fn test() {
    let file = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/test.vk").canonicalize().unwrap();
    let output = Path::new(env!("CARGO_MANIFEST_DIR")).join("target/debug/valkyrie/test.wasm");
    let mut cache = FileCache::default();
    let file = cache.load_local(file).unwrap();
    let mut reolver = ModuleResolver::default();
    for error in reolver.parse(file, &mut cache) {
        error.as_report().eprint(&cache).unwrap()
    }
    let wasm = reolver.build_wasm();
    let _ = wasm.build_module(output).unwrap();
    let o = Command::new("valor").arg("build").output().unwrap();
    println!("{}", String::from_utf8_lossy(&o.stdout))
}

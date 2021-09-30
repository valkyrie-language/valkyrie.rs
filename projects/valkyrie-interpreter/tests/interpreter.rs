use valkyrie_interpreter::Interpreter;

#[test]
fn builtin_runtimes_match_legion_run_hosts() {
    let runtimes = Interpreter::builtin_runtimes();

    assert_eq!(runtimes.len(), 5);
    assert!(runtimes.iter().any(|runtime| {
        runtime.family == "clr" && runtime.abi == "managed" && runtime.launcher == "dotnet"
    }));
    assert!(runtimes.iter().any(|runtime| {
        runtime.family == "jvm" && runtime.abi == "managed" && runtime.launcher == "java"
    }));
    assert!(runtimes.iter().any(|runtime| {
        runtime.family == "wasi" && runtime.abi == "wasip1" && runtime.launcher == "wasmtime"
    }));
    assert!(runtimes.iter().any(|runtime| {
        runtime.family == "wasm" && runtime.abi == "node" && runtime.launcher == "node"
    }));
    assert!(runtimes.iter().any(|runtime| {
        runtime.family == "windows" && runtime.abi == "exe" && runtime.launcher == "exe"
    }));
}

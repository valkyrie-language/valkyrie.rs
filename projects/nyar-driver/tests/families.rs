use jvm_backend::{JvmClassFile, JvmCodeBody, JvmInstruction, JvmMethodDescriptor, JvmMethodSignature, JvmTypeDescriptor};
use nyar::{backends::CompilationOptions, BinaryArch, BinaryFlavor, BinaryTarget, HostProjectionBoundary, RunnerFamily, TargetFamily};
use nyar_driver::{compile_with_bundled_backends, DriverBackendInput, DriverCompileRequest};
use tempfile::tempdir;
use wasi_backend::WasmBinaryModule;

fn compilation_options(target: BinaryTarget, artifact_name: &str) -> CompilationOptions {
    CompilationOptions { target, artifact_name: artifact_name.to_string(), emit_debug_symbols: false, optimize: false }
}

fn demo_jvm_input(output_dir: &std::path::Path) -> DriverBackendInput {
    let mut class_file = JvmClassFile::new("demo/Main");
    class_file.methods.push(JvmMethodSignature {
        name: "main".to_string(),
        descriptor: JvmMethodDescriptor::new(Vec::new(), JvmTypeDescriptor::Int),
        access_flags: 0x0001 | 0x0008,
        code: Some(JvmCodeBody { max_stack: 1, max_locals: 0, instructions: vec![JvmInstruction::IConst(1), JvmInstruction::IReturn] }),
    });
    DriverBackendInput::Jvm(jvm_backend::JvmBinaryBackendInput { class_file, output_dir: output_dir.to_path_buf(), emit_class_file: true })
}

fn demo_wasm_input(output_dir: &std::path::Path, host_boundary: HostProjectionBoundary) -> DriverBackendInput {
    let mut module = WasmBinaryModule::new();
    module.push_custom_section("demo.module", b"demo".to_vec());
    DriverBackendInput::Wasm(wasi_backend::WasmBinaryBackendInput {
        module,
        output_dir: output_dir.to_path_buf(),
        host_boundary,
        imports: Vec::new(),
    })
}

#[test]
fn creates_jvm_run_contract_via_bundled_compiler() {
    let output_dir = tempdir().expect("temp dir");
    let options = compilation_options(BinaryTarget::new(TargetFamily::Jvm, BinaryArch::Any, BinaryFlavor::ManagedClr), "demo");
    let input = demo_jvm_input(output_dir.path());
    let report = compile_with_bundled_backends(DriverCompileRequest {
        artifact_name: "demo",
        requirement: input.requirement(options.target.clone()),
        input,
        runner_family: RunnerFamily::Jvm,
        generate_runtime_config: false,
        options: &options,
    })
    .expect("compile ok");

    let contract = report.run_contract.expect("missing run contract");
    assert_eq!(contract.physical_entry, "demo.jar");
    assert_eq!(contract.invocation, "java");
    assert_eq!(contract.validate, "java -jar demo.jar");
    assert!(contract.logical_entry.ends_with(".Main") || contract.logical_entry == "Main");
}

#[test]
fn creates_node_and_wasi_run_contracts_via_bundled_compiler() {
    let output_dir = tempdir().expect("temp dir");
    let node_options = compilation_options(BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native), "demo_node");
    let node_input = demo_wasm_input(output_dir.path(), HostProjectionBoundary::WasmJsGlue);
    let node_report = compile_with_bundled_backends(DriverCompileRequest {
        artifact_name: "demo_node",
        requirement: node_input.requirement(node_options.target.clone()),
        input: node_input,
        runner_family: RunnerFamily::Node,
        generate_runtime_config: false,
        options: &node_options,
    })
    .expect("node compile ok");
    let node_contract = node_report.run_contract.expect("missing node contract");
    assert_eq!(node_contract.logical_entry, "main");
    assert_eq!(node_contract.physical_entry, "demo_node.mjs");
    assert_eq!(node_contract.invocation, "node");

    let wasi_options = compilation_options(BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native), "demo_wasi");
    let wasi_input = demo_wasm_input(output_dir.path(), HostProjectionBoundary::WasiComponent);
    let wasi_report = compile_with_bundled_backends(DriverCompileRequest {
        artifact_name: "demo_wasi",
        requirement: wasi_input.requirement(wasi_options.target.clone()),
        input: wasi_input,
        runner_family: RunnerFamily::Wasi,
        generate_runtime_config: false,
        options: &wasi_options,
    })
    .expect("wasi compile ok");
    let wasi_contract = wasi_report.run_contract.expect("missing wasi contract");
    assert_eq!(wasi_contract.logical_entry, "_start");
    assert_eq!(wasi_contract.physical_entry, "demo_wasi.wasm");
    assert_eq!(wasi_contract.validate, "wasmtime demo_wasi.wasm");
}

#[test]
fn rejects_unsupported_backend_requirement() {
    let output_dir = tempdir().expect("temp dir");
    let options = compilation_options(BinaryTarget::new(TargetFamily::NyarVm, BinaryArch::Any, BinaryFlavor::Native), "demo");
    let input = demo_jvm_input(output_dir.path());
    let error = compile_with_bundled_backends(DriverCompileRequest {
        artifact_name: "demo",
        requirement: input.requirement(options.target.clone()),
        input,
        runner_family: RunnerFamily::Node,
        generate_runtime_config: false,
        options: &options,
    })
    .expect_err("should reject unsupported backend requirement");
    assert!(error.to_string().contains("尚未接入"));
}

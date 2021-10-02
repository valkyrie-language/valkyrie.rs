use std::time::{SystemTime, UNIX_EPOCH};

use nyar::{
    backends::{CompilationOptions, TargetCodeGenBackend},
    BinaryArch, BinaryFlavor, BinaryTarget, HostProjectionBoundary, TargetFamily,
};
use nyar_backend_wasi::{WasmBinaryBackend, WasmBinaryBackendInput, WasmBinaryModule, WasmSection};

#[test]
fn round_trips_custom_and_standard_sections() {
    let mut module = WasmBinaryModule::new();
    module.push_custom_section("name", b"demo".to_vec());
    module.sections.push(WasmSection { id: 1, name: None, bytes: vec![0x01, 0x60, 0x00, 0x00] });

    let bytes = module.to_bytes().unwrap();
    let decoded = WasmBinaryModule::from_bytes(&bytes).unwrap();

    assert_eq!(decoded.version, 1);
    assert_eq!(decoded.sections.len(), 2);
    assert_eq!(decoded.custom_sections()[0].name, "name");
    assert_eq!(decoded.sections[1].id, 1);
}

#[test]
fn wasm_js_glue_builder_emits_launcher() {
    let output_dir = unique_output_dir("js-glue");
    let backend = WasmBinaryBackend::new();
    let options = demo_options();
    let input = WasmBinaryBackendInput {
        module: WasmBinaryModule::new(),
        output_dir: output_dir.clone(),
        host_boundary: HostProjectionBoundary::WasmJsGlue,
        imports: vec![("env".to_string(), "get_input".to_string())],
    };

    let artifacts = backend.compile(input, &options).unwrap();

    assert!(output_dir.join("demo.mjs").exists());
    assert_eq!(artifacts.artifacts.len(), 2);

    let _ = std::fs::remove_dir_all(output_dir);
}

#[test]
fn wasi_component_builder_keeps_binding_generation_separate() {
    let output_dir = unique_output_dir("component");
    let backend = WasmBinaryBackend::new();
    let options = demo_options();
    let input = WasmBinaryBackendInput {
        module: WasmBinaryModule::new(),
        output_dir: output_dir.clone(),
        host_boundary: HostProjectionBoundary::WasiComponent,
        imports: vec![("env".to_string(), "get_input".to_string())],
    };

    let artifacts = backend.compile(input, &options).unwrap();

    assert!(output_dir.join("demo.wasm").exists());
    assert!(!output_dir.join("demo.mjs").exists());
    let wit_path = output_dir.join("demo.component.wit");
    assert!(wit_path.exists());
    let wit_text = std::fs::read_to_string(&wit_path).unwrap();
    assert!(wit_text.contains("package nyar:demo@0.1.0;"));
    assert!(wit_text.contains("interface env {"));
    assert!(wit_text.contains("get_input: func();"));
    assert_eq!(artifacts.artifacts.len(), 2);

    let _ = std::fs::remove_dir_all(output_dir);
}

fn demo_options() -> CompilationOptions {
    CompilationOptions {
        target: BinaryTarget::new(TargetFamily::Wasm, BinaryArch::Any, BinaryFlavor::Native),
        artifact_name: "demo".to_string(),
        emit_debug_symbols: false,
        optimize: false,
    }
}

fn unique_output_dir(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("nyar-backend-wasi-{label}-{nonce}"))
}

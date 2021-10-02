use std::fs;

use nyar::{
    backends::{CompilationOptions, TargetCodeGenBackend},
    BinaryArch, BinaryFlavor, BinaryTarget, TargetFamily,
};
use nyar_backend_jvm::{
    internal_name_to_binary_name, JvmBinaryBackend, JvmBinaryBackendInput, JvmClassFile, JvmCodeBody, JvmInstruction, JvmMethodDescriptor,
    JvmMethodSignature, JvmTypeDescriptor,
};
use tempfile::tempdir;

const ACC_PUBLIC: u16 = 0x0001;
const ACC_STATIC: u16 = 0x0008;

fn jvm_options(artifact_name: &str) -> CompilationOptions {
    CompilationOptions {
        target: BinaryTarget::new(TargetFamily::Jvm, BinaryArch::Any, BinaryFlavor::ManagedClr),
        artifact_name: artifact_name.to_string(),
        emit_debug_symbols: false,
        optimize: false,
    }
}

#[test]
fn converts_internal_name_to_binary_name() {
    assert_eq!(internal_name_to_binary_name("demo/Main"), "demo.Main");
    assert_eq!(internal_name_to_binary_name("demo/Main.class"), "demo.Main");
}

#[test]
fn creates_java_launcher_for_zero_arg_main() {
    let mut class_file = JvmClassFile::new("demo/Main");
    class_file.methods.push(JvmMethodSignature {
        name: "main".to_string(),
        descriptor: JvmMethodDescriptor::new(Vec::new(), JvmTypeDescriptor::Int),
        access_flags: ACC_PUBLIC | ACC_STATIC,
        code: Some(JvmCodeBody { max_stack: 1, max_locals: 0, instructions: vec![JvmInstruction::IConst(1), JvmInstruction::IReturn] }),
    });

    let output_dir = tempdir().expect("temp dir");
    let backend = JvmBinaryBackend::new();
    backend
        .compile(JvmBinaryBackendInput { class_file, output_dir: output_dir.path().to_path_buf(), emit_class_file: true }, &jvm_options("demo"))
        .expect("compile ok");

    let class_bytes = fs::read(output_dir.path().join("demo.class")).expect("class bytes");
    let decoded = JvmClassFile::from_bytes(&class_bytes).expect("decode ok");
    let launcher_descriptor = JvmMethodDescriptor::new(
        vec![JvmTypeDescriptor::array(JvmTypeDescriptor::Object("java/lang/String".to_string()))],
        JvmTypeDescriptor::Void,
    );
    let launcher =
        decoded.methods.iter().find(|method| method.name == "main" && method.descriptor == launcher_descriptor).expect("缺少 Java 启动入口");
    assert_eq!(launcher.access_flags, ACC_PUBLIC | ACC_STATIC);
    assert_eq!(decoded.methods.len(), 2, "应同时保留原零参入口和生成的 Java 启动入口");
}

#[test]
fn keeps_existing_java_launcher() {
    let launcher_descriptor = JvmMethodDescriptor::new(
        vec![JvmTypeDescriptor::array(JvmTypeDescriptor::Object("java/lang/String".to_string()))],
        JvmTypeDescriptor::Void,
    );
    let mut class_file = JvmClassFile::new("demo/Main");
    class_file.methods.push(JvmMethodSignature {
        name: "main".to_string(),
        descriptor: launcher_descriptor,
        access_flags: ACC_PUBLIC | ACC_STATIC,
        code: Some(JvmCodeBody { max_stack: 0, max_locals: 1, instructions: vec![JvmInstruction::Return] }),
    });

    let output_dir = tempdir().expect("temp dir");
    let backend = JvmBinaryBackend::new();
    backend
        .compile(JvmBinaryBackendInput { class_file, output_dir: output_dir.path().to_path_buf(), emit_class_file: true }, &jvm_options("demo"))
        .expect("compile ok");

    let class_bytes = fs::read(output_dir.path().join("demo.class")).expect("class bytes");
    let decoded = JvmClassFile::from_bytes(&class_bytes).expect("decode ok");
    assert_eq!(decoded.methods.len(), 1);
}

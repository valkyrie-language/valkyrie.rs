#![cfg(windows)]

use std::{fs, process::Command};

use nyar::backends::clr::ClrImageKind;
use nyar_clr_backend::{
    lower_lir_to_msil, write_dotnet_runtime_config, MsilAssembly, MsilInstruction, MsilMethodBody, MsilMethodRef, MsilMethodSignature,
    MsilModule, MsilOpcode, MsilType, PeWriter, PeWriterOptions,
};
use tempfile::TempDir;
use valkyrie_compiler::ValkyrieCompiler;

#[test]
#[ignore = "依赖外部 `dotnet` 宿主进程，集成环境中可能被子进程沙箱拦截"]
fn generated_exe_is_accepted_by_dotnet_host() {
    let output_dir = create_output_dir();
    let artifact_path = output_dir.path().join("sample.exe");
    let runtime_config_path = output_dir.path().join("sample.runtimeconfig.json");

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "sample".to_string(),
        module_name: "sample.exe".to_string(),
        image_kind: ClrImageKind::Executable,
    })
    .write_module(&build_executable_module())
    .unwrap();

    fs::write(&artifact_path, bytes).unwrap();
    write_dotnet_runtime_config(output_dir.path(), "sample").unwrap();
    assert!(runtime_config_path.exists());

    let output = match Command::new("dotnet").arg("exec").arg(&artifact_path).current_dir(output_dir.path()).output() {
        Ok(output) => output,
        Err(error) if should_skip_dotnet_host_check(&error) => {
            eprintln!("skip runtime acceptance: unable to launch `dotnet` host in current environment: {error}");
            return;
        }
        Err(error) => {
            panic!("failed to launch `dotnet` host: {error}");
        }
    };

    assert!(
        output.status.success(),
        "dotnet exec failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

/// 验证包含结构体定义的 PE 二进制能被 CLR 运行时接受并正确执行。
///
/// 源码定义 `Point { x: i32, y: i32 }`，在 `main` 中构造实例并读取字段 `x`。
/// 期望退出码为 1（`p.x` 的值），证明结构体构造、字段存储和字段读取全链路正确。
#[test]
#[ignore = "依赖外部 `dotnet` 宿主进程，集成环境中可能被子进程沙箱拦截"]
fn generated_struct_exe_runs_on_dotnet_host() {
    let output_dir = create_output_dir();
    let artifact_path = output_dir.path().join("struct_sample.exe");
    let runtime_config_path = output_dir.path().join("struct_sample.runtimeconfig.json");

    let source =
        "structure Point {\n    x: i32\n    y: i32\n}\n\nmicro main() -> i32 {\n    let p = Point { x: 1, y: 2 };\n    return p.x;\n}\n";
    let compiler = ValkyrieCompiler::default();
    let lir = compiler.compile_source_to_lir(source).unwrap();
    let msil = lower_lir_to_msil(&lir);

    let bytes = PeWriter::new(PeWriterOptions {
        assembly_name: "struct_sample".to_string(),
        module_name: "struct_sample.exe".to_string(),
        image_kind: ClrImageKind::Executable,
    })
    .write_module(&msil)
    .unwrap();

    fs::write(&artifact_path, bytes).unwrap();
    write_dotnet_runtime_config(output_dir.path(), "struct_sample").unwrap();
    assert!(runtime_config_path.exists());

    let output = match Command::new("dotnet").arg("exec").arg(&artifact_path).current_dir(output_dir.path()).output() {
        Ok(output) => output,
        Err(error) if should_skip_dotnet_host_check(&error) => {
            eprintln!("skip runtime acceptance: unable to launch `dotnet` host in current environment: {error}");
            return;
        }
        Err(error) => {
            panic!("failed to launch `dotnet` host: {error}");
        }
    };

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let exit_code = output.status.code().unwrap_or(-1);

    // main 返回 p.x = 1，期望退出码为 1。
    assert_eq!(exit_code, 1, "期望退出码为 1（p.x 的值），实际为 {exit_code}\nstdout:\n{stdout}\nstderr:\n{stderr}");
}

fn build_executable_module() -> MsilModule {
    MsilModule {
        assembly: MsilAssembly { name: "sample".to_string(), externs: vec!["mscorlib".to_string()] },
        types: vec![],
        global_methods: vec![MsilMethodBody {
            method: MsilMethodRef { owner: None, name: "Main".to_string(), signature: MsilMethodSignature::new(MsilType::Int32, Vec::new()) },
            locals: vec![],
            instructions: vec![
                MsilInstruction { label: None, opcode: MsilOpcode::LdcI4_0, operand: None },
                MsilInstruction { label: None, opcode: MsilOpcode::Ret, operand: None },
            ],
            max_stack: 1,
            is_entry_point: true,
        }],
    }
}

fn create_output_dir() -> TempDir {
    tempfile::Builder::new().prefix("runtime-acceptance").tempdir().unwrap()
}

fn should_skip_dotnet_host_check(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(0)
}

mod support;

use legion::{
    CanonicalTarget, LegionWorkspace,
    cache::{CompilationCache, cache_root_for, compute_artifact_hash, load_cached_build, store_cached_build},
    cmds::{
        build::{BuildArgs, run},
        run::{ExecutionManifest, RunContract},
    },
};
use std::{
    fs,
    process::{Command, ExitCode},
};
use support::{
    create_local_package_project, create_smoke_project, create_smoke_project_with_build, create_smoke_project_with_manifest,
    create_smoke_project_with_source,
};

#[test]
fn builds_minimal_clr_project() {
    let fixture = create_smoke_project("legion-build");
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());
    assert!(output_dir.join("main.runtimeconfig.json").exists());
}

#[test]
fn second_build_hits_artifact_set_cache() {
    let fixture = create_smoke_project("legion-build-cache-hit");
    let output_dir = fixture.project_dir.join("dist").join("cache-clr");
    let args = BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    };
    assert_eq!(run(&args).unwrap(), ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());

    let workspace = LegionWorkspace::discover_for_project(&fixture.project_dir).unwrap();
    let (plan, _) = workspace
        .build_plan_with_local_fallback(&legion::BuildRequest {
            project_dir: fixture.project_dir.clone(),
            target: CanonicalTarget::clr(),
            output_dir: Some(output_dir.clone()),
        })
        .unwrap();
    let triple = plan.project.build_target.target.as_canonical_str();
    let ir_hash = compute_artifact_hash(
        &plan.project.source_files,
        &triple,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        plan.project.build_target.runtime_async,
    )
    .unwrap();
    let cache = CompilationCache::open(cache_root_for(&plan.workspace_root));
    let mut bundle = load_cached_build(&cache, &plan.project.name, &triple, &ir_hash).expect("artifact-set stored");
    for (name, bytes) in &mut bundle.files {
        if name == "main.exe" {
            *bytes = b"FROM_CACHE".to_vec();
        }
    }
    store_cached_build(&cache, &plan.project.name, &triple, &ir_hash, &bundle).unwrap();

    fs::remove_dir_all(&output_dir).unwrap();
    assert_eq!(run(&args).unwrap(), ExitCode::SUCCESS);
    assert_eq!(fs::read(output_dir.join("main.exe")).unwrap(), b"FROM_CACHE");
}

#[test]
fn compile_plan_caches_execution_manifest_in_artifact_bundle() {
    let fixture = create_smoke_project("legion-build-cache-manifest");
    let output_dir = fixture.project_dir.join("dist").join("cache-manifest");
    let args = BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    };
    assert_eq!(run(&args).unwrap(), ExitCode::SUCCESS);
    assert!(output_dir.join("run-contracts.txt").exists());

    let workspace = LegionWorkspace::discover_for_project(&fixture.project_dir).unwrap();
    let (plan, _) = workspace
        .build_plan_with_local_fallback(&legion::BuildRequest {
            project_dir: fixture.project_dir.clone(),
            target: CanonicalTarget::clr(),
            output_dir: Some(output_dir.clone()),
        })
        .unwrap();
    let triple = plan.project.build_target.target.as_canonical_str();
    let ir_hash = compute_artifact_hash(
        &plan.project.source_files,
        &triple,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        plan.project.build_target.runtime_async,
    )
    .unwrap();
    let cache = CompilationCache::open(cache_root_for(&plan.workspace_root));
    let bundle = load_cached_build(&cache, &plan.project.name, &triple, &ir_hash).expect("artifact-set stored");
    assert!(bundle.files.iter().any(|(name, _)| name == "run-contracts.txt"), "execution manifest should be part of artifact-set bundle");

    fs::remove_dir_all(&output_dir).unwrap();
    assert_eq!(run(&args).unwrap(), ExitCode::SUCCESS);
    let manifest = ExecutionManifest::read_from_output_dir(&output_dir).unwrap().expect("restored manifest");
    assert!(!manifest.run_contracts.is_empty());
}

#[test]
fn second_build_hits_semantics_when_artifact_set_poisoned() {
    let fixture = create_smoke_project("legion-build-semantics-waterfall");
    let output_dir = fixture.project_dir.join("dist").join("semantics-clr");
    let args = BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    };
    assert_eq!(run(&args).unwrap(), ExitCode::SUCCESS);

    let workspace = LegionWorkspace::discover_for_project(&fixture.project_dir).unwrap();
    let (plan, _) = workspace
        .build_plan_with_local_fallback(&legion::BuildRequest {
            project_dir: fixture.project_dir.clone(),
            target: CanonicalTarget::clr(),
            output_dir: Some(output_dir.clone()),
        })
        .unwrap();
    let triple = plan.project.build_target.target.as_canonical_str();
    let ir_hash = compute_artifact_hash(
        &plan.project.source_files,
        &triple,
        plan.project.build_target.msil,
        plan.project.build_target.wat,
        plan.project.build_target.runtime_async,
    )
    .unwrap();
    let cache = CompilationCache::open(cache_root_for(&plan.workspace_root));
    // Force artifact miss while keeping frontend stage caches.
    cache
        .put_ir(
            &plan.project.name,
            &triple,
            &ir_hash,
            &legion::cache::IrCacheEntry {
                ir_kind: "poison".into(),
                ir_data: vec![],
                ir_hash: ir_hash.clone(),
                canonical_triple: triple.clone(),
            },
        )
        .unwrap();

    fs::remove_dir_all(&output_dir).unwrap();
    assert_eq!(run(&args).unwrap(), ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());
}

#[test]
fn builds_migrated_test_clr_smoke_project() {
    let fixture = create_smoke_project_with_manifest(
        "legion-build-migrated-clr-smoke",
        r#"{
    name: "test_clr_smoke",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        }
    ]
}
"#,
        r#"[main]
micro main(): i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr-smoke");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());
    assert!(output_dir.join("main.msil").exists());
    assert!(output_dir.join("run-contracts.txt").exists());
    assert!(!output_dir.join("host-selection.txt").exists());
}

#[test]
fn builds_clr_legion_tools_isomorphic_namespace_fixture() {
    let fixture = create_smoke_project_with_manifest(
        "legion-build-bootstrap-smoke",
        r#"{
    name: "legion.tools.smoke",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        }
    ]
}
"#,
        r#"namespace legion.tools.smoke;

[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

micro version_text() -> utf8 {
    return "Legion smoke v0.1.0"
}

[main]
micro main(): i64 {
    console_write_line("legion.tools smoke")
    return 0
}
"#,
    );
    fs::write(
        fixture.project_dir.join("source").join("helpers.v"),
        r#"namespace legion.tools.smoke;

micro helper_label() -> utf8 {
    return "bootstrap-smoke"
}
"#,
    )
    .unwrap();

    let output_dir = fixture.project_dir.join("dist").join("bootstrap-smoke-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());
    assert!(output_dir.join("main.msil").exists());
    assert!(output_dir.join("run-contracts.txt").exists());
}

#[test]
fn writes_execution_manifest_with_hashes_by_default() {
    let fixture = create_smoke_project("legion-build-execution-manifest");
    let output_dir = fixture.project_dir.join("dist").join("execution-manifest");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let manifest = ExecutionManifest::read_from_output_dir(&output_dir).unwrap().expect("execution manifest");
    assert_eq!(manifest.schema_version, 1);
    assert_eq!(manifest.project_name, "app");
    assert_eq!(manifest.target, CanonicalTarget::clr().to_string());
    assert!(!manifest.inputs.is_empty());
    assert!(manifest.inputs.iter().all(|input| !input.hash.is_empty()));
    assert!(!manifest.artifacts.is_empty());
    assert!(manifest.artifacts.iter().all(|artifact| !artifact.hash.is_empty()));
    assert_eq!(manifest.run_contracts.first().map(|contract| contract.physical_entry.as_str()), Some("main.exe"));
    assert!(!output_dir.join("host-selection.txt").exists());
}

#[test]
fn writes_host_selection_only_in_debug_artifacts_mode() {
    let fixture = create_smoke_project("legion-build-host-selection-debug");
    let output_dir = fixture.project_dir.join("dist").join("debug-artifacts");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: true,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("host-selection.txt").exists());
    assert!(output_dir.join("run-contracts.txt").exists());
}

#[test]
fn builds_clr_project_with_real_external_call_edge_in_msil() {
    let fixture = create_smoke_project_with_manifest(
        "legion-build-clr-interop-stdout",
        r#"{
    name: "test_clr_interop_stdout",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "clr",
            msil: true
        }
    ]
}
"#,
        r#"[clr("mscorlib", "System.Console", "WriteLine")]
micro console_write_line(message: utf16): unit;

[main]
micro main() -> i64 {
    console_write_line("hello from clr")
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr-interop");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let msil = fs::read_to_string(output_dir.join("main.msil")).unwrap();
    assert!(msil.contains("ldstr \"hello from clr\""));
    assert!(msil.contains("call void [System.Console]System.Console::WriteLine(string)"));
    assert!(msil.lines().any(|line| line.contains("call") && line.contains("__main()")));
}

#[test]
fn uses_main_attribute_instead_of_function_name_for_entry_selection() {
    let fixture = create_smoke_project_with_source(
        "legion-build-entry",
        r#"[main]
micro helper_entry() -> i64 {
    return 0;
}

micro main() -> i64 {
    return 1;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let msil = fs::read_to_string(output_dir.join("helper_entry.msil")).unwrap();
    assert!(msil.lines().any(|line| line.contains("call") && line.contains("__helper_entry()")));
    assert!(!msil.lines().any(|line| line.contains("call") && line.contains("__main()")));
    let run_contract = fs::read_to_string(output_dir.join("run-contracts.txt")).unwrap();
    assert!(run_contract.contains("logical_entry: \"Main\""));
}

#[test]
fn builds_multiple_artifacts_from_multiple_main_annotations() {
    let fixture = create_smoke_project_with_build(
        "legion-build-multi-main-node",
        r#"{
            target: "node"
        }"#,
        r#"[main]
micro alpha_entry() -> i64 {
    return 0;
}

[main]
micro beta_entry() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-node");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("node").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let manifest = ExecutionManifest::read_from_output_dir(&output_dir).unwrap().expect("execution manifest");
    assert_eq!(manifest.run_contracts.len(), 2);
    assert!(manifest.run_contracts.iter().any(|contract| contract.physical_entry.contains("alpha_entry")));
    assert!(manifest.run_contracts.iter().any(|contract| contract.physical_entry.contains("beta_entry")));
    assert!(output_dir.join("run-contract.txt").exists());
}

#[test]
fn builds_clr_project_with_tuple_pattern_let_and_loop_in() {
    let fixture = create_smoke_project_with_source(
        "legion-build-pattern",
        r#"micro main() -> i64 {
    let ((x, _), y) = ((1, 2), 3);
    loop ((a, _), b) in [((4, 5), 6)] {
        return x + y + a + b;
    }
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());
    assert!(output_dir.join("main.msil").exists());
}

#[test]
fn builds_node_wasm_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-node",
        r#"{
            target: "node",
            wat: true
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-node");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("node").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.wasm").exists());
    assert!(output_dir.join("main.wat").exists());
    assert!(output_dir.join("main.mjs").exists());
    let run_contract = fs::read_to_string(output_dir.join("run-contracts.txt")).unwrap();
    assert!(run_contract.contains("physical_entry: \"main.mjs\""));
    assert!(run_contract.contains("invocation: \"node\""));
}

#[test]
fn builds_wasi_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-wasi",
        r#"{
            target: "wasi",
            wat: true
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-wasi");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasi").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.wasi").exists());
    assert!(output_dir.join("main.wat").exists());
    let run_contract = fs::read_to_string(output_dir.join("run-contracts.txt")).unwrap();
    assert!(run_contract.contains("physical_entry: \"main.wasi\""));
    assert!(run_contract.contains("invocation: \"wasmtime\""));
}

#[test]
fn builds_native_msvc_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-native",
        r#"{
            target: "x86_64-pc-windows-msvc"
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-native");
    let canonical_target = CanonicalTarget::parse("x86_64-pc-windows-msvc").unwrap();
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: canonical_target,
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.obj").exists());
    let exe_path = output_dir.join("main.exe");
    assert!(exe_path.exists());
    let run_contract = fs::read_to_string(output_dir.join("run-contracts.txt")).unwrap();
    assert!(run_contract.contains("physical_entry: \"main.exe\""));
    assert!(run_contract.contains("invocation: \"windows\""));
    #[cfg(windows)]
    {
        let output = Command::new(&exe_path).output().expect("run native exe");
        assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
#[cfg(windows)]
fn builds_native_msvc_project_with_print() {
    let fixture = create_smoke_project_with_manifest(
        "legion-build-native-interop-stdout",
        r#"{
    name: "test_native_interop_stdout",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "x86_64-pc-windows-msvc"
        }
    ]
}
"#,
        r#"[c("kernel32", "WriteFile")]
micro console_write(message: utf8): i32;

[main]
micro main() -> i64 {
    console_write("hello from native")
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-native-interop");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("x86_64-pc-windows-msvc").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let exe_path = output_dir.join("main.exe");
    assert!(exe_path.exists());
    assert!(output_dir.join("main.obj").exists());

    let output = Command::new(&exe_path).output().expect("run native exe");
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello from native");
}

#[test]
fn builds_native_linux_gnu_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-native-linux",
        r#"{
            target: "x86_64-unknown-linux-gnu"
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-linux");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("x86_64-unknown-linux-gnu").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let elf_path = output_dir.join("main");
    assert!(elf_path.exists(), "expected ELF at {}", elf_path.display());
    assert!(!output_dir.join("main.exe").exists());
    let bytes = fs::read(&elf_path).unwrap();
    assert_eq!(&bytes[0..4], b"\x7fELF");
    let run_contract = fs::read_to_string(output_dir.join("run-contracts.txt")).unwrap();
    assert!(run_contract.contains("physical_entry: \"main\""));
    assert!(run_contract.contains("invocation: \"linux\""));

    let output = run_linux_elf(&elf_path);
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
}

#[test]
fn builds_native_linux_gnu_project_with_print() {
    let fixture = create_smoke_project_with_manifest(
        "legion-build-linux-interop-stdout",
        r#"{
    name: "test_linux_interop_stdout",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        {
            target: "x86_64-unknown-linux-gnu"
        }
    ]
}
"#,
        r#"[syscall(1)]
micro console_write(message: utf8): i32;

[main]
micro main() -> i64 {
    console_write("hello from linux")
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-linux-interop");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("x86_64-unknown-linux-gnu").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let elf_path = output_dir.join("main");
    assert!(elf_path.exists());
    let bytes = fs::read(&elf_path).unwrap();
    assert_eq!(&bytes[0..4], b"\x7fELF");

    let output = run_linux_elf(&elf_path);
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "hello from linux");
}

/// Unix 直接 exec；Windows 通过 WSL exec。不可用或路径转换失败时 panic（不允许静默跳过）。
fn run_linux_elf(path: &std::path::Path) -> std::process::Output {
    #[cfg(unix)]
    {
        return std::process::Command::new(path).output().expect("run linux elf");
    }
    #[cfg(windows)]
    {
        let wsl_check = Command::new("wsl").args(["-e", "true"]).status().expect("WSL required to execute Linux ELF on Windows");
        assert!(wsl_check.success(), "WSL required to execute Linux ELF on Windows");
        let win_path = windows_path_for_wsl(path);
        let wslpath = Command::new("wsl").args(["-e", "wslpath", "-a", &win_path]).output().expect("wslpath");
        assert!(wslpath.status.success(), "wslpath failed for {}: {}", win_path, String::from_utf8_lossy(&wslpath.stderr));
        let linux_path = String::from_utf8_lossy(&wslpath.stdout).trim().to_string();
        assert!(!linux_path.is_empty(), "wslpath returned empty path for {win_path}");
        let chmod = Command::new("wsl").args(["-e", "chmod", "+x", &linux_path]).status().expect("chmod");
        assert!(chmod.success(), "chmod +x failed for {linux_path}");
        Command::new("wsl").args(["-e", &linux_path]).output().expect("wsl run elf")
    }
    #[cfg(not(any(unix, windows)))]
    {
        panic!("Linux ELF runtime verification unsupported on this host");
    }
}

#[cfg(windows)]
fn windows_path_for_wsl(path: &std::path::Path) -> String {
    let s = path.to_string_lossy();
    let s = s.strip_prefix(r"\\?\").or_else(|| s.strip_prefix("//?/")).unwrap_or(&s);
    s.replace('\\', "/")
}

#[test]
fn builds_migrated_test_wasm_minimal_project() {
    let fixture = create_smoke_project_with_build(
        "legion-build-migrated-wasm-minimal",
        r#"{
            target: "wasm32-unknown-web-webassembly"
        }"#,
        r#"namespace test;

[main]
micro main(): unit {
    var _ = hello()
}

micro hello(): i64 {
    return 42
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("custom-web-wasm");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasm32-unknown-web-webassembly").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.wasm").exists());
    assert!(output_dir.join("main.mjs").exists());
    assert!(output_dir.join("run-contracts.txt").exists());
}

#[test]
fn builds_migrated_test_wasm_hello_project() {
    let source = r#"[main]
micro main(): i64 {
    return 42;
}
"#;
    let node_fixture = create_smoke_project_with_build(
        "legion-build-migrated-wasm-hello-node",
        r#"{
            target: "node"
        }"#,
        source,
    );
    let node_output_dir = node_fixture.project_dir.join("dist").join("custom-node");
    let node_status = run(&BuildArgs {
        project_dir: node_fixture.project_dir.clone(),
        target: CanonicalTarget::parse("node").unwrap(),
        output_dir: Some(node_output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(node_status, ExitCode::SUCCESS);
    assert!(node_output_dir.join("main.wasm").exists());
    assert!(node_output_dir.join("main.mjs").exists());

    let wasi_fixture = create_smoke_project_with_build(
        "legion-build-migrated-wasm-hello-wasi",
        r#"{
            target: "wasi"
        }"#,
        source,
    );
    let wasi_output_dir = wasi_fixture.project_dir.join("dist").join("custom-wasi");
    let wasi_status = run(&BuildArgs {
        project_dir: wasi_fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasi").unwrap(),
        output_dir: Some(wasi_output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(wasi_status, ExitCode::SUCCESS);
    assert!(wasi_output_dir.join("main.wasi").exists());
    assert!(wasi_output_dir.join("run-contracts.txt").exists());
}

#[test]
fn builds_local_package_when_project_is_not_registered_in_workspace_members() {
    let fixture = create_local_package_project(
        "legion-build-local-package",
        r#"{
            target: "clr",
            msil: true
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
    );
    let output_dir = fixture.project_dir.join("dist").join("local-package");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    assert!(output_dir.join("main.exe").exists());
    assert!(output_dir.join("run-contracts.txt").exists());
}

const WITNESS_SMOKE_SOURCE: &str = r#"trait Animal {
    micro make_sound(): utf8
}

class Dog {
}

imply Dog: Animal {
    micro make_sound() -> utf8 {
        return "woof"
    }
}

[main]
micro main() -> i64 {
    return 0;
}
"#;

#[test]
fn builds_native_linux_gnu_witness_dispatch() {
    let fixture = create_smoke_project_with_build(
        "legion-build-linux-witness",
        r#"{
            target: "x86_64-unknown-linux-gnu"
        }"#,
        WITNESS_SMOKE_SOURCE,
    );
    let output_dir = fixture.project_dir.join("dist").join("linux-witness");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("x86_64-unknown-linux-gnu").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let elf_path = output_dir.join("main");
    assert!(elf_path.exists());
    let output = run_linux_elf(&elf_path);
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "woof");
}

#[test]
fn builds_native_msvc_witness_dispatch() {
    let fixture = create_smoke_project_with_build(
        "legion-build-msvc-witness",
        r#"{
            target: "x86_64-pc-windows-msvc"
        }"#,
        WITNESS_SMOKE_SOURCE,
    );
    let output_dir = fixture.project_dir.join("dist").join("msvc-witness");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("x86_64-pc-windows-msvc").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let exe_path = output_dir.join("main.exe");
    assert!(exe_path.exists());
    let output = Command::new(&exe_path).output().expect("run native exe");
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "woof");
}

#[test]
fn builds_wasm_wasi_witness_dispatch() {
    let fixture = create_smoke_project_with_build(
        "legion-build-wasi-witness",
        r#"{
            target: "wasi"
        }"#,
        WITNESS_SMOKE_SOURCE,
    );
    let output_dir = fixture.project_dir.join("dist").join("wasi-witness");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("wasi").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let wasm_path = output_dir.join("main.wasi");
    assert!(wasm_path.exists());
    let output = run_wasmtime(&wasm_path);
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout), "woof");
}

#[test]
fn builds_clr_witness_dispatch() {
    let fixture = create_smoke_project_with_build("legion-build-clr-witness", r#"{ target: "clr" }"#, WITNESS_SMOKE_SOURCE);
    let output_dir = fixture.project_dir.join("dist").join("clr-witness");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let exe_path = output_dir.join("main.exe");
    assert!(exe_path.exists());
    if !command_exists("dotnet") {
        return;
    }
    let output = Command::new("dotnet").arg("exec").arg(&exe_path).output().expect("dotnet exec");
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "woof");
}

#[test]
fn builds_jvm_witness_dispatch() {
    let fixture =
        create_smoke_project_with_build("legion-build-jvm-witness", r#"{ target: "jvm-openjdk-unknown-managed" }"#, WITNESS_SMOKE_SOURCE);
    let output_dir = fixture.project_dir.join("dist").join("jvm-witness");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();

    assert_eq!(status, ExitCode::SUCCESS);
    let jar_path = output_dir.join("main.jar");
    assert!(jar_path.exists());
    if !command_exists("java") {
        return;
    }
    let output = match run_java_from_run_contracts(&output_dir) {
        Some(output) => output,
        None => match run_java_witness_jar(&jar_path) {
            Some(output) => output,
            None => {
                eprintln!("skip jvm witness dispatch: could not locate runnable JVM entry in {}", output_dir.display());
                return;
            }
        },
    };
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    assert_eq!(String::from_utf8_lossy(&output.stdout).trim(), "woof");
}

fn run_java_witness_jar(jar_path: &std::path::Path) -> Option<std::process::Output> {
    use std::{io::Read, process::Command};
    let file = std::fs::File::open(jar_path).ok()?;
    let mut archive = zip::ZipArchive::new(file).ok()?;
    let mut main_class = None::<String>;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).ok()?;
        if entry.name().eq_ignore_ascii_case("META-INF/MANIFEST.MF") {
            let mut manifest = String::new();
            entry.read_to_string(&mut manifest).ok()?;
            main_class = manifest.lines().find_map(|line| line.strip_prefix("Main-Class:").map(|value| value.trim().to_string()));
            break;
        }
    }
    let main_class = main_class?;
    Command::new("java").arg("-cp").arg(windows_friendly_path(jar_path)).arg(main_class).output().ok()
}

fn windows_friendly_path(path: &std::path::Path) -> std::path::PathBuf {
    let rendered = path.to_string_lossy();
    if let Some(stripped) = rendered.strip_prefix(r"\\?\") { std::path::PathBuf::from(stripped) } else { path.to_path_buf() }
}

fn run_java_from_run_contracts(output_dir: &std::path::Path) -> Option<std::process::Output> {
    run_java_from_run_contracts_filtered(output_dir, |_| true)
}

fn run_java_suspend_from_run_contracts(output_dir: &std::path::Path) -> Option<std::process::Output> {
    run_java_from_run_contracts_filtered(output_dir, |physical| physical.contains("suspend"))
        .or_else(|| run_java_from_run_contracts(output_dir))
}

fn run_java_from_run_contracts_filtered(output_dir: &std::path::Path, filter: impl Fn(&str) -> bool) -> Option<std::process::Output> {
    use std::process::Command;
    let contracts = RunContract::read_all_from_output_dir(output_dir).ok()?;
    for contract in &contracts {
        if contract.physical_entry.is_empty() || !filter(&contract.physical_entry) {
            continue;
        }
        let jar_path = windows_friendly_path(&output_dir.join(&contract.physical_entry));
        if !jar_path.is_file() || !jar_path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("jar"))
        {
            continue;
        }
        let main_class = if contract.logical_entry.is_empty() {
            jar_path.file_stem().and_then(|value| value.to_str()).map(str::to_string)?
        }
        else {
            contract.logical_entry.clone()
        };
        if let Some(output) = Command::new("java").arg("-jar").arg(&jar_path).output().ok() {
            if output.status.success() {
                return Some(output);
            }
        }
        return Command::new("java").arg("-cp").arg(&jar_path).arg(main_class).output().ok();
    }
    None
}

fn jar_path_from_run_contracts(output_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let contracts = RunContract::read_all_from_output_dir(output_dir).ok()?;
    for contract in &contracts {
        if contract.physical_entry.is_empty() {
            continue;
        }
        let path = output_dir.join(&contract.physical_entry);
        if path.is_file() && path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("jar")) {
            return Some(path);
        }
    }
    None
}

fn run_wasmtime(path: &std::path::Path) -> std::process::Output {
    Command::new("wasmtime").arg(path).output().unwrap_or_else(|error| panic!("wasmtime required to execute WASI witness sample: {error}"))
}

const AWAIT_FUTURE_SUSPEND_SOURCE: &str = include_str!("fixtures/runtime_smoke/await_future.valkyrie");
const SUSPEND_TRAIT_COMBO_SOURCE: &str = include_str!("fixtures/runtime_smoke/suspend_trait_combo.valkyrie");

fn multi_lane_suspend_manifest() -> String {
    r#"{
    name: "test_suspend",
    version: "0.1.0",
    dependencies: {
        "std": false,
        "core": false
    },
    build: [
        { target: "clr", msil: true },
        { target: "jvm-openjdk-unknown-managed" },
        { target: "node" },
        { target: "wasi" },
        { target: "x86_64-unknown-linux-gnu" },
        { target: "x86_64-pc-windows-msvc" }
    ]
}
"#
    .to_string()
}

#[test]
fn builds_clr_suspend_await_future() {
    let fixture =
        create_smoke_project_with_manifest("legion-build-clr-await-future", &multi_lane_suspend_manifest(), AWAIT_FUTURE_SUSPEND_SOURCE);
    let output_dir = fixture.project_dir.join("dist").join("clr");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::clr(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();
    assert_eq!(status, ExitCode::SUCCESS);
    // 单 [main] entry 时，partitioner 将 sync + suspend 函数合并进单一 partition，
    // 产物名为 main.exe（与 builds_clr_witness_dispatch 一致）。
    let exe_path = output_dir.join("main.exe");
    assert!(exe_path.exists(), "expected CLR executable in {}", output_dir.display());
    if command_exists("dotnet") {
        let output = Command::new("dotnet").arg(&exe_path).output().expect("dotnet run");
        assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
    }
}

#[test]
fn builds_jvm_suspend_trait_combo() {
    let fixture =
        create_smoke_project_with_manifest("legion-build-jvm-suspend-combo", &multi_lane_suspend_manifest(), SUSPEND_TRAIT_COMBO_SOURCE);
    let output_dir = fixture.project_dir.join("dist").join("jvm");
    let status = run(&BuildArgs {
        project_dir: fixture.project_dir.clone(),
        target: CanonicalTarget::parse("jvm-openjdk-unknown-managed").unwrap(),
        output_dir: Some(output_dir.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .unwrap();
    assert_eq!(status, ExitCode::SUCCESS);
    let jar_path = jar_path_from_run_contracts(&output_dir)
        .unwrap_or_else(|| panic!("expected JVM jar artifact from run-contracts.txt physical_entry in {}", output_dir.display()));
    assert!(jar_path.exists(), "expected JVM jar at {}", jar_path.display());
    if !command_exists("java") {
        return;
    }
    if std::env::var_os("LEGION_TEST_DEBUG_JAR").is_some() {
        for entry in fs::read_dir(&output_dir).into_iter().flatten().flatten() {
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()).is_some_and(|value| value.eq_ignore_ascii_case("jar")) {
                eprintln!("jar artifact: {}", path.display());
                if let Ok(file) = std::fs::File::open(&path) {
                    if let Ok(mut archive) = zip::ZipArchive::new(file) {
                        for index in 0..archive.len() {
                            if let Ok(item) = archive.by_index(index) {
                                eprintln!("  {}", item.name());
                            }
                        }
                    }
                }
            }
        }
        if let Ok(contracts) = RunContract::read_all_from_output_dir(&output_dir) {
            for contract in contracts {
                eprintln!("run contract: logical={} physical={}", contract.logical_entry, contract.physical_entry);
            }
        }
    }
    let output = match run_java_suspend_from_run_contracts(&output_dir) {
        Some(output) => output,
        None => {
            eprintln!("skip jvm suspend trait combo: could not locate runnable JVM entry in {}", output_dir.display());
            return;
        }
    };
    assert!(output.status.success(), "exit={:?} stderr={}", output.status, String::from_utf8_lossy(&output.stderr));
}

fn command_exists(command: &str) -> bool {
    std::env::var_os("PATH").is_some_and(|path| {
        std::env::split_paths(&path).any(|dir| {
            let base = dir.join(command);
            base.is_file()
                || [".exe", ".cmd", ".bat", ".com"].iter().map(|ext| dir.join(format!("{command}{ext}"))).any(|candidate| candidate.is_file())
        })
    })
}

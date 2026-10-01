//! 正式 capability：完整 legion.tools manifests/source closure → Node/Wasm 产物。
//! 源项目必须由门禁显式传入，禁止用 bootstrap fixture 替代编译器源码闭包。
//! 不依赖 native `legion` 二进制；直接调用库内 `legion build` 实现。

use legion::cmds::build::{BuildArgs, run};
use nyar_language::CanonicalTarget;
use std::path::{Path, PathBuf};

fn capability_source_root() -> PathBuf {
    let root = PathBuf::from(std::env::var("LEGION_CAPABILITY_SOURCE")
        .expect("正式门禁必须传入 LEGION_CAPABILITY_SOURCE，不允许 fixture fallback"));
    assert_eq!(root.file_name().and_then(|name| name.to_str()), Some("legion.tools"));
    root
}

fn capability_out_root() -> PathBuf {
    std::env::var("LEGION_CAPABILITY_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").join("dist/legion-node-capability"))
}

fn artifact_dir(out_root: &Path) -> PathBuf {
    let nested = out_root.join("wasm32-node-unknown-wasm");
    if nested.join("legion.wasm").is_file() {
        return nested;
    }
    out_root.to_path_buf()
}

#[test]
fn assemble_vcc_unknown_wasm32_capability() {
    let fixture = capability_source_root();
    assert!(fixture.join("legion.von").is_file(), "legion.tools manifest missing");

    let out_root = capability_out_root();
    std::fs::create_dir_all(&out_root).expect("create capability output dir");

    let status = run(&BuildArgs {
        project_dir: fixture.clone(),
        target: CanonicalTarget::parse("node").expect("node target"),
        output_dir: Some(out_root.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .expect("legion build complete compiler source closure");

    assert_eq!(status, std::process::ExitCode::SUCCESS);

    let artifacts = artifact_dir(&out_root);
    let wasm_path = artifacts.join("legion.wasm");
    assert!(wasm_path.is_file(), "missing legion.wasm under {}", artifacts.display());
    assert!(artifacts.join("run-contracts.txt").is_file(), "missing run-contracts.txt under {}", artifacts.display());
    let wasm_bytes = std::fs::metadata(&wasm_path).expect("legion.wasm metadata").len();
    assert!(wasm_bytes >= 1024, "legion.wasm is {wasm_bytes} bytes (< 1024); executable closure is likely empty");
}

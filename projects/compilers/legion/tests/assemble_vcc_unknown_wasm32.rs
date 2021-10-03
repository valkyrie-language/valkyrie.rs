//! CI：`valkyrie.v` 的 `legion.tools` → Node wasm 产物，供 `scripts/build.mjs capability` 装配。
//!
//! 不依赖 native `legion` 二进制；直接调用库内 `legion build` 实现。

use legion::cmds::build::{BuildArgs, run};
use nyar_language::CanonicalTarget;
use std::path::{Path, PathBuf};

fn valkyrie_v_root() -> PathBuf {
    std::env::var("VALKYRIE_V")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").join("valkyrie.v"))
}

fn capability_out_root() -> PathBuf {
    std::env::var("LEGION_CAPABILITY_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..").join("dist/legion-node-capability"))
}

fn find_tools_project(valkyrie_v: &Path) -> PathBuf {
    for candidate in [
        valkyrie_v.join("projects/legion._/projects/legion.tools"),
        valkyrie_v.join("projects/legion.tools"),
    ] {
        if candidate.join("legion.von").is_file() {
            return candidate;
        }
    }
    panic!("legion.tools not found under {}", valkyrie_v.display());
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
    let valkyrie_v = valkyrie_v_root();
    assert!(valkyrie_v.is_dir(), "missing valkyrie.v at {}", valkyrie_v.display());

    let tools = find_tools_project(&valkyrie_v);
    let out_root = capability_out_root();
    std::fs::create_dir_all(&out_root).expect("create capability output dir");

    let status = run(&BuildArgs {
        project_dir: tools.clone(),
        target: CanonicalTarget::parse("node").expect("node target"),
        output_dir: Some(out_root.clone()),
        workspace: false,
        debug_artifacts: false,
    })
    .expect("legion build legion.tools for node");

    assert_eq!(status, std::process::ExitCode::SUCCESS);

    let artifacts = artifact_dir(&out_root);
    assert!(
        artifacts.join("legion.wasm").is_file(),
        "missing legion.wasm under {}",
        artifacts.display()
    );
    assert!(
        artifacts.join("run-contracts.txt").is_file(),
        "missing run-contracts.txt under {}",
        artifacts.display()
    );
}

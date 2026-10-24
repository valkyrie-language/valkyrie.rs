//! 用 `nyar-vm.rs` 的生产 CLI 入口对 `projects/fixtures` 语料做黑盒验收。
//!
//! `vcc plan` 只解析 manifest 与 workspace，不进入 Compiler；因此它证明语料自身的工程输入仍然可解析，
//! 而不在本仓重新实现任何编译逻辑。

use std::{
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../fixtures")
}

fn project_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).unwrap_or_else(|err| panic!("无法读取 {}: {err}", dir.display())) {
            let entry = entry.expect("读取目录项失败");
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            }
            else if path.file_name().and_then(|name| name.to_str()) == Some("legion.von") {
                if let Some(parent) = path.parent() {
                    dirs.push(parent.to_path_buf());
                }
            }
        }
    }
    dirs.sort();
    dirs
}

#[test]
fn fixture_projects_resolve_plan() {
    let projects = project_dirs(&fixtures_root());
    assert!(!projects.is_empty(), "projects/fixtures 下没有 legion.von 语料工程");

    let mut failures = Vec::new();
    for project in &projects {
        // 语料的 `legion.von` 都声明 `target: node`；`plan` 的默认目标是 clr，因此显式指定。
        let argv: Vec<OsString> = vec!["vcc".into(), "plan".into(), project.clone().into_os_string(), "--target".into(), "node".into()];
        let (code, stdout, stderr) = legion::cli::run_with_argv_captured_on_current_thread(argv);
        if code != 0 {
            failures.push(format!("{} 退出码 {code}\nstdout:\n{stdout}\nstderr:\n{stderr}", project.display()));
        }
    }

    assert!(failures.is_empty(), "{} 个语料工程无法解析 plan:\n{}", failures.len(), failures.join("\n\n"));
}

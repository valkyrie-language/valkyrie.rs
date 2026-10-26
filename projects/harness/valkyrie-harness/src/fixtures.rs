//! `projects/fixtures` 语料工程发现（黑盒 / 白盒 harness 共用）。

use std::path::{Path, PathBuf};

/// `projects/fixtures` 根目录。
pub fn fixtures_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures")
}

/// 递归收集含 `legion.von` 的 fixture 工程目录。
pub fn project_dirs(root: &Path) -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in std::fs::read_dir(&dir).unwrap_or_else(|err| panic!("无法读取 {}: {err}", dir.display())) {
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

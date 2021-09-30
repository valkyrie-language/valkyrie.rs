use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

pub struct SmokeProject {
    _temp_dir: TempDir,
    pub project_dir: PathBuf,
}

pub fn create_smoke_project(prefix: &str) -> SmokeProject {
    create_fixture(prefix, default_build_manifest(), "micro main() -> i64 {\n    return 0;\n}\n")
}

pub fn create_smoke_project_with_source(prefix: &str, source: &str) -> SmokeProject {
    create_fixture(prefix, default_build_manifest(), source)
}

pub fn create_smoke_project_with_build(prefix: &str, build_block: &str, source: &str) -> SmokeProject {
    create_fixture(prefix, build_block, source)
}

fn create_fixture(prefix: &str, build_block: &str, source: &str) -> SmokeProject {
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let root = temp_dir.path();
    let project_dir = root.join("app");
    let source_dir = project_dir.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(root.join("legions.von"), "{\n    name: \"runtime-smoke\",\n    members: [\n        \"app\"\n    ]\n}\n").unwrap();
    fs::write(project_dir.join("legion.von"), format!("{{\n    name: \"app\",\n    build: [\n        {}\n    ]\n}}\n", build_block)).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();

    SmokeProject { _temp_dir: temp_dir, project_dir: canonicalize_lossy(&project_dir) }
}

fn default_build_manifest() -> &'static str {
    "{\n            target: \"clr\",\n            msil: true\n        }"
}

fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

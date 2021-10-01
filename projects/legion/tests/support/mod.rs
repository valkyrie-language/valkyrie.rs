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
    create_fixture(
        prefix,
        default_build_manifest(),
        r#"micro main() -> i64 {
    return 0;
}
"#,
    )
}

pub fn create_smoke_project_with_source(prefix: &str, source: &str) -> SmokeProject {
    create_fixture(prefix, default_build_manifest(), source)
}

pub fn create_smoke_project_with_build(prefix: &str, build_block: &str, source: &str) -> SmokeProject {
    create_fixture(prefix, build_block, source)
}

pub fn create_smoke_project_with_manifest(prefix: &str, manifest: &str, source: &str) -> SmokeProject {
    create_manifest_fixture(prefix, manifest, source, true)
}

pub fn create_local_package_project(prefix: &str, build_block: &str, source: &str) -> SmokeProject {
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let root = temp_dir.path();
    let project_dir = root.join("app");
    let source_dir = project_dir.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        root.join("legions.von"),
        r#"{
    name: "runtime-smoke",
    members: []
}
"#,
    )
    .unwrap();
    fs::write(
        project_dir.join("legion.von"),
        format!(
            r#"{{
    name: "app",
    build: [
        {}
    ]
}}
"#,
            build_block
        ),
    )
    .unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();

    SmokeProject { _temp_dir: temp_dir, project_dir: canonicalize_lossy(&project_dir) }
}

fn create_fixture(prefix: &str, build_block: &str, source: &str) -> SmokeProject {
    let manifest = format!(
        r#"{{
    name: "app",
    build: [
        {}
    ]
}}
"#,
        build_block
    );
    create_manifest_fixture(prefix, &manifest, source, true)
}

fn create_manifest_fixture(prefix: &str, manifest: &str, source: &str, include_workspace_member: bool) -> SmokeProject {
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let root = temp_dir.path();
    let project_dir = root.join("app");
    let source_dir = project_dir.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    let workspace_manifest = if include_workspace_member {
        r#"{
    name: "runtime-smoke",
    members: [
        "app"
    ]
}
"#
    }
    else {
        r#"{
    name: "runtime-smoke",
    members: []
}
"#
    };
    fs::write(root.join("legions.von"), workspace_manifest).unwrap();
    fs::write(project_dir.join("legion.von"), manifest).unwrap();
    fs::write(source_dir.join("main.v"), source).unwrap();

    SmokeProject { _temp_dir: temp_dir, project_dir: canonicalize_lossy(&project_dir) }
}

fn default_build_manifest() -> &'static str {
    r#"{
            target: "clr",
            msil: true
        }"#
}

fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

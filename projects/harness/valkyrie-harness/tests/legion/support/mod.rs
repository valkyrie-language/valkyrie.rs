//! Harness support for Legion blackbox tests.
//!
//! 内联 smoke 工程脚手架移植自兄弟仓 `rusty-valkyrie/tests/legion/support/mod.rs`
//! （corpus 侧脚手架；本仓不实现编译器，只构造临时工程供白盒 API 消费）。

pub mod run_fixture;
pub mod runtime_fixture;

use std::{
    fs,
    path::{Path, PathBuf},
};

use tempfile::TempDir;

pub fn legion_corpus_root() -> std::path::PathBuf {
    legion_fixtures::root()
}

pub fn legion_corpus_subdir(relative: &str) -> std::path::PathBuf {
    legion_fixtures::root().join(relative)
}

pub struct SmokeProject {
    _temp_dir: TempDir,
    pub project_dir: PathBuf,
}

/// 运行时夹具的 `main` 以 `ExitCode` 表达进程退出码；该名字来自 `std` 适配层，
/// 夹具项目声明 `"std": false`，因此在源级给出等价别名，避免依赖未链接的 std 组。
const EXIT_CODE_DECLARATIONS: &str = concat!(
    "type ExitCode = i32;\n",
    "micro ExitCode(value: i32): i32 { return value }\n",
);

/// 源级 `std/primitive` 声明族：数值原始类型名不是语言内建，必须绑定到源码声明身份。
/// 夹具项目声明 `"std": false`，因此在 `source/` 下写入等价声明文件；身份是
/// `std::primitive::*`，声明落在该身份的规范 namespace（闭包身份来源门按绝对限定位置
/// 解析来源，只有占据规范位置的声明才是 owner）。`usize`/`isize` 是 std 的指针宽度别名，
/// 不是独立 primitive 身份。
const CORE_PRIMITIVE_DECLARATIONS: &str = concat!(
    "namespace std.primitive;\n",
    "[primitive(\"std::primitive::i8\")] structure i8 { }\n",
    "[primitive(\"std::primitive::i16\")] structure i16 { }\n",
    "[primitive(\"std::primitive::i32\")] structure i32 { }\n",
    "[primitive(\"std::primitive::i64\")] structure i64 { }\n",
    "[primitive(\"std::primitive::i128\")] structure i128 { }\n",
    "[primitive(\"std::primitive::u8\")] structure u8 { }\n",
    "[primitive(\"std::primitive::u16\")] structure u16 { }\n",
    "[primitive(\"std::primitive::u32\")] structure u32 { }\n",
    "[primitive(\"std::primitive::u64\")] structure u64 { }\n",
    "[primitive(\"std::primitive::u128\")] structure u128 { }\n",
    "[primitive(\"std::primitive::f32\")] structure f32 { }\n",
    "[primitive(\"std::primitive::f64\")] structure f64 { }\n",
    "[primitive(\"std::primitive::f128\")] structure f128 { }\n",
    "[primitive(\"std::primitive::byte\")] structure byte { }\n",
    "[primitive(\"std::primitive::sbyte\")] structure sbyte { }\n",
    "type usize = u64;\n",
    "type isize = i64;\n",
);

/// 写入夹具入口源码，并同时提供源级 primitive 声明。
fn write_main_source(source_dir: &Path, source: &str) {
    fs::write(source_dir.join("main.v"), source).unwrap();
    write_core_primitives(source_dir);
}

fn write_core_primitives(source_dir: &Path) {
    // `ExitCode` 是全局 namespace 的入口别名，写在规范 `std.primitive` 段之前。
    fs::write(source_dir.join("core_primitive.v"), format!("{EXIT_CODE_DECLARATIONS}{CORE_PRIMITIVE_DECLARATIONS}")).unwrap();
}

pub fn create_smoke_project_with_build(prefix: &str, build_block: &str, source: &str) -> SmokeProject {
    create_fixture(prefix, build_block, source)
}

pub fn create_smoke_project_with_manifest(prefix: &str, manifest: &str, source: &str) -> SmokeProject {
    create_manifest_fixture(prefix, manifest, source, true)
}

pub fn create_local_package_project_with_manifest(prefix: &str, manifest: &str, source: &str) -> SmokeProject {
    create_manifest_fixture(prefix, manifest, source, false)
}

pub fn create_script_project_with_manifest(prefix: &str, manifest: &str, source: &str) -> SmokeProject {
    create_standalone_manifest_fixture(prefix, manifest, source)
}

pub fn create_nested_workspace_member_project_with_manifest(prefix: &str, manifest: &str, source: &str) -> SmokeProject {
    create_nested_workspace_fixture(prefix, Path::new("subspace").join("projects").join("app"), manifest, source)
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
    create_nested_manifest_fixture(prefix, PathBuf::from("app"), manifest, source, include_workspace_member)
}

fn create_nested_manifest_fixture(
    prefix: &str,
    project_relative_dir: PathBuf,
    manifest: &str,
    source: &str,
    include_workspace_member: bool,
) -> SmokeProject {
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let root = temp_dir.path();
    let project_dir = root.join(&project_relative_dir);
    let source_dir = project_dir.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    let workspace_member = project_relative_dir.to_string_lossy().replace('\\', "/");
    let workspace_manifest = if include_workspace_member {
        format!(
            r#"{{
    name: "runtime-smoke",
    members: [
        "{}"
    ]
}}
"#,
            workspace_member
        )
    }
    else {
        r#"{
    name: "runtime-smoke",
    members: []
}
"#
        .to_string()
    };
    fs::write(root.join("legions.von"), workspace_manifest).unwrap();
    fs::write(project_dir.join("legion.von"), manifest).unwrap();
    write_main_source(&source_dir, source);

    SmokeProject { _temp_dir: temp_dir, project_dir: canonicalize_lossy(&project_dir) }
}

fn create_nested_workspace_fixture(prefix: &str, project_relative_dir: PathBuf, manifest: &str, source: &str) -> SmokeProject {
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let root = temp_dir.path();
    let subspace_dir = root.join("subspace");
    let project_dir = root.join(&project_relative_dir);
    let source_dir = project_dir.join("source");
    let nested_member = project_relative_dir
        .strip_prefix("subspace")
        .unwrap_or(project_relative_dir.as_path())
        .to_string_lossy()
        .trim_start_matches(['\\', '/'])
        .replace('\\', "/");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(
        root.join("legions.von"),
        r#"{
    name: "runtime-smoke",
    members: [
        "subspace"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        subspace_dir.join("legions.von"),
        format!(
            r#"{{
    name: "nested-runtime-smoke",
    members: [
        "{nested_member}"
    ]
}}
"#
        ),
    )
    .unwrap();
    fs::write(project_dir.join("legion.von"), manifest).unwrap();
    write_main_source(&source_dir, source);

    SmokeProject { _temp_dir: temp_dir, project_dir: canonicalize_lossy(&project_dir) }
}

fn create_standalone_manifest_fixture(prefix: &str, manifest: &str, source: &str) -> SmokeProject {
    let temp_dir = tempfile::Builder::new().prefix(prefix).tempdir().unwrap();
    let root = temp_dir.path().to_path_buf();
    let source_dir = root.join("source");
    fs::create_dir_all(&source_dir).unwrap();
    fs::write(root.join("legion.von"), manifest).unwrap();
    write_main_source(&source_dir, source);

    SmokeProject { _temp_dir: temp_dir, project_dir: canonicalize_lossy(&root) }
}

/// 规范化路径（失败时保留原路径）。
fn canonicalize_lossy(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

//! 样式合同的负例证据：在临时工程副本里写入越界 `<style>`，真实执行
//! `asgard build --static`，断言结构化拒绝信息与 fail-closed 的产物行为。
//! 仓库内示例保持合法，不为了负例把正向示例改坏。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

/// 复制 `asgard-static` 到临时工作区，并把入口的 `<style>` 内容替换为给定文本。
fn example_with_style(dest: &Path, style: &str) -> PathBuf {
    let project = materialize_example_workspace("asgard-static", dest);
    let entry = project.join("source").join("index.awsl");
    let source = fs::read_to_string(&entry).expect("读取示例入口失败");
    let open = source.find("<style>").expect("示例入口应含 `<style>`");
    let close = source.find("</style>").expect("示例入口应含 `</style>`");
    let rewritten = format!("{}<style>{style}</style>{}", &source[..open], &source[close + "</style>".len()..]);
    fs::write(&entry, rewritten).expect("写入临时入口失败");
    project
}

fn build(project: &Path) -> Output {
    let dist = project.join("dist");
    let args = ["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径"), "--output", dist.to_str().expect("UTF-8 路径")];
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    asgard_cli::run(&owned).expect("无法启动 asgard CLI")
}

fn expect_rejected(project: &Path, needle: &str) {
    let output = build(project);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!output.status.success(), "越界样式应让构建失败（needle `{needle}`）:\nstdout:\n{}\nstderr:\n{stderr}", String::from_utf8_lossy(&output.stdout));
    assert!(stderr.contains(needle), "stderr 应包含 `{needle}`:\n{stderr}");
    // fail-closed：拒绝时不得留下页面产物。
    assert!(!project.join("dist").join("index.html").exists(), "构建被拒绝后不应产出 index.html");
}

#[test]
fn legacy_class_selector_is_rejected() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = example_with_style(temp.path(), ".hero { color: #c67b34 }");
    expect_rejected(&project, "只允许 root / slot <name> / #<id>");
}

#[test]
fn unknown_property_is_rejected() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = example_with_style(temp.path(), "root { margin: 8 }");
    expect_rejected(&project, "未定义属性 `margin`");
}

#[test]
fn relative_unit_is_rejected() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = example_with_style(temp.path(), "root { inset: 1rem }");
    expect_rejected(&project, "禁止依赖宿主盒模型的单位");
}

#[test]
fn important_is_rejected() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = example_with_style(temp.path(), "root { spacing: 8 !important }");
    expect_rejected(&project, "禁止 `!important`");
}

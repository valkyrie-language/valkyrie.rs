//! SSG 预渲染内容证据：直接读取交付产物 `index.html` 的文本，不加载 `boot.js`、
//! 不挂载岛、不执行任何 hydration。
//!
//! hydration 会先清空预渲染 DOM 再按状态重建，所以任何“执行后再断言”的用例都无法
//! 证明页面文件里原本就有内容。本文件与 `asgard_*_interaction` 的宿主行为证据分开，
//! 不互相冒充。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

/// 以显式输出目录构建示例，返回（临时工程根，预渲染页面文本）。
/// 输出目录不依赖 CLI 默认的 `.cache/asgard`。
fn build_and_read_page(name: &str, entry: &str, dest: &Path) -> (PathBuf, String) {
    let project = materialize_example_workspace(name, dest);
    let dist = project.join("dist");
    let args = ["build", "--static", entry, project.to_str().expect("UTF-8 路径"), "--output", dist.to_str().expect("UTF-8 路径")];
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    let output: Output = asgard_cli::run(&owned).expect("无法启动 asgard CLI");
    assert!(
        output.status.success(),
        "示例 `{name}` 静态构建应成功\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let html = fs::read_to_string(dist.join("index.html")).expect("读取预渲染页面失败");
    (project, html)
}

fn assert_page_has(html: &str, needle: &str) {
    assert!(html.contains(needle), "预渲染页面应包含 `{needle}`:\n{html}");
}

fn assert_page_lacks(html: &str, needle: &str) {
    assert!(!html.contains(needle), "预渲染页面不应包含 `{needle}`:\n{html}");
}

#[test]
fn static_page_prerenders_initial_state() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-static", "source/index.awsl", temp.path());

    assert_page_has(&html, "你好，静态世界");
    assert_page_has(&html, "Label: off");
    assert_page_has(&html, r#"value="off""#);
    assert_page_has(&html, "Turn on");
    assert_page_has(&html, "Turn off");
    assert_page_has(&html, "Rebuild island");
    assert_page_has(&html, r#"data-island="hydrated""#);
    // 未执行 hydration：页面里仍是服务端渲染的初始值，而不是运行时状态。
    assert_page_has(&html, "class=\"state-label sc-id-state-label\"");
}

#[test]
fn conditional_page_prerenders_only_the_selected_branch() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-static-conditional", "source/index.awsl", temp.path());

    assert_page_has(&html, "Status Board — online");
    assert_page_has(&html, "expanded body");
    assert_page_lacks(&html, "details hidden");
    assert_page_has(&html, "Show");
    assert_page_has(&html, "Hide");
    assert_page_has(&html, "Refresh");
}

#[test]
fn derived_page_prerenders_computed_values() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-static-derived", "source/index.awsl", temp.path());

    assert_page_has(&html, "Status: red");
    assert_page_has(&html, "hello");
    assert_page_has(&html, "Advance");
    assert_page_has(&html, "Shout");
}

#[test]
fn form_page_prerenders_bound_input_values_and_echoes() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-static-form", "source/index.awsl", temp.path());

    assert_page_has(&html, "Profile");
    assert_page_has(&html, r#"value="Ada""#);
    assert_page_has(&html, r#"value="Paris""#);
    assert_page_has(&html, "Name: Ada");
    assert_page_has(&html, "City: Paris");
    assert_page_has(&html, "Reset");
}

#[test]
fn lifecycle_page_prerenders_initial_mark() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-static-lifecycle", "source/index.awsl", temp.path());

    assert_page_has(&html, "Mark: off");
    assert_page_has(&html, "Mark");
    assert_page_has(&html, "Rebuild island");
}

#[test]
fn nested_page_prerenders_regions_and_state() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-static-nested", "source/index.awsl", temp.path());

    assert_page_has(&html, "Console");
    assert_page_has(&html, "Nav: open");
    assert_page_has(&html, r#"value="first""#);
    assert_page_has(&html, "Note: first");
    assert_page_has(&html, "Menu");
    assert_page_has(&html, "Clear");
}

#[test]
fn webview_page_prerenders_initial_status_text() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, html) = build_and_read_page("asgard-webview", "source/index.awsl", temp.path());

    assert_page_has(&html, "Valkyrie Desk");
    assert_page_has(&html, "WebView desktop host example");
    assert_page_has(&html, "All services online");
    assert_page_has(&html, "Refresh");
}

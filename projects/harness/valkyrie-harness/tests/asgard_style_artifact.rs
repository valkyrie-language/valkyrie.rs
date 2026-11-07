//! 样式合同 → DOM 后端的 CSS 语义证据：断言真实产物 `index.css` 里由合同句柄
//! （`root` / `#id`）落下的声明，而不是只看 CSS 文件是否存在。
//!
//! 断言按“规则块 + 关键声明”读取，不固化整段 CSS 文本，避免合法 lowering 调整
//! 导致无条件失败；同时断言旧类选择器没有回流进产物。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

/// 以显式输出目录构建示例并读取 `index.css`；输出目录不依赖 CLI 默认值。
fn build_and_read_css(name: &str, dest: &Path) -> (PathBuf, String) {
    let project = materialize_example_workspace(name, dest);
    let dist = project.join("dist");
    let args = ["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径"), "--output", dist.to_str().expect("UTF-8 路径")];
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    let output: Output = asgard_cli::run(&owned).expect("无法启动 asgard CLI");
    assert!(
        output.status.success(),
        "示例 `{name}` 静态构建应成功\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let css = fs::read_to_string(dist.join("index.css")).expect("读取样式产物失败");
    (project, css)
}

/// 取某条规则的声明体；选择器必须真实存在，缺失即失败。
fn rule_body<'a>(css: &'a str, selector: &str) -> &'a str {
    let start = css.find(selector).unwrap_or_else(|| panic!("CSS 缺少规则 `{selector}`:\n{css}"));
    let open = css[start..].find('{').map(|i| start + i).expect("规则缺少 `{`");
    let close = css[open..].find('}').map(|i| open + i).expect("规则缺少 `}`");
    &css[open + 1..close]
}

fn assert_rule_has(css: &str, selector: &str, declaration: &str) {
    let body = rule_body(css, selector);
    assert!(body.contains(declaration), "规则 `{selector}` 应包含 `{declaration}`，实际:\n{body}");
}

fn assert_legacy_class_selector_absent(css: &str, class_name: &str) {
    assert!(!css.contains(&format!(".{class_name} {{")), "旧类选择器 `.{class_name}` 不应出现在产物:\n{css}");
}

#[test]
fn static_style_artifact_keeps_visual_intent() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-static", temp.path());

    // `root { color: @warn spacing: 8 }`：令牌降级为具体色值，间距降级为 gap。
    assert_rule_has(&css, ".sc-root {", "color: #f97316");
    assert_rule_has(&css, ".sc-root {", "gap: 8px");
    // `#state-label { text: @title }`：标题令牌降级为字号与字重。
    assert_rule_has(&css, ".sc-id-state-label {", "font-weight: 700");
    assert!(css.contains(".sc-id-state-label {"), "id 句柄应落成 `sc-id-*` 类:\n{css}");
    assert_legacy_class_selector_absent(&css, "hero");
    assert_legacy_class_selector_absent(&css, "status-label");
}

#[test]
fn conditional_style_artifact_keeps_root_inset_and_headline() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-static-conditional", temp.path());

    assert_rule_has(&css, ".sc-root {", "padding: 12px");
    assert_rule_has(&css, ".sc-id-headline {", "font-size: 20px");
    assert_legacy_class_selector_absent(&css, "board");
    assert_legacy_class_selector_absent(&css, "headline");
}

#[test]
fn derived_style_artifact_keeps_token_color() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-static-derived", temp.path());

    assert_rule_has(&css, ".sc-root {", "padding: 16px");
    // `#message-label { color: @brand }`：品牌令牌降级为品牌色。
    assert_rule_has(&css, ".sc-id-message-label {", "color: #38bdf8");
    // `#status-label { text: @body }`：正文令牌降级为正文度量。
    assert_rule_has(&css, ".sc-id-status-label {", "font-size: 15px");
    assert_legacy_class_selector_absent(&css, "message-label");
}

#[test]
fn form_style_artifact_styles_title_and_both_echoes() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-static-form", temp.path());

    assert_rule_has(&css, ".sc-root {", "padding: 16px");
    assert_rule_has(&css, ".sc-id-form-title {", "font-weight: 700");
    assert_rule_has(&css, ".sc-id-echo-name {", "font-size: 15px");
    assert_rule_has(&css, ".sc-id-echo-city {", "font-size: 15px");
    assert_legacy_class_selector_absent(&css, "form-title");
}

#[test]
fn lifecycle_style_artifact_keeps_reserved_line_height() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-static-lifecycle", temp.path());

    assert_rule_has(&css, ".sc-root {", "padding: 16px");
    assert_rule_has(&css, ".sc-id-mark-line {", "height: 20px");
    assert_legacy_class_selector_absent(&css, "mark-line");
}

#[test]
fn nested_style_artifact_keeps_layout_regions() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-static-nested", temp.path());

    assert_rule_has(&css, ".sc-root {", "flex-direction: column");
    assert_rule_has(&css, ".sc-id-header {", "flex-direction: row");
    assert_rule_has(&css, ".sc-id-header {", "justify-content: space-between");
    assert_rule_has(&css, ".sc-id-content {", "flex-direction: row");
    assert_rule_has(&css, ".sc-id-sidebar {", "width: 160px");
    assert_rule_has(&css, ".sc-id-main {", "width: 100%");
    assert_rule_has(&css, ".sc-id-nav-toggle {", "font-weight: 700");
    assert_rule_has(&css, ".sc-id-clear-note {", "color: #94a3b8");
    assert_legacy_class_selector_absent(&css, "sidebar");
    assert_legacy_class_selector_absent(&css, "primary");
}

#[test]
fn webview_style_artifact_keeps_desk_typography() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let (_project, css) = build_and_read_css("asgard-webview", temp.path());

    assert_rule_has(&css, ".sc-root {", "padding: 24px");
    assert_rule_has(&css, ".sc-id-desk-title {", "font-size: 20px");
    assert_rule_has(&css, ".sc-id-desk-status {", "font-size: 15px");
    assert_legacy_class_selector_absent(&css, "desk");
}

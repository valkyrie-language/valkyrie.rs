//! `projects/examples/asgard-webview` 的桌面 WebView 示例语料黑盒测试。
//!
//! 复现命令（自示例目录出发，CLI 入口经 `valkyrie_harness::asgard_cli` 解析）：
//!
//! ```text
//! node <nyar-vm.rs>/projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs \
//!   build --static source/index.awsl <examples/asgard-webview>
//! ```
//!
//! 证据边界（重要，勿以绿灯掩盖）：
//! - 本示例是单一入口（规范后缀 `source/index.awsl`，`.w` 同语义可接受）的合法 Legion
//!   工程；首轮 `artifact.target` 只支持 `static`，桌面差异在 target 策略与宿主打包，
//!   不在配置拼写。
//! - 当前可执行的只有浏览器静态交付线：模板经 Legion 内置 Odin VM 进程内执行产出
//!   预渲染页面，交付产物为页面 + wasm chunk + 浏览器 glue + 组件胶水 + boot +
//!   manifest + CSS（完整链证据见 `asgard_delivery_artifact`）；本文件只断言工程
//!   结构、产物树形态与页面壳合同，不断言组件级渲染形态。
//! - 桌面 WebView 验收门（干净打包启动、点击/键盘状态往返、离线资源、宿主权限/导航
//!   隔离、关闭清理）依赖宿主选型与 `asgard.ui` V 库，当前全部 blocked，
//!   记录于 `asgard_delivery_contract`，不以静态预渲染冒充桌面交付。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

fn example_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/asgard-webview")
}

/// 配置声明的唯一入口（相对项目根）；结构断言与构建命令共用同一常量。
/// `.awsl` 为规范拼写（03 §2，设计仓 `54d13b9`），`.w` 同语义仍可接受，
/// 由 `asgard_static` 的同义用例证明，本文件不把任一后缀写成唯一合法形态。
const CONFIGURED_ENTRY: &str = "source/index.awsl";

fn copy_example_into(dest: &Path) -> PathBuf {
    materialize_example_workspace("asgard-webview", dest)
}

fn asgard(args: &[&str]) -> Output {
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    asgard_cli::run(&owned).expect("无法启动 asgard CLI")
}

fn stdout_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_of(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn expect_success(output: &Output) -> String {
    let (stdout, stderr) = (stdout_of(output), stderr_of(output));
    assert!(output.status.success(), "asgard 应以 0 退出，实际 {:?}\nstdout:\n{stdout}\nstderr:\n{stderr}", output.status);
    stdout
}

/// 剥离 `#` 行注释，只保留代码文本；配置形态检查针对作者实际书写的代码。
fn code_only(source: &str) -> String {
    source.lines().filter(|line| !line.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n")
}

/// 收集输出目录内文件清单（相对输出根，`/` 分隔，排序）。
fn artifact_tree(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files);
    files.sort();
    files
}

fn collect_files(root: &Path, dir: &Path, out: &mut Vec<String>) {
    for entry in fs::read_dir(dir).expect("读取输出目录失败") {
        let path = entry.expect("目录项").path();
        if path.is_dir() {
            collect_files(root, &path, out);
        }
        else {
            out.push(path.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/"));
        }
    }
}

#[test]
fn webview_example_is_single_entry_legion_project() {
    let project = example_project();
    for required in ["legion.von", "asgard.config.v"] {
        let path = project.join(required);
        assert!(path.is_file(), "示例工程缺少 `{required}`: {}", path.display());
    }
    let manifest = fs::read_to_string(project.join("legion.von")).expect("读取 legion.von 失败");
    assert!(manifest.contains("asgard-webview"), "legion.von 应声明工程身份:\n{manifest}");

    // 入口唯一：配置声明的 `source/index.w` 之外不得存在平行入口页。
    let source_dir = project.join("source");
    let mut entries = Vec::new();
    for entry in fs::read_dir(&source_dir).expect("读取 source 目录失败") {
        entries.push(entry.expect("目录项").path().to_string_lossy().replace('\\', "/"));
    }
    assert_eq!(entries.len(), 1, "WebView 示例应只有一个入口文件: {entries:?}");

    let raw = fs::read_to_string(project.join("asgard.config.v")).expect("读取 asgard.config.v 失败");
    let config = code_only(&raw);
    assert!(config.contains("define_config({"), "应以规范作者形态书写:\n{config}");
    assert!(!config.contains("define_config(asgard)"), "旧文本 DSL 不得保留:\n{config}");
    for forbidden in ["Some(", "None", "ArtifactTarget", "AsgardConfig {"] {
        assert!(!config.contains(forbidden), "配置不得引用未冻结 SDK 拼写 `{forbidden}`:\n{config}");
    }
    assert!(config.contains(&format!("entry: \"{CONFIGURED_ENTRY}\"")), "配置入口应与唯一入口同源:\n{config}");
    assert!(config.contains("target: \"static\""), "首轮 target 为文本 `static`:\n{config}");
}

#[test]
fn entry_uses_framework_components_and_v_text_types() {
    let path = example_project().join("source").join("index.awsl");
    let source = fs::read_to_string(&path).unwrap_or_else(|err| panic!("示例入口缺失 {}: {err}", path.display()));
    assert!(source.contains("<widget index>"), "入口应声明根组件 index:\n{source}");
    assert!(source.contains("<script>"), "入口应携带脚本状态:\n{source}");
    assert!(source.contains("<style>"), "入口应携带样式块:\n{source}");
    for html_tag in ["<div", "<h1", "<h2", "<span", "<section"] {
        assert!(!source.contains(html_tag), "跨平台组件树不应使用 HTML 标签 `{html_tag}`:\n{source}");
    }
    for uncontracted in ["<Link", "<Box", "<Flex", "<Slot"] {
        assert!(!source.contains(uncontracted), "`{uncontracted}` 不在跨平台组件合同内，示例不得使用:\n{source}");
    }
    assert!(source.contains("<Column"), "应使用跨平台框架布局组件:\n{source}");
    assert!(source.contains("<Button @click="), "应声明框架 Button 与事件绑定:\n{source}");
    // 脚本状态使用 V 文本类型 `utf8`，不写臆造的 `string` 类型。
    assert!(!source.contains(": string"), "V 文本类型是 `utf8`，不存在 `string`:\n{source}");
    assert!(source.contains(": utf8"), "脚本状态应使用 `utf8`:\n{source}");
}

#[test]
fn webview_entry_builds_complete_artifact_tree() {
    // 当前唯一可执行交付：浏览器静态交付线。断言产物树形态与页面壳合同；
    // 组件级渲染形态不作为断言对象（见文件头证据边界）。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let dist = project.join("dist");
    // 输出目录显式指定为临时工程内的 `dist`，不依赖 CLI 默认的 `.cache/asgard`。
    let output = asgard(&["build", "--static", CONFIGURED_ENTRY, project.to_str().expect("UTF-8 路径"), "--output", dist.to_str().expect("UTF-8 路径")]);
    let stdout = expect_success(&output);
    assert!(stdout.contains("asgard build (static) complete"), "stdout 应有完成行:\n{stdout}");

    // CSS 产物名取入口页 stem（`index.css`），chunk 产物名取工程名。
    let expected = [
        "asgard-webview.mjs",
        "asgard-webview.wasm",
        "asgard-webview.wat",
        "boot.js",
        "c/index.js",
        "index.css",
        "index.html",
        "manifest.json",
    ]
    .map(str::to_string);
    assert_eq!(artifact_tree(&dist), expected, "WebView 示例应产出完整静态交付链");
    let html = fs::read_to_string(dist.join("index.html")).expect("读取 index.html 失败");
    assert!(html.contains("<!DOCTYPE html>"), "应为完整 HTML 页面:\n{html}");
    assert!(html.contains("<meta charset=\"utf-8\">"), "页面壳应声明字符集:\n{html}");
    assert!(!html.contains("<base"), "根部署页面壳不输出 `<base>`（ssg 合同：base `/` 省略标签；配置驱动 base 属缺口）:\n{html}");
    assert!(html.contains("<title>index</title>"), "标题缺省应回退根组件名:\n{html}");
    assert!(html.contains(r#"<script src="boot.js"></script>"#), "页面应引用 boot 启动脚本:\n{html}");
    assert!(html.contains("data-component=\"index\""), "页面应声明岛挂载点:\n{html}");
}

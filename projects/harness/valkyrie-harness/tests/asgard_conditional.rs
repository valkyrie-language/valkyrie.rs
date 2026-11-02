//! `projects/examples/asgard-static-conditional` 的黑盒验收：条件渲染、字符串聚合
//! 与事件绑定形态的正式 Legion 工程结构 + 当前静态交付线的产物证据 + 错误路径。
//!
//! 复现命令（自示例目录出发，CLI 入口经 `valkyrie_harness::asgard_cli` 解析）：
//!
//! ```text
//! node <nyar-vm.rs>/projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs \
//!   build --static source/index.awsl <examples/asgard-static-conditional>
//! ```
//!
//! 证据边界（重要，勿以绿灯掩盖）：
//! - 本示例覆盖的 `<if>`/`<else>` 条件构造当前越出 SSG 准入子集，构建按准入门
//!   结构化失败；该缺口以交付合同矩阵 `conditional-aggregation-shape` blocked 条目
//!   记录（含坐标），本文件不把构建失败当作测试绿灯，也不把缺口写成通过。
//! - `each`/`<loop>` 列表形态同样越出准入子集（结构化拒绝，非崩溃），见合同矩阵
//!   `list-each-shape` blocked 条目。
//! - 入口根组件名 ↔ 文件 stem 对应是 CLI 实际执行的错误路径合同，本文件双向断言；
//!   准入子集内的入口必须真实构建出完整交付链（页面 + chunk + glue + manifest）。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::{asgard_cli, asgard_workspace::materialize_example_workspace};

/// 条件形态示例工程根。
fn example_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/asgard-static-conditional")
}

/// 配置声明的入口（相对项目根）；结构断言与构建命令共用同一常量。
/// `.awsl` 为规范拼写（03 §2，设计仓 `54d13b9`），`.w` 同语义仍可接受
/// （同义用例见 `asgard_static`），不把任一后缀写成唯一合法形态。
const CONFIGURED_ENTRY: &str = "source/index.awsl";

fn copy_example_into(dest: &Path) -> PathBuf {
    materialize_example_workspace("asgard-static-conditional", dest)
}

fn asgard(args: &[&str]) -> Output {
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    asgard_cli::run(&owned).expect("无法启动 asgard CLI（node 或 bin/asgard.mjs 缺失）")
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

fn expect_failure(output: &Output, needle: &str) -> String {
    let stderr = stderr_of(output);
    assert!(!output.status.success(), "asgard 应非 0 退出（needle `{needle}`）\nstdout:\n{}\nstderr:\n{stderr}", stdout_of(output));
    assert!(stderr.contains(needle), "stderr 应包含 `{needle}`:\n{stderr}");
    stderr
}

/// 剥离 `#` 行注释，只保留代码文本。
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
fn conditional_example_is_a_legal_legion_project() {
    let project = example_project();
    for required in ["legion.von", "asgard.config.v"] {
        let path = project.join(required);
        assert!(path.is_file(), "示例工程缺少 `{required}`: {}", path.display());
    }
    let manifest = fs::read_to_string(project.join("legion.von")).expect("读取 legion.von 失败");
    assert!(manifest.contains("asgard-static-conditional"), "legion.von 应声明工程身份:\n{manifest}");

    let raw = fs::read_to_string(project.join("asgard.config.v")).expect("读取 asgard.config.v 失败");
    let config = code_only(&raw);
    assert!(config.contains("define_config({"), "应以规范作者形态书写:\n{config}");
    assert!(!config.contains("define_config(asgard)"), "旧文本 DSL 不得保留:\n{config}");
    for forbidden in ["Some(", "None", "ArtifactTarget", "AsgardConfig {"] {
        assert!(!config.contains(forbidden), "配置不得引用未冻结 SDK 拼写 `{forbidden}`:\n{config}");
    }
    assert!(config.contains("target: \"static\""), "首轮 target 为文本 `static`:\n{config}");
    assert!(config.contains(&format!("entry: \"{CONFIGURED_ENTRY}\"")), "配置入口应与验收入口同源:\n{config}");
}

#[test]
fn entry_shape_uses_contract_components_with_control_flow() {
    // 组件树形状合同：只用 03 §11 跨平台组件（Column/Row/Text/Button）表达条件、
    // 聚合与事件声明；不含 HTML 标签、未合同组件或崩溃中的 `each` 列表形态。
    let path = example_project().join("source").join("index.awsl");
    let source = fs::read_to_string(&path).unwrap_or_else(|err| panic!("示例入口缺失 {}: {err}", path.display()));
    assert!(source.contains("<widget index>"), "入口应声明根组件 index:\n{source}");
    assert!(source.contains("<if condition="), "应声明条件渲染形态:\n{source}");
    assert!(source.contains("<else>"), "应声明条件 else 分支:\n{source}");
    assert!(source.contains("{title} — {status}"), "应声明字符串聚合插值:\n{source}");
    assert!(source.contains("<Button @click="), "应声明事件绑定:\n{source}");
    assert!(source.contains("micro on_toggle()"), "事件 handler 应为 V `micro` 函数:\n{source}");
    assert!(source.contains(": utf8"), "脚本状态应使用 `utf8`:\n{source}");
    assert!(!source.contains(": string"), "V 不存在 `string` 类型:\n{source}");
    assert!(!source.contains("each="), "`each` 列表形态越出 SSG 准入子集（见合同矩阵 `list-each-shape`），不得进入可构建示例:\n{source}");
    for html_tag in ["<div", "<h1", "<span", "<section"] {
        assert!(!source.contains(html_tag), "跨平台组件树不应使用 HTML 标签 `{html_tag}`:\n{source}");
    }
    for uncontracted in ["<Link", "<Box", "<Flex", "<Slot", "<List", "<Item"] {
        assert!(!source.contains(uncontracted), "`{uncontracted}` 不在跨平台组件合同内，示例不得使用:\n{source}");
    }
}

#[test]
fn widget_name_must_match_entry_file_stem() {
    // 错误路径：入口根组件名与文件 stem 不一致必须结构化拒绝。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    fs::write(project.join("source").join("mismatch.awsl"), "<widget other>\n<Text>x</Text>\n</widget>\n").expect("写入错配入口失败");
    let output = asgard(&["build", "--static", "source/mismatch.awsl", project.to_str().expect("UTF-8 路径")]);
    let stderr = expect_failure(&output, "does not match file stem");
    assert!(stderr.contains("mismatch"), "诊断应点名错配文件:\n{stderr}");
}

#[test]
fn matching_widget_stem_builds_under_component_name() {
    // 正向对应：stem 与根组件名一致的入口按该名产出页面与组件胶水
    //（入口↔组件↔产物对应合同在真实交付链上成立）。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    fs::remove_file(project.join("source").join("index.awsl")).expect("移除默认入口失败");
    fs::write(
        project.join("source").join("panel.awsl"),
        "<widget panel>\n<Column><Text>{label}</Text></Column>\n</widget>\n<script>\nlet label: utf8 = \"Panel\"\n</script>\n",
    )
    .expect("写入 panel 入口失败");
    let stdout = expect_success(&asgard(&["build", "--static", "source/panel.awsl", project.to_str().expect("UTF-8 路径")]));
    assert!(stdout.contains("component: panel"), "组件名应取根组件:\n{stdout}");

    let dist = project.join("dist");
    let tree = artifact_tree(&dist);
    for required in ["index.html", "manifest.json", "boot.js", "c/panel.js"] {
        assert!(tree.iter().any(|entry| entry == required), "产物树应含 `{required}`: {tree:?}");
    }
    let html = fs::read_to_string(dist.join("index.html")).expect("读取页面失败");
    assert!(html.contains("<title>panel</title>"), "标题缺省应回退根组件名 panel:\n{html}");
    assert!(html.contains("data-component=\"panel\""), "页面应声明 panel 岛挂载点:\n{html}");
}

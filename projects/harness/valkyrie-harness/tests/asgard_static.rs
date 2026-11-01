//! `projects/examples/asgard-static` 的黑盒验收：正式工程合同的结构断言 +
//! 当前交付线（`asgard build --static`）的真实 CLI 执行证据 + 配置驱动链路的
//! 缺口证据。两条证据面分开，不互相冒充。
//!
//! 复现命令（自示例目录出发，CLI 入口经 `valkyrie_harness::asgard_cli` 解析）：
//!
//! ```text
//! node <nyar-vm.rs>/projects/nyar.ts/emulators/valkyrie/bin/asgard.mjs \
//!   build --static source/index.w <examples/asgard-static>
//! ```
//!
//! 证据边界（重要，勿以绿灯掩盖）：
//! - 当前静态交付只产出 `index.html` 预渲染页面；页面内容（组件→DOM 映射、插值、
//!   样式透传、`asgard-static-*` 作用域前缀）全部由 `nyar-vm.rs` 的 Rust 内置组件
//!   语义产生（`awsl/render_ir/surface.rs::is_intrinsic_tag` 名称表、
//!   `codegen/html_render.rs::map_html_tag`、`codegen/expr_eval.rs` 文本求值），
//!   不是 `asgard.ui` V 库经 OVM 执行的产物。因此本文件只断言产物完整性、页面壳
//!   （doctype/charset/viewport/title；根基址 `/` 时 `ssg` 按合同不输出 `<base>`，
//!   见 `nyar-vm.rs/.../asgard/ssg/mod.rs::generate_static_page_with_base`）与
//!   准入/拒绝合同，不断言组件渲染形态——后者属于待迁移
//!   实现细节，固化成断言会把 Rust 特判锁成合同。
//! - 入口/输出/基址当前来自 CLI 裸参数与硬编码默认（`--static <entry>`、缺省 `dist`、
//!   `base` 固定 `/`），`asgard.config.v` 尚未被求值消费；配置驱动构建
//!   （`asgard build <project>`）以结构化失败闭合，本文件把该失败作为缺口证据断言。
//! - `--static` 路径经 `asgardBuildStatic` 直达 `build_static_site`，未执行
//!   `legion.von`/`asgard.config.v` 准入（准入只在 Rust `run_build` 生效）；缺 manifest/
//!   配置的拒绝合同当前在黑盒面不成立，作为 blocked 记录于 `asgard_delivery_contract`，
//!   不以断言固化该绕过。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::asgard_cli;

/// 静态交付示例工程根（自包含，不依赖 `valkyrie.v` submodule）。
fn example_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/asgard-static")
}

/// 配置声明的入口（相对项目根）；结构断言与构建命令共用同一常量，防止二者偶然一致。
const CONFIGURED_ENTRY: &str = "source/index.w";

/// 把示例工程复制进目标目录并返回副本根（构建只写临时目录，不落仓库）。
fn copy_example_into(dest: &Path) -> PathBuf {
    let target = dest.join("asgard-static");
    copy_dir(&example_project(), &target);
    target
}

fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("创建目标目录失败");
    for entry in fs::read_dir(from).expect("读取示例目录失败") {
        let entry = entry.expect("目录项");
        let path = entry.path();
        let target = to.join(entry.file_name());
        if path.is_dir() {
            copy_dir(&path, &target);
        }
        else {
            fs::copy(&path, &target).expect("复制示例文件失败");
        }
    }
}

fn asgard(args: &[&str]) -> Output {
    let owned: Vec<&OsStr> = args.iter().map(OsStr::new).collect();
    asgard_cli::run(&owned).expect("无法启动 asgard CLI（node 或 bin/asgard.mjs 缺失；见 valkyrie_harness::asgard_cli）")
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

/// 剥离 `#` 行注释，只保留代码文本；配置形态检查针对作者实际书写的代码，
/// 注释中的合同说明文字（如被禁止的旧 DSL 拼写）不应触发 forbidden 检查。
fn code_only(source: &str) -> String {
    source.lines().filter(|line| !line.trim_start().starts_with('#')).collect::<Vec<_>>().join("\n")
}

/// 提取 stdout 合同字段值（如 `component:` / `output:` / `page:` 行）。
fn stdout_field(stdout: &str, key: &str) -> String {
    let prefix = format!("{key}: ");
    stdout
        .lines()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("stdout 应含 `{prefix}` 合同行:\n{stdout}"))
        .to_string()
}

fn canonical(path: &str) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|err| panic!("规范化路径失败 {path}: {err}"))
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

/// 以配置声明的入口执行当前唯一可用的静态交付命令（不带任何覆盖 flag）。
fn build_static(project: &Path) -> Output {
    asgard(&["build", "--static", CONFIGURED_ENTRY, project.to_str().expect("UTF-8 路径")])
}

#[test]
fn example_fixture_is_a_legal_legion_project() {
    // 正向示例必须是合法 Legion 项目并具备真实 Asgard 配置（03 §4：二者不可替代）。
    let project = example_project();
    for required in ["legion.von", "asgard.config.v"] {
        let path = project.join(required);
        assert!(path.is_file(), "示例工程缺少 `{required}`: {}", path.display());
    }
    let manifest = fs::read_to_string(project.join("legion.von")).expect("读取 legion.von 失败");
    assert!(manifest.contains("asgard-static"), "legion.von 应声明工程身份:\n{manifest}");
}

#[test]
fn config_uses_canonical_define_config_shape() {
    // 03 §4：作者形态是 `define_config({...})`（或末尾直接 Von 字面量），
    // 字段与取值形状对齐 `resolve_static_config` 的结构合同；旧文本 DSL 与
    // 臆造的 Option/枚举包装不得出现。
    let path = example_project().join("asgard.config.v");
    let raw = fs::read_to_string(&path).unwrap_or_else(|err| panic!("配置缺失 {}: {err}", path.display()));
    let config = code_only(&raw);
    assert!(config.contains("define_config({"), "应以规范作者形态 `define_config({{...}})` 书写:\n{config}");
    assert!(!config.contains("define_config(asgard)"), "旧 `define_config(asgard) {{}}` 文本 DSL 不得保留:\n{config}");
    for forbidden in ["Some(", "None", "ArtifactTarget", "AsgardConfig {"] {
        assert!(!config.contains(forbidden), "配置不得引用未冻结 SDK 拼写 `{forbidden}`:\n{config}");
    }
    for field in ["artifact", "target", "entry", "output", "base"] {
        assert!(config.contains(field), "配置应声明结构合同字段 `{field}`:\n{config}");
    }
    assert!(config.contains("target: \"static\""), "首轮 target 取值为文本 `static`:\n{config}");
    assert!(config.contains(&format!("entry: \"{CONFIGURED_ENTRY}\"")), "配置入口应与验收入口同源:\n{config}");
    assert!(config.contains("output: \"dist\""), "输出目录显式声明:\n{config}");
    assert!(config.contains("base: \"/\""), "基址显式声明:\n{config}");
}

#[test]
fn entry_uses_framework_components_only() {
    // 03 §2：跨平台正向页面以框架组件表达，不以 HTML 标签为基础组件树；
    // 未进入框架合同表面的组件（如 `Link`/`Box` 这类仅 DOM adapter 认识的标签）
    // 不得出现在示例里。
    let entry = example_project().join("source").join("index.w");
    let source = fs::read_to_string(&entry).unwrap_or_else(|err| panic!("示例入口缺失 {}: {err}", entry.display()));
    assert!(source.contains("<widget index>"), "示例入口应声明根组件 index:\n{source}");
    assert!(source.contains("<style>"), "示例入口应携带样式块:\n{source}");
    for html_tag in ["<div", "<h1", "<h2", "<span", "<section"] {
        assert!(!source.contains(html_tag), "跨平台组件树不应使用 HTML 标签 `{html_tag}`:\n{source}");
    }
    assert!(source.contains("<Column"), "示例入口应使用跨平台框架布局组件:\n{source}");
    for uncontracted in ["<Link", "<Box", "<Flex", "<Slot"] {
        assert!(!source.contains(uncontracted), "`{uncontracted}` 不在跨平台组件合同内，示例不得使用:\n{source}");
    }
}

#[test]
fn static_build_emits_complete_single_page_artifact() {
    // 当前交付合同的产物完整性：产物树恰为一个 `index.html`，无 loader/manifest/wasm
    // （JS/Wasm 交付链缺失属于缺口，见文件头与 asgard_delivery_contract 矩阵）。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let stdout = expect_success(&build_static(&project));

    assert!(stdout.contains("asgard build (static) complete"), "stdout 应有完成行:\n{stdout}");
    assert_eq!(stdout_field(&stdout, "component"), "index");
    let dist = project.join("dist");
    assert_eq!(canonical(&stdout_field(&stdout, "output")), canonical(dist.to_str().expect("UTF-8 路径")));
    assert_eq!(canonical(&stdout_field(&stdout, "page")), canonical(dist.join("index.html").to_str().expect("UTF-8 路径")));
    assert_eq!(artifact_tree(&dist), vec!["index.html".to_string()], "静态产物树应只有 index.html");

    // 页面壳合同（与组件渲染语义无关，可固化）：完整文档、charset/viewport、标题回退根组件名。
    // 基址：当前构建的 base 由 N-API 硬编码 `/`，`ssg::generate_static_page_with_base`
    // 对根部署按合同不输出 `<base>`；配置驱动的 `artifact.base` 消费属缺口（见交付合同矩阵
    // `mime-and-base-subdir`），故此处断言"根页面壳无 base 标签"，而非伪造 `<base href="/">`。
    let html = fs::read_to_string(dist.join("index.html")).expect("读取 index.html 失败");
    assert!(html.contains("<!DOCTYPE html>"), "应为完整 HTML 页面:\n{html}");
    assert!(html.contains("<meta charset=\"utf-8\">"), "页面壳应声明字符集:\n{html}");
    assert!(!html.contains("<base"), "根部署页面壳不输出 `<base>`（ssg 合同：base `/` 省略标签）:\n{html}");
    assert!(html.contains("<title>index</title>"), "标题缺省应回退根组件名:\n{html}");
    assert!(!html.contains("<script"), "当前静态产物不携带 runtime script（glue/Wasm 链未建立）:\n{html}");
}

#[test]
fn awsl_entry_builds_identical_page_to_w() {
    // `.w` 与 `.awsl` 同格式同语义（03 §2）：同内容入口应产出逐字节相同的页面。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let source = fs::read(example_project().join("source").join("index.w")).expect("读取示例入口失败");

    expect_success(&build_static(&project));
    let w_page = fs::read(project.join("dist").join("index.html")).expect("读取 .w 页面失败");

    fs::write(project.join("source").join("index.awsl"), &source).expect("写入 .awsl 入口失败");
    let stdout = expect_success(&asgard(&["build", "--static", "source/index.awsl", project.to_str().expect("UTF-8 路径")]));
    assert_eq!(stdout_field(&stdout, "component"), "index");
    let awsl_page = fs::read(project.join("dist").join("index.html")).expect("读取 .awsl 页面失败");
    assert_eq!(w_page, awsl_page, ".w 与 .awsl 入口产物应逐字节相同");
    assert_eq!(artifact_tree(&project.join("dist")), vec!["index.html".to_string()]);
}

#[test]
fn rebuild_is_deterministic_and_leaves_no_strays() {
    // 重复构建确定：同一输入两次构建产物树不变、页面字节一致（相同输入 manifest 确定的
    // 最小可验证面；chunk/manifest 维度见交付合同矩阵）。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    expect_success(&build_static(&project));
    let first = fs::read(project.join("dist").join("index.html")).expect("读取第一次页面失败");
    expect_success(&build_static(&project));
    let second = fs::read(project.join("dist").join("index.html")).expect("读取第二次页面失败");
    assert_eq!(first, second, "同输入重建应逐字节确定");
    assert_eq!(artifact_tree(&project.join("dist")), vec!["index.html".to_string()]);
}

#[test]
fn static_build_requires_explicit_entry() {
    let output = asgard(&["build", "--static"]);
    expect_failure(&output, "需要显式入口");
}

#[test]
fn static_build_rejects_unknown_option() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", "--static", CONFIGURED_ENTRY, "--bogus", project.to_str().expect("UTF-8 路径")]);
    expect_failure(&output, "未知选项");
}

#[test]
fn static_build_rejects_unknown_entry_extension() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let source = fs::read(example_project().join("source").join("index.w")).expect("读取示例入口失败");
    fs::write(project.join("source").join("index.v"), source).expect("写入 .v 入口失败");
    let output = asgard(&["build", "--static", "source/index.v", project.to_str().expect("UTF-8 路径")]);
    let stderr = expect_failure(&output, "静态站点入口必须是");
    assert!(stderr.contains(".w") && stderr.contains(".awsl"), "诊断应说明两种后缀同语义:\n{stderr}");
}

#[test]
fn static_build_rejects_missing_entry() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", "--static", "source/missing.w", project.to_str().expect("UTF-8 路径")]);
    expect_failure(&output, "missing.w");
}

#[test]
fn static_build_rejects_entry_escaping_project() {
    // 入口真实存在于项目根之外：越界检查必须先于读取失败闭合。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let source = fs::read(example_project().join("source").join("index.w")).expect("读取示例入口失败");
    fs::write(temp.path().join("outside.awsl"), source).expect("写入越界入口失败");
    let output = asgard(&["build", "--static", "../outside.awsl", project.to_str().expect("UTF-8 路径")]);
    expect_failure(&output, "越出项目根");
}

#[test]
fn config_driven_build_fails_closed_as_gap_evidence() {
    // 正式合同下 `asgard build <project>` 应读配置产出完整 static 产物（03 §6）；
    // 配置求值（C0/C2）未接入，当前必须结构化失败，不得静默回退裸参数路径。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", project.to_str().expect("UTF-8 路径")]);
    let stderr = expect_failure(&output, "当前可用入口");
    assert!(stderr.contains("配置求值"), "诊断应说明配置求值链路未闭合:\n{stderr}");
}

#[test]
fn unwired_commands_fail_closed() {
    for command in ["serve", "dev", "pack", "fmt"] {
        let output = asgard(&[command]);
        expect_failure(&output, "尚未接入");
    }
}

#[test]
fn unknown_command_fails_closed() {
    let output = asgard(&["no-such-command"]);
    expect_failure(&output, "unknown asgard command");
}

#[test]
fn meta_commands_succeed() {
    let version = expect_success(&asgard(&["version"]));
    assert!(version.trim().starts_with("@nyar/valkyrie "), "version 应打印包名与版本:\n{version}");
    expect_success(&asgard(&["help"]));
}

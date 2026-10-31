//! `asgard build --static` 示例产物黑盒交互测试：以真实 CLI 路径（`node` +
//! `bin/asgard.mjs` + `valkyrie-napi`）驱动 `projects/examples/asgard-static`，
//! 断言退出码、stdout 合同行、产物树、页面内容与失败闭合诊断。
//!
//! 当前静态交付线是纯预渲染——兄弟仓产物树合同测试固化其形态：产物仅
//! `index.html`，无 runtime script、无任何 `src`/`href` 外链。页面因此不存在
//! 浏览器侧交互面，本文件不为此引入 jsdom；「交互」锚定在 CLI 参数面
//! （flag 消费、未知选项、越界入口、未接线命令）与产物合同核验。

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

#[test]
fn example_fixture_is_self_contained() {
    // 示例必须自带 `.w` 入口与根组件声明，且不依赖 submodule 语料。
    let entry = example_project().join("source").join("index.w");
    let source = fs::read_to_string(&entry).unwrap_or_else(|err| panic!("示例入口缺失 {}: {err}", entry.display()));
    assert!(source.contains("<widget index>"), "示例入口应声明根组件 index:\n{source}");
    assert!(source.contains("<style>"), "示例入口应携带样式块:\n{source}");
}

#[test]
fn static_build_emits_self_contained_page() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", "--static", "source/index.w", "--title", "静态示例", project.to_str().expect("UTF-8 路径")]);
    let stdout = expect_success(&output);

    assert!(stdout.contains("asgard build (static) complete"), "stdout 应有完成行:\n{stdout}");
    assert_eq!(stdout_field(&stdout, "component"), "index");
    let dist = project.join("dist");
    assert_eq!(canonical(&stdout_field(&stdout, "output")), canonical(dist.to_str().expect("UTF-8 路径")));
    assert_eq!(canonical(&stdout_field(&stdout, "page")), canonical(dist.join("index.html").to_str().expect("UTF-8 路径")));

    // 当前静态交付合同：产物树只有 index.html（兄弟仓 static_artifact_tree 同型锁定）。
    assert_eq!(artifact_tree(&dist), vec!["index.html".to_string()], "静态产物树应只有 index.html");

    let html = fs::read_to_string(dist.join("index.html")).expect("读取 index.html 失败");
    assert!(html.contains("<!DOCTYPE html>"), "应为完整 HTML 页面:\n{html}");
    assert!(html.contains("<title>静态示例</title>"), "--title 应写入页面:\n{html}");
    assert!(html.contains("asgard-static-index"), "应包含作用域 class:\n{html}");
    assert!(html.contains(".hero { color: #c67b34; }"), "应透传组件样式文本:\n{html}");
    assert!(html.contains("<div class=\"hero\"><h1>你好，静态世界</h1></div>"), "应包含初始状态插值结果:\n{html}");
    assert!(!html.contains("<script"), "静态页面不应包含 runtime script:\n{html}");
    assert!(!html.contains("src="), "页面应自包含、无外链 src:\n{html}");
    assert!(!html.contains("href="), "页面应自包含、无外链 href:\n{html}");
}

#[test]
fn awsl_entry_builds_identical_page_to_w() {
    // `.w` 与 `.awsl` 同格式同语义：同内容入口应产出逐字节相同的页面。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let source = fs::read(example_project().join("source").join("index.w")).expect("读取示例入口失败");
    fs::write(project.join("source").join("index.awsl"), &source).expect("写入 .awsl 入口失败");

    // `-o` 传给 native 的是原样路径，相对值会落在进程 cwd 而非项目根，故显式给绝对路径。
    let dist_w = project.join("dist-w");
    let dist_awsl = project.join("dist-awsl");
    let w_stdout = expect_success(&asgard(&["build", "--static", "source/index.w", "-o", dist_w.to_str().expect("UTF-8 路径"), project.to_str().expect("UTF-8 路径")]));
    let awsl_stdout = expect_success(&asgard(&["build", "--static", "source/index.awsl", "-o", dist_awsl.to_str().expect("UTF-8 路径"), project.to_str().expect("UTF-8 路径")]));
    assert_eq!(stdout_field(&w_stdout, "component"), "index");
    assert_eq!(stdout_field(&awsl_stdout, "component"), "index");

    let w_page = fs::read(dist_w.join("index.html")).expect("读取 .w 页面失败");
    let awsl_page = fs::read(dist_awsl.join("index.html")).expect("读取 .awsl 页面失败");
    assert_eq!(w_page, awsl_page, ".w 与 .awsl 入口产物应逐字节相同");
}

#[test]
fn default_title_falls_back_to_component_name() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    expect_success(&asgard(&["build", "--static", "source/index.w", project.to_str().expect("UTF-8 路径")]));
    let html = fs::read_to_string(project.join("dist").join("index.html")).expect("读取 index.html 失败");
    assert!(html.contains("<title>index</title>"), "缺省标题应为根组件名:\n{html}");
}

#[test]
fn explicit_output_dir_is_honored() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let site_out = project.join("site-out");
    let output = asgard(&["build", "--static", "source/index.w", "--output", site_out.to_str().expect("UTF-8 路径"), project.to_str().expect("UTF-8 路径")]);
    let stdout = expect_success(&output);
    assert_eq!(canonical(&stdout_field(&stdout, "output")), canonical(site_out.to_str().expect("UTF-8 路径")));
    assert_eq!(artifact_tree(&site_out), vec!["index.html".to_string()], "-o 目录应只落 index.html");
    assert!(!site_out.join("..").join("dist").exists(), "显式 -o 后不应再创建默认 dist/");
}

#[test]
fn rebuild_refreshes_page() {
    // 重复构建应成功并刷新页面内容（当前实现不清理输出目录，只覆写 index.html）。
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    expect_success(&asgard(&["build", "--static", "source/index.w", "--title", "第一次", project.to_str().expect("UTF-8 路径")]));
    expect_success(&asgard(&["build", "--static", "source/index.w", "--title", "第二次", project.to_str().expect("UTF-8 路径")]));
    let html = fs::read_to_string(project.join("dist").join("index.html")).expect("读取 index.html 失败");
    assert!(html.contains("<title>第二次</title>"), "重建应覆写页面标题:\n{html}");
    assert!(!html.contains("第一次"), "旧标题不应残留:\n{html}");
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
    let output = asgard(&["build", "--static", "source/index.w", "--bogus", project.to_str().expect("UTF-8 路径")]);
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
fn build_without_static_fails_closed() {
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    let output = asgard(&["build", project.to_str().expect("UTF-8 路径")]);
    let stderr = expect_failure(&output, "当前可用入口");
    assert!(stderr.contains("asgard build --static"), "诊断应指向唯一可用入口:\n{stderr}");
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

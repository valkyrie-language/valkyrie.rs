//! `projects/examples/asgard-webview` 的桌面 WebView 示例语料黑盒测试。
//!
//! 这些示例是同一套 Asgard DOM 组件的桌面宿主语料：当前静态交付线只验证
//! 自包含 `index.html` 的初始状态，交互行为与桌面壳自动化仍由后续浏览器/
//! WebView 验收门承担。示例不依赖 `valkyrie.v` submodule。

use std::{
    ffi::OsStr,
    fs,
    path::{Path, PathBuf},
    process::Output,
};

use tempfile::TempDir;
use valkyrie_harness::asgard_cli;

fn example_project() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/asgard-webview")
}

fn copy_example_into(dest: &Path) -> PathBuf {
    let target = dest.join("asgard-webview");
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

fn stdout_field(stdout: &str, key: &str) -> String {
    let prefix = format!("{key}: ");
    stdout.lines().find_map(|line| line.strip_prefix(&prefix)).unwrap_or_else(|| panic!("stdout 应含 `{prefix}` 合同行:\n{stdout}")).to_string()
}

/// 三个示例入口都必须自带根组件、脚本状态和样式，且不引用外部语料。
#[test]
fn webview_examples_are_self_contained() {
    for (entry, widget) in [("index.w", "index"), ("settings.w", "settings"), ("dashboard.w", "dashboard")] {
        let path = example_project().join("source").join(entry);
        let source = fs::read_to_string(&path).unwrap_or_else(|err| panic!("示例入口缺失 {}: {err}", path.display()));
        assert!(source.contains(&format!("<widget {widget}>")), "{entry} 应声明根组件 {widget}:\n{source}");
        assert!(source.contains("<script>"), "{entry} 应携带脚本状态:\n{source}");
        assert!(source.contains("<style>"), "{entry} 应携带样式块:\n{source}");
    }
}

/// 每个 WebView 示例都能经真实 CLI 静态渲染为自包含页面，并保留初始状态。
#[test]
fn webview_examples_build_static_pages() {
    let cases = [
        ("index.w", "index", "Valkyrie Desk", ".desk { padding: 24px; }"),
        ("settings.w", "settings", "Theme: light", ".settings { padding: 24px; }"),
        ("dashboard.w", "dashboard", "Workspace status", ".dashboard { padding: 24px; }"),
    ];
    let temp = TempDir::new().expect("创建临时目录失败");
    let project = copy_example_into(temp.path());
    for (entry, component, marker, css) in cases {
        let output_dir = project.join(format!("dist-{component}"));
        let output = asgard(&[
            "build",
            "--static",
            &format!("source/{entry}"),
            "--output",
            output_dir.to_str().expect("UTF-8 路径"),
            project.to_str().expect("UTF-8 路径"),
        ]);
        let stdout = expect_success(&output);
        assert_eq!(stdout_field(&stdout, "component"), component, "{entry} 的组件名应来自入口 stem");
        let page = fs::read_to_string(output_dir.join("index.html")).expect("读取 index.html 失败");
        assert!(page.contains(marker), "{entry} 应渲染初始状态 `{marker}`:\n{page}");
        assert!(page.contains(css), "{entry} 应透传组件样式:\n{page}");
        assert!(!page.contains("<script"), "{entry} 当前静态产物不应携带 runtime script:\n{page}");
    }
}

//! 终端 TUI 打包：纯 V 源码（app.v）+ 原生可执行文件。
//!
//! 终端路径是完全纯 Valkyrie 实现：所有 TUI 业务逻辑（widget 树构建、绑定存储、
//! 焦点导航、渲染调度、事件循环）由 V 源码承担，宿主只提供字符级 I/O 原语
//!（`host_contract`）。本模块将 AWSL 降低的 V 源码与 tui.v 运行时合并为 `app.v`，
//! 并生成原生可执行文件骨架（V→native 编译器补全后接入）。

use crate::{awsl::LoweredComponent, codegen::build_awsl_terminal_source, package::artifact::PackageArtifact};
use std_data::binary::host_exe::build_host_executable;

/// 终端打包产物。
#[derive(Debug, Clone)]
pub struct TerminalPackageOutput {
    /// 平台标识（固定为 "terminal"）。
    pub platform: String,
    /// 相对路径 → 产物。
    pub files: Vec<(String, PackageArtifact)>,
}

/// 生成终端 TUI dist（纯 V 源码 app.v + 原生可执行文件）。
///
/// 与桌面打包不同，终端打包不调用 `embed_host_native_section` / `embed_asgard_ui_section`：
/// UI 逻辑全部在 `app.v`（V 源码）中，native_logic 通过 `build_host_executable` 的
/// rodata_tail 参数编入可执行文件（V→native 编译器补全后生效）。
pub fn package_terminal_project(components: &[LoweredComponent], module_name: &str, native_logic: &[u8]) -> TerminalPackageOutput {
    let host_platform = std::env::consts::OS;
    let exe_name = if host_platform == "windows" { format!("{module_name}.exe") } else { module_name.to_string() };
    let app_v = build_awsl_terminal_source(components);
    let executable = build_host_executable(host_platform, None, native_logic).expect("终端宿主可执行构建失败");
    let manifest = format!("# Asgard terminal manifest\nplatform: terminal\nname: {module_name}\nexecutable: {exe_name}\nsource: app.v\n");
    let files = vec![
        ("manifest.toml".into(), PackageArtifact::Text(manifest)),
        (exe_name, PackageArtifact::Bytes(executable)),
        ("app.v".into(), PackageArtifact::Text(app_v)),
    ];
    TerminalPackageOutput { platform: "terminal".into(), files }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::awsl::{LoweringOptions, lower_component};
    use std_data::text::awsl::AwslParser;

    fn demo_component() -> LoweredComponent {
        let source = r#"<widget Demo><text>hi</text></widget>"#;
        let root = AwslParser::parse_root(source).expect("parse");
        lower_component(&root, "demo", "demo.awsl", &LoweringOptions::default())
    }

    #[test]
    fn terminal_package_has_all_artifacts() {
        let component = demo_component();
        let out = package_terminal_project(&[component], "app", &[]);
        assert!(out.files.iter().any(|(n, _)| n == "manifest.toml"), "missing manifest.toml");
        assert!(out.files.iter().any(|(n, _)| n == "app.v"), "missing app.v");
        let exe_count = out.files.iter().filter(|(n, _)| n.starts_with("app")).count();
        assert!(exe_count >= 1, "missing executable");
    }

    #[test]
    fn app_v_contains_widget_runtime() {
        let component = demo_component();
        let out = package_terminal_project(&[component], "app", &[]);
        let (_, artifact) = out.files.iter().find(|(n, _)| n == "app.v").expect("app.v");
        let text = match artifact {
            PackageArtifact::Text(text) => text,
            _ => panic!("app.v must be text"),
        };
        assert!(text.contains("structure TuiRuntime"), "app.v 必须包含 TuiRuntime: {text}");
        assert!(text.contains("widget_text"), "app.v 必须包含 widget_text 调用");
    }

    #[test]
    fn manifest_declares_terminal_platform() {
        let component = demo_component();
        let out = package_terminal_project(&[component], "app", &[]);
        let (_, artifact) = out.files.iter().find(|(n, _)| n == "manifest.toml").expect("manifest.toml");
        let text = match artifact {
            PackageArtifact::Text(text) => text,
            _ => panic!("manifest must be text"),
        };
        assert!(text.contains("platform: terminal"), "manifest must declare platform: terminal");
    }

    #[test]
    fn no_desktop_or_mobile_artifacts() {
        let component = demo_component();
        let out = package_terminal_project(&[component], "app", &[]);
        for (name, _) in &out.files {
            let lower = name.to_ascii_lowercase();
            assert!(!lower.contains("index.html"), "unexpected index.html: {name}");
            assert!(!lower.contains("classes.dex"), "unexpected classes.dex: {name}");
            assert!(!lower.contains("asgardhost"), "unexpected AsgardHost: {name}");
            assert!(!lower.contains("asgardterminalruntime.c"), "unexpected C runtime: {name}");
        }
    }

    #[test]
    fn app_v_has_host_contract_declarations() {
        let component = demo_component();
        let out = package_terminal_project(&[component], "app", &[]);
        let (_, artifact) = out.files.iter().find(|(n, _)| n == "app.v").expect("app.v");
        let text = match artifact {
            PackageArtifact::Text(text) => text,
            _ => panic!("app.v must be text"),
        };
        assert!(text.contains("[host_contract] micro clear"), "app.v 缺少 host_contract 声明");
        assert!(text.contains("[host_contract] micro put_char"), "app.v 缺少 put_char 声明");
    }

    #[test]
    fn manifest_declares_app_v_source() {
        let component = demo_component();
        let out = package_terminal_project(&[component], "app", &[]);
        let (_, artifact) = out.files.iter().find(|(n, _)| n == "manifest.toml").expect("manifest.toml");
        let text = match artifact {
            PackageArtifact::Text(text) => text,
            _ => panic!("manifest must be text"),
        };
        assert!(text.contains("source: app.v"), "manifest must declare source: app.v");
    }
}

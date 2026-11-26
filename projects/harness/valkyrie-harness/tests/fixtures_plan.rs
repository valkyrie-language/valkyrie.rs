//! 黑盒：用 `@nyar/valkyrie` 的 `legion plan` 对 `projects/fixtures` 语料验收。

use std::ffi::OsStr;

use valkyrie_harness::{fixtures, legion_cli};

#[test]
fn fixture_projects_resolve_plan_blackbox() {
    let projects = fixtures::project_dirs(&fixtures::fixtures_root());
    assert!(!projects.is_empty(), "projects/fixtures 下没有 legion.von 语料工程");

    let mut failures = Vec::new();
    for project in &projects {
        let is_bootstrap_node = project.to_string_lossy().replace('\\', "/").contains("bootstrap_node/");
        for target in ["node", "wasm", "wasm-node"] {
            // `node` 占位符结算为机器 addon 车道（本机 x86_64-pc-windows-msvc）。
            // bootstrap 工具工程的交付身份是 Node 承载的 Wasm（manifest 声明 `wasm`，
            // 产物 legion.wasm + legion.mjs），不属于 addon 车道，跳过该请求；
            // library 工程的 `--target node` 由 resolver 合成 build 项路径覆盖。
            if target == "node" && is_bootstrap_node {
                continue;
            }
            let output = legion_cli::run(&[OsStr::new("plan"), project.as_os_str(), OsStr::new("--target"), OsStr::new(target)])
                .unwrap_or_else(|err| panic!("无法启动 @nyar/valkyrie legion（请先 pnpm install 或设置 VALKYRIE_LEGION）: {err}"));

            if !output.status.success() {
                failures.push(format!(
                    "{} target={target} 退出码 {}\nstdout:\n{}\nstderr:\n{}",
                    project.display(),
                    output.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr),
                ));
            }
        }
    }

    assert!(failures.is_empty(), "{} 个语料工程无法解析 plan（黑盒）:\n{}", failures.len(), failures.join("\n\n"));
}

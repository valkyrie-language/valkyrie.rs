//! 黑盒：用 `@nyar/valkyrie` 的 `legion plan` 对 `projects/fixtures` 语料验收。

use std::ffi::OsStr;

use valkyrie_harness::{fixtures, legion_cli};

#[test]
fn fixture_projects_resolve_plan_blackbox() {
    let projects = fixtures::project_dirs(&fixtures::fixtures_root());
    assert!(!projects.is_empty(), "projects/fixtures 下没有 legion.von 语料工程");

    let mut failures = Vec::new();
    for project in &projects {
        let output = legion_cli::run(&[OsStr::new("plan"), project.as_os_str(), OsStr::new("--target"), OsStr::new("node")])
            .unwrap_or_else(|err| panic!("无法启动 @nyar/valkyrie legion（请先 pnpm install 或设置 VALKYRIE_LEGION）: {err}"));

        if !output.status.success() {
            failures.push(format!(
                "{} 退出码 {}\nstdout:\n{}\nstderr:\n{}",
                project.display(),
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ));
        }
    }

    assert!(failures.is_empty(), "{} 个语料工程无法解析 plan（黑盒）:\n{}", failures.len(), failures.join("\n\n"));
}

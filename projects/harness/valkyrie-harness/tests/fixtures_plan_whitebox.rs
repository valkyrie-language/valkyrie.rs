//! 白盒：用 git 依赖的 `rusty-valkyrie` `legion::plan_project` 对语料验收。

use rusty_valkyrie::{CanonicalTarget, legion::plan_project};
use valkyrie_harness::fixtures;

#[test]
fn fixture_projects_resolve_plan_whitebox() {
    let projects = fixtures::project_dirs(&fixtures::fixtures_root());
    assert!(!projects.is_empty(), "projects/fixtures 下没有 legion.von 语料工程");

    let target = CanonicalTarget::parse("node").expect("node target");
    let mut failures = Vec::new();
    for project in &projects {
        match plan_project(project, target.clone()) {
            Ok(summary) => {
                assert!(!summary.project.is_empty(), "{} plan 返回空项目名", project.display());
            }
            Err(error) => failures.push(format!("{}: {error}", project.display())),
        }
    }

    assert!(failures.is_empty(), "{} 个语料工程无法解析 plan（白盒）:\n{}", failures.len(), failures.join("\n\n"));
}

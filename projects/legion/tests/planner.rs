mod support;

use legion::{
    planner::{canonical_target, BuildRequest, LegionWorkspace},
    CanonicalTarget,
};
use miette::{GraphicalReportHandler, Report};
use support::create_smoke_project_with_build;
use tempfile::Builder;

#[test]
fn canonicalizes_short_targets() {
    assert_eq!(canonical_target("clr").unwrap().to_string(), "clr-microsoft-unknown-managed");
    assert_eq!(canonical_target("jvm").unwrap().to_string(), "jvm-openjdk-unknown-managed");
    assert_eq!(canonical_target("wasm").unwrap().to_string(), "wasm32-unknown-browser-wasm");
}

#[test]
fn discovers_temp_workspace_build_plan() {
    let fixture = create_smoke_project_with_build(
        "legion-planner",
        "{\n            target: \"clr\"\n        }",
        "micro main() -> i64 {\n    return 0;\n}\n",
    );
    let workspace = LegionWorkspace::discover(&fixture.project_dir).unwrap();
    let plan = workspace
        .build_plan(&BuildRequest { project_dir: fixture.project_dir.clone(), target: CanonicalTarget::clr(), output_dir: None })
        .unwrap();

    assert_eq!(plan.project.name, "app");
    assert_eq!(plan.project.build_target.target, CanonicalTarget::clr());
    assert!(plan.project.source_files.iter().any(|path| path.ends_with(std::path::Path::new("source").join("main.v"))));
}

#[test]
fn renders_pretty_report_for_missing_workspace() {
    let temp_dir = Builder::new().prefix("legion-miette").tempdir().unwrap();
    let error = LegionWorkspace::discover(temp_dir.path()).unwrap_err();
    let report = Report::new(error);
    let mut rendered = String::new();

    GraphicalReportHandler::new().with_links(false).with_urls(false).render_report(&mut rendered, report.as_ref()).unwrap();

    assert!(rendered.contains("legion::planner::missing_workspace"));
    assert!(rendered.contains("cannot locate `legions.von`"));
    assert!(rendered.contains("请在工作区根目录放置 `legions.von`"));
}

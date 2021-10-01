mod support;

use std::{
    fs,
    path::{Path, PathBuf},
};

use legion::{
    planner::{canonical_target, BuildRequest, LegionWorkspace},
    CanonicalTarget,
};
use miette::{GraphicalReportHandler, Report};
use support::create_smoke_project_with_build;
use tempfile::Builder;

fn workspace_repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..").join("..").join("valkyrie.v")
}

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
        r#"{
            target: "clr"
        }"#,
        r#"micro main() -> i64 {
    return 0;
}
"#,
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

#[test]
fn builds_actual_legion_tools_plan_with_explicit_dependency_closure() {
    let workspace_root = workspace_repo_root();
    let project_dir = fs::canonicalize(workspace_root.join("projects").join("legion.tools")).unwrap();
    let workspace = LegionWorkspace::discover(&project_dir).unwrap();
    let plan =
        workspace.build_plan(&BuildRequest { project_dir: project_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap();

    let dependency_names: Vec<&str> = plan.project.dependencies.iter().map(|item| item.name.as_str()).collect();
    assert_eq!(dependency_names, vec!["core", "nyar", "std", "std.data.text.von"]);

    assert!(plan
        .project
        .source_files
        .iter()
        .any(|path| path.ends_with(Path::new("projects").join("legion.tools").join("source").join("build_context.v"))));
    assert!(plan.project.source_files.iter().any(|path| {
        path.ends_with(Path::new("projects").join("nyar").join("source").join("types").join("targets").join("CanonicalTarget.v"))
    }));
    assert!(plan.project.source_files.iter().any(|path| path.ends_with(Path::new("projects").join("std").join("source").join("_.v"))));
    assert!(plan
        .project
        .source_files
        .iter()
        .any(|path| { path.ends_with(Path::new("projects").join("std.data.text.von").join("source").join("_.v")) }));
    assert!(!plan.project.source_files.iter().any(|path| path.to_string_lossy().contains("examples/test.module_system/test/")));
    assert!(!plan.project.source_files.iter().any(|path| path.to_string_lossy().contains("/test/")));
}

#[test]
fn actual_legion_tools_build_context_keeps_nyar_import_explicit() {
    let build_context =
        fs::read_to_string(workspace_repo_root().join("projects").join("legion.tools").join("source").join("build_context.v")).unwrap();

    assert!(build_context.contains("using nyar;"));
    assert!(build_context.contains("micro legion_parse_canonical_target(target: utf8) -> CanonicalTarget {"));
    assert!(build_context.contains("return parse_target(canonical)"));
    assert!(build_context.contains("return format_target(parsed)"));
}

#[test]
fn filters_sdk_vendor_dependencies_by_target() {
    let temp_dir = Builder::new().prefix("legion-target-filter").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");
    let clr_dir = root.join("std.adaptor.clr");
    let wasm_dir = root.join("std.adaptor.wasm");

    fs::create_dir_all(app_dir.join("source")).unwrap();
    fs::create_dir_all(clr_dir.join("source")).unwrap();
    fs::create_dir_all(wasm_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-filter",
    members: [
        "app",
        "std.adaptor.clr",
        "std.adaptor.wasm"
    ]
}
"#,
    )
    .unwrap();

    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    dependencies: {
        "std.adaptor.clr": "workspace",
        "std.adaptor.wasm": "workspace"
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

[host_contract]
micro write(message: utf8): unit

micro main() -> i64 {
    return 0;
}
"#,
    )
    .unwrap();

    fs::write(
        clr_dir.join("legion.von"),
        r#"{
    name: "std.adaptor.clr",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        clr_dir.join("source").join("_.v"),
        r#"namespace std.adaptor.clr;

[host_provider(demo.write)]
micro write(message: utf8): unit {
}
"#,
    )
    .unwrap();

    fs::write(
        wasm_dir.join("legion.von"),
        r#"{
    name: "std.adaptor.wasm",
    sdk-vendor: {
        organization: "valkyrie",
        host: "browser",
        kind: "platform-sdk",
        targets: ["wasm"]
    },
    build: [
        {
            target: "wasm"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        wasm_dir.join("source").join("_.v"),
        r#"namespace std.adaptor.wasm;

[host_provider(demo.write)]
micro write(message: utf8): unit {
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let plan = workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap();

    let dependency_names: Vec<&str> = plan.project.dependencies.iter().map(|item| item.name.as_str()).collect();
    assert_eq!(dependency_names, vec!["std.adaptor.clr"]);
    assert!(plan.project.source_files.iter().any(|path| path.ends_with(Path::new("std.adaptor.clr").join("source").join("_.v"))));
    assert!(!plan.project.source_files.iter().any(|path| path.ends_with(Path::new("std.adaptor.wasm").join("source").join("_.v"))));
    assert_eq!(plan.project.host_contracts.len(), 1);
    assert_eq!(plan.project.host_provider_candidates.len(), 1);
    assert_eq!(plan.project.selected_host_providers.len(), 1);
    assert_eq!(plan.project.selected_host_providers[0].contract, "demo.write");
    assert!(plan.project.selected_host_providers[0].source_file.ends_with(Path::new("std.adaptor.clr").join("source").join("_.v")));
}

#[test]
fn collects_host_contract_with_default_body_without_provider() {
    let temp_dir = Builder::new().prefix("legion-host-default").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");

    fs::create_dir_all(app_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-host-default",
    members: [
        "app"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

[host_contract]
micro write(message: utf8): unit {
    if message.length == 0 {
        return
    }
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let plan = workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap();

    assert_eq!(plan.project.host_contracts.len(), 1);
    assert_eq!(plan.project.host_contracts[0].id, "demo.write");
    assert!(plan.project.host_provider_candidates.is_empty());
    assert!(plan.project.selected_host_providers.is_empty());
}

#[test]
fn collects_method_level_host_contract_and_provider_with_symbol_reference() {
    let temp_dir = Builder::new().prefix("legion-host-method").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");
    let sdk_dir = root.join("sdk");

    fs::create_dir_all(app_dir.join("source")).unwrap();
    fs::create_dir_all(sdk_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-host-method",
    members: [
        "app",
        "sdk"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    dependencies: {
        "sdk": "workspace"
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

class Writer {}

imply Writer {
    [host_contract]
    micro write(self, message: utf8): unit {
        if message.length == 0 {
            return
        }
    }
}
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("legion.von"),
        r#"{
    name: "sdk",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("source").join("_.v"),
        r#"namespace sdk;

imply demo.Writer {
    [host_provider(demo::Writer::write)]
    micro write(self, message: utf8): unit {
    }
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let plan = workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap();

    assert_eq!(plan.project.host_contracts.len(), 1);
    assert_eq!(plan.project.host_contracts[0].id, "demo::Writer::write");
    assert_eq!(plan.project.host_provider_candidates.len(), 1);
    assert_eq!(plan.project.host_provider_candidates[0].contract, "demo::Writer::write");
    assert_eq!(plan.project.host_provider_candidates[0].symbol, "sdk::Writer::write");
    assert_eq!(plan.project.selected_host_providers.len(), 1);
}

#[test]
fn collects_host_provider_from_combined_attribute_list() {
    let temp_dir = Builder::new().prefix("legion-host-provider-attrs").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");
    let sdk_dir = root.join("sdk");

    fs::create_dir_all(app_dir.join("source")).unwrap();
    fs::create_dir_all(sdk_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-host-provider-attrs",
    members: [
        "app",
        "sdk"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    dependencies: {
        "sdk": "workspace"
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

[host_contract]
micro clear(): unit
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("legion.von"),
        r#"{
    name: "sdk",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("source").join("_.v"),
        r#"namespace sdk;

[host_provider(demo.clear), inline(always)]
micro clear(): unit {
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let plan = workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap();

    assert_eq!(plan.project.host_contracts.len(), 1);
    assert_eq!(plan.project.host_contracts[0].id, "demo.clear");
    assert_eq!(plan.project.host_provider_candidates.len(), 1);
    assert_eq!(plan.project.host_provider_candidates[0].contract, "demo.clear");
    assert_eq!(plan.project.host_provider_candidates[0].symbol, "sdk.clear");
    assert_eq!(plan.project.selected_host_providers.len(), 1);
}

#[test]
fn keeps_legacy_string_host_provider_attribute_compatible() {
    let temp_dir = Builder::new().prefix("legion-host-provider-string").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");
    let sdk_dir = root.join("sdk");

    fs::create_dir_all(app_dir.join("source")).unwrap();
    fs::create_dir_all(sdk_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-host-provider-string",
    members: [
        "app",
        "sdk"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    dependencies: {
        "sdk": "workspace"
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

[host_contract]
micro clear(): unit
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("legion.von"),
        r#"{
    name: "sdk",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("source").join("_.v"),
        r#"namespace sdk;

[host_provider("demo.clear")]
micro clear(): unit {
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let plan = workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap();

    assert_eq!(plan.project.host_contracts.len(), 1);
    assert_eq!(plan.project.host_contracts[0].id, "demo.clear");
    assert_eq!(plan.project.host_provider_candidates.len(), 1);
    assert_eq!(plan.project.host_provider_candidates[0].contract, "demo.clear");
    assert_eq!(plan.project.host_provider_candidates[0].symbol, "sdk.clear");
    assert_eq!(plan.project.selected_host_providers.len(), 1);
}

#[test]
fn reports_conflicting_host_providers_during_planning() {
    let temp_dir = Builder::new().prefix("legion-host-conflict").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");
    let left_dir = root.join("sdk.left");
    let right_dir = root.join("sdk.right");

    fs::create_dir_all(app_dir.join("source")).unwrap();
    fs::create_dir_all(left_dir.join("source")).unwrap();
    fs::create_dir_all(right_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-host-conflict",
    members: [
        "app",
        "sdk.left",
        "sdk.right"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    dependencies: {
        "sdk.left": "workspace",
        "sdk.right": "workspace"
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

[host_contract]
micro write(message: utf8): unit
"#,
    )
    .unwrap();

    for project_dir in [&left_dir, &right_dir] {
        fs::write(
            project_dir.join("legion.von"),
            r#"{
    name: "sdk",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
        )
        .unwrap();
    }

    fs::write(
        left_dir.join("legion.von"),
        r#"{
    name: "sdk.left",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        right_dir.join("legion.von"),
        r#"{
    name: "sdk.right",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();

    fs::write(
        left_dir.join("source").join("_.v"),
        r#"namespace sdk.left;

[host_provider(demo.write)]
micro write(message: utf8): unit {
}
"#,
    )
    .unwrap();
    fs::write(
        right_dir.join("source").join("_.v"),
        r#"namespace sdk.right;

[host_provider(demo.write)]
micro write(message: utf8): unit {
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let error =
        workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap_err();
    let report = Report::new(error);
    let mut rendered = String::new();
    GraphicalReportHandler::new().with_links(false).with_urls(false).render_report(&mut rendered, report.as_ref()).unwrap();

    assert!(rendered.contains("legion::planner::conflicting_host_providers"));
    assert!(rendered.contains("demo.write"));
    assert!(rendered.contains("sdk.left.write"));
    assert!(rendered.contains("sdk.right.write"));
}

#[test]
fn reports_unknown_host_provider_contract_during_planning() {
    let temp_dir = Builder::new().prefix("legion-host-unknown-contract").tempdir().unwrap();
    let root = temp_dir.path();
    let app_dir = root.join("app");
    let sdk_dir = root.join("sdk");

    fs::create_dir_all(app_dir.join("source")).unwrap();
    fs::create_dir_all(sdk_dir.join("source")).unwrap();

    fs::write(
        root.join("legions.von"),
        r#"{
    name: "planner-host-unknown",
    members: [
        "app",
        "sdk"
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("legion.von"),
        r#"{
    name: "app",
    dependencies: {
        "sdk": "workspace"
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        app_dir.join("source").join("main.v"),
        r#"namespace demo;

[host_contract]
micro write(message: utf8): unit
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("legion.von"),
        r#"{
    name: "sdk",
    sdk-vendor: {
        organization: "valkyrie",
        host: "clr",
        kind: "platform-sdk",
        targets: ["clr"]
    },
    build: [
        {
            target: "clr"
        }
    ]
}
"#,
    )
    .unwrap();
    fs::write(
        sdk_dir.join("source").join("_.v"),
        r#"namespace sdk;

[host_provider(demo.missing)]
micro write(message: utf8): unit {
}
"#,
    )
    .unwrap();

    let workspace = LegionWorkspace::discover(&app_dir).unwrap();
    let error =
        workspace.build_plan(&BuildRequest { project_dir: app_dir.clone(), target: CanonicalTarget::clr(), output_dir: None }).unwrap_err();
    let report = Report::new(error);
    let mut rendered = String::new();
    GraphicalReportHandler::new().with_links(false).with_urls(false).render_report(&mut rendered, report.as_ref()).unwrap();

    assert!(rendered.contains("legion::planner::unknown_host_provider_contract"));
    assert!(rendered.contains("demo.missing"));
    assert!(rendered.contains("sdk.write"));
}

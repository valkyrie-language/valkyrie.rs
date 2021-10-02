use legion::{CanonicalTarget, DependencySpec, ProjectManifest, PublishFormat, RunnerFamily, RunnerSelector, WorkspaceManifest};

fn is_workspace_like_dependency(spec: &DependencySpec) -> bool {
    match spec {
        DependencySpec::Workspace => true,
        DependencySpec::Detailed { version: Some(version), path: None, abi: None } => version == "workspace",
        _ => false,
    }
}

#[test]
fn parses_project_manifest_dependencies() {
    let source = r#"
    {
        name: "test.hello_world",
        dependencies: {
            "std": true,
            "std.data.text.von": {
                version: "workspace",
                abi: "clr"
            }
        },
        build: [
            {
                target: "clr"
            }
        ]
    }
    "#;

    let manifest = ProjectManifest::parse(source).unwrap();
    assert_eq!(manifest.name, "test.hello_world");
    assert_eq!(manifest.build.len(), 1);
    assert_eq!(manifest.build[0].target, CanonicalTarget::clr());
    assert_eq!(manifest.dependencies.get("std"), Some(&DependencySpec::Workspace));
}

#[test]
fn parses_workspace_members() {
    let source = r#"
    {
        name: "workspace",
        members: [
            "examples/test.io",
            "projects/std"
        ],
        workspace: {
            version: "0.1.0"
        }
    }
    "#;

    let manifest = WorkspaceManifest::parse(source).unwrap();
    assert_eq!(manifest.members.len(), 2);
    assert_eq!(manifest.workspace.version.as_deref(), Some("0.1.0"));
}

#[test]
fn parses_workspace_runner_bindings() {
    let source = r#"
    {
        members: [
            "examples/test.io"
        ],
        runner: [
            {
                target: "clr",
                command: "dotnet",
                args: ["exec", "{artifact}"]
            },
            {
                target: "node",
                command: "node",
                args: ["{artifact}"]
            }
        ]
    }
    "#;

    let manifest = WorkspaceManifest::parse(source).unwrap();
    assert_eq!(manifest.runner.len(), 2);
    assert_eq!(manifest.runner[0].target, RunnerSelector::Family(RunnerFamily::Clr));
    assert_eq!(manifest.runner[1].command, "node");
}

#[test]
fn parses_publish_formats() {
    let source = r#"
    {
        name: "test.publish",
        publish: [
            {
                target: "wasm",
                type: "web-app"
            }
        ]
    }
    "#;

    let manifest = ProjectManifest::parse(source).unwrap();
    assert_eq!(manifest.publish.len(), 1);
    assert_eq!(manifest.publish[0].publish_format, Some(PublishFormat::WebApp));
}

#[test]
fn parses_sdk_vendor_metadata() {
    let source = r#"
    {
        name: "std.adaptor.wasm",
        sdk-vendor: {
            organization: "valkyrie",
            host: "browser",
            kind: "platform-sdk",
            targets: ["wasm32-unknown-browser-wasm"],
            publish: ["web-app"]
        }
    }
    "#;

    let manifest = ProjectManifest::parse(source).unwrap();
    let sdk_vendor = manifest.sdk_vendor.as_ref().unwrap();
    assert_eq!(sdk_vendor.organization.as_deref(), Some("valkyrie"));
    assert_eq!(sdk_vendor.host.as_deref(), Some("browser"));
    assert_eq!(sdk_vendor.kind.as_deref(), Some("platform-sdk"));
    assert_eq!(sdk_vendor.targets, vec!["wasm32-unknown-browser-wasm".to_string()]);
    assert_eq!(sdk_vendor.publish, vec!["web-app".to_string()]);
}

#[test]
fn parses_legion_tools_like_manifest_with_explicit_module_dependencies() {
    let source = r#"
    {
        name: "legion.tools",
        version: "workspace",
        description: "Legion 构造工具",
        auto_link: {
            core: true,
            std: false
        },
        dependencies: {
            "nyar": { version: "workspace" },
            "std": { version: "workspace" },
            "std.data.text.von": { version: "workspace" }
        },
        build: [
            {
                target: "clr"
            },
            {
                target: "jvm"
            },
            {
                target: "wasm"
            },
            {
                target: "nyar"
            }
        ]
    }
    "#;
    let manifest = ProjectManifest::parse(&source).unwrap();

    assert_eq!(manifest.name, "legion.tools");
    assert!(manifest.auto_link.core);
    assert!(!manifest.auto_link.std);

    assert!(is_workspace_like_dependency(manifest.dependencies.get("nyar").unwrap()));
    assert!(is_workspace_like_dependency(manifest.dependencies.get("std").unwrap()));
    assert!(is_workspace_like_dependency(manifest.dependencies.get("std.data.text.von").unwrap()));

    assert!(manifest.build.iter().any(|item| item.target == CanonicalTarget::clr()));
    assert!(manifest.build.iter().any(|item| item.target == CanonicalTarget::parse("jvm").unwrap()));
    assert!(manifest.build.iter().any(|item| item.target == CanonicalTarget::parse("wasm").unwrap()));
    assert!(manifest.build.iter().any(|item| item.target == CanonicalTarget::parse("nyar").unwrap()));
}

#[test]
fn parses_actual_workspace_members_after_examples_are_narrowed() {
    let source = r#"
    {
        name: "valkyrie-super-workspace",
        members: [
            "examples/demo.wechat.game",
            "examples/demo.unity.game",
            "projects/nyar",
            "projects/std",
            "projects/legion.tools"
        ],
        workspace: {
            version: "0.1.0",
            auto_link: {
                core: false,
                std: false
            }
        }
    }
    "#;
    let manifest = WorkspaceManifest::parse(&source).unwrap();

    assert!(manifest.members.contains(&"projects/legion.tools".to_string()));
    assert!(manifest.members.contains(&"projects/nyar".to_string()));
    assert!(manifest.members.contains(&"projects/std".to_string()));
    assert!(manifest.members.contains(&"examples/demo.unity.game".to_string()));
    assert!(!manifest.members.contains(&"examples/test.module_system".to_string()));
    assert!(!manifest.workspace.auto_link.core);
    assert!(!manifest.workspace.auto_link.std);
}

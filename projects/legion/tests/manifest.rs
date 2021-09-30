use legion::{CanonicalTarget, DependencySpec, ProjectManifest, PublishFormat, RunnerFamily, RunnerSelector, WorkspaceManifest};

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

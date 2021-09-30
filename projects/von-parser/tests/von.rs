use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{de::IntoDeserializer, Deserialize, Serialize};

use von_parser::{from_str, to_string, to_string_pretty, to_value, VonParser, VonValue};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ManifestLike {
    name: String,
    version: String,
    build: Vec<BuildItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct BuildItem {
    target: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct ToolConfig {
    mode: ToolMode,
    note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
enum ToolMode {
    Clr,
    Script { command: String },
}

#[test]
fn parses_manifest_like_document() {
    let source = r#"
    {
        name: "legion.tools",
        version: "workspace",
        auto_link: {
            core: true,
            std: true
        },
        build: [
            {
                target: "clr"
            },
            {
                target: "wasm32-unknown-browser-wasm"
            }
        ]
    }
    "#;

    let parsed = VonParser::parse(source).unwrap();
    let root = parsed.as_object().unwrap();
    assert_eq!(root.get("name").and_then(VonValue::as_str), Some("legion.tools"));
    assert_eq!(root.get("version").and_then(VonValue::as_str), Some("workspace"));
    assert_eq!(root.get("build").and_then(VonValue::as_array).map(|items| items.len()), Some(2));
}

#[test]
fn parses_null_literal() {
    let parsed = VonParser::parse("null").unwrap();
    assert_eq!(parsed, VonValue::Null);
}

#[test]
fn deserializes_typed_value_from_von() {
    let source = r#"
    {
        name: "legion.tools",
        version: "workspace",
        build: [
            {
                target: "clr"
            }
        ]
    }
    "#;

    let parsed: ManifestLike = from_str(source).unwrap();
    assert_eq!(parsed.name, "legion.tools");
    assert_eq!(parsed.build[0].target, "clr");
}

#[test]
fn serializes_typed_value_into_von() {
    let value = ManifestLike {
        name: "legion.tools".to_string(),
        version: "workspace".to_string(),
        build: vec![BuildItem { target: "clr".to_string() }],
    };

    let von = to_string(&value).unwrap();
    assert!(von.contains("name: \"legion.tools\""));
    assert!(von.contains("target: \"clr\""));
}

#[test]
fn pretty_von_round_trips_through_serde() {
    let value = ManifestLike {
        name: "legion.tools".to_string(),
        version: "workspace".to_string(),
        build: vec![BuildItem { target: "clr".to_string() }, BuildItem { target: "wasm".to_string() }],
    };

    let von = to_string_pretty(&value).unwrap();
    let decoded: ManifestLike = from_str(&von).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn round_trips_enum_and_option_through_von_serde() {
    let value = ToolConfig { mode: ToolMode::Script { command: "dotnet".to_string() }, note: None };

    let von = to_string_pretty(&value).unwrap();
    assert!(von.contains("note: null"));

    let decoded: ToolConfig = from_str(&von).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn keeps_boolean_deserialization_strict() {
    let value = VonValue::String("true".to_string());
    let result = bool::deserialize(value.into_deserializer());
    assert!(result.is_err());
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum UntaggedSpec {
    Bool(bool),
    String(String),
    Detailed { version: Option<String> },
}

#[test]
fn deserializes_untagged_enum_from_object() {
    let source = r#"{ version: "workspace" }"#;
    let parsed = VonParser::parse(source).unwrap();
    let result: UntaggedSpec = from_str(source).unwrap_or_else(|e| panic!("{e:?}"));
    let _ = parsed;
    match result {
        UntaggedSpec::Detailed { version } => assert_eq!(version, Some("workspace".to_string())),
        other => panic!("expected Detailed, got {other:?}"),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum DependencySpecDef {
    Bool(bool),
    String(String),
    Detailed(DetailedDependencySpec),
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct DetailedDependencySpec {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    abi: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
struct ProjectManifestLike {
    name: String,
    #[serde(default)]
    dependencies: std::collections::BTreeMap<String, DependencySpecDef>,
}

#[test]
fn deserializes_manifest_with_detailed_dependency() {
    let source = r#"
    {
        name: "legion.tools",
        dependencies: {
            "std.data.text.von": { version: "workspace" }
        }
    }
    "#;
    let result: ProjectManifestLike = from_str(source).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(result.name, "legion.tools");
    let dep = result.dependencies.get("std.data.text.von").unwrap();
    match dep {
        DependencySpecDef::Detailed(d) => assert_eq!(d.version, Some("workspace".to_string())),
        other => panic!("expected Detailed, got {other:?}"),
    }
}

#[test]
fn deserializes_full_legion_tools_manifest() {
    let source = r#"
    {
        name: "legion.tools",
        version: "workspace",
        description: "Legion 构造工具",
        auto_link: {
            core: true,
            std: true
        },
        dependencies: {
            "std.data.text.von": { version: "workspace" }
        },
        build: [
            { target: "clr" },
            { target: "jvm" },
            { target: "wasm" },
            { target: "nyar" }
        ],
        publish: [
            {
                target: "clr",
                type: "nuget",
                package_id: "LoL.Legion",
                version: "2020.0.0.0"
            }
        ]
    }
    "#;
    let result: ProjectManifestLike = from_str(source).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(result.name, "legion.tools");
    let dep = result.dependencies.get("std.data.text.von").unwrap();
    match dep {
        DependencySpecDef::Detailed(d) => assert_eq!(d.version, Some("workspace".to_string())),
        other => panic!("expected Detailed, got {other:?}"),
    }
}

#[test]
fn serializes_option_none_as_null() {
    let value = ToolConfig { mode: ToolMode::Clr, note: None };
    let encoded = to_value(&value).unwrap();
    let root = encoded.as_object().unwrap();
    assert_eq!(root.get("note"), Some(&VonValue::Null));
}

/// 递归收集指定目录下所有名为 `legion.von` 的文件路径
fn collect_legion_von_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir)
    else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_legion_von_files(&path, out);
        }
        else if path.file_name().map(|name| name == "legion.von").unwrap_or(false) {
            out.push(path);
        }
    }
}

/// 遍历 `valkyrie.v` 仓库下所有 `legion.von` 文件，使用 `ProjectManifestLike`
/// （复现 `legion::manifest::ProjectManifest` 中 `DependencySpecDef` 的 `untagged` 反序列化）
/// 尝试解析，定位触发 "data did not match any variant of untagged enum DependencySpecDef" 的文件
#[test]
#[ignore]
fn find_failing_manifests() {
    let root = Path::new(r"e:\RiderProjects\valkyrie.v");
    let mut files = Vec::new();
    collect_legion_von_files(root, &mut files);

    println!("在 {} 下找到 {} 个 `legion.von` 文件", root.display(), files.len());

    let mut failures: Vec<(PathBuf, String)> = Vec::new();

    for file in &files {
        match fs::read_to_string(file) {
            Ok(contents) => match from_str::<ProjectManifestLike>(&contents) {
                Ok(_) => println!("OK    {}", file.display()),
                Err(error) => {
                    let message = error.to_string();
                    println!("FAIL  {}", file.display());
                    println!("      {message}");
                    failures.push((file.clone(), message));
                }
            },
            Err(error) => {
                let message = error.to_string();
                println!("READ  {} {message}", file.display());
                failures.push((file.clone(), message));
            }
        }
    }

    println!();
    println!("文件总数: {}", files.len());
    println!("失败数量: {}", failures.len());

    for (path, message) in &failures {
        println!();
        println!("FAILED: {}", path.display());
        println!("  {message}");
    }
}

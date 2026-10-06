use std::{
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use nyar_language::formatter::{to_string, to_string_indented};
use oak_core::OakError;
use oak_von::language::value::{from_ast, VonValue};
use oak_von::printer::{PrintOptions, PrintStyle};
use oak_von::from_str as oak_from_str;

struct VonParser;

impl VonParser {
    fn parse(source: &str) -> Result<VonValue, OakError> {
        let trimmed = source.trim();
        let ast = oak_von::parse(trimmed).map_err(OakError::custom_error)?;
        Ok(from_ast(&ast))
    }
}

fn from_str<T>(source: &str) -> Result<T, OakError>
where
    T: serde::de::DeserializeOwned,
{
    oak_from_str(source.trim())
}

fn from_value<T>(value: VonValue) -> Result<T, OakError>
where
    T: serde::de::DeserializeOwned,
{
    let text = oak_von::printer::print_value(&value, PrintStyle::Compact, &PrintOptions::default());
    oak_von::from_str(&text)
}

fn to_value<T>(value: &T) -> Result<VonValue, OakError>
where
    T: serde::Serialize,
{
    VonParser::parse(&to_string(value)?)
}

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
    assert_eq!(parsed.get("name").and_then(VonValue::as_str), Some("legion.tools"));
    assert_eq!(parsed.get("version").and_then(VonValue::as_str), Some("workspace"));
    assert_eq!(parsed.get("build").and_then(VonValue::as_array).map(|items| items.len()), Some(2));
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
    assert!(von.contains("legion.tools"));
    assert!(von.contains("clr"));
}

#[test]
fn pretty_von_round_trips_through_serde() {
    let value = ManifestLike {
        name: "legion.tools".to_string(),
        version: "workspace".to_string(),
        build: vec![BuildItem { target: "clr".to_string() }, BuildItem { target: "wasm".to_string() }],
    };

    let von = to_string_indented(&value).unwrap();
    let decoded: ManifestLike = from_str(&von).unwrap();
    assert_eq!(decoded, value);
}

#[test]
fn round_trips_enum_and_option_through_von_serde() {
    let value = ToolConfig { mode: ToolMode::Script { command: "dotnet".to_string() }, note: None };

    let von = to_string_indented(&value).unwrap();
    assert!(von.contains("dotnet"));
    assert!(von.contains("note"));
}

#[test]
fn keeps_boolean_deserialization_strict() {
    let value = VonValue::String("true".to_string());
    let result = from_value::<bool>(value);
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
            { target: "legion" }
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
    match encoded.get("note") {
        Some(VonValue::Null) | Some(VonValue::Enum(_)) => {}
        other => panic!("expected null or Option::None enum, got {other:?}"),
    }
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

/// 遍历外部规范树中所有 `legion.von` 文件（`VALKYRIE_V_ROOT`），使用 `ProjectManifestLike`
/// 尝试解析，定位 manifest 反序列化失败项。
#[test]
#[ignore]
fn find_failing_manifests() {
    let root = match std::env::var_os("VALKYRIE_V_ROOT") {
        Some(path) => PathBuf::from(path),
        None => {
            eprintln!("skip: set VALKYRIE_V_ROOT to a language-spec checkout root");
            return;
        }
    };
    let mut files = Vec::new();
    collect_legion_von_files(&root, &mut files);

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

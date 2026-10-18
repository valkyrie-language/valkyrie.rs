use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
    marker::PhantomData,
    ops::Range,
};

use miette::{Diagnostic, LabeledSpan, Severity};

use crate::oak::{labeled_span, source_point_span};
use nyar_language::{CanonicalTarget, PublishFormat, RunnerSelector};
use oak_core::OakError as VonError;
use oak_von::from_str;
use serde::{
    Deserialize, Deserializer, Serialize, Serializer,
    de::{self, MapAccess, Visitor},
};

/// Oak VON 标量可直接写字符串；serde 的 `Option<T>` 默认需要显式 option 包装。
fn deserialize_option_string<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OptionStringVisitor;

    impl<'de> Visitor<'de> for OptionStringVisitor {
        type Value = Option<String>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("string, null, or absent optional string")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(Some(value.to_owned()))
        }

        fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
            Ok(Some(value))
        }
    }

    deserializer.deserialize_any(OptionStringVisitor)
}

/// Oak VON 嵌套对象直接写 map；serde 的 `Option<T>` 默认需要显式 option 包装。
fn deserialize_option_nested<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct OptionNestedVisitor<T>(PhantomData<T>);

    impl<'de, T> Visitor<'de> for OptionNestedVisitor<T>
    where
        T: Deserialize<'de>,
    {
        type Value = Option<T>;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("object, null, or absent optional nested value")
        }

        fn visit_none<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            Ok(Some(T::deserialize(de::value::MapAccessDeserializer::new(map))?))
        }
    }

    deserializer.deserialize_any(OptionNestedVisitor(PhantomData))
}

/// Consumer package artifact mode (`legion build` target interpretation).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProjectArtifactKind {
    /// CLI / command: `@main` entry, glue auto-runs `main`.
    #[default]
    Binary,
    /// Importable library: `[export]` surface, glue exposes `callExport` only.
    Library,
}

impl<'de> Deserialize<'de> for ProjectArtifactKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct KindVisitor;

        impl<'de> Visitor<'de> for KindVisitor {
            type Value = ProjectArtifactKind;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                formatter.write_str("'binary' or 'library'")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value.to_ascii_lowercase().as_str() {
                    "binary" => Ok(ProjectArtifactKind::Binary),
                    "library" => Ok(ProjectArtifactKind::Library),
                    other => Err(E::custom(format!("unsupported artifact kind '{other}'"))),
                }
            }

            fn visit_string<E>(self, value: String) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                self.visit_str(&value)
            }
        }

        deserializer.deserialize_any(KindVisitor)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct AutoLinkConfig {
    #[serde(default)]
    pub core: bool,
    #[serde(default)]
    pub std: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DependencySpec {
    Disabled,
    Workspace,
    Detailed {
        version: Option<String>,
        path: Option<String>,
        abi: Option<String>,
        source: Option<String>,
        registry: Option<String>,
        git: Option<String>,
        git_ref: Option<String>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencySourcePreference {
    Auto,
    Workspace,
    Registry,
    Path,
    Git,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LocalLegionConfig {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub name: Option<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
}

impl LocalLegionConfig {
    pub fn parse(source: &str) -> Result<Self, ManifestError> {
        from_str(source).map_err(ManifestError::from)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildTargetSpec {
    #[serde(default = "default_canonical_target")]
    pub target: CanonicalTarget,
    #[serde(default)]
    pub source_map: bool,
    #[serde(default)]
    pub typescript: bool,
    #[serde(default)]
    pub wat: bool,
    #[serde(default)]
    pub exclude_directories: Vec<String>,
    #[serde(default)]
    pub exclude_files: Vec<String>,
    /// 发布格式过滤（如 `mini-game` / `mini-program` / `web-app`）。
    #[serde(default)]
    pub publish: Vec<String>,
    /// CLR Runtime Async V2（.NET 11+ await-only）；默认 state machine PE。
    #[serde(default)]
    pub runtime_async: bool,
}

impl Default for BuildTargetSpec {
    fn default() -> Self {
        Self {
            target: default_canonical_target(),
            source_map: false,
            typescript: false,
            wat: false,
            exclude_directories: Vec::new(),
            exclude_files: Vec::new(),
            publish: Vec::new(),
            runtime_async: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishTargetSpec {
    #[serde(default = "default_canonical_target")]
    pub target: CanonicalTarget,
    /// Registry (`npm` / `jsr`) or artifact publish format (`web-app`, …).
    #[serde(rename = "type", default, deserialize_with = "deserialize_option_string")]
    pub channel_type: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub package_id: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub version: Option<String>,
}

impl PublishTargetSpec {
    /// When `type` names an artifact publish format, return the parsed enum.
    pub fn artifact_publish_format(&self) -> Option<PublishFormat> {
        self.channel_type.as_deref().and_then(|value| value.parse::<PublishFormat>().ok())
    }
}

impl Default for PublishTargetSpec {
    fn default() -> Self {
        Self { target: default_canonical_target(), channel_type: None, package_id: None, version: None }
    }
}

/// 第三方构建器插件配置（如 Unity 工程导出）。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct BuildPluginSpec {
    pub kind: String,
    #[serde(default)]
    pub sdk: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(rename = "input_directory", default)]
    pub input_directory: Option<String>,
    #[serde(rename = "output_directory", default)]
    pub output_directory: Option<String>,
    #[serde(rename = "next_step", default)]
    pub next_step: Option<String>,
    #[serde(rename = "export_routes", default)]
    pub export_routes: BTreeMap<String, String>,
}

fn deserialize_option_sdk_vendor<'de, D>(deserializer: D) -> Result<Option<SdkVendorConfig>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_option_nested(deserializer)
}

fn deserialize_option_build_plugin<'de, D>(deserializer: D) -> Result<Option<BuildPluginSpec>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_option_nested(deserializer)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SdkVendorConfig {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub organization: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub host: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub kind: Option<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub publish: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkspaceDefaults {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub author: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub license: Option<String>,
    #[serde(default)]
    pub auto_link: AutoLinkConfig,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RunnerBinding {
    pub target: RunnerSelector,
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceManifest {
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub version: Option<String>,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default)]
    pub runner: Vec<RunnerBinding>,
    #[serde(default)]
    pub workspace: WorkspaceDefaults,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectManifest {
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub version: Option<String>,
    #[serde(default, deserialize_with = "deserialize_option_string")]
    pub description: Option<String>,
    /// `binary`（默认）或 `library`；可执行入口与导出表面由源码 `[main]` / `[export]` 标记，manifest 不声明单文件路径。
    #[serde(default)]
    pub artifact: ProjectArtifactKind,
    #[serde(default)]
    pub auto_link: AutoLinkConfig,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
    #[serde(default)]
    pub build: Vec<BuildTargetSpec>,
    #[serde(default)]
    pub publish: Vec<PublishTargetSpec>,
    #[serde(default, deserialize_with = "deserialize_option_sdk_vendor")]
    pub sdk_vendor: Option<SdkVendorConfig>,
    #[serde(default, deserialize_with = "deserialize_option_build_plugin")]
    pub build_plugin: Option<BuildPluginSpec>,
    /// 工具链版本约束（如 `{ vcc: "0.1.*" }`）；解析后由上层消费，manifest 层仅保留。
    #[serde(default)]
    pub engines: BTreeMap<String, String>,
}

#[derive(Debug)]
pub enum ManifestError {
    Parse { error: VonError, span: Option<Range<usize>> },
}

impl ManifestError {
    fn from_von(error: VonError) -> Self {
        let span = source_point_span(&error);
        Self::Parse { error, span }
    }
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse { error, .. } => Display::fmt(error, f),
        }
    }
}

impl std::error::Error for ManifestError {}

impl Diagnostic for ManifestError {
    fn code<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("legion::manifest::parse"))
    }

    fn severity(&self) -> Option<Severity> {
        Some(Severity::Error)
    }

    fn help<'a>(&'a self) -> Option<Box<dyn Display + 'a>> {
        Some(Box::new("请检查 `legion.von` / `legions.von` 的 `VON` 语法和字段结构"))
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = LabeledSpan> + '_>> {
        match self {
            Self::Parse { span: Some(span), .. } => Some(Box::new(std::iter::once(labeled_span("VON 解析失败位置", span)))),
            _ => None,
        }
    }
}

impl From<VonError> for ManifestError {
    fn from(value: VonError) -> Self {
        Self::from_von(value)
    }
}

impl WorkspaceManifest {
    pub fn parse(source: &str) -> Result<Self, ManifestError> {
        from_str(source).map_err(ManifestError::from)
    }
}

impl ProjectManifest {
    pub fn parse(source: &str) -> Result<Self, ManifestError> {
        from_str(source).map_err(ManifestError::from)
    }
}

impl DependencySpec {
    pub fn source_preference(&self) -> DependencySourcePreference {
        match self {
            Self::Disabled => DependencySourcePreference::Auto,
            Self::Workspace => DependencySourcePreference::Workspace,
            Self::Detailed { version: Some(version), path: None, abi: None, source: None, registry: None, git: None, git_ref: None }
                if version == "workspace" =>
            {
                DependencySourcePreference::Workspace
            }
            Self::Detailed { source, path, git, .. } => match source.as_deref().map(|value| value.trim().to_ascii_lowercase()) {
                Some(value) if value == "workspace" => DependencySourcePreference::Workspace,
                Some(value) if value == "registry" => DependencySourcePreference::Registry,
                Some(value) if value == "path" => DependencySourcePreference::Path,
                Some(value) if value == "git" => DependencySourcePreference::Git,
                _ if git.is_some() => DependencySourcePreference::Git,
                _ if path.is_some() => DependencySourcePreference::Path,
                _ => DependencySourcePreference::Auto,
            },
        }
    }

    pub fn version_hint(&self) -> Option<&str> {
        match self {
            Self::Detailed { version: Some(version), .. } => Some(version.as_str()),
            Self::Workspace => Some("workspace"),
            _ => None,
        }
    }

    pub fn registry_hint(&self) -> Option<&str> {
        match self {
            Self::Detailed { registry: Some(registry), .. } => Some(registry.as_str()),
            _ => None,
        }
    }

    pub fn path_hint(&self) -> Option<&str> {
        match self {
            Self::Detailed { path: Some(path), .. } => Some(path.as_str()),
            _ => None,
        }
    }

    pub fn git_url(&self) -> Option<&str> {
        match self {
            Self::Detailed { git: Some(git), .. } => Some(git.as_str()),
            _ => None,
        }
    }

    pub fn git_ref_hint(&self) -> Option<&str> {
        match self {
            Self::Detailed { git_ref: Some(git_ref), .. } => Some(git_ref.as_str()),
            _ => None,
        }
    }
}

impl Serialize for DependencySpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Disabled => false.serialize(serializer),
            Self::Workspace => true.serialize(serializer),
            Self::Detailed { version, path, abi, source, registry, git, git_ref }
                if path.is_none() && abi.is_none() && source.is_none() && registry.is_none() && git.is_none() && git_ref.is_none() =>
            {
                version.serialize(serializer)
            }
            Self::Detailed { version, path, abi, source, registry, git, git_ref } => DetailedDependencySpec {
                version: version.clone(),
                path: path.clone(),
                abi: abi.clone(),
                source: source.clone(),
                registry: registry.clone(),
                git: git.clone(),
                git_ref: git_ref.clone(),
            }
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for DependencySpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        match DependencySpecDef::deserialize(deserializer)? {
            DependencySpecDef::Bool(false) => Ok(Self::Disabled),
            DependencySpecDef::Bool(true) => Ok(Self::Workspace),
            DependencySpecDef::String(version) => {
                Ok(Self::Detailed { version: Some(version), path: None, abi: None, source: None, registry: None, git: None, git_ref: None })
            }
            DependencySpecDef::Detailed(value) => Ok(Self::Detailed {
                version: value.version,
                path: value.path,
                abi: value.abi,
                source: value.source,
                registry: value.registry,
                git: value.git,
                git_ref: value.git_ref,
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
struct DetailedDependencySpec {
    #[serde(default)]
    version: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    abi: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    registry: Option<String>,
    #[serde(default)]
    git: Option<String>,
    #[serde(default, rename = "ref")]
    git_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(untagged)]
enum DependencySpecDef {
    Bool(bool),
    String(String),
    Detailed(DetailedDependencySpec),
}

fn default_canonical_target() -> CanonicalTarget {
    CanonicalTarget::clr()
}

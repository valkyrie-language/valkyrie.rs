use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use miette::{Diagnostic, Severity};
use nyar_language::{CanonicalTarget, PublishFormat, RunnerSelector};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std_data::text::von::{VonError, from_str};

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
    Detailed { version: Option<String>, path: Option<String>, abi: Option<String>, source: Option<String>, registry: Option<String> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencySourcePreference {
    Auto,
    Workspace,
    Registry,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildTargetSpec {
    #[serde(default = "default_canonical_target")]
    pub target: CanonicalTarget,
    #[serde(default)]
    pub msil: bool,
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
            msil: false,
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
    #[serde(rename = "type", default)]
    pub channel_type: Option<String>,
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct SdkVendorConfig {
    #[serde(default)]
    pub organization: Option<String>,
    #[serde(default)]
    pub host: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
    #[serde(default)]
    pub targets: Vec<String>,
    #[serde(default)]
    pub publish: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WorkspaceDefaults {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
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
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub auto_link: AutoLinkConfig,
    #[serde(default)]
    pub dependencies: BTreeMap<String, DependencySpec>,
    #[serde(default)]
    pub build: Vec<BuildTargetSpec>,
    #[serde(default)]
    pub publish: Vec<PublishTargetSpec>,
    #[serde(rename = "sdk-vendor", default)]
    pub sdk_vendor: Option<SdkVendorConfig>,
    #[serde(rename = "build_plugin", default)]
    pub build_plugin: Option<BuildPluginSpec>,
}

#[derive(Debug)]
pub enum ManifestError {
    Parse(VonError),
}

impl Display for ManifestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(error) => Display::fmt(error, f),
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

    fn diagnostic_source(&self) -> Option<&dyn Diagnostic> {
        match self {
            ManifestError::Parse(error) => Some(error),
        }
    }
}

impl From<VonError> for ManifestError {
    fn from(value: VonError) -> Self {
        Self::Parse(value)
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
            Self::Detailed { source, .. } => match source.as_deref().map(|value| value.trim().to_ascii_lowercase()) {
                Some(value) if value == "workspace" => DependencySourcePreference::Workspace,
                Some(value) if value == "registry" => DependencySourcePreference::Registry,
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
}

impl Serialize for DependencySpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Disabled => false.serialize(serializer),
            Self::Workspace => true.serialize(serializer),
            Self::Detailed { version, path, abi, source, registry }
                if path.is_none() && abi.is_none() && source.is_none() && registry.is_none() =>
            {
                version.serialize(serializer)
            }
            Self::Detailed { version, path, abi, source, registry } => DetailedDependencySpec {
                version: version.clone(),
                path: path.clone(),
                abi: abi.clone(),
                source: source.clone(),
                registry: registry.clone(),
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
                Ok(Self::Detailed { version: Some(version), path: None, abi: None, source: None, registry: None })
            }
            DependencySpecDef::Detailed(value) => {
                Ok(Self::Detailed { version: value.version, path: value.path, abi: value.abi, source: value.source, registry: value.registry })
            }
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

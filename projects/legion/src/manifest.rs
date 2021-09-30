use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use miette::{Diagnostic, Severity};
use nyar::{CanonicalTarget, PublishFormat, RunnerSelector};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use von_parser::{from_str, VonError};

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
    Detailed { version: Option<String>, path: Option<String>, abi: Option<String> },
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishTargetSpec {
    #[serde(default = "default_canonical_target")]
    pub target: CanonicalTarget,
    #[serde(rename = "type", default)]
    pub publish_format: Option<PublishFormat>,
    #[serde(default)]
    pub package_id: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
}

impl Default for PublishTargetSpec {
    fn default() -> Self {
        Self { target: default_canonical_target(), publish_format: None, package_id: None, version: None }
    }
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

impl Serialize for DependencySpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Disabled => false.serialize(serializer),
            Self::Workspace => true.serialize(serializer),
            Self::Detailed { version, path, abi } if path.is_none() && abi.is_none() => version.serialize(serializer),
            Self::Detailed { version, path, abi } => {
                DetailedDependencySpec { version: version.clone(), path: path.clone(), abi: abi.clone() }.serialize(serializer)
            }
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
            DependencySpecDef::String(version) => Ok(Self::Detailed { version: Some(version), path: None, abi: None }),
            DependencySpecDef::Detailed(value) => Ok(Self::Detailed { version: value.version, path: value.path, abi: value.abi }),
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

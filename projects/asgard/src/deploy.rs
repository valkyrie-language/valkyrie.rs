//! Deploy profile：校验契约并打印制品矩阵（编排构建为 stub / planned）。

use std::path::{Path, PathBuf};

use miette::{IntoDiagnostic, Result, WrapErr, miette};
use serde::{Deserialize, Serialize};
use std_data::text::von::{VonParser, VonValue};

/// Profile 实现状态。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProfileStatus {
    /// 契约与当前 CLI 能力已落地（编排仍可能 stub）。
    Implemented,
    /// 仅文档/契约；构建或打包后置。
    Planned,
}

impl ProfileStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Implemented => "implemented",
            Self::Planned => "planned",
        }
    }
}

impl Default for ProfileStatus {
    fn default() -> Self {
        Self::Planned
    }
}

/// 流水线标识。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployPipeline {
    /// Asgard / VOA 客户端 GUI。
    Asgard,
    /// Atlas 服务端 / server 岛。
    Atlas,
}

impl DeployPipeline {
    fn as_str(self) -> &'static str {
        match self {
            Self::Asgard => "asgard",
            Self::Atlas => "atlas",
        }
    }
}

/// 单条制品矩阵条目。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployArtifact {
    /// `asgard` 或 `atlas`。
    pub pipeline: DeployPipeline,
    /// Asgard `platform`（atlas 可省略）。
    #[serde(default)]
    pub platform: Option<String>,
    /// 拓扑角色，如 `static-web`、`server-islands`。
    pub role: String,
    /// 预期输出目录（相对 workspace）。
    pub output: String,
    /// 条目状态；缺省继承 profile。
    #[serde(default)]
    pub status: Option<ProfileStatus>,
}

/// 部署拓扑提示。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployTopology {
    /// 静态资源挂载（如 `cdn`）。
    #[serde(default)]
    pub static_host: Option<String>,
    /// 服务挂载（如 `serverless`、`single-host`）。
    #[serde(default)]
    pub server: Option<String>,
    /// 其它自由字段由 VON 保留时忽略亦可；此处仅常用键。
    #[serde(default)]
    pub notes: Option<String>,
}

/// Deploy profile 根对象。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployProfile {
    /// Profile 名。
    pub name: String,
    /// 说明。
    #[serde(default)]
    pub description: String,
    /// Profile 级状态。
    #[serde(default)]
    pub status: ProfileStatus,
    /// 制品矩阵。
    pub artifacts: Vec<DeployArtifact>,
    /// 拓扑提示。
    #[serde(default)]
    pub topology: DeployTopology,
}

/// 校验警告（非致命）。
#[derive(Debug, Clone)]
pub struct DeployProfileWarning {
    /// 警告文本。
    pub message: String,
}

/// 校验 + 解析结果。
#[derive(Debug, Clone)]
pub struct DeployPlan {
    /// 解析后的 profile。
    pub profile: DeployProfile,
    /// 非致命警告。
    pub warnings: Vec<DeployProfileWarning>,
    /// 源文件路径。
    pub source: PathBuf,
}

impl DeployProfile {
    /// 从文件加载并校验。
    pub fn load(path: &Path) -> Result<DeployPlan> {
        let source =
            std::fs::read_to_string(path).into_diagnostic().wrap_err_with(|| format!("读取 deploy profile 失败: {}", path.display()))?;
        let mut plan = Self::parse(&source)?;
        plan.source = path.to_path_buf();
        Ok(plan)
    }

    /// 解析 VON 文本并校验。
    pub fn parse(source: &str) -> Result<DeployPlan> {
        let value = VonParser::parse(source).map_err(|error| miette!("{error:?}"))?;
        let json = von_to_json(&value);
        let profile: DeployProfile = serde_json::from_value(json).into_diagnostic().wrap_err("解析 deploy profile 失败")?;
        let warnings = validate_profile(&profile)?;
        Ok(DeployPlan { profile, warnings, source: PathBuf::from("<memory>") })
    }
}

fn validate_profile(profile: &DeployProfile) -> Result<Vec<DeployProfileWarning>> {
    if profile.name.trim().is_empty() {
        return Err(miette!("deploy profile 缺少 name"));
    }
    if profile.artifacts.is_empty() {
        return Err(miette!("deploy profile artifacts 不能为空"));
    }
    let mut warnings = Vec::new();
    for (index, artifact) in profile.artifacts.iter().enumerate() {
        if artifact.role.trim().is_empty() {
            return Err(miette!("artifacts[{index}] 缺少 role"));
        }
        if artifact.output.trim().is_empty() {
            return Err(miette!("artifacts[{index}] 缺少 output"));
        }
        if matches!(artifact.pipeline, DeployPipeline::Asgard) && artifact.platform.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true) {
            warnings.push(DeployProfileWarning { message: format!("artifacts[{index}] pipeline=asgard 建议提供 platform") });
        }
    }
    Ok(warnings)
}

fn von_to_json(value: &VonValue) -> serde_json::Value {
    match value {
        VonValue::Null => serde_json::Value::Null,
        VonValue::Bool(v) => serde_json::Value::Bool(*v),
        VonValue::Number(v) => serde_json::Value::Number((*v).into()),
        VonValue::String(v) => serde_json::Value::String(v.clone()),
        VonValue::Array(items) => serde_json::Value::Array(items.iter().map(von_to_json).collect()),
        VonValue::Object(map) => {
            let mut object = serde_json::Map::new();
            for (key, item) in map {
                // topology 常用别名：static → static_host
                let key = if key == "static" { "static_host".to_string() } else { key.clone() };
                object.insert(key, von_to_json(item));
            }
            serde_json::Value::Object(object)
        }
    }
}

/// 列出目录下的 `*.von` profile 文件名（按字典序）。
pub fn list_deploy_profiles(dir: &Path) -> Result<Vec<String>> {
    let entries = std::fs::read_dir(dir).into_diagnostic().wrap_err_with(|| format!("读取 profiles 目录失败: {}", dir.display()))?;
    let mut names = Vec::new();
    for entry in entries {
        let entry = entry.into_diagnostic()?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("von") {
            continue;
        }
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            names.push(name.to_string());
        }
    }
    names.sort();
    Ok(names)
}

/// 打印制品矩阵（编排 stub）。
pub fn print_deploy_plan(plan: &DeployPlan) {
    let p = &plan.profile;
    println!("asgard plan");
    println!("profile: {}", p.name);
    println!("source: {}", plan.source.display());
    println!("status: {}", p.status.as_str());
    if !p.description.is_empty() {
        println!("description: {}", p.description);
    }
    if let Some(static_host) = &p.topology.static_host {
        println!("topology.static: {static_host}");
    }
    if let Some(server) = &p.topology.server {
        println!("topology.server: {server}");
    }
    for warning in &plan.warnings {
        println!("warning: {}", warning.message);
    }
    println!();
    println!("artifact matrix (orchestration: stub / planned)");
    println!("{:<8} {:<10} {:<18} {:<16} {}", "pipeline", "platform", "role", "status", "output");
    println!("{}", "-".repeat(72));
    for artifact in &p.artifacts {
        let status = artifact.status.unwrap_or(p.status).as_str();
        let platform = artifact.platform.as_deref().unwrap_or("-");
        println!("{:<8} {:<10} {:<18} {:<16} {}", artifact.pipeline.as_str(), platform, artifact.role, status, artifact.output);
    }
    println!();
    println!("note: 不会执行 asgard build / atlas；仅校验并打印矩阵。");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_cdn_serverless_profile() {
        let source = r#"{
  name: "cdn+serverless",
  description: "SSG to CDN + optional Atlas server islands",
  status: "implemented",
  topology: {
    static: "cdn",
    server: "serverless"
  },
  artifacts: [
    {
      pipeline: "asgard",
      platform: "browser",
      role: "static-web",
      output: "apps/web/dist",
      status: "implemented"
    },
    {
      pipeline: "atlas",
      role: "server-islands",
      output: "apps/api/dist",
      status: "planned"
    }
  ]
}"#;
        let plan = DeployProfile::parse(source).expect("parse");
        assert_eq!(plan.profile.name, "cdn+serverless");
        assert_eq!(plan.profile.artifacts.len(), 2);
        assert_eq!(plan.profile.topology.static_host.as_deref(), Some("cdn"));
        assert!(plan.warnings.is_empty());
    }

    #[test]
    fn rejects_empty_artifacts() {
        let source = r#"{ name: "x", artifacts: [] }"#;
        assert!(DeployProfile::parse(source).is_err());
    }

    #[test]
    fn loads_repo_cdn_profile_file() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../valkyrie.v/projects/asgard._/projects/asgard/deploy/profiles/cdn+serverless.von");
        let plan = DeployProfile::load(&path).expect("load repo profile");
        assert_eq!(plan.profile.name, "cdn+serverless");
        assert_eq!(plan.profile.artifacts.len(), 2);
    }

    #[test]
    fn loads_fullstack_cdn_profile() {
        let path =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../valkyrie.v/examples/test.fullstack/deploy/profiles/cdn+serverless.von");
        let plan = DeployProfile::load(&path).expect("load fullstack profile");
        assert_eq!(plan.profile.name, "cdn+serverless");
        assert!(plan.profile.artifacts.iter().any(|a| a.output.contains("apps/shell")));
        assert!(plan.profile.artifacts.iter().any(|a| a.output.contains("apps/atlas")));
    }

    #[test]
    fn lists_fullstack_profiles_dir() {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../valkyrie.v/examples/test.fullstack/deploy/profiles");
        let names = list_deploy_profiles(&dir).expect("list");
        assert!(names.iter().any(|n| n == "cdn+serverless.von"));
        assert!(names.iter().any(|n| n == "portable.von"));
    }
}

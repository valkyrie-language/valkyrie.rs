//! `CLR` 宿主相关产物。
//!
//! 负责生成 `dotnet` 宿主运行所需的辅助文件，
//! 避免命令层手写 `JSON` 文本。

use std::path::Path;

use miette::{IntoDiagnostic, Result, WrapErr};
use serde::Serialize;

/// `dotnet` 运行时配置。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DotNetRuntimeConfig {
    /// 运行时选项。
    #[serde(rename = "runtimeOptions")]
    pub runtime_options: DotNetRuntimeOptions,
}

/// `dotnet` 运行时选项。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DotNetRuntimeOptions {
    /// 目标框架标识。
    pub tfm: String,
    /// 目标宿主框架。
    pub framework: DotNetFramework,
}

/// `dotnet` 宿主框架信息。
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DotNetFramework {
    /// 框架名称。
    pub name: String,
    /// 框架版本。
    pub version: String,
}

impl Default for DotNetRuntimeConfig {
    fn default() -> Self {
        Self {
            runtime_options: DotNetRuntimeOptions {
                tfm: "net9.0".to_string(),
                framework: DotNetFramework { name: "Microsoft.NETCore.App".to_string(), version: "9.0.0".to_string() },
            },
        }
    }
}

/// 将 `dotnet` 运行时配置写入输出目录。
pub fn write_dotnet_runtime_config(output_dir: &Path, artifact_name: &str) -> Result<()> {
    let runtime_config_path = output_dir.join(format!("{}.runtimeconfig.json", artifact_name));
    let runtime_config = serde_json::to_string_pretty(&DotNetRuntimeConfig::default())
        .into_diagnostic()
        .wrap_err_with(|| format!("序列化运行时配置失败：{}", runtime_config_path.display()))?;

    std::fs::write(&runtime_config_path, format!("{}\n", runtime_config))
        .into_diagnostic()
        .wrap_err_with(|| format!("写入运行时配置失败：{}", runtime_config_path.display()))
}

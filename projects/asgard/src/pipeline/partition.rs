//! 源码分流：AWSL 与 V 逻辑分离。

use std::{fs, path::Path};

use miette::{IntoDiagnostic, Result, WrapErr};

use super::discover::DiscoveredSources;

/// 合并 V 逻辑与 synthetic V 为单一编译单元。
pub fn combine_v_sources(sources: &DiscoveredSources, synthetic_v: &str) -> Result<String> {
    let mut combined = String::new();
    for v_file in &sources.v_files {
        let content =
            fs::read_to_string(&v_file.path).into_diagnostic().wrap_err_with(|| format!("读取 V 源码失败: {}", v_file.path.display()))?;
        let trimmed = content.strip_prefix('\u{FEFF}').unwrap_or(&content);
        combined.push_str(trimmed);
        combined.push('\n');
    }
    combined.push_str(synthetic_v);
    Ok(combined)
}

/// 加载单个 AWSL 文件内容。
pub fn read_awsl_file(path: &Path) -> Result<String> {
    let content = fs::read_to_string(path).into_diagnostic().wrap_err_with(|| format!("读取 AWSL 失败: {}", path.display()))?;
    Ok(content.strip_prefix('\u{FEFF}').unwrap_or(&content).to_string())
}

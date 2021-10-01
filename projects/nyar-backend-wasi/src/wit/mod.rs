#![doc = include_str!("readme.md")]

use std::fmt::{Display, Formatter};

use serde::{Deserialize, Serialize};

/// `WIT` 解析错误。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WitError {
    /// 包声明缺失或格式不正确。
    InvalidPackage,
    /// 接口块格式不正确。
    InvalidInterface(String),
}

impl Display for WitError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPackage => write!(f, "无效的 `WIT` package 声明"),
            Self::InvalidInterface(message) => write!(f, "无效的 `WIT` interface：{message}"),
        }
    }
}

impl std::error::Error for WitError {}

/// `WIT` 接口项。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WitInterface {
    /// 接口名。
    pub name: String,
    /// 函数签名列表。
    pub functions: Vec<String>,
}

/// `WIT` 接口包模型。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct WitPackage {
    /// 包名。
    pub package_name: String,
    /// 接口列表。
    pub interfaces: Vec<WitInterface>,
}

impl WitPackage {
    /// 创建一个新的 `WIT` 接口包。
    pub fn new(package_name: impl Into<String>) -> Self {
        Self { package_name: package_name.into(), interfaces: Vec::new() }
    }

    /// 追加一个接口定义。
    pub fn push_interface(&mut self, name: impl Into<String>, functions: Vec<String>) {
        self.interfaces.push(WitInterface { name: name.into(), functions });
    }

    /// 解析 `WIT` 文本。
    pub fn parse(source: &str) -> Result<Self, WitError> {
        let normalized = source.replace("\r\n", "\n");
        let mut lines = normalized.lines().map(str::trim).filter(|line| !line.is_empty() && !line.starts_with("//")).peekable();

        let Some(package_line) = lines.next()
        else {
            return Err(WitError::InvalidPackage);
        };
        let Some(package_name) = package_line.strip_prefix("package ").and_then(|value| value.strip_suffix(';')).map(str::trim)
        else {
            return Err(WitError::InvalidPackage);
        };

        let mut package = Self::new(package_name);
        while let Some(line) = lines.next() {
            if !line.starts_with("interface ") {
                return Err(WitError::InvalidInterface(format!("无法识别的顶层项：{line}")));
            }

            let header = line.trim_start_matches("interface ").trim();
            let name = header.strip_suffix('{').map(str::trim).ok_or_else(|| WitError::InvalidInterface(format!("接口头缺少 `{{`：{line}")))?;

            let mut functions = Vec::new();
            loop {
                let Some(body_line) = lines.next()
                else {
                    return Err(WitError::InvalidInterface(format!("接口 `{name}` 没有闭合")));
                };
                if body_line == "}" {
                    break;
                }
                let function = body_line
                    .strip_suffix(';')
                    .map(str::trim)
                    .ok_or_else(|| WitError::InvalidInterface(format!("函数声明缺少 `;`：{body_line}")))?;
                functions.push(function.to_string());
            }
            package.push_interface(name, functions);
        }
        Ok(package)
    }

    /// 将模型格式化为 `WIT` 文本。
    pub fn to_text(&self) -> String {
        let mut result = format!("package {};\n", self.package_name);
        for interface in &self.interfaces {
            result.push('\n');
            result.push_str(&format!("interface {} {{\n", interface.name));
            for function in &interface.functions {
                result.push_str("  ");
                result.push_str(function.trim());
                result.push_str(";\n");
            }
            result.push_str("}\n");
        }
        result
    }
}

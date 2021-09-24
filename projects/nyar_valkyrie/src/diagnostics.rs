//! 诊断管理模块
//!
//! 提供统一的错误和警告管理功能，支持多种输出格式。

use std::{
    collections::HashMap,
    fmt,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

use miette::{Diagnostic, SourceSpan};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use nyar_core::{Position, Range};

use crate::{config::DiagnosticsConfig, error::RuntimeResult};

/// 诊断严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Severity {
    /// 错误
    Error = 1,
    /// 警告
    Warning = 2,
    /// 信息
    Information = 3,
    /// 提示
    Hint = 4,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => write!(f, "error"),
            Severity::Warning => write!(f, "warning"),
            Severity::Information => write!(f, "info"),
            Severity::Hint => write!(f, "hint"),
        }
    }
}

/// 诊断类别
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiagnosticCategory {
    /// 语法错误
    Syntax,
    /// 类型错误
    Type,
    /// 语义错误
    Semantic,
    /// 运行时错误
    Runtime,
    /// 性能警告
    Performance,
    /// 代码风格
    Style,
    /// 弃用警告
    Deprecation,
    /// 安全警告
    Security,
    /// 自定义类别
    Custom(String),
}

impl fmt::Display for DiagnosticCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiagnosticCategory::Syntax => write!(f, "syntax"),
            DiagnosticCategory::Type => write!(f, "type"),
            DiagnosticCategory::Semantic => write!(f, "semantic"),
            DiagnosticCategory::Runtime => write!(f, "runtime"),
            DiagnosticCategory::Performance => write!(f, "performance"),
            DiagnosticCategory::Style => write!(f, "style"),
            DiagnosticCategory::Deprecation => write!(f, "deprecation"),
            DiagnosticCategory::Security => write!(f, "security"),
            DiagnosticCategory::Custom(name) => write!(f, "custom:{}", name),
        }
    }
}

/// 诊断信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticInfo {
    /// 唯一标识符
    pub id: String,
    /// 严重程度
    pub severity: Severity,
    /// 类别
    pub category: DiagnosticCategory,
    /// 消息
    pub message: String,
    /// 详细描述
    pub detail: Option<String>,
    /// 文件路径
    pub file_path: Option<PathBuf>,
    /// 位置范围
    pub range: Option<Range>,
    /// 源码片段
    pub source_span: Option<SourceSpan>,
    /// 相关信息
    pub related_information: Vec<RelatedInformation>,
    /// 建议的修复
    pub fixes: Vec<DiagnosticFix>,
    /// 标签
    pub tags: Vec<DiagnosticTag>,
    /// 错误代码
    pub code: Option<String>,
    /// 创建时间
    pub timestamp: Instant,
}

/// 相关信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedInformation {
    /// 位置
    pub location: DiagnosticLocation,
    /// 消息
    pub message: String,
}

/// 诊断位置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticLocation {
    /// 文件路径
    pub file_path: PathBuf,
    /// 范围
    pub range: Range,
}

/// 诊断修复建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticFix {
    /// 标题
    pub title: String,
    /// 描述
    pub description: Option<String>,
    /// 编辑操作
    pub edits: Vec<TextEdit>,
    /// 是否为首选修复
    pub preferred: bool,
}

/// 文本编辑
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEdit {
    /// 编辑范围
    pub range: Range,
    /// 新文本
    pub new_text: String,
}

/// 诊断标签
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticTag {
    /// 不必要的代码
    Unnecessary,
    /// 已弃用
    Deprecated,
}

/// 诊断统计
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiagnosticStats {
    /// 错误数量
    pub error_count: usize,
    /// 警告数量
    pub warning_count: usize,
    /// 信息数量
    pub info_count: usize,
    /// 提示数量
    pub hint_count: usize,
    /// 按类别统计
    pub category_counts: HashMap<DiagnosticCategory, usize>,
    /// 按文件统计
    pub file_counts: HashMap<PathBuf, usize>,
}

impl DiagnosticStats {
    /// 添加诊断到统计
    pub fn add_diagnostic(&mut self, diagnostic: &DiagnosticInfo) {
        match diagnostic.severity {
            Severity::Error => self.error_count += 1,
            Severity::Warning => self.warning_count += 1,
            Severity::Information => self.info_count += 1,
            Severity::Hint => self.hint_count += 1,
        }

        *self.category_counts.entry(diagnostic.category.clone()).or_insert(0) += 1;

        if let Some(file_path) = &diagnostic.file_path {
            *self.file_counts.entry(file_path.clone()).or_insert(0) += 1;
        }
    }

    /// 移除诊断从统计
    pub fn remove_diagnostic(&mut self, diagnostic: &DiagnosticInfo) {
        match diagnostic.severity {
            Severity::Error => self.error_count = self.error_count.saturating_sub(1),
            Severity::Warning => self.warning_count = self.warning_count.saturating_sub(1),
            Severity::Information => self.info_count = self.info_count.saturating_sub(1),
            Severity::Hint => self.hint_count = self.hint_count.saturating_sub(1),
        }

        if let Some(count) = self.category_counts.get_mut(&diagnostic.category) {
            *count = count.saturating_sub(1);
            if *count == 0 {
                self.category_counts.remove(&diagnostic.category);
            }
        }

        if let Some(file_path) = &diagnostic.file_path {
            if let Some(count) = self.file_counts.get_mut(file_path) {
                *count = count.saturating_sub(1);
                if *count == 0 {
                    self.file_counts.remove(file_path);
                }
            }
        }
    }

    /// 获取总数量
    pub fn total_count(&self) -> usize {
        self.error_count + self.warning_count + self.info_count + self.hint_count
    }

    /// 是否有错误
    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }

    /// 是否有警告
    pub fn has_warnings(&self) -> bool {
        self.warning_count > 0
    }
}

/// 诊断过滤器
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticFilter {
    /// 最小严重程度
    pub min_severity: Option<Severity>,
    /// 包含的类别
    pub include_categories: Option<Vec<DiagnosticCategory>>,
    /// 排除的类别
    pub exclude_categories: Option<Vec<DiagnosticCategory>>,
    /// 包含的文件模式
    pub include_files: Option<Vec<String>>,
    /// 排除的文件模式
    pub exclude_files: Option<Vec<String>>,
    /// 包含的错误代码
    pub include_codes: Option<Vec<String>>,
    /// 排除的错误代码
    pub exclude_codes: Option<Vec<String>>,
}

impl Default for DiagnosticFilter {
    fn default() -> Self {
        Self {
            min_severity: None,
            include_categories: None,
            exclude_categories: None,
            include_files: None,
            exclude_files: None,
            include_codes: None,
            exclude_codes: None,
        }
    }
}

impl DiagnosticFilter {
    /// 检查诊断是否通过过滤器
    pub fn matches(&self, diagnostic: &DiagnosticInfo) -> bool {
        // 检查严重程度
        if let Some(min_severity) = self.min_severity {
            if diagnostic.severity > min_severity {
                return false;
            }
        }

        // 检查类别包含
        if let Some(include_categories) = &self.include_categories {
            if !include_categories.contains(&diagnostic.category) {
                return false;
            }
        }

        // 检查类别排除
        if let Some(exclude_categories) = &self.exclude_categories {
            if exclude_categories.contains(&diagnostic.category) {
                return false;
            }
        }

        // 检查错误代码包含
        if let Some(include_codes) = &self.include_codes {
            if let Some(code) = &diagnostic.code {
                if !include_codes.contains(code) {
                    return false;
                }
            }
            else {
                return false;
            }
        }

        // 检查错误代码排除
        if let Some(exclude_codes) = &self.exclude_codes {
            if let Some(code) = &diagnostic.code {
                if exclude_codes.contains(code) {
                    return false;
                }
            }
        }

        // TODO: 实现文件模式匹配

        true
    }
}

/// 诊断格式化器
pub trait DiagnosticFormatter: Send + Sync {
    /// 格式化单个诊断
    fn format_diagnostic(&self, diagnostic: &DiagnosticInfo) -> String;

    /// 格式化诊断列表
    fn format_diagnostics(&self, diagnostics: &[DiagnosticInfo]) -> String {
        diagnostics.iter().map(|d| self.format_diagnostic(d)).collect::<Vec<_>>().join("\n")
    }

    /// 格式化统计信息
    fn format_stats(&self, stats: &DiagnosticStats) -> String;
}

/// 简单文本格式化器
pub struct SimpleTextFormatter;

impl DiagnosticFormatter for SimpleTextFormatter {
    fn format_diagnostic(&self, diagnostic: &DiagnosticInfo) -> String {
        let mut result = String::new();

        // 添加文件路径和位置
        if let Some(file_path) = &diagnostic.file_path {
            result.push_str(&file_path.display().to_string());

            if let Some(range) = &diagnostic.range {
                result.push_str(&format!(":{}:{}", range.start.line + 1, range.start.column + 1));
            }

            result.push_str(": ");
        }

        // 添加严重程度
        result.push_str(&format!("{}: ", diagnostic.severity));

        // 添加消息
        result.push_str(&diagnostic.message);

        // 添加错误代码
        if let Some(code) = &diagnostic.code {
            result.push_str(&format!(" [{}]", code));
        }

        // 添加类别
        result.push_str(&format!(" ({})", diagnostic.category));

        result
    }

    fn format_stats(&self, stats: &DiagnosticStats) -> String {
        format!(
            "Total: {} (Errors: {}, Warnings: {}, Info: {}, Hints: {})",
            stats.total_count(),
            stats.error_count,
            stats.warning_count,
            stats.info_count,
            stats.hint_count
        )
    }
}

/// JSON 格式化器
pub struct JsonFormatter;

impl DiagnosticFormatter for JsonFormatter {
    fn format_diagnostic(&self, diagnostic: &DiagnosticInfo) -> String {
        serde_json::to_string(diagnostic).unwrap_or_else(|_| "Invalid diagnostic".to_string())
    }

    fn format_diagnostics(&self, diagnostics: &[DiagnosticInfo]) -> String {
        serde_json::to_string(diagnostics).unwrap_or_else(|_| "[]".to_string())
    }

    fn format_stats(&self, stats: &DiagnosticStats) -> String {
        serde_json::to_string(stats).unwrap_or_else(|_| "{}".to_string())
    }
}

/// 诊断管理器
pub struct DiagnosticsManager {
    /// 配置
    config: DiagnosticsConfig,
    /// 诊断信息存储
    diagnostics: Arc<RwLock<HashMap<String, Vec<DiagnosticInfo>>>>,
    /// 统计信息
    stats: Arc<RwLock<DiagnosticStats>>,
    /// 格式化器
    formatter: Box<dyn DiagnosticFormatter>,
}

impl DiagnosticsManager {
    /// 创建新的诊断管理器
    pub fn new(config: DiagnosticsConfig) -> Self {
        let formatter: Box<dyn DiagnosticFormatter> = match config.output_format.as_str() {
            "json" => Box::new(JsonFormatter),
            _ => Box::new(SimpleTextFormatter),
        };

        Self {
            config,
            diagnostics: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(DiagnosticStats::default())),
            formatter,
        }
    }

    /// 添加诊断信息
    pub fn add_diagnostic(&self, file_id: &str, diagnostic: DiagnosticInfo) {
        let mut diagnostics = self.diagnostics.write();
        let mut stats = self.stats.write();

        stats.add_diagnostic(&diagnostic);

        diagnostics.entry(file_id.to_string()).or_insert_with(Vec::new).push(diagnostic);
    }

    /// 添加多个诊断信息
    pub fn add_diagnostics(&self, file_id: &str, new_diagnostics: Vec<DiagnosticInfo>) {
        let mut diagnostics = self.diagnostics.write();
        let mut stats = self.stats.write();

        for diagnostic in &new_diagnostics {
            stats.add_diagnostic(diagnostic);
        }

        diagnostics.entry(file_id.to_string()).or_insert_with(Vec::new).extend(new_diagnostics);
    }

    /// 清除文件的诊断信息
    pub fn clear_file_diagnostics(&self, file_id: &str) {
        let mut diagnostics = self.diagnostics.write();
        let mut stats = self.stats.write();

        if let Some(file_diagnostics) = diagnostics.remove(file_id) {
            for diagnostic in &file_diagnostics {
                stats.remove_diagnostic(diagnostic);
            }
        }
    }

    /// 获取文件的诊断信息
    pub fn get_file_diagnostics(&self, file_id: &str) -> Vec<DiagnosticInfo> {
        let diagnostics = self.diagnostics.read();
        diagnostics.get(file_id).cloned().unwrap_or_default()
    }

    /// 获取所有诊断信息
    pub fn get_all_diagnostics(&self) -> HashMap<String, Vec<DiagnosticInfo>> {
        self.diagnostics.read().clone()
    }

    /// 获取过滤后的诊断信息
    pub fn get_filtered_diagnostics(&self, filter: &DiagnosticFilter) -> Vec<DiagnosticInfo> {
        let diagnostics = self.diagnostics.read();
        let mut result = Vec::new();

        for file_diagnostics in diagnostics.values() {
            for diagnostic in file_diagnostics {
                if filter.matches(diagnostic) {
                    result.push(diagnostic.clone());
                }
            }
        }

        result
    }

    /// 获取统计信息
    pub fn get_stats(&self) -> DiagnosticStats {
        self.stats.read().clone()
    }

    /// 格式化诊断信息
    pub fn format_diagnostics(&self, diagnostics: &[DiagnosticInfo]) -> String {
        self.formatter.format_diagnostics(diagnostics)
    }

    /// 格式化统计信息
    pub fn format_stats(&self) -> String {
        let stats = self.stats.read();
        self.formatter.format_stats(&stats)
    }

    /// 检查是否有错误
    pub fn has_errors(&self) -> bool {
        self.stats.read().has_errors()
    }

    /// 检查是否有警告
    pub fn has_warnings(&self) -> bool {
        self.stats.read().has_warnings()
    }

    /// 获取错误数量
    pub fn error_count(&self) -> usize {
        self.stats.read().error_count
    }

    /// 获取警告数量
    pub fn warning_count(&self) -> usize {
        self.stats.read().warning_count
    }

    /// 清除所有诊断信息
    pub fn clear_all(&self) {
        let mut diagnostics = self.diagnostics.write();
        let mut stats = self.stats.write();

        diagnostics.clear();
        *stats = DiagnosticStats::default();
    }
}

/// 诊断构建器
pub struct DiagnosticBuilder {
    diagnostic: DiagnosticInfo,
}

impl DiagnosticBuilder {
    /// 创建新的诊断构建器
    pub fn new(severity: Severity, message: String) -> Self {
        Self {
            diagnostic: DiagnosticInfo {
                id: uuid::Uuid::new_v4().to_string(),
                severity,
                category: DiagnosticCategory::Syntax,
                message,
                detail: None,
                file_path: None,
                range: None,
                source_span: None,
                related_information: Vec::new(),
                fixes: Vec::new(),
                tags: Vec::new(),
                code: None,
                timestamp: Instant::now(),
            },
        }
    }

    /// 设置类别
    pub fn category(mut self, category: DiagnosticCategory) -> Self {
        self.diagnostic.category = category;
        self
    }

    /// 设置详细描述
    pub fn detail(mut self, detail: String) -> Self {
        self.diagnostic.detail = Some(detail);
        self
    }

    /// 设置文件路径
    pub fn file_path(mut self, file_path: PathBuf) -> Self {
        self.diagnostic.file_path = Some(file_path);
        self
    }

    /// 设置范围
    pub fn range(mut self, range: Range) -> Self {
        self.diagnostic.range = Some(range);
        self
    }

    /// 设置源码片段
    pub fn source_span(mut self, source_span: SourceSpan) -> Self {
        self.diagnostic.source_span = Some(source_span);
        self
    }

    /// 添加相关信息
    pub fn related_information(mut self, info: RelatedInformation) -> Self {
        self.diagnostic.related_information.push(info);
        self
    }

    /// 添加修复建议
    pub fn fix(mut self, fix: DiagnosticFix) -> Self {
        self.diagnostic.fixes.push(fix);
        self
    }

    /// 添加标签
    pub fn tag(mut self, tag: DiagnosticTag) -> Self {
        self.diagnostic.tags.push(tag);
        self
    }

    /// 设置错误代码
    pub fn code(mut self, code: String) -> Self {
        self.diagnostic.code = Some(code);
        self
    }

    /// 构建诊断信息
    pub fn build(self) -> DiagnosticInfo {
        self.diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DiagnosticsConfig;
    use std::path::PathBuf;

    #[test]
    fn test_diagnostic_builder() {
        let diagnostic = DiagnosticBuilder::new(Severity::Error, "Test error message".to_string())
            .category(DiagnosticCategory::Type)
            .detail("This is a test error".to_string())
            .file_path(PathBuf::from("test.ny"))
            .code("E001".to_string())
            .build();

        assert_eq!(diagnostic.severity, Severity::Error);
        assert_eq!(diagnostic.category, DiagnosticCategory::Type);
        assert_eq!(diagnostic.message, "Test error message");
        assert_eq!(diagnostic.detail, Some("This is a test error".to_string()));
        assert_eq!(diagnostic.code, Some("E001".to_string()));
    }

    #[test]
    fn test_diagnostics_manager() {
        let config = DiagnosticsConfig::default();
        let manager = DiagnosticsManager::new(config);

        let diagnostic = DiagnosticBuilder::new(Severity::Warning, "Test warning".to_string()).build();

        manager.add_diagnostic("test.ny", diagnostic.clone());

        let file_diagnostics = manager.get_file_diagnostics("test.ny");
        assert_eq!(file_diagnostics.len(), 1);
        assert_eq!(file_diagnostics[0].message, "Test warning");

        let stats = manager.get_stats();
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.total_count(), 1);
    }

    #[test]
    fn test_diagnostic_filter() {
        let filter = DiagnosticFilter {
            min_severity: Some(Severity::Warning),
            include_categories: Some(vec![DiagnosticCategory::Type]),
            ..Default::default()
        };

        let error_diagnostic =
            DiagnosticBuilder::new(Severity::Error, "Error message".to_string()).category(DiagnosticCategory::Type).build();

        let info_diagnostic = DiagnosticBuilder::new(Severity::Information, "Info message".to_string())
            .category(DiagnosticCategory::Type)
            .build();

        let syntax_error =
            DiagnosticBuilder::new(Severity::Error, "Syntax error".to_string()).category(DiagnosticCategory::Syntax).build();

        assert!(filter.matches(&error_diagnostic));
        assert!(!filter.matches(&info_diagnostic)); // 严重程度不够
        assert!(!filter.matches(&syntax_error)); // 类别不匹配
    }

    #[test]
    fn test_diagnostic_stats() {
        let mut stats = DiagnosticStats::default();

        let error = DiagnosticBuilder::new(Severity::Error, "Error".to_string()).build();

        let warning = DiagnosticBuilder::new(Severity::Warning, "Warning".to_string()).build();

        stats.add_diagnostic(&error);
        stats.add_diagnostic(&warning);

        assert_eq!(stats.error_count, 1);
        assert_eq!(stats.warning_count, 1);
        assert_eq!(stats.total_count(), 2);
        assert!(stats.has_errors());
        assert!(stats.has_warnings());

        stats.remove_diagnostic(&error);
        assert_eq!(stats.error_count, 0);
        assert!(!stats.has_errors());
    }

    #[test]
    fn test_formatters() {
        let diagnostic = DiagnosticBuilder::new(Severity::Error, "Test error".to_string())
            .file_path(PathBuf::from("test.ny"))
            .code("E001".to_string())
            .build();

        let simple_formatter = SimpleTextFormatter;
        let formatted = simple_formatter.format_diagnostic(&diagnostic);
        assert!(formatted.contains("error:"));
        assert!(formatted.contains("Test error"));
        assert!(formatted.contains("E001"));

        let json_formatter = JsonFormatter;
        let json_formatted = json_formatter.format_diagnostic(&diagnostic);
        assert!(json_formatted.contains("\"severity\":"));
        assert!(json_formatted.contains("\"message\":"));
    }
}

//! 诊断信息管理
//!
//! 将 Valkyrie 编译器的诊断信息转换为 LSP 格式

use core::range::Range;
use oak_lsp::types::{Diagnostic, DiagnosticSeverity};
use valkyrie_types::{HelpMessage, LabeledSpan, ReportKind, ValkyrieError, ValkyrieErrorKind};

/// 错误码前缀常量
const ERROR_CODE_PREFIX: &str = "E";

/// 错误码范围定义
pub mod error_codes {
    /// I/O 错误起始码
    pub const IO_ERROR_START: u32 = 0x0001;
    /// 解析错误起始码
    pub const PARSE_ERROR_START: u32 = 0x0002;
    /// 类型错误起始码
    pub const TYPE_ERROR_START: u32 = 0x0003;
    /// 运行时错误起始码
    pub const RUNTIME_ERROR_START: u32 = 0x0004;
    /// 语法错误起始码
    pub const SYNTAX_ERROR_START: u32 = 0x0005;
    /// 编译错误起始码
    pub const COMPILE_ERROR_START: u32 = 0x2001;
    /// VM 错误起始码
    pub const VM_ERROR_START: u32 = 0x1000;
    /// 未知错误码
    pub const UNKNOWN_ERROR: u32 = 0xFFFF;
}

/// 诊断信息管理器
///
/// 负责将 Valkyrie 编译器的诊断信息转换为 LSP 标准格式
pub struct DiagnosticsManager {
    /// 诊断过滤配置
    filter_config: DiagnosticFilterConfig,
}

impl DiagnosticsManager {
    /// 创建新的诊断管理器
    pub fn new() -> Self {
        Self { filter_config: DiagnosticFilterConfig::default() }
    }

    /// 使用指定配置创建诊断管理器
    pub fn with_config(config: DiagnosticFilterConfig) -> Self {
        Self { filter_config: config }
    }

    /// 将 Valkyrie 编译器诊断信息转换为 LSP 诊断信息
    ///
    /// # 参数
    /// - `compiler_diagnostics`: 编译器产生的诊断信息列表
    /// - `source`: 源代码文本
    ///
    /// # 返回
    /// 转换后的 LSP 诊断信息列表
    pub fn convert_to_lsp_diagnostics(&self, compiler_diagnostics: &[ValkyrieError], source: &str) -> Vec<Diagnostic> {
        let mut result = Vec::new();

        for diag in compiler_diagnostics {
            if self.should_filter(diag) {
                continue;
            }

            if let Some(primary_diagnostic) = self.convert_single_diagnostic(diag, source) {
                result.push(primary_diagnostic);
            }

            for related_diagnostic in self.convert_related_diagnostics(diag, source) {
                result.push(related_diagnostic);
            }
        }

        result
    }

    /// 判断是否应该过滤该诊断
    fn should_filter(&self, diag: &ValkyrieError) -> bool {
        if self.filter_config.ignore_warnings && diag.level == ReportKind::Warning {
            return true;
        }

        if self.filter_config.ignore_hints && diag.level == ReportKind::Help {
            return true;
        }

        if let Some(code) = self.extract_error_code_string(diag) {
            if self.filter_config.excluded_codes.contains(&code) {
                return true;
            }
        }

        false
    }

    /// 转换单个诊断信息
    fn convert_single_diagnostic(&self, diag: &ValkyrieError, source: &str) -> Option<Diagnostic> {
        let range = self.extract_primary_range(diag, source)?;
        let severity = self.map_severity(diag);
        let code = self.extract_error_code_string(diag);
        let message = self.build_diagnostic_message(diag);

        Some(Diagnostic { range, severity: Some(severity), code, source: Some("valkyrie".to_string()), message })
    }

    /// 转换相关的诊断信息（次要标签）
    fn convert_related_diagnostics(&self, diag: &ValkyrieError, source: &str) -> Vec<Diagnostic> {
        let mut related = Vec::new();

        for label in &diag.labels {
            if !label.primary {
                if let Some(diagnostic) = self.convert_secondary_label(diag, label, source) {
                    related.push(diagnostic);
                }
            }
        }

        if let Some(ref help) = diag.help {
            if let Some(diagnostic) = self.convert_help_message(diag, help, source) {
                related.push(diagnostic);
            }
        }

        related
    }

    /// 转换次要标签为诊断信息
    fn convert_secondary_label(&self, diag: &ValkyrieError, label: &LabeledSpan, source: &str) -> Option<Diagnostic> {
        let range = self.label_to_range(label, source)?;
        let severity = self.map_secondary_severity(diag);
        let message = self.build_label_message(label);

        Some(Diagnostic { range, severity: Some(severity), code: None, source: Some("valkyrie".to_string()), message })
    }

    /// 转换帮助信息为诊断
    fn convert_help_message(&self, diag: &ValkyrieError, help: &HelpMessage, source: &str) -> Option<Diagnostic> {
        let range = self.extract_primary_range(diag, source)?;
        let message = self.build_help_message(help);

        Some(Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::Hint),
            code: None,
            source: Some("valkyrie".to_string()),
            message,
        })
    }

    /// 提取主要位置范围
    fn extract_primary_range(&self, diag: &ValkyrieError, source: &str) -> Option<Range<usize>> {
        for label in &diag.labels {
            if label.primary {
                return self.label_to_range(label, source);
            }
        }

        diag.labels.first().and_then(|label| self.label_to_range(label, source))
    }

    /// 将标签转换为范围
    fn label_to_range(&self, label: &LabeledSpan, _source: &str) -> Option<Range<usize>> {
        let start_offset = label.span.get_start() as usize;
        let end_offset = label.span.get_end() as usize;

        if start_offset > end_offset {
            return None;
        }

        Some(Range { start: start_offset, end: end_offset })
    }

    /// 映射诊断严重程度
    fn map_severity(&self, diag: &ValkyrieError) -> DiagnosticSeverity {
        match diag.level {
            ReportKind::Error => DiagnosticSeverity::Error,
            ReportKind::Warning => DiagnosticSeverity::Warning,
            ReportKind::Note => DiagnosticSeverity::Information,
            ReportKind::Help => DiagnosticSeverity::Hint,
        }
    }

    /// 映射次要标签的严重程度
    fn map_secondary_severity(&self, diag: &ValkyrieError) -> DiagnosticSeverity {
        match diag.level {
            ReportKind::Error => DiagnosticSeverity::Information,
            ReportKind::Warning => DiagnosticSeverity::Hint,
            ReportKind::Note => DiagnosticSeverity::Hint,
            ReportKind::Help => DiagnosticSeverity::Hint,
        }
    }

    /// 提取错误码字符串
    fn extract_error_code_string(&self, diag: &ValkyrieError) -> Option<String> {
        let code = diag.code();
        Some(format!("{}{:04X}", ERROR_CODE_PREFIX, code))
    }

    /// 构建诊断消息
    fn build_diagnostic_message(&self, diag: &ValkyrieError) -> String {
        let mut message = String::new();

        let kind_message = self.format_error_kind(&diag.kind);
        message.push_str(&kind_message);

        for label in &diag.labels {
            if label.primary {
                if let Some(ref key) = label.key {
                    message.push_str("\n  ");
                    message.push_str(key);
                }

                for (k, v) in &label.data {
                    message.push_str(&format!("\n    {}: {}", k, v));
                }
            }
        }

        message
    }

    /// 格式化错误类型消息
    fn format_error_kind(&self, kind: &ValkyrieErrorKind) -> String {
        match kind {
            ValkyrieErrorKind::IoError { message, path } => {
                if let Some(p) = path {
                    format!("I/O 错误: {} (路径: {})", message, p)
                }
                else {
                    format!("I/O 错误: {}", message)
                }
            }
            ValkyrieErrorKind::ParseError { message } => {
                format!("解析错误: {}", message)
            }
            ValkyrieErrorKind::SyntaxError { message } => {
                format!("语法错误: {}", message)
            }
            ValkyrieErrorKind::TypeError { expected, found } => {
                format!("类型错误: 期望 '{}', 实际 '{}'", expected, found)
            }
            ValkyrieErrorKind::RuntimeError { message } => {
                format!("运行时错误: {}", message)
            }
            ValkyrieErrorKind::VmError { code, key, message } => {
                format!("虚拟机错误 [E{:04X}]: {} - {}", code, key, message)
            }
            ValkyrieErrorKind::CompileError { message } => {
                format!("编译错误: {}", message)
            }
            ValkyrieErrorKind::Unknown => "未知错误".to_string(),
        }
    }

    /// 构建标签消息
    fn build_label_message(&self, label: &LabeledSpan) -> String {
        let mut message = String::new();

        if let Some(ref key) = label.key {
            message.push_str(key);
        }
        else {
            message.push_str("备注");
        }

        for (k, v) in &label.data {
            message.push_str(&format!("\n  {}: {}", k, v));
        }

        message
    }

    /// 构建帮助消息
    fn build_help_message(&self, help: &HelpMessage) -> String {
        let mut message = format!("提示: {}", help.key);

        for (k, v) in &help.data {
            message.push_str(&format!("\n  {}: {}", k, v));
        }

        message
    }

    /// 计算诊断统计信息
    pub fn compute_stats(&self, diagnostics: &[Diagnostic]) -> DiagnosticStats {
        let mut stats = DiagnosticStats::default();

        for diag in diagnostics {
            if let Some(severity) = &diag.severity {
                match severity {
                    DiagnosticSeverity::Error => stats.errors += 1,
                    DiagnosticSeverity::Warning => stats.warnings += 1,
                    DiagnosticSeverity::Information => stats.infos += 1,
                    DiagnosticSeverity::Hint => stats.hints += 1,
                }
            }
        }

        stats
    }
}

impl Default for DiagnosticsManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 诊断统计信息
#[derive(Debug, Clone, Default)]
pub struct DiagnosticStats {
    /// 错误数量
    pub errors: usize,
    /// 警告数量
    pub warnings: usize,
    /// 信息数量
    pub infos: usize,
    /// 提示数量
    pub hints: usize,
}

impl DiagnosticStats {
    /// 创建新的统计信息
    pub fn new() -> Self {
        Self::default()
    }

    /// 获取总问题数量
    pub fn total(&self) -> usize {
        self.errors + self.warnings + self.infos + self.hints
    }

    /// 是否有任何错误
    pub fn has_errors(&self) -> bool {
        self.errors > 0
    }

    /// 是否有任何问题
    pub fn has_any(&self) -> bool {
        self.total() > 0
    }
}

/// 诊断过滤配置
#[derive(Debug, Clone)]
pub struct DiagnosticFilterConfig {
    /// 是否忽略警告
    pub ignore_warnings: bool,
    /// 是否忽略提示
    pub ignore_hints: bool,
    /// 排除的错误代码
    pub excluded_codes: Vec<String>,
}

impl Default for DiagnosticFilterConfig {
    fn default() -> Self {
        Self { ignore_warnings: false, ignore_hints: false, excluded_codes: Vec::new() }
    }
}

impl DiagnosticFilterConfig {
    /// 创建新的过滤配置
    pub fn new() -> Self {
        Self::default()
    }

    /// 设置是否忽略警告
    pub fn with_ignore_warnings(mut self, ignore: bool) -> Self {
        self.ignore_warnings = ignore;
        self
    }

    /// 设置是否忽略提示
    pub fn with_ignore_hints(mut self, ignore: bool) -> Self {
        self.ignore_hints = ignore;
        self
    }

    /// 添加排除的错误代码
    pub fn with_excluded_code(mut self, code: impl Into<String>) -> Self {
        self.excluded_codes.push(code.into());
        self
    }
}

/// 错误码工具函数
pub mod error_code_utils {
    use super::*;

    /// 解析错误码字符串
    pub fn parse_error_code(code: &str) -> Option<u32> {
        if code.starts_with(ERROR_CODE_PREFIX) {
            u32::from_str_radix(&code[1..], 16).ok()
        }
        else {
            None
        }
    }

    /// 获取错误码对应的错误类型名称
    pub fn get_error_category(code: u32) -> &'static str {
        match code {
            error_codes::IO_ERROR_START => "I/O 错误",
            error_codes::PARSE_ERROR_START => "解析错误",
            error_codes::TYPE_ERROR_START => "类型错误",
            error_codes::RUNTIME_ERROR_START => "运行时错误",
            error_codes::SYNTAX_ERROR_START => "语法错误",
            error_codes::COMPILE_ERROR_START..=0x2FFF => "编译错误",
            error_codes::VM_ERROR_START..=0x1FFF => "虚拟机错误",
            error_codes::UNKNOWN_ERROR => "未知错误",
            _ => "其他错误",
        }
    }

    /// 格式化错误码
    pub fn format_error_code(code: u32) -> String {
        format!("{}{:04X}", ERROR_CODE_PREFIX, code)
    }
}


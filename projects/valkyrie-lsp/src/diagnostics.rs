//! 诊断信息管理
//!
//! 将 Nyar 编译器的诊断信息转换为 LSP 格式

use nyar_error::NyarError;
use tower_lsp::lsp_types::*;
use tracing::{debug, warn};

/// 诊断信息管理器
pub struct DiagnosticsManager {
    // 可以添加诊断信息的缓存和过滤逻辑
}

impl DiagnosticsManager {
    pub fn new() -> Self {
        Self {}
    }

    /// 将 Nyar 编译器诊断信息转换为 LSP 诊断信息
    pub fn convert_to_lsp_diagnostics(&self, compiler_diagnostics: &[NyarError]) -> Vec<Diagnostic> {
        compiler_diagnostics.iter().filter_map(|diag| self.convert_single_diagnostic(diag)).collect()
    }

    /// 转换单个诊断信息
    fn convert_single_diagnostic(&self, diag: &NyarError) -> Option<Diagnostic> {
        // 获取诊断信息的位置
        let range = self.extract_range_from_diagnostic(diag)?;

        // 确定诊断严重程度
        let severity = self.map_severity(diag);

        // 提取错误代码
        let code = self.extract_error_code(diag);

        // 构建 LSP 诊断信息
        Some(Diagnostic {
            range,
            severity: Some(severity),
            code: code.map(NumberOrString::String),
            code_description: None,
            source: Some("valkyrie-lsp".to_string()),
            message: diag.message().to_string(),
            related_information: self.extract_related_information(diag),
            tags: self.extract_diagnostic_tags(diag),
            data: None,
        })
    }

    /// 从诊断信息中提取位置范围
    fn extract_range_from_diagnostic(&self, diag: &NyarError) -> Option<Range> {
        // 这里需要根据 CompilerDiagnostic 的实际结构来实现
        // 目前返回一个默认范围

        // TODO: 实际的位置提取逻辑
        // let span = diag.span()?;
        // let start_pos = self.offset_to_position(span.start)?;
        // let end_pos = self.offset_to_position(span.end)?;

        // 模拟返回
        Some(Range::new(Position::new(0, 0), Position::new(0, 10)))
    }

    /// 映射诊断严重程度
    fn map_severity(&self, diag: &NyarError) -> DiagnosticSeverity {
        // 根据 CompilerDiagnostic 的类型映射严重程度
        match diag.severity() {
            nyar_error::Severity::Error => DiagnosticSeverity::ERROR,
            nyar_error::Severity::Warning => DiagnosticSeverity::WARNING,
            nyar_error::Severity::Info => DiagnosticSeverity::INFORMATION,
            nyar_error::Severity::Hint => DiagnosticSeverity::HINT,
        }
    }

    /// 提取错误代码
    fn extract_error_code(&self, diag: &NyarError) -> Option<String> {
        // TODO: 从 CompilerDiagnostic 中提取错误代码
        diag.code().map(|code| code.to_string())
    }

    /// 提取相关信息
    fn extract_related_information(&self, diag: &NyarError) -> Option<Vec<DiagnosticRelatedInformation>> {
        // TODO: 从 CompilerDiagnostic 中提取相关信息
        // let related = diag.related_information();

        None // 暂时返回 None
    }

    /// 提取诊断标签
    fn extract_diagnostic_tags(&self, diag: &NyarError) -> Option<Vec<DiagnosticTag>> {
        let mut tags = Vec::new();

        // 根据诊断类型添加标签
        if self.is_deprecated_warning(diag) {
            tags.push(DiagnosticTag::DEPRECATED);
        }

        if self.is_unnecessary_code(diag) {
            tags.push(DiagnosticTag::UNNECESSARY);
        }

        if tags.is_empty() {
            None
        }
        else {
            Some(tags)
        }
    }

    /// 检查是否为弃用警告
    fn is_deprecated_warning(&self, diag: &NyarError) -> bool {
        // TODO: 实际的弃用检查逻辑
        diag.message().contains("deprecated")
    }

    /// 检查是否为不必要的代码
    fn is_unnecessary_code(&self, diag: &NyarError) -> bool {
        // TODO: 实际的不必要代码检查逻辑
        diag.message().contains("unused") || diag.message().contains("dead code")
    }

    /// 将字节偏移转换为 LSP 位置
    fn offset_to_position(&self, _offset: usize, _text: &str) -> Option<Position> {
        // TODO: 实现字节偏移到行列位置的转换
        // 这需要根据文本内容计算行号和列号

        Some(Position::new(0, 0)) // 暂时返回默认位置
    }

    /// 过滤诊断信息
    pub fn filter_diagnostics(&self, diagnostics: Vec<Diagnostic>, filter_config: &DiagnosticFilterConfig) -> Vec<Diagnostic> {
        diagnostics.into_iter().filter(|diag| self.should_include_diagnostic(diag, filter_config)).collect()
    }

    /// 检查是否应该包含诊断信息
    fn should_include_diagnostic(&self, diag: &Diagnostic, config: &DiagnosticFilterConfig) -> bool {
        // 检查严重程度过滤
        if let Some(severity) = diag.severity {
            if !config.enabled_severities.contains(&severity) {
                return false;
            }
        }

        // 检查错误代码过滤
        if let Some(code) = &diag.code {
            let code_str = match code {
                NumberOrString::Number(n) => n.to_string(),
                NumberOrString::String(s) => s.clone(),
            };

            if config.ignored_codes.contains(&code_str) {
                return false;
            }
        }

        true
    }
}

/// 诊断过滤配置
#[derive(Debug, Clone)]
pub struct DiagnosticFilterConfig {
    pub enabled_severities: Vec<DiagnosticSeverity>,
    pub ignored_codes: Vec<String>,
    pub max_diagnostics_per_file: Option<usize>,
}

impl Default for DiagnosticFilterConfig {
    fn default() -> Self {
        Self {
            enabled_severities: vec![
                DiagnosticSeverity::ERROR,
                DiagnosticSeverity::WARNING,
                DiagnosticSeverity::INFORMATION,
                DiagnosticSeverity::HINT,
            ],
            ignored_codes: Vec::new(),
            max_diagnostics_per_file: Some(100),
        }
    }
}

/// 诊断信息统计
#[derive(Debug, Clone, Default)]
pub struct DiagnosticStats {
    pub error_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    pub hint_count: usize,
}

impl DiagnosticStats {
    pub fn from_diagnostics(diagnostics: &[Diagnostic]) -> Self {
        let mut stats = Self::default();

        for diag in diagnostics {
            match diag.severity {
                Some(DiagnosticSeverity::ERROR) => stats.error_count += 1,
                Some(DiagnosticSeverity::WARNING) => stats.warning_count += 1,
                Some(DiagnosticSeverity::INFORMATION) => stats.info_count += 1,
                Some(DiagnosticSeverity::HINT) => stats.hint_count += 1,
                None => {} // 忽略没有严重程度的诊断
            }
        }

        stats
    }

    pub fn total_count(&self) -> usize {
        self.error_count + self.warning_count + self.info_count + self.hint_count
    }

    pub fn has_errors(&self) -> bool {
        self.error_count > 0
    }
}

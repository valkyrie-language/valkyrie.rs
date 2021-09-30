#![feature(new_range_api)]

mod diagnostics {
    include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/diagnostics.rs"));

    use valkyrie_types::{HelpMessage, LabeledSpan, SourceID, SourceSpan, ValkyrieError, ValkyrieErrorKind};

    fn create_test_span(start: u32, end: u32) -> SourceSpan {
        SourceSpan::new(SourceID(0), start, end)
    }

    fn create_test_error() -> ValkyrieError {
        let span = create_test_span(0, 10);
        let label = LabeledSpan {
            span,
            primary: true,
            key: Some("此处发生错误".to_string()),
            data: vec![("变量".to_string(), "x".to_string())],
        };

        ValkyrieError {
            level: ReportKind::Error,
            kind: ValkyrieErrorKind::TypeError {
                expected: "Integer".to_string(),
                found: "String".to_string(),
            },
            labels: vec![label],
            help: Some(HelpMessage {
                key: "尝试使用类型转换".to_string(),
                data: vec![("建议".to_string(), "使用 `as` 关键字".to_string())],
            }),
        }
    }

    #[test]
    fn test_convert_single_diagnostic() {
        let manager = DiagnosticsManager::new();
        let error = create_test_error();
        let source = "let x: Integer = \"hello\"";

        let diagnostics = manager.convert_to_lsp_diagnostics(&[error], source);

        assert!(!diagnostics.is_empty());
        assert_eq!(diagnostics[0].severity, Some(DiagnosticSeverity::Error));
        assert!(diagnostics[0].code.is_some());
        assert!(diagnostics[0].message.contains("类型错误"));
    }

    #[test]
    fn test_error_code_extraction() {
        let manager = DiagnosticsManager::new();

        let io_error = ValkyrieError::io_error("file not found".to_string(), Some("test.vk".to_string()));
        let parse_error = ValkyrieError::parse_error("unexpected token".to_string());
        let type_error = ValkyrieError::type_error("Int".to_string(), "String".to_string());

        assert_eq!(manager.extract_error_code_string(&io_error), Some("E0001".to_string()));
        assert_eq!(manager.extract_error_code_string(&parse_error), Some("E0002".to_string()));
        assert_eq!(manager.extract_error_code_string(&type_error), Some("E0003".to_string()));
    }

    #[test]
    fn test_severity_mapping() {
        let manager = DiagnosticsManager::new();

        let mut error = create_test_error();

        error.level = ReportKind::Error;
        assert_eq!(manager.map_severity(&error), DiagnosticSeverity::Error);

        error.level = ReportKind::Warning;
        assert_eq!(manager.map_severity(&error), DiagnosticSeverity::Warning);

        error.level = ReportKind::Note;
        assert_eq!(manager.map_severity(&error), DiagnosticSeverity::Information);

        error.level = ReportKind::Help;
        assert_eq!(manager.map_severity(&error), DiagnosticSeverity::Hint);
    }

    #[test]
    fn test_diagnostic_stats() {
        let manager = DiagnosticsManager::new();

        let diagnostics = vec![
            Diagnostic {
                range: Range { start: 0, end: 1 },
                severity: Some(DiagnosticSeverity::Error),
                code: None,
                source: None,
                message: String::new(),
            },
            Diagnostic {
                range: Range { start: 0, end: 1 },
                severity: Some(DiagnosticSeverity::Warning),
                code: None,
                source: None,
                message: String::new(),
            },
            Diagnostic {
                range: Range { start: 0, end: 1 },
                severity: Some(DiagnosticSeverity::Information),
                code: None,
                source: None,
                message: String::new(),
            },
            Diagnostic {
                range: Range { start: 0, end: 1 },
                severity: Some(DiagnosticSeverity::Hint),
                code: None,
                source: None,
                message: String::new(),
            },
        ];

        let stats = manager.compute_stats(&diagnostics);

        assert_eq!(stats.errors, 1);
        assert_eq!(stats.warnings, 1);
        assert_eq!(stats.infos, 1);
        assert_eq!(stats.hints, 1);
        assert_eq!(stats.total(), 4);
    }

    #[test]
    fn test_filter_config() {
        let config = DiagnosticFilterConfig::new()
            .with_ignore_warnings(true)
            .with_excluded_code("E0001");

        assert!(config.ignore_warnings);
        assert!(config.excluded_codes.contains(&"E0001".to_string()));
    }

    #[test]
    fn test_error_code_utils() {
        use super::diagnostics::error_code_utils::*;

        assert_eq!(parse_error_code("E0001"), Some(0x0001));
        assert_eq!(parse_error_code("EFFFF"), Some(0xFFFF));
        assert_eq!(parse_error_code("X0001"), None);

        assert_eq!(get_error_category(0x0001), "I/O 错误");
        assert_eq!(get_error_category(0x0003), "类型错误");
        assert_eq!(get_error_category(0x2001), "编译错误");

        assert_eq!(format_error_code(0x0001), "E0001");
    }
}

//! LSP 服务器能力配置
//!
//! 定义 Valkyrie LSP 服务器支持的功能

use tower_lsp::lsp_types::*;

/// 返回服务器支持的能力
pub fn server_capabilities() -> ServerCapabilities {
    ServerCapabilities {
        // 文本文档同步
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::INCREMENTAL)),

        // 悬停支持
        hover_provider: Some(HoverProviderCapability::Simple(true)),

        // 补全支持
        completion_provider: Some(CompletionOptions {
            resolve_provider: Some(true),
            trigger_characters: Some(vec![
                ".".to_string(),
                "::".to_string(),
                "(".to_string(),
                "[".to_string(),
                "{".to_string(),
            ]),
            all_commit_characters: None,
            work_done_progress_options: WorkDoneProgressOptions::default(),
            completion_item: Some(CompletionOptionsCompletionItem { label_details_support: Some(true) }),
        }),

        // 定义跳转
        definition_provider: Some(OneOf::Left(true)),

        // 类型定义跳转
        type_definition_provider: Some(TypeDefinitionProviderCapability::Simple(true)),

        // 实现跳转
        implementation_provider: Some(ImplementationProviderCapability::Simple(true)),

        // 引用查找
        references_provider: Some(OneOf::Left(true)),

        // 文档高亮
        document_highlight_provider: Some(OneOf::Left(true)),

        // 文档符号
        document_symbol_provider: Some(OneOf::Left(true)),

        // 工作区符号
        workspace_symbol_provider: Some(OneOf::Left(true)),

        // 代码操作
        code_action_provider: Some(CodeActionProviderCapability::Options(CodeActionOptions {
            code_action_kinds: Some(vec![
                CodeActionKind::QUICKFIX,
                CodeActionKind::REFACTOR,
                CodeActionKind::REFACTOR_EXTRACT,
                CodeActionKind::REFACTOR_INLINE,
                CodeActionKind::REFACTOR_REWRITE,
                CodeActionKind::SOURCE,
                CodeActionKind::SOURCE_ORGANIZE_IMPORTS,
            ]),
            resolve_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),

        // 代码镜头
        code_lens_provider: Some(CodeLensOptions { resolve_provider: Some(true) }),

        // 文档格式化
        document_formatting_provider: Some(OneOf::Left(true)),

        // 范围格式化
        document_range_formatting_provider: Some(OneOf::Left(true)),

        // 输入时格式化
        document_on_type_formatting_provider: Some(DocumentOnTypeFormattingOptions {
            first_trigger_character: "}".to_string(),
            more_trigger_character: Some(vec![";".to_string(), "\n".to_string()]),
        }),

        // 重命名
        rename_provider: Some(OneOf::Right(RenameOptions {
            prepare_provider: Some(true),
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),

        // 折叠范围
        folding_range_provider: Some(FoldingRangeProviderCapability::Simple(true)),

        // 选择范围
        selection_range_provider: Some(SelectionRangeProviderCapability::Simple(true)),

        // 语义标记
        semantic_tokens_provider: Some(SemanticTokensServerCapabilities::SemanticTokensOptions(SemanticTokensOptions {
            work_done_progress_options: WorkDoneProgressOptions::default(),
            legend: SemanticTokensLegend { token_types: semantic_token_types(), token_modifiers: semantic_token_modifiers() },
            range: Some(true),
            full: Some(SemanticTokensFullOptions::Bool(true)),
        })),

        // 内联提示
        inlay_hint_provider: Some(OneOf::Left(true)),

        // 诊断
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(DiagnosticOptions {
            identifier: Some("valkyrie-lsp".to_string()),
            inter_file_dependencies: true,
            workspace_diagnostics: true,
            work_done_progress_options: WorkDoneProgressOptions::default(),
        })),

        // 工作区配置
        workspace: Some(WorkspaceServerCapabilities {
            workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                supported: Some(true),
                change_notifications: Some(OneOf::Left(true)),
            }),
            file_operations: Some(WorkspaceFileOperationsServerCapabilities {
                did_create: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: Some("file".to_string()),
                        pattern: FileOperationPattern {
                            glob: "**/*.val".to_string(),
                            matches: Some(FileOperationPatternKind::File),
                            options: None,
                        },
                    }],
                }),
                will_create: None,
                did_rename: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: Some("file".to_string()),
                        pattern: FileOperationPattern {
                            glob: "**/*.val".to_string(),
                            matches: Some(FileOperationPatternKind::File),
                            options: None,
                        },
                    }],
                }),
                will_rename: None,
                did_delete: Some(FileOperationRegistrationOptions {
                    filters: vec![FileOperationFilter {
                        scheme: Some("file".to_string()),
                        pattern: FileOperationPattern {
                            glob: "**/*.val".to_string(),
                            matches: Some(FileOperationPatternKind::File),
                            options: None,
                        },
                    }],
                }),
                will_delete: None,
            }),
        }),

        // 其他能力
        call_hierarchy_provider: Some(CallHierarchyServerCapability::Simple(true)),
        linked_editing_range_provider: Some(OneOf::Left(true)),
        moniker_provider: Some(OneOf::Left(true)),

        ..Default::default()
    }
}

/// 语义标记类型
fn semantic_token_types() -> Vec<SemanticTokenType> {
    vec![
        SemanticTokenType::NAMESPACE,
        SemanticTokenType::TYPE,
        SemanticTokenType::CLASS,
        SemanticTokenType::ENUM,
        SemanticTokenType::INTERFACE,
        SemanticTokenType::STRUCT,
        SemanticTokenType::TYPE_PARAMETER,
        SemanticTokenType::PARAMETER,
        SemanticTokenType::VARIABLE,
        SemanticTokenType::PROPERTY,
        SemanticTokenType::ENUM_MEMBER,
        SemanticTokenType::EVENT,
        SemanticTokenType::FUNCTION,
        SemanticTokenType::METHOD,
        SemanticTokenType::MACRO,
        SemanticTokenType::KEYWORD,
        SemanticTokenType::MODIFIER,
        SemanticTokenType::COMMENT,
        SemanticTokenType::STRING,
        SemanticTokenType::NUMBER,
        SemanticTokenType::REGEXP,
        SemanticTokenType::OPERATOR,
    ]
}

/// 语义标记修饰符
fn semantic_token_modifiers() -> Vec<SemanticTokenModifier> {
    vec![
        SemanticTokenModifier::DECLARATION,
        SemanticTokenModifier::DEFINITION,
        SemanticTokenModifier::READONLY,
        SemanticTokenModifier::STATIC,
        SemanticTokenModifier::DEPRECATED,
        SemanticTokenModifier::ABSTRACT,
        SemanticTokenModifier::ASYNC,
        SemanticTokenModifier::MODIFICATION,
        SemanticTokenModifier::DOCUMENTATION,
        SemanticTokenModifier::DEFAULT_LIBRARY,
    ]
}

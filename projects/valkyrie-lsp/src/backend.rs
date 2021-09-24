//! Valkyrie LSP Backend Implementation
//!
//! 基于 Nyar 编译器基础设施的 LSP 后端实现

use dashmap::DashMap;
use parking_lot::RwLock;
use serde_json::Value;
use std::{collections::HashMap, sync::Arc};
use tower_lsp::{
    jsonrpc::{Error, Result},
    lsp_types::*,
    Client, LanguageServer,
};
use tracing::{debug, error, info, warn};

use nyar_ast::AstNode;
use nyar_core::{
    ids::{FileId, ModuleId},
    symbol::{Symbol, SymbolId},
};
use nyar_error::NyarError;
use nyar_hir::HirNode;
use nyar_query::QueryEngine;

use crate::{capabilities::server_capabilities, diagnostics::DiagnosticsManager, state::ServerState};

/// Valkyrie LSP 后端
///
/// 这个结构体实现了 LSP 协议，并使用 Nyar 编译器基础设施
/// 来提供语言服务功能。
pub struct ValkyrieBackend {
    client: Client,
    state: Arc<ServerState>,
    diagnostics: Arc<DiagnosticsManager>,
}

impl ValkyrieBackend {
    pub fn new(client: Client) -> Self {
        Self { client, state: Arc::new(ServerState::new()), diagnostics: Arc::new(DiagnosticsManager::new()) }
    }

    /// 自定义方法：获取 AST
    pub async fn get_ast(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;

        debug!("Getting AST for {}", uri);

        match self.state.get_ast(uri) {
            Some(ast) => {
                // 将 AST 序列化为 JSON
                serde_json::to_value(&ast).map_err(|e| Error::internal_error(format!("Failed to serialize AST: {}", e)))
            }
            None => Err(Error::invalid_request(format!("No AST available for {}", uri))),
        }
    }

    /// 自定义方法：获取 HIR
    pub async fn get_hir(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;

        debug!("Getting HIR for {}", uri);

        match self.state.get_hir(uri) {
            Some(hir) => {
                serde_json::to_value(&hir).map_err(|e| Error::internal_error(format!("Failed to serialize HIR: {}", e)))
            }
            None => Err(Error::invalid_request(format!("No HIR available for {}", uri))),
        }
    }

    /// 自定义方法：查询符号
    pub async fn query_symbol(&self, params: Value) -> Result<Value> {
        let uri = params.get("uri").and_then(|v| v.as_str()).ok_or_else(|| Error::invalid_params("Missing uri parameter"))?;

        let position = params.get("position").ok_or_else(|| Error::invalid_params("Missing position parameter"))?;

        let position: Position =
            serde_json::from_value(position.clone()).map_err(|e| Error::invalid_params(format!("Invalid position: {}", e)))?;

        debug!("Querying symbol at {}:{}:{}", uri, position.line, position.character);

        match self.state.query_symbol_at_position(uri, position) {
            Some(symbol_info) => serde_json::to_value(&symbol_info)
                .map_err(|e| Error::internal_error(format!("Failed to serialize symbol info: {}", e))),
            None => Ok(Value::Null),
        }
    }

    /// 编译文档并更新诊断信息
    async fn compile_and_diagnose(&self, uri: &str, text: &str) {
        debug!("Compiling document: {}", uri);

        match self.state.compile_document(uri, text).await {
            Ok(diagnostics) => {
                // 转换诊断信息为 LSP 格式
                let lsp_diagnostics = self.diagnostics.convert_to_lsp_diagnostics(&diagnostics);

                // 发送诊断信息到客户端
                self.client.publish_diagnostics(Url::parse(uri).unwrap(), lsp_diagnostics, None).await;
            }
            Err(e) => {
                error!("Compilation failed for {}: {}", uri, e);

                // 发送编译错误作为诊断信息
                let error_diagnostic = Diagnostic {
                    range: Range::new(Position::new(0, 0), Position::new(0, 0)),
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("valkyrie-lsp".to_string()),
                    message: format!("Compilation failed: {}", e),
                    related_information: None,
                    tags: None,
                    data: None,
                };

                self.client.publish_diagnostics(Url::parse(uri).unwrap(), vec![error_diagnostic], None).await;
            }
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for ValkyrieBackend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        info!("Initializing Valkyrie LSP Server");

        // 存储客户端能力
        if let Some(capabilities) = params.capabilities.text_document {
            self.state.set_client_capabilities(capabilities).await;
        }

        Ok(InitializeResult {
            capabilities: server_capabilities(),
            server_info: Some(ServerInfo {
                name: "valkyrie-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        info!("Valkyrie LSP Server initialized");
    }

    async fn shutdown(&self) -> Result<()> {
        info!("Shutting down Valkyrie LSP Server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;

        info!("Document opened: {}", uri);

        // 编译文档并发送诊断信息
        self.compile_and_diagnose(&uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        // 获取最新的文档内容
        if let Some(change) = params.content_changes.into_iter().last() {
            debug!("Document changed: {}", uri);

            // 重新编译并发送诊断信息
            self.compile_and_diagnose(&uri, &change.text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        info!("Document closed: {}", uri);

        // 清理文档状态
        self.state.remove_document(&uri).await;

        // 清除诊断信息
        self.client.publish_diagnostics(params.text_document.uri, vec![], None).await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        debug!("Hover request at {}:{}:{}", uri, position.line, position.character);

        match self.state.get_hover_info(&uri, position) {
            Some(hover_info) => Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent { kind: MarkupKind::Markdown, value: hover_info }),
                range: None,
            })),
            None => Ok(None),
        }
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params.text_document_position_params.text_document.uri.to_string();
        let position = params.text_document_position_params.position;

        debug!("Go to definition at {}:{}:{}", uri, position.line, position.character);

        match self.state.get_definition_location(&uri, position) {
            Some(location) => Ok(Some(GotoDefinitionResponse::Scalar(location))),
            None => Ok(None),
        }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        debug!("Completion request at {}:{}:{}", uri, position.line, position.character);

        match self.state.get_completions(&uri, position) {
            Some(completions) => Ok(Some(CompletionResponse::Array(completions))),
            None => Ok(None),
        }
    }
}

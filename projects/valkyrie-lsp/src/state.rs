//! LSP 服务器状态管理
//!
//! 管理文档状态、编译结果和查询缓存

use dashmap::DashMap;
use parking_lot::RwLock;
use std::{collections::HashMap, sync::Arc};
use tower_lsp::lsp_types::*;
use tracing::{debug, error, info};

use nyar_ast::AstNode;
use nyar_core::{
    ids::{FileId, ModuleId},
    symbol::{Symbol, SymbolId},
};
use nyar_error::NyarError;
use nyar_hir::HirNode;
use nyar_query::QueryEngine;

/// 文档编译结果
#[derive(Debug, Clone)]
pub struct DocumentState {
    pub uri: String,
    pub version: i32,
    pub text: String,
    pub file_id: Option<FileId>,
    pub ast: Option<AstNode>,
    pub hir: Option<HirNode>,
    pub diagnostics: Vec<NyarError>,
    pub symbols: HashMap<SymbolId, Symbol>,
}

impl DocumentState {
    pub fn new(uri: String, version: i32, text: String) -> Self {
        Self { uri, version, text, file_id: None, ast: None, hir: None, diagnostics: Vec::new(), symbols: HashMap::new() }
    }
}

/// 符号查询结果
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: String,
    pub type_info: Option<String>,
    pub documentation: Option<String>,
    pub location: Location,
}

/// LSP 服务器状态
pub struct ServerState {
    /// 文档状态缓存
    documents: DashMap<String, DocumentState>,

    /// 查询引擎
    query_engine: Arc<RwLock<QueryEngine>>,

    /// 客户端能力
    client_capabilities: RwLock<Option<TextDocumentClientCapabilities>>,

    /// 工作区根目录
    workspace_root: RwLock<Option<String>>,
}

impl ServerState {
    pub fn new() -> Self {
        Self {
            documents: DashMap::new(),
            query_engine: Arc::new(RwLock::new(QueryEngine::new())),
            client_capabilities: RwLock::new(None),
            workspace_root: RwLock::new(None),
        }
    }

    /// 设置客户端能力
    pub async fn set_client_capabilities(&self, capabilities: TextDocumentClientCapabilities) {
        *self.client_capabilities.write() = Some(capabilities);
    }

    /// 设置工作区根目录
    pub async fn set_workspace_root(&self, root: String) {
        *self.workspace_root.write() = Some(root);
    }

    /// 编译文档
    pub async fn compile_document(
        &self,
        uri: &str,
        text: &str,
    ) -> Result<Vec<NyarError>, Box<dyn std::error::Error + Send + Sync>> {
        debug!("Compiling document: {}", uri);

        // 创建或更新文档状态
        let mut doc_state = DocumentState::new(uri.to_string(), 1, text.to_string());

        // 使用 Nyar 编译器进行编译
        match self.compile_with_nyar(text).await {
            Ok((ast, hir, diagnostics)) => {
                doc_state.ast = Some(ast);
                doc_state.hir = Some(hir);
                doc_state.diagnostics = diagnostics.clone();

                // 更新符号表
                self.update_symbols(&mut doc_state).await;

                // 缓存编译结果
                self.documents.insert(uri.to_string(), doc_state);

                Ok(diagnostics)
            }
            Err(e) => {
                error!("Compilation failed: {}", e);

                // 即使编译失败也要缓存状态
                self.documents.insert(uri.to_string(), doc_state);

                Err(e)
            }
        }
    }

    /// 使用 Nyar 编译器编译代码
    async fn compile_with_nyar(
        &self,
        text: &str,
    ) -> Result<(AstNode, HirNode, Vec<NyarError>), Box<dyn std::error::Error + Send + Sync>> {
        // 这里应该调用 Nyar 编译器的 API
        // 目前返回模拟数据

        // TODO: 实际的编译逻辑
        // let compiler = NyarCompiler::new();
        // let result = compiler.compile_text(text)?;

        // 模拟编译结果
        let ast = AstNode::default(); // 实际应该从编译器获取
        let hir = HirNode::default(); // 实际应该从编译器获取
        let diagnostics = Vec::new(); // 实际应该从编译器获取

        Ok((ast, hir, diagnostics))
    }

    /// 更新符号表
    async fn update_symbols(&self, doc_state: &mut DocumentState) {
        // 从 HIR 中提取符号信息
        if let Some(_hir) = &doc_state.hir {
            // TODO: 实际的符号提取逻辑
            // let symbols = extract_symbols_from_hir(hir);
            // doc_state.symbols = symbols;
        }
    }

    /// 获取 AST
    pub fn get_ast(&self, uri: &str) -> Option<AstNode> {
        self.documents.get(uri).and_then(|doc| doc.ast.clone())
    }

    /// 获取 HIR
    pub fn get_hir(&self, uri: &str) -> Option<HirNode> {
        self.documents.get(uri).and_then(|doc| doc.hir.clone())
    }

    /// 查询位置处的符号
    pub fn query_symbol_at_position(&self, uri: &str, position: Position) -> Option<SymbolInfo> {
        let doc = self.documents.get(uri)?;

        // 使用查询引擎查找符号
        let query_engine = self.query_engine.read();

        // TODO: 实际的符号查询逻辑
        // let symbol = query_engine.find_symbol_at_position(&doc.hir?, position)?;

        // 模拟返回
        Some(SymbolInfo {
            name: "example_symbol".to_string(),
            kind: "function".to_string(),
            type_info: Some("fn() -> i32".to_string()),
            documentation: Some("Example function documentation".to_string()),
            location: Location { uri: Url::parse(uri).ok()?, range: Range::new(position, position) },
        })
    }

    /// 获取悬停信息
    pub fn get_hover_info(&self, uri: &str, position: Position) -> Option<String> {
        let symbol_info = self.query_symbol_at_position(uri, position)?;

        let mut hover_text = format!("**{}**\n\n", symbol_info.name);

        if let Some(type_info) = symbol_info.type_info {
            hover_text.push_str(&format!("Type: `{}`\n\n", type_info));
        }

        if let Some(doc) = symbol_info.documentation {
            hover_text.push_str(&doc);
        }

        Some(hover_text)
    }

    /// 获取定义位置
    pub fn get_definition_location(&self, uri: &str, position: Position) -> Option<Location> {
        let symbol_info = self.query_symbol_at_position(uri, position)?;
        Some(symbol_info.location)
    }

    /// 获取补全建议
    pub fn get_completions(&self, uri: &str, position: Position) -> Option<Vec<CompletionItem>> {
        let _doc = self.documents.get(uri)?;

        // TODO: 实际的补全逻辑
        // 这里应该基于上下文和符号表生成补全建议

        // 模拟补全建议
        Some(vec![
            CompletionItem {
                label: "println".to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("fn println(format: &str, ...)".to_string()),
                documentation: Some(Documentation::String("Print a line to stdout".to_string())),
                insert_text: Some("println!(\"{}\")".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
            CompletionItem {
                label: "let".to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("Variable binding".to_string()),
                insert_text: Some("let ${1:name} = ${2:value};".to_string()),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            },
        ])
    }

    /// 移除文档
    pub async fn remove_document(&self, uri: &str) {
        self.documents.remove(uri);
        info!("Document removed: {}", uri);
    }

    /// 获取所有文档
    pub fn get_all_documents(&self) -> Vec<String> {
        self.documents.iter().map(|entry| entry.key().clone()).collect()
    }
}

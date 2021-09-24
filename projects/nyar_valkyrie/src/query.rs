//! 查询引擎模块
//!
//! 提供符号查询、类型推断、引用查找等功能。

use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::time::timeout;

use nyar_core::{Position, Range, SourceSpan};
use nyar_hir::{HirNode, Module, Symbol, Type};

use crate::{
    config::QueryConfig,
    error::{QueryError, RuntimeResult},
};

/// 符号信息
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    /// 符号名称
    pub name: String,
    /// 符号类型
    pub symbol_type: SymbolType,
    /// 定义位置
    pub definition: Position,
    /// 类型信息
    pub type_info: Option<String>,
    /// 文档注释
    pub documentation: Option<String>,
    /// 可见性
    pub visibility: Visibility,
    /// 所属模块
    pub module: String,
}

/// 符号类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SymbolType {
    Function,
    Variable,
    Type,
    Module,
    Constant,
    Parameter,
    Field,
    Method,
    Trait,
    Enum,
    Struct,
}

/// 可见性
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

/// 查询请求
#[derive(Debug, Clone)]
pub struct QueryRequest {
    /// 查询类型
    pub query_type: QueryType,
    /// 文件标识
    pub file_id: String,
    /// 查询位置
    pub position: Option<Position>,
    /// 查询范围
    pub range: Option<Range>,
    /// 查询参数
    pub parameters: HashMap<String, String>,
}

/// 查询类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryType {
    /// 符号定义
    Definition,
    /// 符号引用
    References,
    /// 类型信息
    TypeInfo,
    /// 补全建议
    Completion,
    /// 悬停信息
    Hover,
    /// 符号搜索
    SymbolSearch,
    /// 工作区符号
    WorkspaceSymbols,
}

/// 查询结果
#[derive(Debug, Clone)]
pub struct QueryResult {
    /// 查询类型
    pub query_type: QueryType,
    /// 结果数据
    pub data: QueryResultData,
    /// 查询耗时
    pub duration: Duration,
}

/// 查询结果数据
#[derive(Debug, Clone)]
pub enum QueryResultData {
    /// 位置列表
    Positions(Vec<Position>),
    /// 符号信息列表
    Symbols(Vec<SymbolInfo>),
    /// 类型信息
    TypeInfo(String),
    /// 悬停信息
    Hover(HoverInfo),
    /// 补全建议
    Completion(Vec<CompletionItem>),
}

/// 悬停信息
#[derive(Debug, Clone)]
pub struct HoverInfo {
    /// 内容
    pub contents: String,
    /// 范围
    pub range: Option<Range>,
}

/// 补全建议项
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// 标签
    pub label: String,
    /// 详细信息
    pub detail: Option<String>,
    /// 文档
    pub documentation: Option<String>,
    /// 插入文本
    pub insert_text: Option<String>,
    /// 补全类型
    pub kind: CompletionKind,
    /// 排序文本
    pub sort_text: Option<String>,
}

/// 补全类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    Text,
    Method,
    Function,
    Constructor,
    Field,
    Variable,
    Class,
    Interface,
    Module,
    Property,
    Unit,
    Value,
    Enum,
    Keyword,
    Snippet,
    Color,
    File,
    Reference,
    Folder,
    EnumMember,
    Constant,
    Struct,
    Event,
    Operator,
    TypeParameter,
}

/// 符号索引
struct SymbolIndex {
    /// 符号表
    symbols: DashMap<String, Vec<SymbolInfo>>,
    /// 位置到符号的映射
    position_map: DashMap<String, DashMap<Position, String>>,
    /// 类型缓存
    type_cache: DashMap<String, String>,
    /// 引用缓存
    reference_cache: DashMap<String, Vec<Position>>,
}

impl SymbolIndex {
    fn new() -> Self {
        Self {
            symbols: DashMap::new(),
            position_map: DashMap::new(),
            type_cache: DashMap::new(),
            reference_cache: DashMap::new(),
        }
    }

    fn add_symbol(&self, file_id: &str, symbol: SymbolInfo) {
        let mut symbols = self.symbols.entry(file_id.to_string()).or_insert_with(Vec::new);

        // 添加位置映射
        let position_map = self.position_map.entry(file_id.to_string()).or_insert_with(DashMap::new);
        position_map.insert(symbol.definition, symbol.name.clone());

        symbols.push(symbol);
    }

    fn get_symbols(&self, file_id: &str) -> Option<Vec<SymbolInfo>> {
        self.symbols.get(file_id).map(|symbols| symbols.clone())
    }

    fn find_symbol_at_position(&self, file_id: &str, position: Position) -> Option<String> {
        self.position_map.get(file_id)?.get(&position).map(|name| name.clone())
    }

    fn clear_file(&self, file_id: &str) {
        self.symbols.remove(file_id);
        self.position_map.remove(file_id);
        // 清除相关的类型和引用缓存
        self.type_cache.retain(|key, _| !key.starts_with(file_id));
        self.reference_cache.retain(|key, _| !key.starts_with(file_id));
    }
}

/// 查询引擎
pub struct QueryEngine {
    /// 配置
    config: QueryConfig,
    /// 符号索引
    index: Arc<SymbolIndex>,
    /// 模块缓存
    modules: Arc<RwLock<HashMap<String, Arc<Module>>>>,
}

impl QueryEngine {
    /// 创建新的查询引擎
    pub async fn new(config: QueryConfig) -> RuntimeResult<Self> {
        Ok(Self { config, index: Arc::new(SymbolIndex::new()), modules: Arc::new(RwLock::new(HashMap::new())) })
    }

    /// 索引模块
    pub async fn index_module(&self, file_id: &str, module: Arc<Module>) -> RuntimeResult<()> {
        // 清除旧的索引
        self.index.clear_file(file_id);

        // 缓存模块
        {
            let mut modules = self.modules.write();
            modules.insert(file_id.to_string(), module.clone());
        }

        // 提取符号信息
        let symbols = self.extract_symbols(&module).await?;

        // 添加到索引
        for symbol in symbols {
            self.index.add_symbol(file_id, symbol);
        }

        Ok(())
    }

    /// 查询符号
    pub async fn query_symbols(&self, module: &Module) -> RuntimeResult<Vec<SymbolInfo>> {
        let start_time = Instant::now();

        let query_future = self.extract_symbols(module);

        let result = if self.config.query_timeout > Duration::ZERO {
            timeout(self.config.query_timeout, query_future)
                .await
                .map_err(|_| QueryError::Timeout { timeout_ms: self.config.query_timeout.as_millis() as u64 })?
        }
        else {
            query_future.await
        };

        result.map_err(Into::into)
    }

    /// 查询指定位置的类型信息
    pub async fn query_type_at_position(&self, module: &Module, position: Position) -> RuntimeResult<Option<String>> {
        // TODO: 实现类型推断逻辑
        // 这里需要遍历 HIR 并找到指定位置的节点，然后推断其类型
        Ok(Some("unknown".to_string()))
    }

    /// 查询符号定义
    pub async fn query_definition(&self, module: &Module, position: Position) -> RuntimeResult<Option<Position>> {
        // TODO: 实现定义查找逻辑
        Ok(None)
    }

    /// 查询符号引用
    pub async fn query_references(&self, module: &Module, position: Position) -> RuntimeResult<Vec<Position>> {
        // TODO: 实现引用查找逻辑
        Ok(vec![])
    }

    /// 查询补全建议
    pub async fn query_completion(&self, module: &Module, position: Position) -> RuntimeResult<Vec<CompletionItem>> {
        // TODO: 实现补全逻辑
        // 这里需要分析上下文并提供合适的补全建议
        Ok(vec![])
    }

    /// 查询悬停信息
    pub async fn query_hover(&self, module: &Module, position: Position) -> RuntimeResult<Option<HoverInfo>> {
        // TODO: 实现悬停信息逻辑
        Ok(None)
    }

    /// 搜索符号
    pub async fn search_symbols(&self, query: &str, limit: Option<usize>) -> RuntimeResult<Vec<SymbolInfo>> {
        let mut results = Vec::new();
        let limit = limit.unwrap_or(100);

        // 遍历所有文件的符号
        for symbols_ref in self.index.symbols.iter() {
            let symbols = symbols_ref.value();
            for symbol in symbols {
                if self.matches_query(&symbol.name, query) {
                    results.push(symbol.clone());
                    if results.len() >= limit {
                        break;
                    }
                }
            }
            if results.len() >= limit {
                break;
            }
        }

        // 按相关性排序
        results.sort_by(|a, b| {
            let a_score = self.calculate_match_score(&a.name, query);
            let b_score = self.calculate_match_score(&b.name, query);
            b_score.partial_cmp(&a_score).unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(results)
    }

    /// 处理查询请求
    pub async fn handle_query(&self, request: QueryRequest) -> RuntimeResult<QueryResult> {
        let start_time = Instant::now();

        let modules = self.modules.read();
        let module =
            modules.get(&request.file_id).ok_or_else(|| QueryError::SymbolNotFound { symbol: request.file_id.clone() })?;

        let data = match request.query_type {
            QueryType::Definition => {
                if let Some(position) = request.position {
                    let pos = self.query_definition(module, position).await?;
                    QueryResultData::Positions(pos.into_iter().collect())
                }
                else {
                    QueryResultData::Positions(vec![])
                }
            }
            QueryType::References => {
                if let Some(position) = request.position {
                    let refs = self.query_references(module, position).await?;
                    QueryResultData::Positions(refs)
                }
                else {
                    QueryResultData::Positions(vec![])
                }
            }
            QueryType::TypeInfo => {
                if let Some(position) = request.position {
                    let type_info =
                        self.query_type_at_position(module, position).await?.unwrap_or_else(|| "unknown".to_string());
                    QueryResultData::TypeInfo(type_info)
                }
                else {
                    QueryResultData::TypeInfo("unknown".to_string())
                }
            }
            QueryType::Completion => {
                if let Some(position) = request.position {
                    let items = self.query_completion(module, position).await?;
                    QueryResultData::Completion(items)
                }
                else {
                    QueryResultData::Completion(vec![])
                }
            }
            QueryType::Hover => {
                if let Some(position) = request.position {
                    let hover = self
                        .query_hover(module, position)
                        .await?
                        .unwrap_or_else(|| HoverInfo { contents: "No information available".to_string(), range: None });
                    QueryResultData::Hover(hover)
                }
                else {
                    QueryResultData::Hover(HoverInfo { contents: "No information available".to_string(), range: None })
                }
            }
            QueryType::SymbolSearch => {
                let query = request.parameters.get("query").unwrap_or(&String::new());
                let limit = request.parameters.get("limit").and_then(|s| s.parse().ok());
                let symbols = self.search_symbols(query, limit).await?;
                QueryResultData::Symbols(symbols)
            }
            QueryType::WorkspaceSymbols => {
                let symbols = self.query_symbols(module).await?;
                QueryResultData::Symbols(symbols)
            }
        };

        Ok(QueryResult { query_type: request.query_type, data, duration: start_time.elapsed() })
    }

    // 私有辅助方法

    /// 提取模块中的符号
    async fn extract_symbols(&self, module: &Module) -> Result<Vec<SymbolInfo>, QueryError> {
        let mut symbols = Vec::new();

        // 提取函数符号
        for function in &module.functions {
            symbols.push(SymbolInfo {
                name: function.name.clone(),
                symbol_type: SymbolType::Function,
                definition: function.span.start,
                type_info: Some(format!("fn({})", function.parameters.len())),
                documentation: function.documentation.clone(),
                visibility: Visibility::Public, // TODO: 从 HIR 中获取实际可见性
                module: module.name.clone(),
            });

            // 提取参数符号
            for param in &function.parameters {
                symbols.push(SymbolInfo {
                    name: param.name.clone(),
                    symbol_type: SymbolType::Parameter,
                    definition: param.span.start,
                    type_info: Some(param.type_name.clone()),
                    documentation: None,
                    visibility: Visibility::Private,
                    module: module.name.clone(),
                });
            }
        }

        // TODO: 提取其他类型的符号（变量、类型、常量等）

        Ok(symbols)
    }

    /// 检查查询是否匹配符号名称
    fn matches_query(&self, symbol_name: &str, query: &str) -> bool {
        if !self.config.enable_fuzzy_matching {
            return symbol_name.contains(query);
        }

        let score = self.calculate_match_score(symbol_name, query);
        score >= self.config.fuzzy_match_threshold
    }

    /// 计算匹配分数
    fn calculate_match_score(&self, symbol_name: &str, query: &str) -> f64 {
        if symbol_name == query {
            return 1.0;
        }

        if symbol_name.starts_with(query) {
            return 0.9;
        }

        if symbol_name.to_lowercase().contains(&query.to_lowercase()) {
            return 0.7;
        }

        // 简单的模糊匹配算法
        let mut score = 0.0;
        let mut query_chars = query.chars().peekable();

        for ch in symbol_name.chars() {
            if let Some(&query_ch) = query_chars.peek() {
                if ch.to_lowercase().eq(query_ch.to_lowercase()) {
                    score += 1.0;
                    query_chars.next();
                }
            }
        }

        score / query.len() as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::QueryConfig;

    #[tokio::test]
    async fn test_query_engine_creation() {
        let config = QueryConfig::default();
        let engine = QueryEngine::new(config).await;
        assert!(engine.is_ok());
    }

    #[tokio::test]
    async fn test_symbol_search() {
        let config = QueryConfig::default();
        let engine = QueryEngine::new(config).await.unwrap();

        let results = engine.search_symbols("test", Some(10)).await.unwrap();
        assert!(results.is_empty()); // 没有索引任何符号
    }

    #[test]
    fn test_match_score() {
        let config = QueryConfig::default();
        let engine = QueryEngine::new(config).await.unwrap();

        assert_eq!(engine.calculate_match_score("test", "test"), 1.0);
        assert!(engine.calculate_match_score("test_function", "test") > 0.8);
        assert!(engine.calculate_match_score("function_test", "test") > 0.5);
    }

    #[test]
    fn test_symbol_index() {
        let index = SymbolIndex::new();

        let symbol = SymbolInfo {
            name: "test_function".to_string(),
            symbol_type: SymbolType::Function,
            definition: Position { line: 1, column: 0 },
            type_info: Some("fn()".to_string()),
            documentation: None,
            visibility: Visibility::Public,
            module: "test".to_string(),
        };

        index.add_symbol("test.ny", symbol.clone());

        let symbols = index.get_symbols("test.ny").unwrap();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0].name, "test_function");

        let found = index.find_symbol_at_position("test.ny", Position { line: 1, column: 0 });
        assert_eq!(found, Some("test_function".to_string()));
    }
}

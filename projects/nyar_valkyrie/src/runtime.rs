//! 核心运行时实现
//!
//! 提供统一的运行时接口，整合解释器、查询引擎、代码生成等功能。

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use dashmap::DashMap;
use parking_lot::RwLock;
use tokio::sync::RwLock as AsyncRwLock;

use nyar_ast::{AstNode, Program};
use nyar_core::{Position, Range, SourceFile};
use nyar_error::NyarError;
use nyar_hir::{HirNode, Module};

use crate::{
    cache::RuntimeCache,
    codegen::{CodegenEngine, CodegenTarget, GeneratedCode},
    config::RuntimeConfig,
    diagnostics::DiagnosticsManager,
    error::RuntimeError,
    interpreter::{InterpreterEngine, InterpreterResult},
    query::{QueryEngine, QueryRequest, QueryResult, SymbolInfo},
};

/// 文件标识符
pub type FileId = String;

/// 核心运行时结构体
///
/// 提供编译器基础设施的统一访问接口，支持：
/// - 源码管理和编译
/// - AST/HIR 生成和缓存
/// - 解释执行
/// - 符号查询
/// - 代码生成
/// - 诊断信息管理
pub struct Runtime {
    /// 运行时配置
    config: RuntimeConfig,

    /// 源文件管理
    source_files: Arc<DashMap<FileId, SourceFile>>,

    /// 编译缓存
    cache: Arc<RuntimeCache>,

    /// 诊断管理器
    diagnostics: Arc<DiagnosticsManager>,

    /// 解释器引擎
    #[cfg(feature = "interpreter")]
    interpreter: Arc<AsyncRwLock<InterpreterEngine>>,

    /// 查询引擎
    query_engine: Arc<QueryEngine>,

    /// 代码生成引擎
    codegen_engine: Arc<CodegenEngine>,

    /// 工作区根目录
    workspace_root: Option<PathBuf>,
}

impl Runtime {
    /// 创建新的运行时实例
    pub async fn new() -> Result<Self, RuntimeError> {
        Self::with_config(RuntimeConfig::default()).await
    }

    /// 使用指定配置创建运行时实例
    pub async fn with_config(config: RuntimeConfig) -> Result<Self, RuntimeError> {
        let source_files = Arc::new(DashMap::new());
        let cache = Arc::new(RuntimeCache::new(config.cache_config.clone()));
        let diagnostics = Arc::new(DiagnosticsManager::new(config.diagnostics_config.clone()));

        #[cfg(feature = "interpreter")]
        let interpreter = Arc::new(AsyncRwLock::new(InterpreterEngine::new(config.interpreter_config.clone()).await?));

        let query_engine = Arc::new(QueryEngine::new(config.query_config.clone()).await?);
        let codegen_engine = Arc::new(CodegenEngine::new(config.codegen_config.clone()).await?);

        Ok(Self {
            config,
            source_files,
            cache,
            diagnostics,
            #[cfg(feature = "interpreter")]
            interpreter,
            query_engine,
            codegen_engine,
            workspace_root: None,
        })
    }

    /// 设置工作区根目录
    pub fn set_workspace_root<P: AsRef<Path>>(&mut self, root: P) {
        self.workspace_root = Some(root.as_ref().to_path_buf());
    }

    /// 获取工作区根目录
    pub fn workspace_root(&self) -> Option<&Path> {
        self.workspace_root.as_deref()
    }

    /// 添加源文件
    pub async fn add_source(&self, file_id: &str, content: &str) -> Result<FileId, RuntimeError> {
        let source_file = SourceFile::new(file_id.to_string(), content.to_string());
        let file_id = source_file.id().clone();

        self.source_files.insert(file_id.clone(), source_file);

        // 触发编译
        self.compile_file(&file_id).await?;

        Ok(file_id)
    }

    /// 添加源文件从路径
    pub async fn add_source_file<P: AsRef<Path>>(&self, path: P) -> Result<FileId, RuntimeError> {
        let path = path.as_ref();
        let content = tokio::fs::read_to_string(path).await.map_err(|e| RuntimeError::IoError(e.to_string()))?;

        let file_id = path.to_string_lossy().to_string();
        self.add_source(&file_id, &content).await
    }

    /// 更新源文件内容
    pub async fn update_source(&self, file_id: &str, content: &str) -> Result<(), RuntimeError> {
        if let Some(mut source_file) = self.source_files.get_mut(file_id) {
            source_file.update_content(content.to_string());

            // 清除相关缓存
            self.cache.invalidate_file(file_id).await;

            // 重新编译
            self.compile_file(file_id).await?;

            Ok(())
        }
        else {
            Err(RuntimeError::FileNotFound(file_id.to_string()))
        }
    }

    /// 移除源文件
    pub async fn remove_source(&self, file_id: &str) -> Result<(), RuntimeError> {
        self.source_files.remove(file_id);
        self.cache.invalidate_file(file_id).await;
        self.diagnostics.clear_diagnostics(file_id).await;
        Ok(())
    }

    /// 编译指定文件
    pub async fn compile_file(&self, file_id: &str) -> Result<(), RuntimeError> {
        let source_file = self.source_files.get(file_id).ok_or_else(|| RuntimeError::FileNotFound(file_id.to_string()))?;

        // 检查缓存
        if self.cache.is_valid(file_id).await {
            return Ok(());
        }

        // 解析 AST
        let ast = self.parse_ast(&source_file).await?;
        self.cache.store_ast(file_id, ast.clone()).await;

        // 生成 HIR
        let hir = self.lower_to_hir(&ast, file_id).await?;
        self.cache.store_hir(file_id, hir.clone()).await;

        // 更新诊断信息
        let diagnostics = self.collect_diagnostics(file_id, &ast, &hir).await?;
        self.diagnostics.update_diagnostics(file_id, diagnostics).await;

        Ok(())
    }

    /// 获取文件的 AST
    pub async fn get_ast(&self, file_id: &str) -> Result<Arc<Program>, RuntimeError> {
        // 先检查缓存
        if let Some(ast) = self.cache.get_ast(file_id).await {
            return Ok(ast);
        }

        // 重新编译
        self.compile_file(file_id).await?;

        self.cache.get_ast(file_id).await.ok_or_else(|| RuntimeError::CompilationFailed(file_id.to_string()))
    }

    /// 获取文件的 HIR
    pub async fn get_hir(&self, file_id: &str) -> Result<Arc<Module>, RuntimeError> {
        // 先检查缓存
        if let Some(hir) = self.cache.get_hir(file_id).await {
            return Ok(hir);
        }

        // 重新编译
        self.compile_file(file_id).await?;

        self.cache.get_hir(file_id).await.ok_or_else(|| RuntimeError::CompilationFailed(file_id.to_string()))
    }

    /// 解释执行源码
    #[cfg(feature = "interpreter")]
    pub async fn interpret(&self, source: &str) -> Result<InterpreterResult, RuntimeError> {
        let file_id = self.add_source("<input>", source).await?;
        self.interpret_file(&file_id).await
    }

    /// 解释执行指定文件
    #[cfg(feature = "interpreter")]
    pub async fn interpret_file(&self, file_id: &str) -> Result<InterpreterResult, RuntimeError> {
        let hir = self.get_hir(file_id).await?;
        let interpreter = self.interpreter.read().await;
        interpreter.execute(&hir).await
    }

    /// 查询文件中的符号
    pub async fn query_symbols(&self, file_id: &str) -> Result<Vec<SymbolInfo>, RuntimeError> {
        let hir = self.get_hir(file_id).await?;
        self.query_engine.query_symbols(&hir).await
    }

    /// 查询指定位置的类型信息
    pub async fn query_type_at_position(&self, file_id: &str, position: Position) -> Result<Option<String>, RuntimeError> {
        let hir = self.get_hir(file_id).await?;
        self.query_engine.query_type_at_position(&hir, position).await
    }

    /// 查询符号的定义位置
    pub async fn query_definition(&self, file_id: &str, position: Position) -> Result<Option<Position>, RuntimeError> {
        let hir = self.get_hir(file_id).await?;
        self.query_engine.query_definition(&hir, position).await
    }

    /// 查询符号的引用
    pub async fn query_references(&self, file_id: &str, position: Position) -> Result<Vec<Position>, RuntimeError> {
        let hir = self.get_hir(file_id).await?;
        self.query_engine.query_references(&hir, position).await
    }

    /// 生成代码
    pub async fn generate_code(&self, source: &str, target: CodegenTarget) -> Result<GeneratedCode, RuntimeError> {
        let file_id = self.add_source("<codegen>", source).await?;
        self.generate_code_for_file(&file_id, target).await
    }

    /// 为指定文件生成代码
    pub async fn generate_code_for_file(&self, file_id: &str, target: CodegenTarget) -> Result<GeneratedCode, RuntimeError> {
        let hir = self.get_hir(file_id).await?;
        self.codegen_engine.generate(&hir, target).await
    }

    /// 获取诊断信息
    pub async fn get_diagnostics(&self, file_id: &str) -> Result<Vec<NyarError>, RuntimeError> {
        self.diagnostics.get_diagnostics(file_id).await
    }

    /// 获取所有文件的诊断信息
    pub async fn get_all_diagnostics(&self) -> Result<HashMap<FileId, Vec<NyarError>>, RuntimeError> {
        self.diagnostics.get_all_diagnostics().await
    }

    /// 清除指定文件的诊断信息
    pub async fn clear_diagnostics(&self, file_id: &str) -> Result<(), RuntimeError> {
        self.diagnostics.clear_diagnostics(file_id).await;
        Ok(())
    }

    /// 获取运行时统计信息
    pub async fn get_stats(&self) -> RuntimeStats {
        RuntimeStats {
            source_files_count: self.source_files.len(),
            cached_asts: self.cache.ast_count().await,
            cached_hirs: self.cache.hir_count().await,
            total_diagnostics: self.diagnostics.total_count().await,
        }
    }

    // 私有辅助方法

    async fn parse_ast(&self, source_file: &SourceFile) -> Result<Program, RuntimeError> {
        // TODO: 实际的 AST 解析逻辑
        // 这里应该调用 nyar-ast 的解析器
        Err(RuntimeError::NotImplemented("AST parsing".to_string()))
    }

    async fn lower_to_hir(&self, ast: &Program, file_id: &str) -> Result<Module, RuntimeError> {
        // TODO: 实际的 HIR 降级逻辑
        // 这里应该调用 nyar-hir 的降级器
        Err(RuntimeError::NotImplemented("HIR lowering".to_string()))
    }

    async fn collect_diagnostics(&self, file_id: &str, ast: &Program, hir: &Module) -> Result<Vec<NyarError>, RuntimeError> {
        // TODO: 实际的诊断收集逻辑
        // 这里应该收集编译过程中的所有诊断信息
        Ok(vec![])
    }
}

/// 运行时统计信息
#[derive(Debug, Clone)]
pub struct RuntimeStats {
    /// 源文件数量
    pub source_files_count: usize,
    /// 缓存的 AST 数量
    pub cached_asts: usize,
    /// 缓存的 HIR 数量
    pub cached_hirs: usize,
    /// 总诊断信息数量
    pub total_diagnostics: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = Runtime::new().await;
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_source_management() {
        let runtime = Runtime::new().await.unwrap();

        let file_id = runtime.add_source("test.ny", "fn main() {}").await;
        assert!(file_id.is_err()); // Will fail until parsing is implemented

        // Test update and removal would go here
    }

    #[tokio::test]
    async fn test_workspace_root() {
        let mut runtime = Runtime::new().await.unwrap();

        assert!(runtime.workspace_root().is_none());

        runtime.set_workspace_root("/tmp/workspace");
        assert_eq!(runtime.workspace_root(), Some(Path::new("/tmp/workspace")));
    }

    #[tokio::test]
    async fn test_stats() {
        let runtime = Runtime::new().await.unwrap();
        let stats = runtime.get_stats().await;

        assert_eq!(stats.source_files_count, 0);
        assert_eq!(stats.cached_asts, 0);
        assert_eq!(stats.cached_hirs, 0);
        assert_eq!(stats.total_diagnostics, 0);
    }
}

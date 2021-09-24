//! 运行时错误处理模块
//!
//! 定义运行时系统中可能出现的各种错误类型，使用 miette 提供统一的错误处理体验。

use std::fmt;

use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

use nyar_core::{Position, Range};
use nyar_error::NyarError;

/// 运行时错误的主要类型
#[derive(Error, Debug, Diagnostic)]
pub enum RuntimeError {
    /// 编译失败
    #[error("Compilation failed for file: {0}")]
    #[diagnostic(code(nyar::runtime::compilation_failed))]
    CompilationFailed(String),

    /// 文件未找到
    #[error("File not found: {0}")]
    #[diagnostic(code(nyar::runtime::file_not_found))]
    FileNotFound(String),

    /// IO 错误
    #[error("IO error: {0}")]
    #[diagnostic(code(nyar::runtime::io_error))]
    IoError(String),

    /// 解析错误
    #[error("Parse error in {file} at {position}: {message}")]
    #[diagnostic(code(nyar::runtime::parse_error))]
    ParseError {
        file: String,
        position: Position,
        message: String,
        #[source_code]
        source_code: String,
        #[label("Parse error occurred here")]
        span: SourceSpan,
    },

    /// 类型错误
    #[error("Type error in {file}: {message}")]
    #[diagnostic(code(nyar::runtime::type_error))]
    TypeError {
        file: String,
        message: String,
        #[source_code]
        source_code: String,
        #[label("Type error here")]
        span: SourceSpan,
        #[help]
        help: Option<String>,
    },

    /// 解释器错误
    #[cfg(feature = "interpreter")]
    #[error("Interpreter error: {0}")]
    #[diagnostic(code(nyar::runtime::interpreter_error))]
    InterpreterError(#[from] InterpreterError),

    /// 查询错误
    #[error("Query error: {0}")]
    #[diagnostic(code(nyar::runtime::query_error))]
    QueryError(#[from] QueryError),

    /// 代码生成错误
    #[error("Code generation error: {0}")]
    #[diagnostic(code(nyar::runtime::codegen_error))]
    CodegenError(#[from] CodegenError),

    /// 缓存错误
    #[error("Cache error: {0}")]
    #[diagnostic(code(nyar::runtime::cache_error))]
    CacheError(String),

    /// 配置错误
    #[error("Configuration error: {0}")]
    #[diagnostic(code(nyar::runtime::config_error))]
    ConfigError(String),

    /// 超时错误
    #[error("Operation timed out after {timeout_ms}ms: {operation}")]
    #[diagnostic(code(nyar::runtime::timeout))]
    Timeout { operation: String, timeout_ms: u64 },

    /// 内存不足错误
    #[error("Out of memory: {message}")]
    #[diagnostic(code(nyar::runtime::out_of_memory))]
    OutOfMemory {
        message: String,
        #[help]
        help: String,
    },

    /// 功能未实现
    #[error("Feature not implemented: {0}")]
    #[diagnostic(code(nyar::runtime::not_implemented))]
    NotImplemented(String),

    /// 内部错误
    #[error("Internal runtime error: {0}")]
    #[diagnostic(code(nyar::runtime::internal_error))]
    InternalError(String),

    /// 多个错误的聚合
    #[error("Multiple errors occurred ({count} errors)")]
    #[diagnostic(code(nyar::runtime::multiple_errors))]
    MultipleErrors { count: usize, errors: Vec<RuntimeError> },
}

/// 解释器特定错误
#[cfg(feature = "interpreter")]
#[derive(Error, Debug, Diagnostic)]
pub enum InterpreterError {
    /// 栈溢出
    #[error("Stack overflow: maximum stack size ({max_size} bytes) exceeded")]
    #[diagnostic(code(nyar::interpreter::stack_overflow))]
    StackOverflow { max_size: usize },

    /// 堆溢出
    #[error("Heap overflow: maximum heap size ({max_size} bytes) exceeded")]
    #[diagnostic(code(nyar::interpreter::heap_overflow))]
    HeapOverflow { max_size: usize },

    /// 执行超时
    #[error("Execution timeout: exceeded {timeout_ms}ms limit")]
    #[diagnostic(code(nyar::interpreter::execution_timeout))]
    ExecutionTimeout { timeout_ms: u64 },

    /// 运行时错误
    #[error("Runtime error at {position}: {message}")]
    #[diagnostic(code(nyar::interpreter::runtime_error))]
    RuntimeError {
        position: Position,
        message: String,
        #[source_code]
        source_code: String,
        #[label("Error occurred here")]
        span: SourceSpan,
    },

    /// 除零错误
    #[error("Division by zero at {position}")]
    #[diagnostic(code(nyar::interpreter::division_by_zero))]
    DivisionByZero {
        position: Position,
        #[source_code]
        source_code: String,
        #[label("Division by zero here")]
        span: SourceSpan,
    },

    /// 空指针访问
    #[error("Null pointer access at {position}")]
    #[diagnostic(code(nyar::interpreter::null_pointer))]
    NullPointer {
        position: Position,
        #[source_code]
        source_code: String,
        #[label("Null pointer access here")]
        span: SourceSpan,
    },

    /// 数组越界
    #[error("Array index out of bounds: index {index} >= length {length}")]
    #[diagnostic(code(nyar::interpreter::array_bounds))]
    ArrayBounds {
        index: usize,
        length: usize,
        position: Position,
        #[source_code]
        source_code: String,
        #[label("Array access here")]
        span: SourceSpan,
    },
}

/// 查询引擎错误
#[derive(Error, Debug, Diagnostic)]
pub enum QueryError {
    /// 符号未找到
    #[error("Symbol not found: {symbol}")]
    #[diagnostic(code(nyar::query::symbol_not_found))]
    SymbolNotFound { symbol: String },

    /// 查询超时
    #[error("Query timeout: exceeded {timeout_ms}ms limit")]
    #[diagnostic(code(nyar::query::timeout))]
    Timeout { timeout_ms: u64 },

    /// 索引损坏
    #[error("Symbol index corrupted: {message}")]
    #[diagnostic(code(nyar::query::index_corrupted))]
    IndexCorrupted { message: String },

    /// 查询语法错误
    #[error("Invalid query syntax: {message}")]
    #[diagnostic(code(nyar::query::invalid_syntax))]
    InvalidSyntax {
        message: String,
        #[help]
        help: String,
    },

    /// 跨文件查询失败
    #[error("Cross-file query failed: {message}")]
    #[diagnostic(code(nyar::query::cross_file_failed))]
    CrossFileFailed { message: String },
}

/// 代码生成错误
#[derive(Error, Debug, Diagnostic)]
pub enum CodegenError {
    /// 不支持的目标
    #[error("Unsupported target: {target}")]
    #[diagnostic(code(nyar::codegen::unsupported_target))]
    UnsupportedTarget { target: String },

    /// 生成失败
    #[error("Code generation failed for {target}: {message}")]
    #[diagnostic(code(nyar::codegen::generation_failed))]
    GenerationFailed {
        target: String,
        message: String,
        #[help]
        help: Option<String>,
    },

    /// 优化失败
    #[error("Optimization failed: {message}")]
    #[diagnostic(code(nyar::codegen::optimization_failed))]
    OptimizationFailed { message: String },

    /// WebAssembly 特定错误
    #[cfg(feature = "wasm-backend")]
    #[error("WebAssembly error: {0}")]
    #[diagnostic(code(nyar::codegen::wasm_error))]
    WasmError(String),

    /// JavaScript 特定错误
    #[cfg(feature = "js-backend")]
    #[error("JavaScript error: {0}")]
    #[diagnostic(code(nyar::codegen::js_error))]
    JavaScriptError(String),
}

/// 错误结果类型别名
pub type RuntimeResult<T> = Result<T, RuntimeError>;

/// 错误转换辅助函数
impl RuntimeError {
    /// 从编译器诊断创建运行时错误
    pub fn from_diagnostic(diagnostic: NyarError, source_code: String) -> Self {
        match diagnostic.severity {
            nyar_error::DiagnosticSeverity::Error => {
                if let Some(span) = diagnostic.primary_span {
                    Self::ParseError {
                        file: diagnostic.file_id.unwrap_or_else(|| "<unknown>".to_string()),
                        position: Position { line: span.start.line, column: span.start.column },
                        message: diagnostic.message,
                        source_code,
                        span: SourceSpan::new(span.start.offset.into(), (span.end.offset - span.start.offset).into()),
                    }
                }
                else {
                    Self::CompilationFailed(diagnostic.message)
                }
            }
            _ => Self::CompilationFailed(diagnostic.message),
        }
    }

    /// 创建 IO 错误
    pub fn io_error<E: fmt::Display>(error: E) -> Self {
        Self::IoError(error.to_string())
    }

    /// 创建超时错误
    pub fn timeout(operation: impl Into<String>, timeout_ms: u64) -> Self {
        Self::Timeout { operation: operation.into(), timeout_ms }
    }

    /// 创建内存不足错误
    pub fn out_of_memory(message: impl Into<String>) -> Self {
        Self::OutOfMemory { message: message.into(), help: "Try reducing memory usage or increasing memory limits".to_string() }
    }

    /// 创建未实现错误
    pub fn not_implemented(feature: impl Into<String>) -> Self {
        Self::NotImplemented(feature.into())
    }

    /// 创建内部错误
    pub fn internal(message: impl Into<String>) -> Self {
        Self::InternalError(message.into())
    }

    /// 聚合多个错误
    pub fn multiple(errors: Vec<RuntimeError>) -> Self {
        let count = errors.len();
        Self::MultipleErrors { count, errors }
    }

    /// 检查是否为致命错误
    pub fn is_fatal(&self) -> bool {
        matches!(self, Self::OutOfMemory { .. } | Self::InternalError(_) | Self::ConfigError(_) | Self::CacheError(_))
    }

    /// 检查是否可以重试
    pub fn is_retryable(&self) -> bool {
        matches!(self, Self::Timeout { .. } | Self::IoError(_) | Self::CacheError(_))
    }
}

/// 错误上下文扩展
pub trait ErrorContext<T> {
    /// 添加文件上下文
    fn with_file_context(self, file: impl Into<String>) -> RuntimeResult<T>;

    /// 添加操作上下文
    fn with_operation_context(self, operation: impl Into<String>) -> RuntimeResult<T>;
}

impl<T, E> ErrorContext<T> for Result<T, E>
where
    E: Into<RuntimeError>,
{
    fn with_file_context(self, file: impl Into<String>) -> RuntimeResult<T> {
        self.map_err(|e| {
            let mut error = e.into();
            match &mut error {
                RuntimeError::ParseError { file: f, .. } => {
                    if f == "<unknown>" {
                        *f = file.into();
                    }
                }
                RuntimeError::TypeError { file: f, .. } => {
                    if f.is_empty() {
                        *f = file.into();
                    }
                }
                _ => {}
            }
            error
        })
    }

    fn with_operation_context(self, operation: impl Into<String>) -> RuntimeResult<T> {
        self.map_err(|e| {
            let error = e.into();
            match error {
                RuntimeError::InternalError(msg) => RuntimeError::InternalError(format!("{}: {}", operation.into(), msg)),
                other => other,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = RuntimeError::file_not_found("test.ny");
        assert!(matches!(error, RuntimeError::FileNotFound(_)));
    }

    #[test]
    fn test_error_properties() {
        let timeout_error = RuntimeError::timeout("compilation", 5000);
        assert!(timeout_error.is_retryable());
        assert!(!timeout_error.is_fatal());

        let memory_error = RuntimeError::out_of_memory("heap exhausted");
        assert!(!memory_error.is_retryable());
        assert!(memory_error.is_fatal());
    }

    #[test]
    fn test_multiple_errors() {
        let errors = vec![RuntimeError::file_not_found("a.ny"), RuntimeError::file_not_found("b.ny")];
        let multi_error = RuntimeError::multiple(errors);

        if let RuntimeError::MultipleErrors { count, .. } = multi_error {
            assert_eq!(count, 2);
        }
        else {
            panic!("Expected MultipleErrors");
        }
    }

    #[test]
    fn test_error_context() {
        let result: Result<(), std::io::Error> = Err(std::io::Error::new(std::io::ErrorKind::NotFound, "file not found"));

        let runtime_result =
            result.map_err(RuntimeError::io_error).with_file_context("test.ny").with_operation_context("parsing");

        assert!(runtime_result.is_err());
    }
}

// 便利构造函数
impl RuntimeError {
    pub fn file_not_found(file: impl Into<String>) -> Self {
        Self::FileNotFound(file.into())
    }

    pub fn compilation_failed(file: impl Into<String>) -> Self {
        Self::CompilationFailed(file.into())
    }

    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::CacheError(message.into())
    }

    pub fn config_error(message: impl Into<String>) -> Self {
        Self::ConfigError(message.into())
    }
}

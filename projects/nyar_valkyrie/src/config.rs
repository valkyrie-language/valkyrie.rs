//! 运行时配置模块
//!
//! 提供运行时系统的各种配置选项，支持灵活的功能定制。

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// 运行时主配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// 是否启用解释器
    pub enable_interpreter: bool,

    /// 是否启用查询引擎
    pub enable_query_engine: bool,

    /// 是否启用代码生成
    pub enable_codegen: bool,

    /// 是否启用调试信息
    pub enable_debug_info: bool,

    /// 是否启用性能分析
    pub enable_profiling: bool,

    /// 缓存配置
    pub cache_config: CacheConfig,

    /// 诊断配置
    pub diagnostics_config: DiagnosticsConfig,

    /// 解释器配置
    #[cfg(feature = "interpreter")]
    pub interpreter_config: InterpreterConfig,

    /// 查询引擎配置
    pub query_config: QueryConfig,

    /// 代码生成配置
    pub codegen_config: CodegenConfig,

    /// 性能配置
    pub performance_config: PerformanceConfig,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            enable_interpreter: true,
            enable_query_engine: true,
            enable_codegen: true,
            enable_debug_info: cfg!(debug_assertions),
            enable_profiling: false,
            cache_config: CacheConfig::default(),
            diagnostics_config: DiagnosticsConfig::default(),
            #[cfg(feature = "interpreter")]
            interpreter_config: InterpreterConfig::default(),
            query_config: QueryConfig::default(),
            codegen_config: CodegenConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

/// 缓存配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// 是否启用 AST 缓存
    pub enable_ast_cache: bool,

    /// 是否启用 HIR 缓存
    pub enable_hir_cache: bool,

    /// 是否启用符号表缓存
    pub enable_symbol_cache: bool,

    /// AST 缓存最大条目数
    pub max_ast_entries: usize,

    /// HIR 缓存最大条目数
    pub max_hir_entries: usize,

    /// 符号表缓存最大条目数
    pub max_symbol_entries: usize,

    /// 缓存过期时间
    pub cache_ttl: Duration,

    /// 是否启用 LRU 淘汰策略
    pub enable_lru_eviction: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enable_ast_cache: true,
            enable_hir_cache: true,
            enable_symbol_cache: true,
            max_ast_entries: 1000,
            max_hir_entries: 1000,
            max_symbol_entries: 5000,
            cache_ttl: Duration::from_secs(3600), // 1 hour
            enable_lru_eviction: true,
        }
    }
}

/// 诊断配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsConfig {
    /// 是否启用语法错误诊断
    pub enable_syntax_diagnostics: bool,

    /// 是否启用类型错误诊断
    pub enable_type_diagnostics: bool,

    /// 是否启用语义警告
    pub enable_semantic_warnings: bool,

    /// 是否启用性能警告
    pub enable_performance_warnings: bool,

    /// 是否启用风格建议
    pub enable_style_suggestions: bool,

    /// 每个文件最大诊断数量
    pub max_diagnostics_per_file: Option<usize>,

    /// 诊断严重程度过滤
    pub severity_filter: Vec<DiagnosticSeverity>,

    /// 忽略的诊断代码
    pub ignored_diagnostic_codes: Vec<String>,
}

impl Default for DiagnosticsConfig {
    fn default() -> Self {
        Self {
            enable_syntax_diagnostics: true,
            enable_type_diagnostics: true,
            enable_semantic_warnings: true,
            enable_performance_warnings: false,
            enable_style_suggestions: false,
            max_diagnostics_per_file: Some(100),
            severity_filter: vec![DiagnosticSeverity::Error, DiagnosticSeverity::Warning],
            ignored_diagnostic_codes: vec![],
        }
    }
}

/// 诊断严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
    Information,
    Hint,
}

/// 解释器配置
#[cfg(feature = "interpreter")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterpreterConfig {
    /// 是否启用即时编译 (JIT)
    pub enable_jit: bool,

    /// 是否启用优化
    pub enable_optimization: bool,

    /// 栈大小限制 (字节)
    pub stack_size_limit: usize,

    /// 堆大小限制 (字节)
    pub heap_size_limit: Option<usize>,

    /// 执行超时时间
    pub execution_timeout: Option<Duration>,

    /// 是否启用调试模式
    pub debug_mode: bool,

    /// 是否启用性能分析
    pub enable_profiling: bool,
}

#[cfg(feature = "interpreter")]
impl Default for InterpreterConfig {
    fn default() -> Self {
        Self {
            enable_jit: false,
            enable_optimization: true,
            stack_size_limit: 1024 * 1024,           // 1MB
            heap_size_limit: Some(64 * 1024 * 1024), // 64MB
            execution_timeout: Some(Duration::from_secs(30)),
            debug_mode: cfg!(debug_assertions),
            enable_profiling: false,
        }
    }
}

/// 查询引擎配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryConfig {
    /// 是否启用符号索引
    pub enable_symbol_indexing: bool,

    /// 是否启用类型推断缓存
    pub enable_type_inference_cache: bool,

    /// 是否启用跨文件查询
    pub enable_cross_file_queries: bool,

    /// 符号搜索深度限制
    pub max_search_depth: usize,

    /// 查询超时时间
    pub query_timeout: Duration,

    /// 是否启用模糊匹配
    pub enable_fuzzy_matching: bool,

    /// 模糊匹配阈值 (0.0 - 1.0)
    pub fuzzy_match_threshold: f64,
}

impl Default for QueryConfig {
    fn default() -> Self {
        Self {
            enable_symbol_indexing: true,
            enable_type_inference_cache: true,
            enable_cross_file_queries: true,
            max_search_depth: 10,
            query_timeout: Duration::from_millis(5000),
            enable_fuzzy_matching: true,
            fuzzy_match_threshold: 0.7,
        }
    }
}

/// 代码生成配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodegenConfig {
    /// 默认优化级别
    pub optimization_level: OptimizationLevel,

    /// 是否生成调试信息
    pub generate_debug_info: bool,

    /// 是否生成源码映射
    pub generate_source_maps: bool,

    /// WebAssembly 配置
    #[cfg(feature = "wasm-backend")]
    pub wasm_config: WasmConfig,

    /// JavaScript 配置
    #[cfg(feature = "js-backend")]
    pub js_config: JavaScriptConfig,
}

impl Default for CodegenConfig {
    fn default() -> Self {
        Self {
            optimization_level: OptimizationLevel::O2,
            generate_debug_info: cfg!(debug_assertions),
            generate_source_maps: true,
            #[cfg(feature = "wasm-backend")]
            wasm_config: WasmConfig::default(),
            #[cfg(feature = "js-backend")]
            js_config: JavaScriptConfig::default(),
        }
    }
}

/// 优化级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OptimizationLevel {
    /// 无优化，快速编译
    O0,
    /// 基本优化
    O1,
    /// 标准优化
    O2,
    /// 激进优化
    O3,
    /// 大小优化
    Os,
    /// 极致大小优化
    Oz,
}

/// WebAssembly 配置
#[cfg(feature = "wasm-backend")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// 目标 WebAssembly 版本
    pub target_version: WasmVersion,

    /// 是否启用 GC 提案
    pub enable_gc: bool,

    /// 是否启用 SIMD
    pub enable_simd: bool,

    /// 是否启用多线程
    pub enable_threads: bool,

    /// 输出格式
    pub output_format: WasmOutputFormat,
}

#[cfg(feature = "wasm-backend")]
impl Default for WasmConfig {
    fn default() -> Self {
        Self {
            target_version: WasmVersion::V1,
            enable_gc: true,
            enable_simd: false,
            enable_threads: false,
            output_format: WasmOutputFormat::Wat,
        }
    }
}

/// WebAssembly 版本
#[cfg(feature = "wasm-backend")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmVersion {
    V1,
    V2,
}

/// WebAssembly 输出格式
#[cfg(feature = "wasm-backend")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WasmOutputFormat {
    /// WebAssembly 文本格式
    Wat,
    /// WebAssembly 二进制格式
    Wasm,
}

/// JavaScript 配置
#[cfg(feature = "js-backend")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaScriptConfig {
    /// 目标 ECMAScript 版本
    pub target_version: EcmaScriptVersion,

    /// 是否生成 TypeScript 定义文件
    pub generate_typescript_defs: bool,

    /// 是否启用模块系统
    pub module_system: ModuleSystem,

    /// 是否压缩输出
    pub minify: bool,
}

#[cfg(feature = "js-backend")]
impl Default for JavaScriptConfig {
    fn default() -> Self {
        Self {
            target_version: EcmaScriptVersion::ES2020,
            generate_typescript_defs: true,
            module_system: ModuleSystem::ESM,
            minify: false,
        }
    }
}

/// ECMAScript 版本
#[cfg(feature = "js-backend")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EcmaScriptVersion {
    ES5,
    ES2015,
    ES2017,
    ES2018,
    ES2019,
    ES2020,
    ES2021,
    ES2022,
    ESNext,
}

/// 模块系统
#[cfg(feature = "js-backend")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModuleSystem {
    /// CommonJS
    CommonJS,
    /// ES Modules
    ESM,
    /// AMD
    AMD,
    /// UMD
    UMD,
}

/// 性能配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// 工作线程数量
    pub worker_threads: Option<usize>,

    /// 是否启用并行编译
    pub enable_parallel_compilation: bool,

    /// 是否启用增量编译
    pub enable_incremental_compilation: bool,

    /// 内存使用限制 (字节)
    pub memory_limit: Option<usize>,

    /// 是否启用内存池
    pub enable_memory_pool: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            worker_threads: None, // 使用系统默认
            enable_parallel_compilation: true,
            enable_incremental_compilation: true,
            memory_limit: None,
            enable_memory_pool: true,
        }
    }
}

/// 配置构建器
#[derive(Debug, Default)]
pub struct RuntimeConfigBuilder {
    config: RuntimeConfig,
}

impl RuntimeConfigBuilder {
    /// 创建新的配置构建器
    pub fn new() -> Self {
        Self::default()
    }

    /// 启用/禁用解释器
    pub fn interpreter(mut self, enabled: bool) -> Self {
        self.config.enable_interpreter = enabled;
        self
    }

    /// 启用/禁用查询引擎
    pub fn query_engine(mut self, enabled: bool) -> Self {
        self.config.enable_query_engine = enabled;
        self
    }

    /// 启用/禁用代码生成
    pub fn codegen(mut self, enabled: bool) -> Self {
        self.config.enable_codegen = enabled;
        self
    }

    /// 设置优化级别
    pub fn optimization_level(mut self, level: OptimizationLevel) -> Self {
        self.config.codegen_config.optimization_level = level;
        self
    }

    /// 设置缓存配置
    pub fn cache_config(mut self, config: CacheConfig) -> Self {
        self.config.cache_config = config;
        self
    }

    /// 设置诊断配置
    pub fn diagnostics_config(mut self, config: DiagnosticsConfig) -> Self {
        self.config.diagnostics_config = config;
        self
    }

    /// 启用调试模式
    pub fn debug_mode(mut self, enabled: bool) -> Self {
        self.config.enable_debug_info = enabled;
        #[cfg(feature = "interpreter")]
        {
            self.config.interpreter_config.debug_mode = enabled;
        }
        self.config.codegen_config.generate_debug_info = enabled;
        self
    }

    /// 启用性能分析
    pub fn profiling(mut self, enabled: bool) -> Self {
        self.config.enable_profiling = enabled;
        #[cfg(feature = "interpreter")]
        {
            self.config.interpreter_config.enable_profiling = enabled;
        }
        self
    }

    /// 构建配置
    pub fn build(self) -> RuntimeConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RuntimeConfig::default();
        assert!(config.enable_interpreter);
        assert!(config.enable_query_engine);
        assert!(config.enable_codegen);
    }

    #[test]
    fn test_config_builder() {
        let config = RuntimeConfigBuilder::new()
            .interpreter(false)
            .query_engine(true)
            .codegen(true)
            .optimization_level(OptimizationLevel::O3)
            .debug_mode(true)
            .profiling(true)
            .build();

        assert!(!config.enable_interpreter);
        assert!(config.enable_query_engine);
        assert!(config.enable_codegen);
        assert_eq!(config.codegen_config.optimization_level, OptimizationLevel::O3);
        assert!(config.enable_debug_info);
        assert!(config.enable_profiling);
    }

    #[test]
    fn test_cache_config_default() {
        let config = CacheConfig::default();
        assert!(config.enable_ast_cache);
        assert!(config.enable_hir_cache);
        assert!(config.enable_symbol_cache);
        assert_eq!(config.max_ast_entries, 1000);
    }

    #[test]
    fn test_diagnostics_config_default() {
        let config = DiagnosticsConfig::default();
        assert!(config.enable_syntax_diagnostics);
        assert!(config.enable_type_diagnostics);
        assert!(config.enable_semantic_warnings);
        assert!(!config.enable_performance_warnings);
    }
}

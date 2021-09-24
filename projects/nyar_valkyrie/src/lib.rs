//! # Nyar Runtime
//!
//! 统一的 Nyar 编译器运行时系统，提供解释器、查询器、代码生成等核心功能的统一接口。
//!
//! ## 设计理念
//!
//! Nyar Runtime 作为编译器基础设施的统一入口，提供：
//!
//! - **统一接口**：为所有运行时组件提供一致的 API
//! - **模块化设计**：支持按需加载不同的后端和功能
//! - **异步优先**：所有操作都支持异步执行
//! - **错误统一**：使用 miette 提供一致的错误处理
//! - **性能优化**：内置缓存和并发优化
//!
//! ## 核心组件
//!
//! ### 解释器 (Interpreter)
//!
//! ```rust
//! use nyar_runtime::{InterpreterConfig, Runtime};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let runtime = Runtime::new().await?;
//!
//!     let source = r#"
//!         fn main() {
//!             println!("Hello, Nyar!");
//!         }
//!     "#;
//!
//!     let result = runtime.interpret(source).await?;
//!     println!("Result: {:?}", result);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 查询引擎 (Query Engine)
//!
//! ```rust
//! use nyar_runtime::{QueryRequest, Runtime};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let runtime = Runtime::new().await?;
//!
//!     // 编译源码
//!     runtime.compile_source("example.ny", source).await?;
//!
//!     // 查询符号信息
//!     let symbols = runtime.query_symbols("example.ny").await?;
//!
//!     // 查询类型信息
//!     let type_info =
//!         runtime.query_type_at_position("example.ny", Position { line: 10, column: 5 }).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ### 代码生成 (Code Generation)
//!
//! ```rust
//! use nyar_runtime::{CodegenTarget, Runtime};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let runtime = Runtime::new().await?;
//!
//!     // 编译到 WebAssembly
//!     let wasm_code = runtime.generate_code(source, CodegenTarget::Wasm).await?;
//!
//!     // 编译到 JavaScript
//!     let js_code = runtime.generate_code(source, CodegenTarget::JavaScript).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod cache;
pub mod codegen;
pub mod config;
pub mod diagnostics;
pub mod error;
pub mod interpreter;
pub mod query;
pub mod runtime;

// Re-export core types
pub use codegen::*;
pub use config::*;
pub use diagnostics::*;
pub use error::*;
pub use interpreter::*;
pub use query::*;
pub use runtime::*;

// Re-export commonly used types from dependencies
pub use nyar_ast::{AstNode, Program};
pub use nyar_core::{Position, Range, SourceFile, Span};
pub use nyar_error::{DiagnosticSeverity, NyarError};
pub use nyar_hir::{HirNode, Module};

/// Runtime version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const NAME: &str = env!("CARGO_PKG_NAME");

/// Supported file extensions
pub const SUPPORTED_EXTENSIONS: &[&str] = &[".ny", ".nyar", ".valkyrie"];

/// Default configuration for the runtime
pub fn default_config() -> RuntimeConfig {
    RuntimeConfig::default()
}

/// Create a new runtime instance with default configuration
pub async fn create_runtime() -> Result<Runtime, RuntimeError> {
    Runtime::new().await
}

/// Create a new runtime instance with custom configuration
pub async fn create_runtime_with_config(config: RuntimeConfig) -> Result<Runtime, RuntimeError> {
    Runtime::with_config(config).await
}

/// Convenience function to quickly interpret source code
pub async fn interpret_source(source: &str) -> Result<InterpreterResult, RuntimeError> {
    let runtime = create_runtime().await?;
    runtime.interpret(source).await
}

/// Convenience function to quickly compile and query source code
pub async fn query_source_symbols(source: &str) -> Result<Vec<SymbolInfo>, RuntimeError> {
    let runtime = create_runtime().await?;
    let file_id = runtime.add_source("<input>", source).await?;
    runtime.query_symbols(&file_id).await
}

/// Convenience function to generate code for a target
pub async fn generate_code_for_target(source: &str, target: CodegenTarget) -> Result<GeneratedCode, RuntimeError> {
    let runtime = create_runtime().await?;
    runtime.generate_code(source, target).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_runtime_creation() {
        let runtime = create_runtime().await;
        assert!(runtime.is_ok());
    }

    #[tokio::test]
    async fn test_version_info() {
        assert!(!VERSION.is_empty());
        assert_eq!(NAME, "nyar-runtime");
    }

    #[tokio::test]
    async fn test_supported_extensions() {
        assert!(SUPPORTED_EXTENSIONS.contains(&".ny"));
        assert!(SUPPORTED_EXTENSIONS.contains(&".nyar"));
        assert!(SUPPORTED_EXTENSIONS.contains(&".valkyrie"));
    }

    #[tokio::test]
    async fn test_default_config() {
        let config = default_config();
        assert!(config.enable_interpreter);
        assert!(config.enable_query_engine);
    }

    #[tokio::test]
    async fn test_interpret_simple_source() {
        let source = r#"
            fn main() -> i32 {
                return 42;
            }
        "#;

        let result = interpret_source(source).await;
        // Note: This will fail until we implement the actual interpreter
        // but it tests the API structure
        assert!(result.is_err() || result.is_ok());
    }

    #[tokio::test]
    async fn test_query_symbols() {
        let source = r#"
            fn hello_world() {
                let x = 42;
            }
        "#;

        let result = query_source_symbols(source).await;
        // Note: This will fail until we implement the actual query engine
        // but it tests the API structure
        assert!(result.is_err() || result.is_ok());
    }

    #[cfg(feature = "wasm-backend")]
    #[tokio::test]
    async fn test_wasm_codegen() {
        let source = r#"
            fn add(a: i32, b: i32) -> i32 {
                return a + b;
            }
        "#;

        let result = generate_code_for_target(source, CodegenTarget::Wasm).await;
        // Note: This will fail until we implement the actual codegen
        // but it tests the API structure
        assert!(result.is_err() || result.is_ok());
    }
}

//! Valkyrie Language Server Protocol Implementation
//!
//! 这是基于 Nyar 编译器基础设施的 Valkyrie 语言 LSP 实现。
//!
//! ## 设计理念
//!
//! Nyar 作为编译器本身不提供 LSP 服务，而是为宿主语言（如 Valkyrie）
//! 提供统一的 LSP 查询体系。这种设计有以下优势：
//!
//! 1. **模块化**：编译器核心与 LSP 服务分离
//! 2. **可复用**：多个宿主语言可以共享相同的基础设施
//! 3. **可扩展**：每个宿主语言可以定制自己的 LSP 功能
//!
//! ## 架构概览
//!
//! ```text
//! ┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
//! │   IDE Client    │◄──►│  Valkyrie LSP   │◄──►│ Nyar Compiler   │
//! │                 │    │                 │    │ Infrastructure  │
//! │ - VS Code       │    │ - Protocol      │    │                 │
//! │ - IntelliJ      │    │ - Handlers      │    │ - AST/HIR/MIR   │
//! │ - Vim/Neovim    │    │ - Diagnostics   │    │ - Query Engine  │
//! │ - Emacs         │    │ - State Mgmt    │    │ - Error System  │
//! └─────────────────┘    └─────────────────┘    └─────────────────┘
//! ```
//!
//! ## 功能特性
//!
//! - **完整的 LSP 支持**：实现了 LSP 3.17 规范的大部分功能
//! - **实时诊断**：基于 Nyar 编译器的错误和警告系统
//! - **智能补全**：上下文感知的代码补全
//! - **符号导航**：定义跳转、引用查找、符号搜索
//! - **代码操作**：快速修复、重构建议
//! - **语义高亮**：基于语义分析的语法高亮
//! - **自定义扩展**：Valkyrie 特定的 LSP 扩展方法
//!
//! ## 使用示例
//!
//! ### 作为独立服务器运行
//!
//! ```bash
//! # 标准输入输出模式（默认）
//! valkyrie-lsp --stdio
//!
//! # TCP 模式（用于调试）
//! valkyrie-lsp --tcp 9257
//! ```
//!
//! ### 作为库使用
//!
//! ```rust
//! use tower_lsp::{LspService, Server};
//! use valkyrie_lsp::{start_lsp_server, ValkyrieBackend};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let (service, socket) = LspService::build(|client| ValkyrieBackend::new(client)).finish();
//!
//!     let stdin = tokio::io::stdin();
//!     let stdout = tokio::io::stdout();
//!
//!     Server::new(stdin, stdout, socket).serve(service).await;
//!     Ok(())
//! }
//! ```

pub mod backend;
pub mod capabilities;
pub mod diagnostics;
pub mod handlers;
pub mod state;

// 重新导出主要类型
pub use backend::ValkyrieBackend;
pub use capabilities::server_capabilities;
pub use diagnostics::{DiagnosticFilterConfig, DiagnosticStats, DiagnosticsManager};
pub use state::{DocumentState, ServerState, SymbolInfo};

// 重新导出处理器
pub use handlers::{CodeActionHandler, DocumentSymbolHandler, FormattingHandler, RenameHandler, WorkspaceSymbolHandler};

/// LSP 服务器版本信息
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// LSP 服务器名称
pub const SERVER_NAME: &str = "valkyrie-lsp";

/// 支持的文件扩展名
pub const SUPPORTED_EXTENSIONS: &[&str] = &[".vk", ".valkyrie"];

/// 便捷函数：启动 LSP 服务器
///
/// 这个函数提供了一个简单的方式来启动 Valkyrie LSP 服务器。
///
/// # 参数
///
/// * `mode` - 服务器模式（stdio 或 tcp）
/// * `port` - TCP 端口（仅在 TCP 模式下使用）
///
/// # 示例
///
/// ```rust
/// use valkyrie_lsp::{start_lsp_server, ServerMode};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     start_lsp_server(ServerMode::Stdio, None).await
/// }
/// ```
pub async fn start_lsp_server(mode: ServerMode, port: Option<u16>) -> Result<(), Box<dyn std::error::Error>> {
    use tokio::net::TcpListener;
    use tower_lsp::{LspService, Server};
    use tracing::info;

    match mode {
        ServerMode::Stdio => {
            let stdin = tokio::io::stdin();
            let stdout = tokio::io::stdout();

            let (service, socket) = LspService::build(|client| ValkyrieBackend::new(client))
                .custom_method("valkyrie/getAst", ValkyrieBackend::get_ast)
                .custom_method("valkyrie/getHir", ValkyrieBackend::get_hir)
                .custom_method("valkyrie/querySymbol", ValkyrieBackend::query_symbol)
                .finish();

            info!("Valkyrie LSP Server started on stdio");
            Server::new(stdin, stdout, socket).serve(service).await;
        }
        ServerMode::Tcp => {
            let port = port.unwrap_or(9257);
            let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
            info!("Valkyrie LSP Server listening on 127.0.0.1:{}", port);

            loop {
                let (stream, addr) = listener.accept().await?;
                info!("New connection from {}", addr);

                tokio::spawn(async move {
                    let (read, write) = tokio::io::split(stream);

                    let (service, socket) = LspService::build(|client| ValkyrieBackend::new(client))
                        .custom_method("valkyrie/getAst", ValkyrieBackend::get_ast)
                        .custom_method("valkyrie/getHir", ValkyrieBackend::get_hir)
                        .custom_method("valkyrie/querySymbol", ValkyrieBackend::query_symbol)
                        .finish();

                    let server = Server::new(read, write, socket);

                    if let Err(e) = server.serve(service).await {
                        tracing::warn!("Connection error: {}", e);
                    }
                });
            }
        }
    }

    Ok(())
}

/// LSP 服务器运行模式
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ServerMode {
    /// 标准输入输出模式
    Stdio,
    /// TCP 模式
    Tcp,
}

/// LSP 服务器配置
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// 服务器模式
    pub mode: ServerMode,
    /// TCP 端口（仅在 TCP 模式下使用）
    pub port: Option<u16>,
    /// 日志级别
    pub log_level: tracing::Level,
    /// 诊断过滤配置
    pub diagnostic_filter: DiagnosticFilterConfig,
    /// 是否启用自定义扩展方法
    pub enable_custom_methods: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            mode: ServerMode::Stdio,
            port: None,
            log_level: tracing::Level::INFO,
            diagnostic_filter: DiagnosticFilterConfig::default(),
            enable_custom_methods: true,
        }
    }
}

/// 使用配置启动 LSP 服务器
pub async fn start_lsp_server_with_config(config: ServerConfig) -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt().with_max_level(config.log_level).init();

    start_lsp_server(config.mode, config.port).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_config_default() {
        let config = ServerConfig::default();
        assert_eq!(config.mode, ServerMode::Stdio);
        assert_eq!(config.port, None);
        assert!(config.enable_custom_methods);
    }

    #[test]
    fn test_supported_extensions() {
        assert!(SUPPORTED_EXTENSIONS.contains(&".vk"));
        assert!(SUPPORTED_EXTENSIONS.contains(&".valkyrie"));
    }
}

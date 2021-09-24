//! Valkyrie Language Server Protocol Implementation
//!
//! 这是 Valkyrie 语言的 LSP 服务器实现，基于 Nyar 编译器基础设施构建。
//! 注意：Nyar 作为编译器本身不提供 LSP，而是为宿主语言提供统一的 LSP 查询体系。

use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tower_lsp::{LspService, Server};
use tracing::{info, warn};
use tracing_subscriber;

mod backend;
mod capabilities;
mod diagnostics;
mod handlers;
mod state;

use backend::ValkyrieBackend;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志系统
    tracing_subscriber::fmt().with_max_level(tracing::Level::INFO).init();

    info!("Starting Valkyrie LSP Server");

    // 从环境变量或命令行参数获取配置
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("--stdio") => {
            // 标准输入输出模式（默认）
            start_stdio_server().await
        }
        Some("--tcp") => {
            // TCP 模式
            let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(9257);
            start_tcp_server(port).await
        }
        _ => {
            // 默认使用 stdio
            start_stdio_server().await
        }
    }
}

/// 启动基于标准输入输出的 LSP 服务器
async fn start_stdio_server() -> Result<(), Box<dyn std::error::Error>> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| ValkyrieBackend::new(client))
        .custom_method("valkyrie/getAst", ValkyrieBackend::get_ast)
        .custom_method("valkyrie/getHir", ValkyrieBackend::get_hir)
        .custom_method("valkyrie/querySymbol", ValkyrieBackend::query_symbol)
        .finish();

    info!("Valkyrie LSP Server started on stdio");
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}

/// 启动基于 TCP 的 LSP 服务器（用于调试）
async fn start_tcp_server(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", port)).await?;
    info!("Valkyrie LSP Server listening on 127.0.0.1:{}", port);

    loop {
        let (stream, addr) = listener.accept().await?;
        info!("New connection from {}", addr);

        tokio::spawn(handle_connection(stream));
    }
}

/// 处理单个 TCP 连接
async fn handle_connection(stream: TcpStream) {
    let (read, write) = tokio::io::split(stream);

    let (service, socket) = LspService::build(|client| ValkyrieBackend::new(client))
        .custom_method("valkyrie/getAst", ValkyrieBackend::get_ast)
        .custom_method("valkyrie/getHir", ValkyrieBackend::get_hir)
        .custom_method("valkyrie/querySymbol", ValkyrieBackend::query_symbol)
        .finish();

    let server = Server::new(read, write, socket);

    if let Err(e) = server.serve(service).await {
        warn!("Connection error: {}", e);
    }
}

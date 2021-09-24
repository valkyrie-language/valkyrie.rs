# Nyar Runtime

统一的 Nyar 运行时系统，集成解释器、查询引擎、代码生成器等核心组件。

## 概述

Nyar Runtime 是 Nyar 编程语言生态系统的核心运行时，提供了一个统一的接口来访问编译器的各种功能。它将解释器、查询引擎、代码生成器、缓存系统和诊断管理器整合在一起，为上层应用（如 LSP 服务器、REPL、构建工具等）提供简洁易用的 API。

## 设计理念

### 统一接口
- **单一入口点**：通过 `Runtime` 结构体提供所有功能的统一访问
- **一致的 API**：所有组件都遵循相同的设计模式和错误处理机制
- **配置驱动**：通过配置文件灵活控制各组件的行为

### 高性能
- **多层缓存**：内存缓存 + 磁盘缓存，最大化编译性能
- **增量编译**：只重新编译发生变化的部分
- **并发安全**：所有组件都支持多线程并发访问

### 可扩展性
- **插件架构**：支持自定义代码生成器和查询处理器
- **模块化设计**：各组件可以独立使用和测试
- **标准化接口**：便于第三方工具集成

## 核心组件

### Runtime 核心
```rust
use nyar_runtime::{Runtime, RuntimeConfig};

// 创建运行时实例
let config = RuntimeConfig::default();
let runtime = Runtime::new(config).await?;

// 编译源码
let source = "fn main() { println(\"Hello, World!\"); }";
let result = runtime.compile_source("main.ny", source).await?;

// 解释执行
let output = runtime.interpret_module(&result.hir).await?;
println!("Output: {}", output);
```

### 查询引擎
```rust
// 查询符号信息
let symbols = runtime.query_symbols(&module).await?;
for symbol in symbols {
    println!("Symbol: {} ({})", symbol.name, symbol.symbol_type);
}

// 查询类型信息
let position = Position { line: 10, column: 5 };
let type_info = runtime.query_type_at_position(&module, position).await?;
```

### 代码生成
```rust
use nyar_runtime::codegen::{CodegenOptions, CodegenTarget};

// 生成 JavaScript 代码
let options = CodegenOptions {
    target: CodegenTarget::JavaScript,
    optimization_level: OptimizationLevel::Full,
    source_map: true,
    ..Default::default()
};

let result = runtime.generate_code(&module, &options).await?;
println!("Generated code:\n{}", result.code);
```

### 诊断管理
```rust
// 获取诊断信息
let diagnostics = runtime.get_diagnostics("main.ny");
for diagnostic in diagnostics {
    println!("{}: {}", diagnostic.severity, diagnostic.message);
}

// 格式化诊断输出
let formatted = runtime.format_diagnostics(&diagnostics);
println!("{}", formatted);
```

## 功能特性

### 🚀 高性能编译
- **增量编译**：只重新编译修改的文件
- **并行处理**：多线程并行编译多个模块
- **智能缓存**：AST、HIR、类型信息等多层缓存
- **内存优化**：高效的内存管理和对象池

### 🔍 强大的查询系统
- **符号查询**：快速查找定义、引用、类型信息
- **语义分析**：提供丰富的语义信息
- **模糊搜索**：支持模糊匹配的符号搜索
- **工作区支持**：跨文件的全局符号查询

### 🎯 多目标代码生成
- **JavaScript**：生成现代 JavaScript 代码
- **TypeScript**：生成类型定义文件
- **WebAssembly**：生成 WASM 文本格式
- **可扩展**：支持自定义代码生成器

### 📊 完善的诊断系统
- **统一错误处理**：基于 miette 的错误体系
- **多种输出格式**：文本、JSON、结构化输出
- **错误分类**：语法、类型、语义、性能等分类
- **修复建议**：提供自动修复建议

### 💾 智能缓存系统
- **多层缓存**：内存 + 磁盘缓存
- **缓存策略**：LRU、LFU、TTL 等多种策略
- **缓存统计**：详细的缓存命中率统计
- **自动清理**：过期和大小限制的自动清理

## 安装使用

### 添加依赖

在 `Cargo.toml` 中添加：

```toml
[dependencies]
nyar-runtime = { path = "../nyar-runtime" }
tokio = { version = "1.0", features = ["full"] }
```

### 基本使用

```rust
use nyar_runtime::{Runtime, RuntimeConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 创建配置
    let config = RuntimeConfig::builder()
        .enable_cache(true)
        .cache_size(1000)
        .optimization_level(OptimizationLevel::Full)
        .build();
    
    // 创建运行时
    let runtime = Runtime::new(config).await?;
    
    // 设置工作区
    runtime.set_workspace_root("/path/to/project").await?;
    
    // 编译文件
    let source = std::fs::read_to_string("main.ny")?;
    let result = runtime.compile_source("main.ny", &source).await?;
    
    // 检查错误
    if runtime.has_errors() {
        let diagnostics = runtime.get_all_diagnostics();
        eprintln!("Compilation errors:\n{}", runtime.format_diagnostics(&diagnostics));
        return Ok(());
    }
    
    // 解释执行
    let output = runtime.interpret_module(&result.hir).await?;
    println!("Program output: {}", output);
    
    Ok(())
}
```

### 高级配置

```rust
use nyar_runtime::config::*;

let config = RuntimeConfig {
    // 缓存配置
    cache: CacheConfig {
        enable_memory_cache: true,
        enable_disk_cache: true,
        max_entries: 10000,
        max_entry_size: 1024 * 1024, // 1MB
        eviction_policy: EvictionPolicy::LRU,
        disk_cache_dir: "./cache".to_string(),
    },
    
    // 诊断配置
    diagnostics: DiagnosticsConfig {
        max_diagnostics_per_file: 100,
        output_format: "json".to_string(),
        enable_suggestions: true,
        severity_filter: Some(Severity::Warning),
    },
    
    // 解释器配置
    interpreter: InterpreterConfig {
        stack_size: 1024 * 1024, // 1MB
        heap_size: 16 * 1024 * 1024, // 16MB
        enable_profiling: true,
        max_execution_time: Some(Duration::from_secs(30)),
    },
    
    // 查询配置
    query: QueryConfig {
        enable_fuzzy_matching: true,
        fuzzy_match_threshold: 0.6,
        query_timeout: Duration::from_millis(5000),
        max_results: 1000,
    },
    
    // 代码生成配置
    codegen: CodegenConfig {
        default_target: CodegenTarget::JavaScript,
        enable_optimization: true,
        enable_source_maps: true,
        output_directory: "./dist".to_string(),
    },
};
```

## 架构设计

```
┌─────────────────────────────────────────────────────────────┐
│                    Nyar Runtime                             │
├─────────────────────────────────────────────────────────────┤
│  Runtime Core                                               │
│  ├── Configuration Management                               │
│  ├── Source File Management                                 │
│  ├── Compilation Pipeline                                   │
│  └── Error Handling                                         │
├─────────────────────────────────────────────────────────────┤
│  Core Components                                            │
│  ├── Interpreter Engine    ├── Query Engine                │
│  ├── Code Generator        ├── Diagnostics Manager         │
│  └── Cache Manager                                          │
├─────────────────────────────────────────────────────────────┤
│  Foundation Layer                                           │
│  ├── nyar-core           ├── nyar-ast                      │
│  ├── nyar-hir            ├── nyar-diagnostics              │
│  └── External Dependencies (tokio, serde, miette, etc.)    │
└─────────────────────────────────────────────────────────────┘
```

### 组件交互

1. **Runtime Core** 作为统一入口点，协调各个组件
2. **Cache Manager** 为所有组件提供缓存服务
3. **Diagnostics Manager** 收集和管理所有组件的错误信息
4. **Interpreter Engine** 执行编译后的 HIR 代码
5. **Query Engine** 提供符号查询和语义分析
6. **Code Generator** 将 HIR 转换为目标代码

## 性能优化

### 编译性能
- **增量编译**：只重新编译修改的文件和依赖
- **并行编译**：多线程并行处理独立模块
- **缓存策略**：多层缓存减少重复计算
- **内存池**：减少内存分配开销

### 查询性能
- **索引构建**：预构建符号索引加速查询
- **模糊匹配优化**：高效的字符串匹配算法
- **结果缓存**：缓存常用查询结果
- **异步处理**：非阻塞的查询处理

### 内存优化
- **引用计数**：使用 Arc 共享数据结构
- **写时复制**：延迟复制大型数据结构
- **内存映射**：大文件使用内存映射
- **垃圾回收**：定期清理未使用的缓存

## 错误处理

Nyar Runtime 使用统一的错误处理体系：

```rust
use nyar_runtime::error::{RuntimeError, RuntimeResult};

// 所有 API 都返回 RuntimeResult
let result: RuntimeResult<Module> = runtime.compile_source("test.ny", source).await;

match result {
    Ok(module) => {
        println!("Compilation successful");
    }
    Err(RuntimeError::CompilationError { diagnostics, .. }) => {
        eprintln!("Compilation failed with {} errors", diagnostics.len());
        for diagnostic in diagnostics {
            eprintln!("{}", diagnostic);
        }
    }
    Err(e) => {
        eprintln!("Runtime error: {}", e);
    }
}
```

## 测试

运行所有测试：

```bash
cargo test
```

运行特定模块测试：

```bash
cargo test runtime
cargo test query
cargo test codegen
cargo test cache
```

运行基准测试：

```bash
cargo bench
```

## 示例项目

查看 `examples/` 目录中的示例项目：

- `basic_usage.rs` - 基本使用示例
- `lsp_integration.rs` - LSP 集成示例
- `repl.rs` - REPL 实现示例
- `build_tool.rs` - 构建工具示例

## 贡献指南

1. Fork 项目
2. 创建功能分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add some amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 打开 Pull Request

### 开发环境

```bash
# 克隆项目
git clone https://github.com/your-org/nyar-vm.git
cd nyar-vm/projects/nyar-runtime

# 安装依赖
cargo build

# 运行测试
cargo test

# 检查代码格式
cargo fmt --check

# 运行 linter
cargo clippy -- -D warnings
```

## 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](../../LICENSE) 文件了解详情。

## 更新日志

### v0.1.0 (2024-01-XX)
- 初始版本发布
- 实现核心运行时功能
- 支持基本的编译和解释执行
- 提供查询引擎和代码生成功能
- 实现多层缓存系统
- 集成诊断管理器

## 相关项目

- [nyar-core](../nyar-core) - Nyar 核心库
- [nyar-ast](../nyar-ast) - AST 定义和操作
- [nyar-hir](../nyar-hir) - HIR 定义和操作
- [valkyrie-lsp](../valkyrie-lsp) - Valkyrie LSP 服务器
- [nyar-wasm](../nyar-wasm) - WebAssembly 代码生成器
- [nyar-js](../nyar-js) - JavaScript 代码生成器

## 支持

如果您在使用过程中遇到问题，请：

1. 查看 [文档](https://docs.nyar-lang.org/runtime)
2. 搜索 [Issues](https://github.com/your-org/nyar-vm/issues)
3. 创建新的 Issue 描述问题
4. 加入我们的 [Discord 社区](https://discord.gg/nyar-lang)

---

**Nyar Runtime** - 统一、高效、可扩展的 Nyar 运行时系统 🚀
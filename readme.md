# Valkyrie 语言原生运行时

Valkyrie 是一门现代函数式编程语言，本项目包含了该语言的核心运行时设施。

## 项目组件

本工作区包含以下核心组件：

- **[类型系统与错误处理](./projects/valkyrie-types)**: 内置类型定义、类型检查与统一的错误报告
- **[编译器](./projects/valkyrie-compiler)**: 基于 Chomsky 的现代编译器框架
- **[运行时](./projects/valkyrie-interpreter)**: 高性能字节码执行引擎
- **[集成工具](./projects/valkyrie)**: 整合编译器与运行时的入口
- **[命令行工具 (Legion)](./projects/legion)**: 编译器与包管理工具
- **[WASM/WASI 支持](./projects/valkyrie-wasm32-wasi)**: 针对 WebAssembly 平台的后端实现
- **[演练场](./projects/valkyrie-playground)**: 在线尝试 Valkyrie 的 Web 应用
- **[VSCode 扩展](./projects/valkyrie-vscode)**: Visual Studio Code 的语法高亮与插件支持

## 核心特性

- **代数效应**: 灵活处理副作用的控制流抽象。
- **强类型系统**: 静态类型检查与强大的类型推导。
- **跨平台运行**: 支持原生、JavaScript 及 WebAssembly 目标。
- **互操作性**: 支持从 Valkyrie 调用 Rust 编写的 FFI 接口。

## 许可证

本项目采用 MIT 许可证。详见 [LICENSE.md](LICENSE.md) 文件。

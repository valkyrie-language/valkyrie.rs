# Valkyrie 维护指南

本指南面向 `valkyrie.rs` 工作区的**内部维护者**，说明现行架构、模块职责与维护约定。

> **目标读者**: 项目维护者、核心开发成员  
> **内容重点**: 以源码为准的主链；愿景与现行分开书写

## 项目架构概览

Valkyrie 采用 Rust workspace。根 `Cargo.toml` 的成员以 `projects/*` 为准（节选）：

```text
valkyrie.rs/
├── Cargo.toml
├── documentation/
└── projects/
    ├── legion/              # CLI / 清单 / 轻量编排
    ├── nyar-language/       # 语言主链：AST / HIR / MIR …
    ├── nyar/                # 规划、后端能力、打包协议
    ├── nyar-emitter/        # FragmentSubmission → 各目标降低
    ├── nyar-types/          # 平台层类型（含 NyarType）
    ├── nyar-analyzer/       # 分析辅助
    ├── nyar-optimizer/      # 优化 / egraph 宿主（演进中）
    ├── std-data/            # PE / class / wasm 等手写二进制与 MSIL 文本模型
    ├── nvm/                 # Nyar 相关运行时资产
    └── …
```

历史文档中的 `valkyrie-compiler` / `valkyrie-parser` / `oak-valkyrie` / `ProjectChomsky` / `project-gaia` 等名称，**不是**当前 workspace 成员；其中语言主链能力主要落在 `nyar-language`，二进制发射落在 `nyar-emitter` + `std-data`。

## 现行编译主链

```text
source -> AST -> HIR -> MIR
  -> ArtifactPartitionPlan / FragmentSubmission (nyar)
  -> nyar-emitter 按目标降低
  -> std-data 手写目标二进制
```

四目标自举重点：**clr / jvm / wasm / wasi**。约束摘要：

- 与 `valkyrie.v` **同构**；CLR/JVM/WASM 均手写二进制，禁止 ilasm/dnlib/javac/`wit-component` 等作为正式生成器。
- 无语言类型 `string`；文本为 `utf8` / `utf16`。
- `void`（ADT=0）与 `unit`（ADT=1）禁止混淆。

## 核心设计哲学（仍适用）

1. **语义在前端闭合**：emit 不补 trait/row/effect 语义。  
2. **诊断体验**：以 `miette` 等结构化诊断为主。  
3. **抽象对偶**：`match`（数据）与 `catch`/效应（控制）对称。  
4. **多执行面**：开发期运行时与 AOT 产物并存；具体能力以各 crate 现状为准，勿把未落地的 Gaia/Chomsky 写成现行事实。

## 专题索引

- [项目架构](project-architecture.md)
- [编译器架构](compiler-architecture.md)
- [执行模型](execution-models.md)（含现状 vs 愿景）
- [对象降低](object-lowering.md)
- [包管理与符号解析](package-management.md)
- [错误处理](error-handling.md)
- [优化策略](optimization-strategies.md)
- [后端](backends/index.md)

本指南随源码演进更新；**冲突时以源码与本目录较新修订为准**。

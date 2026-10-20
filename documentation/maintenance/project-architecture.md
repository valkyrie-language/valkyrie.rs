# Valkyrie 项目架构与维护指南

本仓拥有工程输入、CLI/host 装配、缓存和产物提交。语言语义与 Nyar 编译框架通过依赖接入，不能按旧目录图在装配仓重新实现。

## 目录与职责

- projects/compilers/legion：CLI、manifest/workspace、Compiler 调用与 build/test 编排。
- projects/compilers/legion-workspace：package graph 与逐文件不可变输入。
- projects/compilers/vcc-napi、vcc-wasm：宿主接口。
- projects/compilers/asgard：上层应用构建编排。
- projects/packages：npm CLI、host wrapper 与平台包。
- scripts：开发、装配和发布脚本。

成员与依赖以 Cargo.toml 为准。nyar-language、nyar-emitter 及其他编译框架 crate 的实现位于 nyar-vm.rs；Oak 拥有文本格式，Acorn 拥有外部二进制格式。本仓不保留混合格式包或第二份 codec/parser。

## 深模块边界

外部接口为 WorkspaceResolver、Compiler、ArtifactStore。Resolver 冻结来源和依赖图；Compiler 内部负责绑定、语义需求及代码生成；ArtifactStore 原子提交完整 ArtifactSet。

装配层不访问 AST/HIR/MIR、语义 planner 或 target lowering 实现。新增 host/CLI 路由必须记录实际 Compiler/host 来源，不得以 wrapper 成功证明源码编译能力。

## 维护流程

先定位缺失事实的唯一生产者，再选择仓和接口。每个工作包说明可观察行为、失败条件、正反验收与证据范围；不能用题目名、operator 或当前报错作为特判依据。

缓存复用前重算期望语义与产物身份。inventory provenance 与语义 freshness 分开；本地 patch 与准确依赖 pin 分开记录。删除旧路线必须迁移消费者、依赖和实现并验证旧入口不可达。

详见 [编译协议](compiler-architecture.md)、[后端边界](backends/index.md)及[诊断](error-handling.md)。

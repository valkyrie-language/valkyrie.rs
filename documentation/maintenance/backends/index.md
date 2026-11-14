# 后端维护指南

语言与目标适配器将本语言 IR 或 residual 降低到目标私有计划和 LIR。Nyar 提供变换、分析、优化和专门化工具，不规定公共语言 MIR。普通函数、合成包装和 runtime helper 共用目标 LIR 编码链路。

Wasm、CLR、JVM、OVM 和 native 各自验证目标表示、调用与导入合同。Acorn 负责适用的外部格式，OVM 编码 `.odin` 供 OdinVM 执行。目标不能按函数名、类型形状、默认值或空载荷补语言语义。

## 验收边界

支持能力必须有正式入口、实际非空 artifact、export/invoke 与真实用例证据。CLR/JVM/WASI/native 等专题设计不构成当前题目矩阵完成。

缺 callable、完整实例、import signature、layout 或 carrier 时，在其所属阶段结构化失败。验证须覆盖全部生产 backend 入口，不能仅验证直接调用 emitter 的测试夹具。

## 专题

- [CLR](clr.md)
- [JVM](jvm.md)
- [Wasm/WASI](wasi.md)
- [Native](native.md)
- [OVM / OdinVM](odin-vm.md)

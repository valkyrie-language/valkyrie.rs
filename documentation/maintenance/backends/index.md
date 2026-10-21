# 后端维护指南

backend 只消费已验证的 TargetExecutionPlan。公共 Canonical 语义、RepresentationPlan 与目标私有计划各有所有者，不建立包含全部阶段的统一物理 IR。

## 数据流

```text
package Semantic MIR → linker / Canonical validation
  → optimization / sparse RepresentationPlan
  → BackendLowering.prepare → pre-encode validation
  → TargetExecutionPlan → format encoding
```

linker 选择 adaptor/import；表示规划选择 carrier/layout；backend 安排目标私有编号并编码。backend 不能按 import field 猜 ABI、按 callee 尾名决定行为、按类型名找布局或缺 payload 补值。

外部 PE/JVM/Wasm 等格式由对应 Acorn crate 编码；Nyar 字节码由 nyar-bytecode 编码。格式校验不能反向改变语言语义。

## 验收边界

支持能力必须有正式入口、实际非空 artifact、export/invoke 与真实用例证据。CLR/JVM/WASI/native 等专题设计不构成当前题目矩阵完成。

缺 callable、完整实例、import signature、layout 或 carrier 时，在其所属阶段结构化失败。验证须覆盖全部生产 backend 入口，不能仅验证直接调用 emitter 的测试夹具。

## 专题

- [CLR](clr.md)
- [JVM](jvm.md)
- [Wasm/WASI](wasi.md)
- [Native](native.md)
- [Nyar VM](nyar-vm.md)

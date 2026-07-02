# 优化策略

## 前言

优化工作分布在语言主链、`nyar-optimizer` 与各后端局部 peephole/布局选择中。本文区分**现状**与**愿景**。

## 现状

1. **语义层**：HIR 脱糖、类型驱动简化、可达性与单态化相关工作在 `nyar-language` / 规划层推进。
2. **优化宿主**：`nyar-optimizer` 提供 egraph 等能力，但**尚未**达到「全面 equality saturation 产品态」（见 `projects/readme.md`）。
3. **后端**：CLR/JVM/WASM 在 MIR→目标指令时做栈平衡、调用约定与布局选择；**不**依赖未落地的 Gaia 代价模型作为现行必经之路。
4. **约束**：优化不得抹平 `void`/`unit`，不得把 `utf8` 优化成 UTF-16 code-unit API，不得引入外部工具产盘捷径。

## 愿景（非现行）

「前端只产 Intent，一切优化由 Chomsky E-Graph 统一完成，再由 Gaia 按后端代价模型提取」——保留为方向，**不要**写成 2026 已全面交付。

## 流水线（现行示意）

```text
源码 -> AST -> HIR -> MIR
  ->（可选）nyar-optimizer / 规划期静态化
  -> FragmentSubmission
  -> 目标 lowering + 手写二进制
```

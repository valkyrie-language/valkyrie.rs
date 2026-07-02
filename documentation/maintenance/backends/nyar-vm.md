# Nyar VM

## 现状（以源码为准）

- 相关 crate：`projects/nvm`、以及工作区中的 `legacy-vm` / `text-vm` 等（以根 `Cargo.toml` 为准）。
- 优化侧：`projects/nyar-optimizer` 提供 egraph 等**演进中**宿主；`projects/readme.md` 明确其尚未达到完整 equality saturation 产品态。
- 四目标 AOT 发射的主路径是 **`nyar-emitter` + `std-data`**，不是外部 `nyar-aot` / `project-gaia` 仓库链接。

## 愿景（历史叙述，勿当现行）

旧文档曾把流水线写成：

```text
Source -> AST -> HIR -> UIR (Chomsky) -> Optimized UIR -> Gaia / NyarAot / NyarJit
```

并链到本机路径（如 `e:/普遍优化/ProjectChomsky`）。这些**不是**当前 `valkyrie.rs` workspace 成员。维护时不要按其补齐依赖、也不要把四目标产盘写成必须先经「UIR → Gaia」。

## 与后端文档的关系

需要改 CLR/JVM/WASM/WASI 行为时，请直接阅读 [index.md](index.md) 与各目标文档及对应 `nyar-emitter`/`std-data` 模块；不要假设必须先经「UIR → Gaia」才能产盘。

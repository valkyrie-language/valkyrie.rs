# 架构优化记录

> **本文是冻结的历史 changelog**，不是现行架构说明。  
> 现行主链与约束见：[index.md](index.md)、[backends/index.md](backends/index.md)、[compiler-architecture.md](compiler-architecture.md)、根 `Cargo.toml`、`projects/readme.md`。

## 对照现状（阅读下文前先看这里）

| 历史叙述 | 现行理解 |
| --- | --- |
| 中心 crate `nyar-driver` + `families/{clr,wasm,native}` | 能力分布在 `nyar`（规划/选择）与 `nyar-emitter`（各目标 lowering）；workspace **无**名为 `nyar-driver` 的成员 |
| 「JVM 尚未接入 bundled 注册表」 | JVM 已有 `nyar-emitter/.../backends/jvm` + `std-data/binary/class` |
| `HIR + LIR + target profile` 驱动产盘 | 推荐：`HIR/MIR` → `FragmentSubmission` → 手写二进制；legacy LIR 过渡中 |
| 用 ilasm / 外部工具补 CLR | CLR 正式路径：`PeWriter`（`std-data/binary/pe`），禁止 ilasm/dnlib 产盘 |

## 历史目标（当时）

- 降低驱动层与具体后端的耦合。
- 按 backend family 做模块化分发。
- 保持当时的 `legion build` 产物协议稳定。

## 历史清单摘录（勿当 TODO）

当时记录过的议题包括：驱动层硬编码分支、WASM/native 与 CLR pipeline 边界不一致、`valkyrie-lsp` 独立编译路径、`nyar` selection 抽象未完全接上、以及「把 JVM 接入 driver 注册表」等。

当时设想的目录形如 `nyar-driver/src/families/{clr,wasm,native}.rs`，以及 `cargo test -p nyar-driver` 等验证步骤——**这些路径在当前 workspace 中不存在或已迁移**，不要按原文补齐或回滚。

## 历史「下一阶段」如何改读

1. ~~JVM 接入 nyar-driver~~ → 维护 JVM 时改 `nyar-emitter` / `std-data`，见 [backends/jvm.md](backends/jvm.md)。  
2. 统一选择入口 → 看 `projects/nyar` 的 planning / selection。  
3. LSP 与主编译链复用 → 以现有 `nyar-language` / legion 集成代码为准，另开议题，不沿用本文路径名。

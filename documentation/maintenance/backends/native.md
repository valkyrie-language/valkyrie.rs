# Native 后端

## 现状（以源码为准）

- 入口：`projects/nyar-emitter/src/lowering/backends/native/`。
- 可与 PE/原生镜像相关的编解码资产位于 `std-data`（含 `binary/pe` 的 native 子模块等）。
- Native 不是四目标自举（clr/jvm/wasm/wasi）的同一优先级文档焦点；维护时以 emitter 与测试现状为准。

## 愿景（非现行主链承诺）

历史材料曾描述「全面由 Nyar VM + Project Gaia 接管 x86/arm64/riscv，并生成 ELF/PE/WASM/JVM/CLR」。其中：

- **Gaia / Chomsky 作为统一多格式发射器**——**未**作为本仓库现行四目标生成路径。
- **CLR / JVM / WASM** 的正式生成由各自手写二进制路径完成（见 [clr.md](clr.md)、[jvm.md](jvm.md)、[wasi.md](wasi.md)），**不**经 Gaia。

## 约束对齐

即便扩展 native，仍须遵守：无语言 `string`（用 utf8/utf16）；`void`≠`unit`；若产出与托管目标重叠，不得改走 ilasm/dnlib/javac 等外部生成器。

# 后端维护指南

本文档说明 `valkyrie.rs` 中各后端的**现行**职责与入口。愿景级组件（历史文档中的 Chomsky UIR / Project Gaia 等）若未在本仓库落地，不得写成现行主链。

## 现行主链（以源码为准）

```text
source
  -> AST / HIR / MIR          (projects/nyar-language)
  -> 规划与分区                 (projects/nyar：ArtifactPartitionPlan 等)
  -> FragmentSubmission
  -> nyar-emitter 按目标降低
  -> std-data 手写二进制编解码
```

- 语言语义在 `HIR`/`MIR` 闭合；emit 层不补语义。
- 面向寄存器机的 legacy `LIR` 主链已标记废弃过渡；栈机目标（CLR/JVM/WASM）从 MIR 直接生成目标指令。
- 四目标自举重点：**clr / jvm / wasm / wasi**，与 `valkyrie.v` 同构手写 emitter。

## 共享约束

1. **手写二进制**：CLR→PE/CLI，JVM→class，WASM/WASI→wasm/component sections。禁止把 `ilasm` / `dnlib` / `csc` / `javac` / `wit-component` 等当作正式生成器。
2. **无语言类型 `string`**：文本必须是 `utf8` 或 `utf16`（及 `Utf8Text` / `Utf16Text`）。
3. **`void` ≠ `unit`**：`void` ADT 代数 = 0（`NyarType::Bottom`）；`unit` ADT 代数 = 1。ABI 返回位可能都塌成宿主 `void`/`V`，语义仍须区分。

## 后端文档

| 目标 | 文档 | 现行实现入口 |
| --- | --- | --- |
| CLR | [clr.md](clr.md) | `nyar-emitter/.../backends/clr` + `std-data/binary/pe` |
| JVM | [jvm.md](jvm.md) | `nyar-emitter/.../backends/jvm` + `std-data/binary/class` |
| WASM / WASI | [wasi.md](wasi.md) | `nyar-emitter/.../backends/wasm` + `std-data/binary/wasm`（WASI 另含 CM/host） |
| Native | [native.md](native.md) | `nyar-emitter/.../backends/native`（现状与愿景见该文） |
| Nyar VM | [nyar-vm.md](nyar-vm.md) | `nvm` / 相关运行时 crate（见该文「现状 vs 愿景」） |

## 设计决策（仍适用）

### 1. 语义与发射分离

优化与语义定界优先在语言主链 / `nyar` 规划层完成；各后端只消费已定界的 `FragmentSubmission`，不重做 trait/row/effect 解析。

### 2. 栈机跳过寄存器分配

对 JVM / CLR / WASM：不经「为寄存器机准备的 LIR 分配」；从 MIR 生成栈指令更自然。

### 3. 目标 FFI 隔离

CLR、JVM、WASM、WASI 的宿主导入与调用约定相互独立，禁止混用垫片。

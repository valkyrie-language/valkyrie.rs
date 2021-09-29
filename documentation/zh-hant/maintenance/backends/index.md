# 後端維護指南

本文檔介紹了 Valkyrie 編譯器支持的各種後端的編譯流程和設計考量。

## 概覽

Valkyrie 現已採用統一的、以 **Nyar VM** 為核心的編譯優化流水線。核心邏輯已從傳統的 LIR 線性轉換演進為基於 E-Graph 的等價飽和優化：

`Source -> AST -> HIR -> UIR (Chomsky) -> Optimized UIR (Nyar VM) -> Target`

這種架構允許我們將複雜的語言特性優化任務交給專業的優化引擎，而前端只需專注於語義降級。

## 後端實現

- [Nyar VM](nyar-vm.md): **核心後端**。
    - **AOT 模式**: 通過 `NyarAot` 與 `Gaia` 發射靜態二進制。
    - **JIT 模式**: 通過 `NyarJit` 實現即時編譯與熱點優化。
- [WASM 後端](wasi.md): 針對 WebAssembly (WASI)，利用 Nyar VM 的 UIR 發射適配。
- [Native 後端](native.md): 現已完全由 Nyar VM / Gaia 架構接管，提供原生指令集支持。
- [JVM/CLR 後端](jvm.md): 傳統後端，針對基於棧的虛擬機，通常跳過 LIR 階段直接從 CFG/UIR 生成指令。

## 設計決策

### 1. 統一優化入口

自 2026 年起，所有的核心優化任務（包括內聯、逃逸分析、死代碼消除等）統一由 Nyar VM 驅動的 Chomsky 引擎完成。這避免了在不同後端重複實現優化 Pass 的維護成本。

### 2. 針對棧機跳過寄存器分配

對於 JVM 或 CLR 等棧機後端：
- **理由**：LIR 是為寄存器機設計的。將 SSA 映射到 LIR 涉及寄存器分配，這對於棧機來說是不必要的。
- **策略**：直接從 CFG 或優化後的 UIR 生成基於棧的指令，可以更自然地利用操作數棧。

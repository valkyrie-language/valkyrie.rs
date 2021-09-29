# Native 後端架構

Valkyrie 的原生指令集支持（x86, x64, arm64, riscv）不再依賴 Cranelift 或 LLVM，而是採用了更強大且完全自主掌控的 **Nyar VM + Project Gaia** 架構。

## 架構概覽

Valkyrie 的原生編譯流程由以下核心組件驅動：

### 1. Nyar VM (核心運行時)
Nyar VM 是 Valkyrie 的主要驅動力，負責協調整個編譯和執行流程。
- **AOT 編譯驅動**：通過 `nyar-aot` 將源碼或字節碼預先編譯為高效的目標平台構件。
- **JIT 執行模式**：支持在運行時根據熱點代碼動態生成機器碼。

### 2. Project Chomsky (優化引擎)
替代了傳統的 LLVM 優化序列，Chomsky 採用了更現代的優化技術。
- **E-Graph 等價飽和**：利用基於 E-Graph 的等價飽和技術進行極致優化。
- **IKun 中間表示**：統一的意圖（Intents）表示，確保 AOT 和 JIT 共享相同的優化邏輯。

### 3. Project Gaia (多目標發射器)
Gaia 是一個極其靈活的後端系統，負責生成最終的可執行文件或庫。
- **多格式支持**：直接支持生成 ELF、PE、WASM、JVM、CLR 等多種格式。
- **全目標發射**：具備為 x86, x64, ARM64, RISC-V 等多種硬件架構發射機器碼的能力。
- **極致掌控**：相比 LLVM，Gaia 允許對內存佈局和指令序列進行更精細的控制，非常適合 OS 內核開發。

## 為什麼選擇自主架構？

1. **代數效應支持**：Valkyrie 的高級特性（如 Effect System）需要底層對棧和延續（Continuation）有特殊處理，自主架構能提供更好的適配。
2. **優化潛力**：E-Graph 技術能探索 LLVM 難以觸及的優化空間。
3. **輕量化與可移植性**：擺脫了對笨重的 LLVM C++ 庫的依賴，整個工具鏈更加緊湊。

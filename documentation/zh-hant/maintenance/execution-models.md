# Valkyrie 執行模型

## 1. 概述

Valkyrie 旨在支持多種執行環境，從高性能的生產環境到高度交互的開發環境。核心執行邏輯由 **Nyar VM** 驅動。

## 2. 現代執行架構 (Modern Execution Architecture)

Valkyrie 採用**統一意圖後端架構 (Unified Intent Backend Architecture)**。所有代碼首先降級為 Chomsky UIR，然後由 Nyar VM 根據執行模式進行分發：

```mermaid
graph TD
    A[Chomsky UIR] -->|Nyar VM| B{執行模式};
    B -->|JIT| C[動態生成機器碼並執行];
    B -->|AOT| D[生成靜態二進制文件];
    B -->|Interpreter| E[基於意圖的解釋執行 (調試用)];
```

### 2.1 JIT 模式 (Just-In-Time)
- **場景**: 開發調試、高性能腳本執行。
- **機制**: Nyar VM 實時分析 UIR 熱點，調用 `Gaia JIT` 引擎將 UIR 意圖直接發射到內存並執行。
- **優勢**: 
    - 結合了動態語言的靈活性和原生代碼的高性能。
    - 支持熱重載 (Hot Reloading)。

### 2.2 AOT 模式 (Ahead-Of-Time)
- **場景**: 生產環境部署、WASM 模塊分發。
- **機制**: 使用 `NyarAot` 靜態掃描整個 UIR 意圖樹，應用深度全局優化後，通過 `Gaia` 發射為目標平台的機器碼或字節碼（如 WASI, x86_64）。
- **優勢**: 
    - 零啟動開銷。
    - 極致的二進制體積優化。

## 3. Nyar VM 核心特性

無論採用何種執行模式，Valkyrie 都共享由 Nyar VM 提供的運行時能力：

- **基於 E-Graph 的全局優化**: 所有的 AOT 和 JIT 優化均由內置的 Chomsky 引擎驅動。
- **原生代數效應支持**: Nyar VM 在底層實現了高效的效應處理器和延續 (Continuation) 捕獲。
- **RAII 與 GC 融合的內存模型**: 為托管語言提供了原生 RAII（資源獲取即初始化）支持。通過 NyarVM 的編譯流水線控制，所有的托管對像在被 GC 回收時都會觸發其對應的終結邏輯（底層映射為 Rust 的 `Drop` 語義），確保了如文件、網絡連接等非內存資源的確定性釋放。
- **統一代價模型**: 開發者只需定義一套後端代價模型，即可同時受益於 AOT 和 JIT 的優化。

## 4. 廢棄的 LIR/SSA 模型 (Legacy Models)

早期的 Valkyrie 曾計劃使用多層線性降級（SSA -> LIR），但為了實現更深度的全局優化，現已全面轉向 **Chomsky UIR + Nyar VM** 架構。

- **為何廢棄**: 
    - 傳統的 SSA/LIR 優化 Pass 順序固定，難以發現跨階段的等價優化機會。
    - 維護多套後端發射邏輯（WASM, Native, VM）成本過高。
    - Nyar VM 的等價飽和技術提供了更強的優化上限。

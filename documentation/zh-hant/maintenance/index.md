# Valkyrie 虛擬機維護指南

本指南面向 Valkyrie 虛擬機項目的**內部維護者和核心開發團隊**，介紹項目架構、模組職責、內部維護流程和系統級設計決策。

> **目標讀者**: 項目維護者、核心開發團隊成員、系統架構師
> **內容重點**: 內部架構、維護流程、系統設計、代碼組織

## 項目架構概覽

Valkyrie 採用 Rust Monorepo (Workspace) 架構，每個組件封裝在獨立的 crate 中，提供清晰的依賴關係、獨立的測試環境和高效的並行編譯。

- **[編譯器架構 (Compiler Architecture)](./compiler-architecture.md)**：深入瞭解 Valkyrie 的多層 IR 架構。
- **[項目架構 (Project Architecture)](./project-architecture.md)**：詳細的目錄結構與模組說明。

```
valkyrie/
├── Cargo.toml         # Workspace 根配置
└── projects/
    ├── valkyrie-types/    # 統一中間表示類型定義 (HIR, UIR/IKun)
    ├── valkyrie-compiler/ # 基於 Chomsky 的現代編譯器框架
    ├── nyar-vm/           # Nyar VM 核心與執行引擎
    ├── valkyrie-error/    # 基於 miette 的診斷系統
    └── legion/            # 命令行工具 (Valkyrie 工具鏈入口)
```

外部依賴:
- `oak-valkyrie`: 新版前端實現 (Lexer, Parser, AST)，位於 `../oaks`
- `ProjectChomsky`: 編譯器後端優化框架，位於 `../ProjectChomsky`
- `project-gaia`: 多目標指令發射器，位於 `../project-gaia`

## 核心設計哲學

Valkyrie 的架構基於五大設計支柱：

### 1. 現代編譯流水線 (Modern Compilation Pipeline)

Valkyrie 的核心設計已演進為以 **Nyar VM** 為中心的現代化架構，將優化與後端生成任務完全解耦：

#### 階段 1: 前端 (Frontend - Oaks)
- **職責**: 語法解析與高級語義處理。
- **當前實現**: 使用 `oak-valkyrie` 作為統一的前端。
- **關鍵處理**:
  - **符號解析 (Symbol Resolution)**: 構建跨模組的符號引用。
  - **類型檢查與推導 (Type Checking & Inference)**: 確保語言層面的類型安全。
  - **模式匹配解糖 (Pattern Matching Desugaring)**: 將複雜的 `match` 結構轉換為決策樹。

#### 階段 2: 降級 (Lowering - Chomsky UIR)
- **職責**: 將高級語義 (HIR) 轉換為通用的、可優化的中間表示 (UIR/IKun)。
- **關鍵處理**: 將語言特定的語義原語映射到通用的 UIR 意圖（Intents）。

#### 階段 3: 優化 (Optimization - Nyar VM / Chomsky)
- **職責**: 執行全局優化，支持 AOT 和 JIT。
- **核心技術**: **E-Graph 等價飽和**。
- **優勢**: 統一的優化邏輯，無需為不同後端重複編寫優化 Pass。

#### 階段 4: 後端發射 (Backend Emission - Nyar VM / Gaia)
- **職責**: 針對特定目標（WASM, Native, JIT 內存空間）發射代碼。
- **關鍵處理**: 寄存器分配、指令選擇。
  - **棧幀優化**: 減少不必要的入棧出棧。
  - **指令調度**: 優化執行流水線以減少延遲。

### 2. 開發者體驗的終極追求 (Uncompromising Developer Experience)

- **診斷即對話**: 使用 `miette` 框架提供 IDE 級別的診斷體驗
- **心流不被打斷**: 通過高效的編譯流水線實現亞秒級響應
- **直覺且強大的語言**: 提供代數效應、強大的模式匹配等高級抽象

### 3. 抽象的統一與對稱 (Unity and Duality of Abstractions)

基於數據與控制的對偶性：
- `match` 表達式：對數據的分解和模式匹配
- `catch` 表達式：對控制流（代數效應）的捕獲和模式匹配

### 4. 執行模型的二元性 (Duality of Execution Models)

- **動態解釋/JIT 執行**: 專為開發、調試和交互式環境設計，內建完整運行時 (Nyar VM)
- **靜態 AOT 編譯**: 專為生產部署設計，通過 Gaia 發射為輕量、高效的原生二進制或 WebAssembly 模塊

### 5. 零成本抽象的最終承諾 (Zero-Cost Abstraction)

高級抽象在編譯後應與手寫的最優底層代碼同樣高效。

### 6. 確定性資源管理 (Deterministic Resource Management)

Valkyrie 通過 **Nyar VM** 實現了 RAII 與垃圾回收的深度整合。由於我們掌控了托管語言從源碼到 UIR 的編譯全流程，我們可以為托管語言提供以下特性：
- **終結器 (Finalizer)**: 托管對像在生命週期結束時自動觸發其終結邏輯（底層映射為 Rust 的 `Drop` 特性或統一的 `Finalizer` Trait）。
- **資源安全**: 即使在 GC 環境下，也能讓托管語言像 C++/Rust 一樣安全、及時地管理非內存資源（如 FFI 對象、文件句柄等）。

## 核心模組詳解

### oak-valkyrie: 編譯器前端實現
**職責**: 提供 Lexer, Parser 和 AST 定義，將源文本解析為抽象語法樹。

### valkyrie-types: 中間表示類型定義
**職責**: 集中管理 HIR, UIR (IKun) 等各階段的中間表示類型定義。

### nyar-vm: 虛擬機與編譯器核心
**職責**: 實現從 HIR 到 UIR 的降級，並驅動 Chomsky 優化與 Gaia 後端生成。

### valkyrie-error: 統一錯誤處理
**職責**: 提供集中的錯誤定義和診斷信息輸出。

## 維護流程

### 代碼審查標準
1. **架構一致性**: 確保新代碼符合五大設計支柱
2. **錯誤處理**: 使用統一的 `valkyrie-error` 系統
3. **性能考慮**: 避免不必要的分配和拷貝

### 調試指南
1. **編譯器錯誤**: 檢查 `valkyrie-error` 的診斷輸出
2. **代碼生成問題**: 使用 `--dump-{ast,hir,cfg,ssa,lir}` 選項轉儲中間層輸出

---

## 設計與實現專題

- [項目架構設計](project-architecture.md)
- [執行模型 (解釋與編譯)](execution-models.md)
- [對象降低 (Object Lowering)](object-lowering.md)
- [包管理與符號解析](package-management.md)
- [基於 Miette 的錯誤處理](error-handling.md)
- [性能優化策略](optimization-strategies.md)
- [後端實現與考量](backends/index.md)

---
本維護指南將隨著項目的發展持續更新。

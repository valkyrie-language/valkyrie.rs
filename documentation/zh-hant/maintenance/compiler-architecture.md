# Valkyrie 編譯器中間表示架構

Valkyrie 編譯器採用多層中間表示（IR）架構，為語言提供高效的編譯和優化基礎設施。通過四層漸進式轉換，Valkyrie 能夠將高級語言特性轉換為優化的目標代碼，支持 WebAssembly、JavaScript 和原生代碼等多種執行環境。

## 編譯器架構概覽

### 設計目標

Valkyrie 編譯器的設計專注於：
- **表達力轉換**: 將複雜的函數式和代數效應特性安全地降級
- **多級優化**: 提供從高級語義到低級指令的多層次優化
- **跨平台支持**: 統一的後端架構，支持多種目標平台
- **開發工具鏈**: 為 IDE 和調試器提供精準的元數據支持

### 編譯流水線 (Compilation Pipeline)

Valkyrie 的編譯過程被建模為從源代碼到機器碼的漸進式降級過程：

#### 1. 前端 (Frontend - Oaks)
- **輸入**: 源代碼
- **輸出**: AST (Abstract Syntax Tree) -> HIR (High-level IR)
- **職責**: 語法解析、名稱解析、類型檢查、Trait 特化、模式匹配解糖。
- **實現**: [oak-valkyrie](file:///e:/普遍优化/oaks/examples/oak-valkyrie)

#### 2. 中間表示降級 (Lowering - Chomsky UIR)
- **輸入**: HIR
- **輸出**: UIR (Universal Intermediate Representation / IKun)
- **職責**: 將高級語義圖轉換為基於 E-Graph 的通用中間表示，為全局優化做準備。
- **實現**: [valkyrie-compiler](file:///e:/普遍优化/valkyrie.rs/projects/valkyrie-compiler)

#### 3. 優化器 (Optimizer - Nyar VM / Chomsky)
- **輸入**: UIR
- **輸出**: 優化後的 UIR (IKun Tree)
- **職責**: 無論是 AOT 還是 JIT 模式，所有的核心優化任務（常數折疊、死代碼消除、循環優化等）均由 **Nyar VM** 驅動的 **Chomsky 優化引擎** 完成。
- **特點**: 基於 E-Graph 的等價飽和（Equality Saturation）技術，能夠發現傳統編譯器難以觸及的深度優化空間。
- **實現**: [ProjectChomsky](file:///e:/普遍优化/ProjectChomsky)

#### 4. 後端生成 (Backend - Nyar VM / Gaia)
- **輸入**: 優化後的 UIR
- **輸出**: 目標機器碼 (AOT) 或 內存可執行代碼 (JIT)
- **職責**: 寄存器分配、指令選擇、棧幀管理及最終代碼發射。
- **實現**: [nyar-vm](file:///e:/普遍优化/nyar-vm) 與 [project-gaia](file:///e:/普遍优化/project-gaia)

## 各層中間表示的職責

### [AST - 抽象語法樹](file:///e:/普遍优化/oaks/examples/oak-valkyrie)

AST 層作為編譯器的入口點，接收來自語法解析器的語法樹，並提供統一的語言特性抽象。

### [HIR - 高級中間表示](file:///e:/普遍优化/valkyrie-compiler/projects/valkyrie-compiler)

HIR 層是語義分析的核心，負責將 AST 轉換為帶有完整類型和語義信息的中間表示。

### [UIR/IKun - 通用中間表示](file:///e:/普遍优化/ProjectChomsky)

UIR 層是優化的核心，將高級語義轉換為基於 E-Graph 的通用中間表示。

### [Nyar IR - 字節碼中間表示](file:///e:/普遍优化/nyar-vm/documentation/zh-hans/maintenance/nyar-ir.md)

Nyar IR 層是代碼生成的核心，為 Nyar VM 提供高效的字節碼抽象。詳見 [nyar-ir](file:///e:/普遍优化/nyar-vm/documentation/zh-hans/maintenance/nyar-ir.md)。

## 編譯器的核心價值

Valkyrie 通過這套多層架構，確保了代碼在保持高級抽象的同時，能夠獲得接近原生的執行性能。開發者可以利用代數效應等前沿特性，而不必擔心運行時的性能損耗，因為編譯器會在轉換過程中通過深度的靜態分析將其優化為高效的控制流。

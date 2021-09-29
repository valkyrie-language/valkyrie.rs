# Nyar VM 維護指南

Nyar VM 是 Valkyrie 的主要運行時和優化後端，支持 AOT (Ahead-of-Time) 編譯和 JIT (Just-in-Time) 執行模式。

## 核心編譯流水線

Valkyrie 編譯器在完成前端語義分析後，將控制權移交給 Nyar VM 進行優化和生成：

`Source -> AST -> HIR -> UIR (Chomsky) -> Optimized UIR -> Backend Emission`

### 1. HIR -> UIR (Lowering)
- **職責**: 將語言特定的高級語義（如 Valkyrie 的 Effect System, Pattern Matching）映射到 Chomsky 的通用中間表示 (UIR/IKun)。
- **實現**: [valkyrie-compiler](file:///e:/普遍优化/valkyrie.rs/projects/valkyrie-compiler/src/pipeline/mod.rs) 中的 `lower_root_to_uir`。

### 2. UIR Optimization (Chomsky)
- **職責**: 核心優化階段。
- **技術**: 使用基於 E-Graph 的等價飽和技術。
- **實現**: [ProjectChomsky](file:///e:/普遍优化/ProjectChomsky) 中的 `UniversalOptimizer`。
- **優勢**: 
    - 無論是 AOT 還是 JIT，共享同一套優化邏輯。
    - 極強的全局優化能力（內聯、死代碼消除、常數傳播等）。

### 3. AOT 模式
- **職責**: 靜態生成可執行二進制文件或 WASM 模塊。
- **流程**: 使用 `NyarAot` 驅動優化流程，並通過 `Gaia` 匯編器生成目標文件。
- **實現**: [nyar-aot](file:///e:/普遍优化/nyar-vm/projects/nyar-aot/src/lib.rs)。

### 4. JIT 模式
- **職責**: 在運行時根據熱點代碼生成並執行機器碼。
- **流程**: 利用 `gaia-jit` 動態發射指令到內存並執行。
- **實現**: [nyar-jit](file:///e:/普遍优化/nyar-vm/projects/nyar-jit/src/lib.rs)。

## 與 ProjectChomsky 的集成

Nyar VM 通過 `IKun` 接口與 Chomsky 深度集成。編譯器生成的意圖（Intents）被送入 Chomsky 的 E-Graph 中，經過規則重寫達到等價飽和狀態後，提取出代價最小的樹結構用於後端生成。

## 維護建議

- **優化規則**: 如果需要添加新的優化邏輯，應在 `ProjectChomsky` 中添加重寫規則。
- **後端適配**: 如果需要支持新的指令集或平台，應在 `project-gaia` 中添加新的匯編器適配。

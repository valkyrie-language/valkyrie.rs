# Valkyrie 編譯器：降級指南 (Lowering Guide)

## 1. 降級哲學：從語義到意圖

Valkyrie 編譯器的核心架構已從傳統的「漸進式降級」演進為**意圖映射 (Intent Mapping)**。核心目標是將高級語言特性降級為 **ProjectChomsky** 可理解的通用中間表示 (UIR/IKun)。

```mermaid
graph TD
    subgraph Frontend (valkyrie-compiler)
        A[源代碼] -->|解析| B(<b>Oaks AST</b>);
        B -->|語義分析| C(<b>HIR</b><br><i>類型, 作用域, Traits</i>);
    end

    subgraph Mid-end (Lowering & Optimization)
        C -->|<b>UIR Lowering</b>| D(<b>Chomsky UIR</b><br><i>意圖圖, IKun Tree</i>);
        D -->|<b>Equality Saturation</b>| E(<b>Optimized UIR</b><br><i>Nyar VM / Chomsky</i>);
    end

    subgraph Backends (Nyar VM / Gaia)
        E -->|AOT Emission| F[<b>Native Binary</b>];
        E -->|JIT Execution| G[<b>Memory Execution</b>];
        E -->|WASI Export| H[<b>WASM Module</b>];
    end
```

## 2. 現代降級流程 (Modern Lowering)

為了充分利用 Nyar VM 的優化能力，Valkyrie 採用統一的降級路徑：

### 2.1 降級到 UIR 的優勢

| 特性 | 傳統 (SSA/LIR) | 現代 (Chomsky UIR) |
| :--- | :--- | :--- |
| **控制流** | 顯式跳轉/基本塊 | 聲明式意圖 (If/Loop Intents) |
| **優化時機** | 固定順序的 Pass | 基於代價模型的全局等價飽和 |
| **後端適配** | 需要為每個後端寫代碼發射 | 統一由 Gaia 驅動，後端只需定義代價模型 |

## 3. 特性降級示例：模式匹配 (Pattern Matching)

模式匹配是 Valkyrie 的核心特性。我們將追蹤它如何降級為 Chomsky 意圖。

### 3.1 AST -> HIR
- **模式解析**: 識別嵌套模式和守衛。
- **類型綁定**: 為每個模式分量分配類型。

### 3.2 HIR -> UIR (Lowering)
- **決策意圖 (Decision Intents)**: 將 `match` 轉化為一系列嵌套的 `Select` 意圖。
- **數據流映射**: 將模式中的變量綁定映射為 UIR 中的 `Define` 或 `Bind` 節點。
- **窮盡性檢查**: 仍在 HIR 階段完成，確保生成的 UIR 樹是邏輯完整的。

### 3.3 UIR Optimization (Chomsky)
- **分支折疊**: 如果匹配器是常量，Chomsky 會通過等價重寫直接消除不必要的分支。
- **等價合併**: 如果多個分支的執行意圖相同，它們會在 E-Graph 中被合併。

## 4. 特性降級示例：控制流與效應 (Control Flow & Effects)

Valkyrie 的高級控制流（如異常處理、異步和代數效應）通過 **Nyar VM** 的原生延續 (Continuation) 支持實現。

### 4.1 HIR -> UIR
- **效應降級**: 將 `try/catch` 映射為 UIR 的 `EffectScope` 和 `Handle` 意圖。
- **延續捕獲**: 將 `raise` 和 `resume` 顯式化為 UIR 的 `Continuation` 調用。

### 4.2 Nyar VM 執行/生成
- **AOT 模式**: Nyar VM 將控制流映射為目標平台的高效狀態機實現（如 CPS 變換或輕量級線程）。
- **JIT 模式**: 直接利用 Nyar VM 的原生協程和延續處理器實現，減少上下文切換開銷。

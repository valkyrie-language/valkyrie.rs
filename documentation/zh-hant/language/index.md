# Valkyrie 語言參考 (Language Reference)

Valkyrie 是一門現代的函數式程式語言，融合了代數效應系統、強型別系統以及極致自由的語法設計。本參考文檔詳細介紹了語言的各個組成部分。

## 目錄結構

### [1. 核心語法 (Core Syntax)](./syntax/index.md)
基礎語法構造、變數定義、控制流以及核心解析邏輯。

### [2. 型別系統 (Type System)](./type-system/index.md)
代數資料型別（ADT）、泛型、關聯型別、依賴型別以及統一的型別求值引擎。

### [3. 程式模式 (Patterns)](./patterns/index.md)
構建器模式（Builder Pattern）等高級程式範式。

### [4. 程式範式 (Paradigms)](./index.md)
- **[物件導向 (Object-Oriented)](./object-oriented/index.md)**：類別、繼承、Trait 系統、Widget 元件。
- **[函數導向 (Function-Oriented)](./function-oriented/index.md)**：匿名函數、模式匹配。
- **[響應式程式設計 (Reactive)](./reactive-programming/index.md)**：訊號（Signal）、流（Stream）、非同步。
- **[元程式設計 (Meta-Programming)](./meta-programming/index.md)**：宏系統、編譯時計算、單元系統。

### [5. 進階特性 (Advanced)](./index.md)
- **[效應系統 (Effect System)](./effect-system/index.md)**：代數效應、非同步、錯誤處理、IoC。
- **[生命週期與記憶體 (Lifetime)](./lifetime/index.md)**：所有權、生命週期、分配器。
- **[模組系統 (Module System)](./module-system/index.md)**：專案管理、工作區、外部函數介面。

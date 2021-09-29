# 編譯時計算 (Compile-time Computation)

Valkyrie 允許在編譯期間執行程式碼，從而實現零開銷的抽象和高度靈活的程式碼生成。

## 常數運算式 (evaluate)

使用 `evaluate` 編譯器內建函式可以強制要求編譯器在編譯時計算運算式的值。

```valkyrie
# 編譯時計算斐波那契數列
let FIB_20: i32 = evaluate(fibonacci(20))

# 編譯時格式化字串
let VERSION: string = evaluate(f"v{1}.{0}.{5}")
```

## 編譯時函式 (@const_fn)

只有標記為 `@const_fn` 的函式才能在編譯時被安全地執行。這些函式必須是純函式（無副作用）。

```valkyrie
@const_fn
micro square(n: i32) -> i32 {
    n * n
}

# 合法呼叫
let X: i32 = evaluate(square(10))
```

## 編譯時反射

編譯器提供了一系列內建指令來獲取型別資訊或環境資訊：

- `type_of(expr)`: 獲取運算式的型別。
- `name_of(sym)`: 獲取符號的名稱字串。
- `env("VAR_NAME")`: 讀取編譯環境的環境變數。
- `is_defined(sym)`: 檢查某個符號是否已定義。

## 外部資源嵌入

你可以在編譯時讀取外部檔案並將其內容嵌入到產生的二進位檔案中：

```valkyrie
# 嵌入文字檔案
let SHADER_SOURCE: string = evaluate(read_file("src/shaders/basic.glsl"))

# 嵌入二進位檔案
let ICON_DATA: [u8] = evaluate(read_bytes("assets/icon.png"))
```

## 為什麼使用編譯時計算？

1. **效能**: 將執行階段開銷轉移到編譯時。
2. **驗證**: 在編譯階段捕捉無效的配置或參數。
3. **靈活性**: 根據環境參數（如開發/生產模式）產生不同的程式碼路徑。

---
**相關章節**:
- [宏系統](./macro-system.md) - 更高級的程式碼生成工具
- [型別函式](../type-system/type-function.md) - 型別層面的編譯時計算

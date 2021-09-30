# 大括號 `{}` 的用法列舉

在 Valkyrie 語言中，大括號 `{}` 承擔了多種語法功能。為了保持解析器的簡潔，我們將大括號的用法分為以下幾類。

## 1. 命名空間與定義區塊 (Namespace & Definition Blocks)

用於組織程式碼結構，內部包含定義語句（如 `let`, `class`, `fn` 等）。

```valkyrie
namespace Core {
    class Point {
        x: f32,
        y: f32,
    }
}
```

## 2. 控制流區塊 (Control Flow Blocks)

用於 `if`, `match`, `loop`, `for`, `while` 等控制流語句。

```valkyrie
if condition {
    // 語句區塊
} match x {
    case 1 => { /* 分支區塊 */ }
}
```

## 3. 應用區塊 (ApplyBlock)

這是 Valkyrie 最具特色的語法構造。當 `{}` 緊跟在表達式（如型別名稱、函式呼叫等）後時，解析器將其解析為 **ApplyBlock**。

### 核心設計：結構優先，語義後置

為了保持解析器的極簡與高效能，Valkyrie 的解析器對 ApplyBlock 採取了「只管掛樹，不問對錯」的策略。

#### 語法同構性
在解析階段，以下形式在語法樹上是完全同構的：
- **物件初始化**：`Point { x: 1.0, y: 2.0 }`
- **尾隨閉包**：`list.map { $ * 2 }`
- **宣告式 DSL**：`Node { child { "Hello" } }`

#### 極致的自由度
ApplyBlock 內部被解析為一個**通用的語句流 (Statement Stream)**。這意味著你可以在區塊內混雜各種節點：
- **賦值語句**：`width = 100`
- **方法/函式定義**：`modifier on_click(self) { ... }`
- **嵌套區塊**：`header { title: "Welcome" }`
- **控制流**：`if condition { ... }`

具體的語義解析由後續的型別系統決定。

---

### 應用場景一：物件構造 (Object Construction)

當 ApplyBlock 的左側是一個類別或結構體時，它被解釋為物件構造模式。

#### 直接初始化 `C { ... }`
區塊內語句直接映射為欄位賦值。
```valkyrie
let config = Config {
    debug = true
    port = 8080
}
```

#### 後初始化 `C() { ... }`
先執行建構函式，再在大括號內進行補充配置。
```valkyrie
let user = User("Alice") {
    verified = true
    bio = "Valkyrie developer"
}
```

---

### 應用場景二：尾隨閉包 (Trailing Closures)

當 ApplyBlock 的左側是一個函式或微函式時，區塊被視為該函式的最後一個參數。

```valkyrie
# 函式呼叫範例
request.send {
    header("Content-Type", "application/json")
    on_success { print("Done!") }
}
```

#### 隱式接收者 (Implicit Receivers)
在某些 DSL 上下文中，ApplyBlock 內部的 `self` 會自動指向當前建構的物件，使得你可以直接呼叫方法而無需顯式引用。

---

### 應用場景三：宣告式 DSL (如 V-Grammar)

ApplyBlock 語法廣泛應用於建構嵌套的宣告式結構。透過利用 ApplyBlock 的自由度，開發者可以在描述結構的同時無縫嵌入邏輯。

有關在 UI 開發中的具體應用，請參閱：
- **[原生 UI 語法 (V-Grammar)](./v-grammar.md)**

---

### 底層邏輯：解析與驗證

#### 解析演算法
解析器將 `{` 視為一個**高優先權中綴運算子**。當遇到 `{` 且左側已有表達式時，解析器會遞迴地將後續內容封裝進 ApplyBlock 節點。

#### 分層驗證機制
1.  **Type Checker**：在語義分析階段，根據左側呼叫者的型別簽名判定區塊內每個節點的含義。
2.  **Linter**：在型別檢查後介入。如果 Linter 發現你在一個純資料結構的初始化區塊中定義了方法，或者在不支援的上下文中使用了型別宣告，它會拋出編譯錯誤。

## 4. 字串插值 (String Interpolation)

在普通字串中使用 `{}` 嵌入表達式。由 Lexer 處理，不進入 Pratt 解析邏輯。
原始字串不做插值；若要輸出字面量花括號，使用 `\{` 與 `\}`。

```valkyrie
print("Hello, {name}!")
print("Template: \{name\}")
```

## 5. 集合字面量 (Collection Literals) - 預留

未來可能用於 Set 或 Map 的字面量表示。

```valkyrie
let set = {1, 2, 3}
let map = {"key" = "value"}
```

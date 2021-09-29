# S-Grammar (字串語法)

Valkyrie 的字串語法設計遵循「詞法與語義分離」原則。由底層的 **S-Grammar** 負責物理邊界掃描，再由應用層的 **DSL** 負責內容解析。

---

## 1. 詞法層：S-Grammar

S-Grammar 是一種極其簡單的字串掃描規則，其核心是 **等量開閉原則 (Symmetric Delimiter)**。

### 1.1 核心原則
- **空字串 ($N=2$)**：兩個連續的引號（`""` 或 `''`）直接識別為空字串。
- **等量開閉原則**：除了 $N=2$ 以外，使用 $N$ 個引號開始，就必須使用 $N$ 個引號結束。
- **天然多行**：任何 $N$ 的字串（包括 $N=1$）都可以包含換行符。
- **無轉義與無插值**：S-Grammar 層面不解析任何字元，所有的 `\` 和 `{}` 都會被原樣掃描。

### 1.2 詞法示例
```valkyrie
# 1. 空字串 (N=2)
let empty = "" 

# 2. 標準字串 (N=1)
let s1 = "Hello, Valkyrie ⚔️"
let s2 = "可以
跨行"

# 3. 對稱匹配 (N=3)
let s3 = """
這裡可以包含 "引號" 而不需要轉義
"""

# 4. 高階匹配 (N=4)
let s4 = """" 這裡可以包含 """ 符號 """"
```

---

## 2. 應用層：DSL (Domain Specific Language)

當 S-Grammar 完成邊界掃描後，其內部內容將交給應用層 DSL 進行解釋。Valkyrie 編譯器僅內置了少數核心前綴的處理邏輯，其它的前綴被視為 **Tagged Strings**，由用戶定義的庫或宏在後續階段（如語義分析或過程宏）進行解析。

### 2.1 Slot String (s / 預設) 🦄
Valkyrie 的插值字串設計旨在提供高效能的內聯插值。它是預設的字串行為。

- **變數插值**：使用 `{name}` 嵌入變數。
- **表達式插值**：支援在 `{}` 中編寫任意 Valkyrie 表達式。
- **符號轉義**：支援 `\{` 和 `\}` 來表示字面量的花括號。
- **字元轉義**：支援 `\n`, `\r`, `\t`, `\\` 等標準轉義，以及 `\u{...}` Unicode 轉義。

```valkyrie
let name = "Alice"
# 預設即為 Slot String
let s1 = "Hello, {name}" 
# 顯式使用 s 前綴效果相同
let s2 = s"Hello, {name}"
```

### 2.2 Localized String (i18n) 🌍
Valkyrie 整合了 **Project Fluent** (Mozilla) 作為其國際化引擎。
通過在插值變數前添加 `߷` (Gwot) 符號，該插值將被標記為「Fluent 變數」。

- **Fluent 整合**：Valkyrie 編譯器會將包含 `߷` 的字串自動映射到 `.ftl` 資源檔案中的訊息。
- **高級語法支援**：支援 Fluent 的複數形式（Plurals）、性別（Gender）以及選擇器（Selectors）。
- **類型安全變數**：插值中的變數名直接對應 Fluent 訊息中的參數。

```valkyrie
let name = "Valkyrie"
let count = 3

# 1. 基礎翻譯
# 映射到 Fluent: hello-user = Hello, { $name }!
let s1 = "Hello, {߷name}!"

# 2. 複數形式與屬性
# 映射到 Fluent: 
# shared-photos = { $userName } added { $photoCount ->
#     [one] a new photo
#    *[other] { $photoCount } new photos
# } to the album.
let s2 = "{߷userName} added {߷photoCount} new photos to the album."
```

- **自動鍵生成規則**：如果沒有顯式指定 Key，編譯器會根據原始文本生成唯一的 Slug 作為 Fluent 識別符。

### 2.3 Format String (f) 🏗️
**Format String** 通過 `f` 前綴啟用。它不捕捉當前作用域，而是聲明一個**函數模板**。它更像 C++ 的 `std::format` 或 `std::vformat`，適用於延遲綁定或日誌庫。

```valkyrie
# 聲明一個模板
let log_template = f"ID:{} - Event: {}"
# 之後再進行綁定
let message = log_template.format(1024, "Login Success")
```

### 2.4 Template String (t) 🎭
通過 `t` 前綴啟用。它支援類似 Handlebars 或 Jinja2 的模板語法，通常用於多行文本生成。

```valkyrie
let tpl = t"
<% loop user in users %>
  - {user.name}
<% end loop %>
"
```

### 2.5 Raw String (r) 🧱
通過 `r` 前綴啟用。它會完全禁用插值和轉義處理，實現真正的「所見即所得」。

```valkyrie
# 路徑無需擔心轉義
let path = r"C:\Windows\System32"
```

### 2.6 其它 DSL (Tagged Strings) 📦
除了上述核心前綴（`s`, `f`, `t`），Valkyrie 支援任意識別符作為前綴。編譯器會將這些字串識別為「帶有標籤的原始文本」，其內容不進行預設的插值解析，由對應的庫在語義階段或宏中進行處理。

- **Regex (`re`)**：由正則庫處理。
- **Byte String (`b`)**：解析為位元組陣列 `[u8]`。
- **JSON5 (`json`)**：解析為 JSON5 物件。

```valkyrie
# 編譯器不硬編碼 re 的邏輯，而是將其作為 Tagged String 傳給正則庫
let pattern = re"^\d+$"

# 用戶也可以定義自己的前綴
let sql = sql"SELECT * FROM users WHERE id = {id}"
```

---

## 3. 最佳實踐總結 💡

1.  **優先使用 Slot String**：內聯插值（`"{name}"`）最直觀，適用於 90% 的字串拼接場景。
2.  **國際化必用 ߷**：在需要翻譯的變數前加上 `߷`，讓編譯器和翻譯引擎自動完成 Key 提取和 Locale 替換，無需手動維護 ResourceBundle。
3.  **日誌與庫開發使用 Format String**：`f"{}"` 帶來的延遲綁定和編譯時檢查非常適合高效能或複用場景。
4.  **善用 N 階差異**：遇到需要包含引號的文本時，不要尋找轉義，而是增加外部引號的數量（如 `"""..."""`）。
5.  **路徑必用 r-string**：避免在 Windows 路徑中陷入 `\\` 的泥潭。
6.  **Emoji 友好**：Valkyrie 源碼和字串均採用 UTF-8 編碼，請放心使用 🚀。

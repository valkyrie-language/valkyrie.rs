# X-Grammar

在掌握了 [Widget](../../examples/web-development/widget.md) 的物件模型和 [V-Grammar](./v-grammar.md) 的閉包語法後，X-Grammar 為我們提供了 UI 邏輯的**視覺投影**。

與 V-Grammar 一致，X-Grammar 同樣提供兩個應用版本：**跨平台組件風格**與 **Web HTML 風格**。

---

## 1. 跨平台組件風格

這一風格將 X-Grammar 標籤映射到跨平台的 UI 組件（如 `Column`, `Button`）。它適用於需要 X-Grammar 視覺結構的非 Web 環境。

```xml
<Column spacing=10 alignment=".center">
    <Image src="logo.png" width=100 height=100 />
    
    <Text font=".headline" color=".blue">
        歡迎回來
    </Text>
    
    <Button on_click={ navigate_to("/dash") }>
        進入控制台
    </Button>
</Column>
```

---

## 2. Web HTML 風格

這一風格直接映射到標準 HTML 標籤，適用於 Web 開發和伺服器端渲染（SSR）。

```xml
<div class="container">
    <h1>歡迎來到 Valkyrie</h1>
    
    <!-- disabled 接受布林值，on_click 接受閉包 -->
    <button disabled=(count >= 10) on_click={ count += 1 }>
        <if (count == 0)> 開始 <else/> 繼續 </if>
    </button>
    
    <p>當前進度：$progress%</p>
</div>
```

---

## 3. 基礎語法與屬性綁定

X-Grammar 使用標籤來描述 UI 結構。所有的交互和數據流動都通過統一的屬性綁定實現：

- **立即屬性 `( )`**：用於需要立即計算並賦值的場景。
    - **字面量簡寫**：`name="value"` 或 `name=10`。
    - **識別符簡寫**：`name=variable`。
    - **表達式求值**：`name=(expression)`。
- **閉包屬性 `{ }`**：用於傳遞邏輯塊（閉包）。在底層，這通常對應於 Widget 的事件註冊方法（如 `on_click`）。
- **內容插值 `${ }` / `$ident`**：在標籤文本內容中，使用 `$` 引導進行動態插值。

```xml
<div class="container">
    <h1>歡迎來到 Valkyrie</h1>
    
    <!-- disabled 接受布林值，on_click 接受閉包 -->
    <button disabled=(count >= 10) on_click={ count += 1 }>
        <if (count == 0)> 開始 <else/> 繼續 </if>
    </button>
    
    <!-- 事件轉發：本質上就是將父組件傳入的閉包 prop 傳遞給子組件 -->
    <CustomWidget on_click=on_click />
    
    <p>當前進度：${progress}%</p>
</div>
```

## 4. 邏輯關鍵字 (Logic Keywords)

在 X-Grammar 模式下，邏輯標籤（`if`, `else`, `match`, `loop`, `slot`）不再是普通的 UI 組件，而是被晉升為 **原生關鍵字 (Native Keywords)**。這意味著它們擁有特殊的 Parser 語法支援，能夠直接映射到 Valkyrie 的核心控制流。

### 條件渲染 (`<if>`)
支援標準的 `if-else` 結構，括號內為布林表達式。由於是關鍵字，它支援更靈活的嵌套和簡寫。
```xml
<if (count > 5)>
    <p>計數已過半</p>
<else/>
    <p>繼續努力</p>
</if>
```

### 模式匹配 (`<match>`)
直接映射到 Valkyrie 的 `match` 語句，支援類型匹配和解構。
```xml
<match (user.role)>
    <case "admin">  <badge>管理員</badge> </case>
    <case "user">   <badge>普通用戶</badge> </case>
    <else>          <badge>訪客</badge>    </else>
</match>
```

### 循環迭代 (`<loop>`)
支援 `loop ... in ...` 語法。由於作為關鍵字處理，Parser 可以更精確地解析迭代器和解構賦值。
```xml
<loop (item, index) in (list)>
    <li key=index>${item.name}</li>
<else/>
    <p>列表為空</p>
</loop>
```

### 內容投影 (`<slot>`)
`<slot>` 是用於內容投影的關鍵字。它不是一個真實的 DOM 節點，而是一個**編譯器佔位符**，指示組件欄位的渲染位置。

#### 1. 聲明與基本用法
在 `widget` 定義中，使用 `$` 引導欄位名來標記槽位：
```valkyrie
widget Card {
    header: Widget
    content: [Widget]
    
    <div class="card">
        <slot $header />
        <div class="card-content">
            <slot $content />
        </div>
    </div>
}
```

#### 2. 作用域插槽 (Scoped Slots)
如果欄位是一個工廠函數（如 `micro`），可以通過屬性語法傳遞參數，實現數據的反向傳遞：
```valkyrie
widget List {
    items: [T]
    render_item: micro(T) -> Widget

    <div class="list">
        <loop item in (items)>
            <slot $render_item=(item) />
        </loop>
    </div>
}
```

#### 3. 預設內容
當父組件未提供對應內容時，渲染標籤內部的子節點：
```xml
<slot $footer>
    <p>這是預設頁腳</p>
</slot>
```

---

## 5. 擴展：單文件組件 (SFC)

單文件組件 (Single File Component) 是 X-Grammar 的一種高級應用模式。它通過頂級標籤組織不同的關注點，其中 `<template>` 塊包含 X-Grammar 視圖。

```xml
<template>
    <div class="container">
        <h1>Hello, $name</h1>
        <!-- handleClick 是一個函數，作為閉包傳遞 -->
        <button on_click=handleClick>
            點擊次數: $count
        </button>
    </div>
</template>

<script>
let name = "Valkyrie";
let count = 0;

micro handleClick() {
    count += 1;
}
</script>
```

頂級標籤通常包括：
- `<template>`：視圖模板（X-Grammar）。
- `<script>`：邏輯程式碼（Valkyrie 程式碼）。
- `<style>`：樣式定義。
- `<router>`：路由配置。
- `<meta>`：元數據定義。

## 6. 語法對比與原理

X-Grammar 沒有任何「魔法指令」，它的所有標籤 and 屬性都會 1:1 地轉換為 [V-Grammar](./v-grammar.md) 中對應的屬性賦值或閉包傳遞。

| X-Grammar | 語義 | V-Grammar 等效程式碼 |
| :--- | :--- | :--- |
| `name=(val)` | 屬性賦值 (立即) | `.name(val)` 或 `name = val` |
| `name={...}` | 閉包傳遞 (延遲) | `name { ... }` 或 `on_name { ... }` |
| `$ident` / `${expr}` | 文本插值 | 轉換為字串並渲染 |
| `<if (cond)>` | 條件分支 | `if cond { ... }` |
| `<match (val)>` | 模式匹配 | `match val { ... }` |
| `<loop (i) in (L)>` | 循環迭代 | `loop i in L { ... }` |

---

## 7. 魔法的真相：邏輯的視覺投影

**雖然 X-Grammar 看上去很魔法，但本質上沒有那麼多魔法。** 它不是一個龐大的運行時框架，而是一層輕薄的、直觀的**語法投影**。

### 1. 零指令設計 (Zero Directive)

Valkyrie 不需要 `v-bind`, `on:` 或 `v-if` 這種「屬性指令」，因為 X-Grammar 深度信任底層的物件模型。

- **回歸編程本質**：如果一個 Widget 有 `on_click` 方法或 `disabled` 欄位，你就在 X-Grammar 裡直接寫 `on_click` 或 `disabled`。
- **Valkyrie 的統一方案**：
    - **邏輯歸關鍵字**：`<if>`, `<loop>`, `<slot>` 等邏輯容器直接處理結構控制。
    - **內容歸插值**：使用 `$ident` 或 `${expr}` 進行動態文本注入。
    - **片段歸屬性**：所有的「插槽傳遞」在 Valkyrie 中都被統一為**帶塊的屬性賦值**。

#### 場景 1：直接在標籤內定義 (Inline Slot)
```xml
<Card>
    <!-- 1. 具名傳遞：header 是 Card 的一個屬性/方法 -->
    <header>
        <Icon name="user" />
        <Text>用戶資訊</Text>
    </header>
    
    <!-- 2. 匿名傳遞：直接寫在標籤內的內容通常映射到 default 屬性 or appendChild -->
    <p>這是卡片的正文內容</p>
</Card>

#### 場景 2：顯式視圖函數 (Explicit View Function)

在某些複雜的邏輯腳本中，你可能不通過「腳本末尾的表達式」來隱式定義視圖。此時可以顯式定義一個 `view` 函數。這個函數不需要放在腳本末尾，它會被自動識別為組件的渲染入口。

```valkyrie
# 顯式定義視圖，無需放在文件末尾
micro view() {
    let x = xxx;
    <div class="layout">
        <header> $title </header>
        <main> $content </main>
    </div>
}

# 邏輯程式碼可以放在 view 之後
let title = "系統概覽";
let content = "這裡是主要的業務邏輯內容...";
```

| 特性 | 隱式視圖 (Trailing Expression) | 顯式視圖 (View Function) |
| :--- | :--- | :--- |
| **定義方式** | 腳本最後一行是一個 X-Grammar 標籤 | 定義一個名為 `view` 的函數 |
| **位置要求** | 必須在末尾 | 可以在腳本的任何位置 |
| **適用場景** | 簡單組件、快速原型 | 複雜邏輯、需要明確入口的組件 |

---

### 2. 括號的力量

通過 `()`, `{}` 和 `${}`，X-Grammar 在編譯階段就明確了「值」、「邏輯」與「插值」的區別。

- **`()` (Value)**: 立即求值的靜態或動態數據。
- **`{}` (Block)**: 延遲執行的程式碼塊或物件配置。
- **`${}` (Hole)**: 注入到文本環境中的動態表達式。

這種區分消除了歧義，並允許編譯器生成最優化的底層程式碼。

### 3. 靜態轉換：消失的開銷

所有的 X-Grammar 語法在編譯階段都會被「拍扁」成最高效的原生方法鏈。這種「非魔法」的設計，讓 Valkyrie 既擁有了 X-Grammar 的直觀，又徹底消除了傳統前端框架帶來的學習成本和運行負擔。一切你看到的「魔法」，最終都只是標準的編程概念在視覺上的延伸。

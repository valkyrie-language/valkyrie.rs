# 交集類型與並集類型 (Intersection & Union Types)

Valkyrie 的類型系統支援代數化的類型組合。通過交集與並集操作，你可以構建出極具表達力的複合類型，精準描述數據的結構特徵。

## 並集類型 (Union Types / Unite)

並集類型表示一個值可以是多種類型中的**其中之一**。Valkyrie 使用顯式 `tag` 的 `unite` 來定義具名的封閉變體族，標準寫法是 `[tag(XXXKind)] unite XXX { }`。

### 1. 具名並集 (`unite`)
這是最常用的形式，通過顯式的標籤（Variants）來區分不同的狀態。

```valkyrie
[tag(ShapeKind)]
unite Shape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Point
}
```

### 2. 匿名並集 (Anonymous Unions)
在某些臨時場景下，可以使用 `|` 符號組合類型：

```valkyrie
# 變數可以是 i32 或 utf8
let data: i32 | utf8 = 42
```

> **設計筆記**：Valkyrie 嚴格區分了**狀態疊加**與**屬性擴展**。`|` 符號現在專門用於並集類型（Union Types），表示「或者是」的關係；而記錄（Record）中的行擴展則採用 `, ...R` 語法，這與模式匹配中的物件展開操作保持語義一致。詳見 [行類型與多態](./row-types.md#語法一致性與設計-syntax--consistency)。

### 3. 語義特徵
- **排他性**：在任一時刻，並集類型的值只能屬於其定義的某一個分支。
- **窮盡性檢查**：編譯器強制要求在 `match` 表達式中處理並集類型的所有可能分支。

---

## 交集類型 (Intersection Types)

交集類型表示一個值必須**同時滿足**多種類型的約束。Valkyrie 使用 `&` 符號來表達交集。

### 1. 結構化交集
交集類型常用於要求一個類型同時實現多個接口（Traits）：

```valkyrie
# 變數必須同時實現 Display 和 Clone 特徵
micro process_data(item: Display & Clone) {
    print(item.fmt())
    let _ = item.clone()
}
```

### 2. 語義特徵
- **能力疊加**：交集類型擁有所有組成成員的方法和屬性總和。
- **多重約束**：它在邏輯上等價於泛型約束中的 `T: TraitA + TraitB`，但可以作為獨立的類型直接使用。

---

## 物理布局與優化

Valkyrie 編譯器會對這些複合類型進行深度物理優化：

1. **並集類型壓縮 (Tag Stripping)**：
   - 對於 `Option⟨ref T⟩` 等特殊並集，編譯器會利用「空位優化」消除標籤，使其物理大小等同於原始指針。
   - 對於欄位互斥的 `unite`，編譯器會通過記憶體重疊（Overlay）技術最小化記憶體佔用。

2. **交集類型扁平化**：
   - 交集類型在底層通常被處理為指向多個特徵虛表（Vtables）的「胖指針」集群，確保在多態調用時依然保持零成本抽象。

## 適用場景

- **並集類型**：狀態機建模、錯誤處理（Result）、可選值（Option）、多態異構容器。
- **交集類型**：插件系統、依賴注入、多特徵組合約束、精細化權限控制。

---

**上一頁**: [行類型與多態](./row-types.md) | **下一頁**: [型變與子類型](./polarity-type.md)

# AIFD 生命週期模型

在 Valkyrie 中，一個物件的完整生命週期被嚴謹地劃分為四個獨立且銜接的階段。這種精細的拆解不僅保證了記憶體安全，更賦予了開發者在不同場景下靈活組合記憶體管理策略的能力。

## 模型定義

1.  **A (Allocate)**：**記憶體分配**。從指定分配器 (Allocator) 中申請符合物件對齊與大小要求的原始記憶體空間。
2.  **I (Initiate)**：**狀態初始化**。在已分配的原始記憶體上進行「就地建構」，確立物件的初始邏輯狀態。
3.  **F (Finalize)**：**邏輯終結**。負責清理物件持有的非記憶體資源（如關閉檔案描述符、斷開網路連線、釋放互斥鎖或遞減外部引用計數）。
4.  **D (Delocate)**：**記憶體釋放**。將不再使用的物理記憶體空間歸還給原分配器，使其可被後續操作重用。

這種職責分離的設計（尤其是 F 與 D 的分離）是 Valkyrie 能夠同時支援 GC 與確定性析構的關鍵。

---

## 核心介面：Initiate 與 Finalize

開發者可以透過實作特定的 Trait 來介入物件的生命週期。

### 1. 狀態初始化：`Initiate`

`Initiate` 定義了如何將原始記憶體轉化為有效的物件狀態。

```valkyrie
trait Initiate⟨Args⟩ {
    # 安全性說明：呼叫者必須確保 ptr 指向的記憶體已透過 A 階段成功分配。
    unsafe micro initiate(ptr: ◆Self, args: Args)
}
```

### 2. 邏輯清理：`Finalize`

`Finalize` 專注於資源清理，**嚴禁**涉及物理記憶體釋放。

```valkyrie
trait Finalize {
    # 允許在物件被物理銷毀前，執行最後的資源釋放工作。
    micro finalize(mut self)
}
```

---

## 宣告式語法與自動化

為了提升開發體驗，Valkyrie 允許在型別定義中直接宣告 `initiate` 和 `finalize` 方法，編譯器會自動將其解構為標準 Trait 實作。

### 自動生成的實作

```valkyrie
class FileBuffer {
    path: Path,
    handle: ◆u8,

    # 映射為 Initiate⟨Path⟩
    initiate(mut self, path: Path) {
        self.path = path
        self.handle = intrinsic::open_file(path)
    }

    # 映射為 Finalize
    finalize(mut self) {
        intrinsic::close_file(self.handle)
    }
}
```

### 偽多載機制 (Pseudo-overloading)

雖然 Valkyrie 核心語法不支援傳統函式多載，但透過 `Initiate⟨Args⟩` 的泛型設計，類別可以擁有多個「建構函式」。

```valkyrie
class FileBuffer {
    path: Path,
    handle: ◆u8,
    is_temp: bool,

    # 映射為 Initiate⟨Path⟩
    initiate(mut self, path: Path) {
        self.path = path
        self.handle = intrinsic::open_file(path)
        self.is_temp = false
    }

    # 映射為 Initiate⟨Path, bool⟩
    initiate(mut self, path: Path, is_temp: bool) {
        self.path = path
        self.handle = intrinsic::open_file(path)
        self.is_temp = is_temp
    }

    # 映射為 Initiate⟨◆u8⟩
    initiate(mut self, handle: ◆u8) {
        self.path = Path::empty()
        self.handle = handle
        self.is_temp = false
    }

    finalize(mut self) {
        intrinsic::close_file(self.handle)
        if self.is_temp {
            intrinsic::delete_file(self.path)
        }
    }
}
```

**黑魔法原理**：當執行 `FileBuffer(...)` 時，編譯器並非在尋找同名函式，而是在尋找滿足 `Self: Initiate⟨T⟩` 約束的 Trait 實例化。這實作了靜態分派的靈活性與高效能。

---

## 型別判定與協定一致性

由於生命週期被抽象為 Trait，所有的元編程與型別判定依然統一：

- **Trait 判定**：`obj is Finalize` 可用於動態檢查物件是否需要清理邏輯。
- **預設行為**：若未顯式定義 `initiate`，編譯器將生成預設的零初始化建構函式；若省略 `finalize`，則該物件被視為「平凡物件 (Trivial)」，在銷毀時無需執行額外操作。
- **一致性協定**：`is` 和 `as` 等關鍵字始終透過 Trait 協定進行判定，確保了泛型約束（例如 `T: Finalize`）能完美相容手動實作的結構體與自動生成的類別。

---

## 下一步

你已經掌握了 AIFD 生命週期模型的基礎理論。接下來，我們將探討編譯器如何利用 **[作用域與自動插入](scope.md)** 技術，在代碼執行過程中自動、精準地編排這些生命週期階段。

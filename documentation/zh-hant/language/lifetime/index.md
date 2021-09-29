# 生命週期與記憶體管理 (Lifetime & Memory Management)

Valkyrie 採用分層設計的記憶體管理系統，旨在平衡開發效率（UX）與對底層的精細控制。Valkyrie 預設提供垃圾回收 (GC) 機制，同時也支援在高效能或嵌入式場景下進行顯式的記憶體管理。

## 核心支柱

Valkyrie 的記憶體管理體系建立在以下核心概念之上：

1.  **[AIFD 生命週期模型](lifecycle.md)**：將物件的生命週期嚴謹地劃分為記憶體獲取 (Allocate)、狀態準備 (Initiate)、邏輯清理 (Finalize) 和記憶體歸還 (Delocate) 四個清晰階段。
2.  **[作用域與靜態分析](scope.md)**：闡述 Valkyrie 如何利用確定性作用域與深度控制流分析，實現生命週期函數的全自動、智能化注入。
3.  **[引用類型 (Class)](class.md)**：作為應用層開發的預設選擇，提供零心智負擔的記憶體安全保障，底層由高效能垃圾回收 (GC) 引擎驅動。
4.  **[值類型 (Structure)](structure.md)**：為效能敏感型場景提供極致的記憶體控制能力，支援資料的內聯儲存。
5.  **[分配器 (Allocator)](allocator.md)**：為底層開發提供精細的記憶體控制，允許精確編排 AIFD 模型中的物理分配與釋放。
6.  **[外部物件 (Foreign Objects)](foreign-objects.md)**：定義了在與 C/C++/Rust 等外部語言進行互操作時，如何嚴謹地管理跨語言邊界的物件生命週期。

---

## 快速導航

- **探索物件從誕生到消亡的全過程？** 請參閱 [AIFD 生命週期](lifecycle.md)。
- **探究編譯器如何確定銷毀的時機？** 請參閱 [作用域與自動插入](scope.md)。
- **深入理解引用類型的運行機制？** 請參閱 [引用類型 (Class)](class.md)。
- **追求極致效能或進行底層系統開發？** 請參閱 [值類型 (Structure)](structure.md)。
- **需要自定義記憶體分配行為？** 請參閱 [分配器 (Allocator)](allocator.md)。
- **需要與現有的 C/Rust 庫進行無縫整合？** 請參閱 [外部物件互操作](foreign-objects.md)。

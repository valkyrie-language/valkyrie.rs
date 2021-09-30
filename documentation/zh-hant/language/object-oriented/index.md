# 物件導向程式設計 (Object-Oriented Programming)

Valkyrie 支援豐富的物件導向程式設計特性，包括類別定義、繼承、Trait 系統和屬性系統。

## 核心概念

### 類別 (Class)
類別是物件的藍圖，定義了資料欄位和行為方法。Valkyrie 的類別支援建構函式、解構函式、屬性和可見性控制。

### 繼承 (Inheritance)
通過繼承，子類可以獲得父類的屬性和方法，實現程式碼重用和多型。

### Trait 系統
Trait 定義了一組行為契約，類別可以實現多個 Trait，實現介面與實現分離。

### 屬性 (Property)
屬性提供了對欄位的封裝訪問，支援 getter 和 setter 的自定義邏輯。

## 主要特性

### [類別與繼承](./inheritance.md)
詳細介紹類別的定義、建構函式、繼承機制和方法重寫。

### [Trait 系統](./trait-system.md)
深入探討 Trait 的定義、實現和高級用法。

### [屬性系統](./property.md)
介紹屬性的定義、存取修飾符和自定義 getter/setter。

### [匿名類別](./anonymous-classes.md)
臨時定義的類別，常用於回調函數和事件處理。

### [值類別](./value-class.md)
輕量級的值型別，用於表示簡單的資料結構。

### [Widget 元件](./widget.md)
宣告式 UI 元件的基礎構建塊。

## 範例

```valkyrie
# 定義一個基礎類別
class Animal {
    name: string
    age: i32
    
    new(name: string, age: i32) {
        self.name = name
        self.age = age
    }
    
    micro speak(self) -> string {
        "Some sound"
    }
}

# 繼承
class Dog : Animal {
    breed: string
    
    new(name: string, age: i32, breed: string) {
        super(name, age)
        self.breed = breed
    }
    
    override micro speak(self) -> string {
        "Woof!"
    }
}

# 實現 Trait
trait Serializable {
    micro to_json(self) -> string
}

impl Serializable for Dog {
    micro to_json(self) -> string {
        f"\{\"name\": \"{self.name}\", \"breed\": \"{self.breed}\"\}"
    }
}
```

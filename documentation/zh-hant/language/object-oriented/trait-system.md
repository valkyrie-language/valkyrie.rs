# Trait 系統

Trait 是 Valkyrie 中定義行為契約的機制，類似於其他語言中的介面（Interface）。通過 Trait，可以實現多型、程式碼重用和關注點分離。

## 定義 Trait

### 基本 Trait

```valkyrie
# 定義一個 Trait
trait Drawable {
    micro draw(self, canvas: Canvas)
    micro get_bounds(self) -> Rectangle
}

# 定義帶預設實現的 Trait
trait Comparable {
    micro compare(self, other: Self) -> i32
    
    # 預設實現
    micro less_than(self, other: Self) -> bool {
        self.compare(other) < 0
    }
    
    micro greater_than(self, other: Self) -> bool {
        self.compare(other) > 0
    }
}
```

### 關聯型別

```valkyrie
trait Container {
    type Item
    type Iterator: Iterator⟨Self::Item⟩
    
    micro add(self, item: Self::Item)
    micro get_iterator(self) -> Self::Iterator
}
```

## 實現 Trait

### 為類別實現 Trait

```valkyrie
class Circle {
    radius: f64
    center: Point
}

impl Drawable for Circle {
    micro draw(self, canvas) {
        canvas.draw_circle(self.center, self.radius)
    }
    
    micro get_bounds(self) -> Rectangle {
        Rectangle {
            x: self.center.x - self.radius,
            y: self.center.y - self.radius,
            width: self.radius * 2,
            height: self.radius * 2
        }
    }
}
```

### 為泛型型別實現 Trait

```valkyrie
impl⟨T: Comparable⟩ Comparable for Array⟨T⟩ {
    micro compare(self, other: Array⟨T⟩) -> i32 {
        let min_len = min(self.length, other.length)
        loop i in 0..min_len {
            let cmp = self[i].compare(other[i])
            if cmp != 0 {
                return cmp
            }
        }
        self.length.compare(other.length)
    }
}
```

## Trait 繼承

### Trait 組合

```valkyrie
# Trait 可以繼承其他 Trait
trait Shape : Drawable {
    micro area(self) -> f64
    micro perimeter(self) -> f64
}

trait ColoredShape : Shape {
    color: Color
    
    micro set_color(mut self, color: Color)
    micro get_color(self) -> Color
}
```

### 多 Trait 繼承

```valkyrie
trait Serializable {
    micro to_json(self) -> string
}

trait Cloneable {
    micro clone(self) -> Self
}

# 組合多個 Trait
trait Persistent : Serializable, Cloneable {
    micro save(self) -> Result⟨Unit⟩
    micro load(id: string) -> Result⟨Self⟩
}
```

## Trait 物件

### 動態分發

```valkyrie
# 使用 Trait 作為型別
let shapes: [Drawable] = [
    Circle { radius: 5.0, center: Point { x: 0, y: 0 } },
    Rectangle { width: 10, height: 20, x: 0, y: 0 }
]

loop shape in shapes {
    shape.draw(canvas)  # 動態分發
}
```

## Trait 約束

### 泛型約束

```valkyrie
# 泛型函數約束
micro sort⟨T: Comparable + Cloneable⟩(items: [T]) -> [T] {
    let mut result = items.map { $.clone() }
    result.sort_by { $1.compare($2) }
    result
}

# 多約束
micro process⟨T⟩(item: T) where T: Serializable + Debug {
    print(item.debug_string())
    let json = item.to_json()
    save_to_file(json)
}
```

### 關聯型別約束

```valkyrie
trait Graph {
    type Node
    type Edge
    
    micro nodes(self) -> [Self::Node]
    micro edges(self) -> [Self::Edge]
    micro neighbors(self, node: Self::Node) -> [Self::Node]
}

micro bfs⟨G: Graph⟩(graph: G, start: G::Node) -> [G::Node] {
    # 廣度優先搜尋實現
}
```

## 標準 Trait

### 常用標準 Trait

```valkyrie
# Debug - 除錯輸出
trait Debug {
    micro debug_string(self) -> string
}

# Eq - 相等比較
trait Eq {
    micro equals(self, other: Self) -> bool
}

# Hash - 雜湊計算
trait Hash {
    micro hash(self) -> u64
}

# Clone - 複製
trait Clone {
    micro clone(self) -> Self
}

# Default - 預設值
trait Default {
    micro default() -> Self
}
```

### 衍生宏

```valkyrie
# 自動衍生 Trait 實現
#[derive(Debug, Clone, Eq, Hash)]
class Point {
    x: i32
    y: i32
}
```

## 高級特性

### 標記 Trait

```valkyrie
# 沒有方法的標記 Trait
trait Send {}
trait Sync {}

# 用於型別約束
micro send_to_thread⟨T: Send⟩(value: T) {
    # 安全地發送到其他執行緒
}
```

### 條件實現

```valkyrie
# 條件性實現 Trait
impl⟨T: Send⟩ Send for Array⟨T⟩ {}

# 或
impl⟨T⟩ Clone for Container⟨T⟩ where T: Clone {
    micro clone(self) -> Self {
        Container { items: self.items.map { $.clone() } }
    }
}
```

## 最佳實踐

### 1. 小而專注的 Trait

```valkyrie
# 好的實踐
trait Read {
    micro read(self, buffer: [u8]) -> i32
}

trait Write {
    micro write(self, data: [u8]) -> i32
}

# 避免
trait IO {
    micro read(self, buffer: [u8]) -> i32
    micro write(self, data: [u8]) -> i32
    micro open(self, path: string)
    micro close(self)
    # ... 太多方法
}
```

### 2. 提供預設實現

```valkyrie
trait Iterator⟨Item⟩ {
    micro next(self) -> Item?
    
    # 提供有用的預設方法
    micro map⟨B⟩(self, f: (Item) -> B) -> Iterator⟨B⟩ {
        MapIterator { source: self, transform: f }
    }
    
    micro filter(self, predicate: (Item) -> bool) -> Iterator⟨Item⟩ {
        FilterIterator { source: self, predicate }
    }
    
    micro collect(self) -> [Item] {
        let mut result = []
        while let Some(item) = self.next() {
            result.push(item)
        }
        result
    }
}
```

### 3. 使用關聯型別

```valkyrie
# 好的實踐：使用關聯型別
trait Container {
    type Item
    micro get(self, index: i32) -> Self::Item
}

# 避免：過度使用泛型參數
trait Container⟨Item⟩ {
    micro get(self, index: i32) -> Item
}
```

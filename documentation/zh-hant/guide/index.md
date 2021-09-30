# Valkyrie 語言快速入門

歡迎使用 Valkyrie 語言！Valkyrie 是一個現代化的函數式編程語言，提供強大的類型系統、靈活的模組系統和豐富的語言特性。

## 什麼是 Valkyrie？

Valkyrie 是一個多範式編程語言，它提供：

- 🎯 **強大的類型系統**：支援泛型、高階類型、類型推導等高級特性
- 🚀 **現代語法**：簡潔直觀的語法，支援模式匹配、閉包等現代特性
- 🔒 **內存安全**：垃圾回收器自動管理內存，避免內存洩漏
- ⚡ **高性能**：零成本抽象，編譯時優化
- 🔧 **靈活的模組系統**：基於命名空間的模組組織方式

## 基本語法

### 變量定義

```valkyrie
# 不可變變量
let name = "Alice"
let age = 30
let is_active = true

# 可變變量
let mut counter = 0
let mut items = []

# 顯式類型註解
let score: i32 = 95
let price: f64 = 29.99
let message: String = "Hello"
```

### 函數定義

```valkyrie
# 基本函數定義
micro greet() {
    print("Hello, World!")
}

# 帶參數和返回值的函數
micro add(a: i32, b: i32) -> i32 {
    a + b
}

# 多參數函數
micro calculate(x: f64, y: f64, operation: String) -> f64 {
    if operation == "add" {
        x + y
    } else if operation == "multiply" {
        x * y
    } else {
        0.0
    }
}
```

### 基本數據類型

```valkyrie
# 整數類型
let a: i32 = 42
let b: u64 = 100

# 浮點類型
let x: f32 = 3.14
let y: f64 = 2.718281828

# 布爾類型
let flag: bool = true

# 字符和字符串
let ch: char = 'A'
let text: String = "Hello, World!"

# 數組類型
let numbers: [i32; 5] = [1, 2, 3, 4, 5]
let dynamic: [String] = ["a", "b", "c"]

# 元組類型
let point: (f64, f64) = (3.0, 4.0)
let mixed: (String, i32, bool) = ("test", 42, true)
```

## 控制流

### 條件語句

```valkyrie
# if 語句
if x > 0 {
    print("正數")
} else {
    print("非正數")
}

# if 表達式
let result = if x > 0 { "positive" } else { "non-positive" }

# 多重條件
if score >= 90 {
    grade = "A"
} else if score >= 80 {
    grade = "B"
} else {
    grade = "F"
}
```

### 循環語句

```valkyrie
# while 循環
while counter < 10 {
    print(counter)
    counter = counter + 1
}

# for 循環
loop i in 0..10 {
    print(i)
}

# 遍歷數組
loop item in items {
    print(item)
}

# 無限循環
loop {
    if should_break {
        break
    }
}
```

## 模式匹配

```valkyrie
# 基本模式匹配
match value {
    case 1: "one"
    case 2: "two"
    case 3: "three"
    case _: "other"
}

# 範圍匹配
match score {
    case 90..=100: "A"
    case 80..=89: "B"
    case 70..=79: "C"
    case _: "F"
}

# 元組解構
match point {
    case (0, 0): "Origin"
    case (x, 0): "On X-axis at {x}"
    case (0, y): "On Y-axis at {y}"
    case (x, y): "Point at ({x}, {y})"
}
```

## 類型定義

### 記錄類型

```valkyrie
# 基本記錄類型
type Point = {
    x: f64,
    y: f64,
}

# 泛型記錄類型
type Container<T> = {
    value: T,
    metadata: String,
}
```

### 聯合類型

```valkyrie
# 基本聯合類型
union Result<T, E> {
    Fine { value: T },
    Fail { error: E }
}

# 使用聯合類型
let result: Result<i32, String> = Fine { value: 42 }
match result {
    case Fine { value }: print("Success: {value}")
    case Fail { error }: print("Error: {error}")
}
```

### 類定義

```valkyrie
# 基本類定義
class Person {
    name: String
    age: i32
    
    new(name: String, age: i32) -> Self {
        Self { name, age }
    }
    
    greet(self) {
        print("Hello, I'm {self.name}")
    }
    
    get_info(self) -> String {
        "{self.name} is {self.age} years old"
    }
}

# 使用類
let person = Person::new("Alice", 30)
person.greet()
let info = person.get_info()
```

## 模組系統

### 命名空間聲明

```valkyrie
# 聲明命名空間
namespace math.geometry {
    class Point {
        x: f64
        y: f64
    }
    
    micro distance(p1: Point, p2: Point) -> f64 {
        let dx = p1.x - p2.x
        let dy = p1.y - p2.y
        (dx * dx + dy * dy).sqrt()
    }
}
```

### 導入系統

```valkyrie
# 導入整個命名空間
using math.geometry.*

# 選擇性導入
using math.geometry.{Point, distance}

# 重命名導入
using math.geometry.Point as GeomPoint

# 使用導入的內容
micro main() {
    let p1 = Point { x: 0.0, y: 0.0 }
    let p2 = Point { x: 3.0, y: 4.0 }
    let dist = distance(p1, p2)
    print("Distance: {dist}")
}
```

## 字面量

### 數值字面量

```valkyrie
# 整數字面量
42
0xFF        # 十六進制
0b1010      # 二進制
0o755       # 八進制
1_000_000   # 帶分隔符

# 浮點數字面量
3.14
1.23e4      # 科學計數法
3.141_592_653  # 帶分隔符
```

### 字符串字面量

```valkyrie
# 普通字符串
"Hello, World!"
'單引號字符串'

# 轉義序列
"換行符：\n"
"製表符：\t"
"Unicode：\u{1F600}"  # 😀 表情符號

# 原始字符串
r"C:\Users\Name\Documents"
r"""多行原始字符串
不處理轉義序列，也不處理插值"""

# 字符串插值
let name = "Alice"
let age = 30
let message = "Hello, {name}! You are {age} years old."

# 字面量花括號
"模板：\{name\}"
```

### 其他字面量

```valkyrie
# 數組字面量
[1, 2, 3, 4, 5]
["a", "b", "c"]

# 對象字面量
{
    name: "Alice",
    age: 30,
    active: true
}

# 元組字面量
(1, 2, 3)
("name", 30, true)

# 範圍字面量
0..=100     # 包含範圍
1..<10      # 排除範圍

# 正則表達式字面量
re"hello"
re"\d+"
re"[a-zA-Z]+"
```

## 閉包和高階函數

```valkyrie
# 基本閉包語法
let square = { $x * $x }
let add = { $x + $y }

# 顯式參數類型
let multiply = { $x: i32, $y: i32 -> $x * $y }

# 多語句閉包
let complex = {
    let result = $x * 2
    result + 1
}

# 高階函數使用
let numbers = [1, 2, 3, 4, 5]
let squares = numbers.map { $x * $x }
let evens = numbers.filter { $x % 2 == 0 }
let sum = numbers.reduce { $acc + $x }
```

## 類型函數 (mezzo)

```valkyrie
# 類型函數定義
mezzo IsEven(z: Type) -> bool {
    # 檢查類型 z 是否表示偶數
    match z {
        i32 if z % 2 == 0 => true,
        _ => false
    }
}

# 類型映射
mezzo MapType<T>(input: T) -> T {
    # 對輸入類型進行映射變換
    match input {
        i32 => i64,
        f32 => f64,
        _ => input
    }
}
```

## 下一步

現在你已經了解了 Valkyrie 語言的基本語法和特性，可以：

1. **深入學習**：查看 [語言特性詳細指南](./features.md)
2. **類型系統**：了解 [類型系統](../language/type-system/index.md) 的高級特性
3. **模式匹配**：掌握 [模式匹配](../language/pattern-match.md) 的強大功能
4. **模組系統**：學習 [模組系統](../language/modules.md) 的組織方式
5. **元編程**：探索 [元編程](../language/meta-programming/index.md) 的高級用法

開始你的 Valkyrie 編程之旅吧！

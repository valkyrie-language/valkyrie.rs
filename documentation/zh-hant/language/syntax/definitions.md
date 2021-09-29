# 定義

Valkyrie 提供了多種定義語法，用於宣告命名空間、變數、函式、型別和其他程式實體。

## 命名空間定義

Valkyrie 使用 `namespace` 或 `namespace!` 關鍵字宣告模組所屬的命名空間。

```valkyrie
# 顯式宣告命名空間
namespace! package.collection.option;

# 或者
namespace package.text;
```

## 變數定義

```valkyrie
# 變數宣告
let name = "Alice"
let age = 30
```

# 修改變數（Valkyrie 預設不可變，需要 mut 關鍵字）

```valkyrie
let mut counter = 0
counter = 1
```

如果不使用 `mut` 關鍵字，變數預設是不可變的：

```valkyrie
let x = 10
x = 20 // 編譯錯誤：無法修改不可變變數
```

```valkyrie
# 顯式型別標註
let score: i32 = 95

# 延遲初始化
let result: i32
if condition {
    result = 42
} else {
    result = 0
}
```

## 函式定義 (micro)

Valkyrie 使用 `micro` 關鍵字定義函式。

### 基本函式定義

```valkyrie
# 無參數函式
micro greet() {
    print("Hello, World!")
}

# 帶參數函式
micro add(a: i32, b: i32) -> i32 {
    a + b
}
```

## 型別定義

Valkyrie 區分結構化資料（`class`）和代數資料型別（顯式 `tag` 的 `unite`）。

### 類別定義 (class)

```valkyrie
class Point {
    x: f64
    y: f64
}
```

### 聯合型別定義 (`unite`)

`unite` 用於定義類似 Rust `enum` 的封閉名義變體族。標準寫法是 `[tag(XXXKind)] unite XXX { }`，判別 tag 需要顯式宣告，不再自動生成。

```valkyrie
[tag(OptionKind)]
unite Option⟨V⟩ {
    Some {
        value: V
    }
    None
}
```

## 實作定義 (imply)

Valkyrie 使用 `imply` 關鍵字為型別實作方法或 Trait。

```valkyrie
imply Option⟨V⟩⸬Some {
    constructor(value: V) {
        this.value = value
    }
}

imply Unicode {
    # 實作方法
}
```

# 具名參數呼叫
let user = create_user(name: "Alice", active: false)
let result = sum(1, 2, 3, 4, 5)

# 引用參數
micro modify_array(arr: &mut [i32]) {
    loop i in 0..<arr.length {
        arr[i] *= 2
    }
}

# 泛型參數
micro identity⟨T⟩(value: T) -> T {
    value
}

micro map⟨T, U⟩(items: [T], transform: micro(T) -> U) -> [U] {
    let mut result = []
    loop item in items {
        result.push(transform(item))
    }
    result
}
```

### 高階函式

```valkyrie
# 函式作為參數
micro apply_operation(x: i32, y: i32, op: micro(i32, i32) -> i32) -> i32 {
    op(x, y)
}

# 返回函式
micro make_adder(n: i32) -> micro(i32) -> i32 {
    micro(x: i32) -> i32 {
        x + n
    }
}

# 閉包
let add_five = make_adder(5)
let result = add_five(10)  # 15

# 匿名函式
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(micro(x) { x * 2 })
let filtered = numbers.filter(micro(x) { x % 2 == 0 })
```

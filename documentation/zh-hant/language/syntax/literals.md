# 字面量

Valkyrie 支援多種字面量型別，用於表示程式中的常數值。

## 數值字面量

### 整數字面量

```valkyrie
# 十進位整數
42
-17
0

# 十六進位整數
0xFF
0x1A2B

# 二進位整數
0b1010
0b11110000

# 八進位整數
0o755
0o644

# 帶下劃線分隔符的整數（提高可讀性）
1_000_000
0xFF_FF_FF
0b1010_1010
```

### 浮點數字面量

```valkyrie
# 標準浮點數
3.14
-2.5
0.0

# 科學記數法
1.23e4
-5.67E-3
2.0e+10

# 帶下劃線分隔符
3.141_592_653
1.234_567e-8
```

### 型別字尾

數值可以帶型別字尾：

```valkyrie
42i32
100u64
3.14f32
```

## 字串字面量

Valkyrie 的字串語法（S-Grammar）支援內插、原始字串和多行模式。

```valkyrie
let simple = "Hello"
let raw = r"C:\path"
let interpolated = "Hello, {name}"
let literal_braces = "Hello, \{name\}"
```

有關字串語法的詳細資訊，請參閱 [S-Grammar](./s-grammar.md)。

## 字元字面量

```valkyrie
# 單個字元
'a'
'中'
'1'

# 轉義字元
'\n'
'\t'
'\\'
'\''
'\$'  # 美元符號轉義

# Unicode 字元
'\u{1F600}'  # 😀
'\u{4E2D}'   # 中
```

## 布林字面量

```valkyrie
# 布林值
true
false
```

## 空值字面量

```valkyrie
# 空值
null
```

## 陣列字面量

```valkyrie
# 空陣列
[]

# 整數陣列
[1, 2, 3, 4, 5]

# 字串陣列
["apple", "banana", "cherry"]

# 混合型別陣列
[1, "hello", true, null]

# 嵌套陣列
[[1, 2], [3, 4], [5, 6]]

# 多行陣列
[
    "first",
    "second",
    "third"
]
```

## 物件字面量

```valkyrie
# 空物件
{}

# 簡單物件
{
    name: "Alice",
    age: 30,
    active: true
}

# 嵌套物件
{
    user: {
        name: "Bob",
        profile: {
            email: "bob@example.com",
            phone: "123-456-7890"
        }
    },
    settings: {
        theme: "dark",
        notifications: true
    }
}

# 帶引號的鍵名
{
    "first-name": "Charlie",
    "last-name": "Brown",
    "age": 25
}
```

## 元組字面量

```valkyrie
# 空元組
()

# 單元素元組（需要逗號）
(42,)

# 多元素元組
(1, 2, 3)
("name", 30, true)

# 嵌套元組
((1, 2), (3, 4))

# 多行元組
(
    "first",
    "second",
    "third"
)
```

## 範圍字面量

```valkyrie
# 包含範圍（推薦語法）
0..=100    # 0 到 100（包含 100）
1..=10     # 1 到 10（包含 10）

# 排除範圍
1..<10     # 1 到 9（不包含 10）

# 注意：以下語法已被禁止
# 1..10      # 錯誤：不明確是否包含結束值
# 1..        # 錯誤：開放範圍語法不支援
# ..10       # 錯誤：開放範圍語法不支援
```

## 閉包字面量

```valkyrie
# 基本閉包語法
micro(x) { x * x }             # 單參數匿名函式
micro(x, y) { x + y }          # 多參數匿名函式
micro() { print("Hello") }     # 無參數匿名函式

# 顯式參數型別
micro(x: i32) { x * 2 }
micro(x: string, y: i32) { "{x}: {y}" }

# 多語句匿名函式
micro(x) {
    let result = x * 2
    result + 1
}

# 尾隨閉包語法
numbers.map { $ * $ }        # 尾隨閉包
numbers.filter { $ > 10 }     # 條件過濾

# 傳統函式式閉包
numbers.map(micro(x) { x * x })  # 顯式函式語法
numbers.filter(micro(x) { x > 10 })
```

## 正規表示式字面量

```valkyrie
# 基本正規表示式
re"hello"
re"\d+"
re"[a-zA-Z]+"

# 推薦使用標準正規表示式
re"hello"         # 標準正規表示式
re"\s+"          # 標準正規表示式
re"test"         # 標準正規表示式
```

## 註釋

Valkyrie 使用 `#` 作為行註釋，`<# #>` 作為塊註釋：

```valkyrie
# 這是行註釋
let x = 42  # 行尾註釋

<# 
這是塊註釋
可以跨越多行
#>

let y = <# 內聯塊註釋 #> 100
```

## 字面量型別推斷

Valkyrie 會根據上下文自動推斷字面量的型別：

```valkyrie
let integer = 42        # 推斷為 i32
let float = 3.14        # 推斷為 f64
let string = "hello"    # 推斷為 string
let boolean = true      # 推斷為 bool
let array = [1, 2, 3]   # 推斷為 [i32; 3]
```

也可以顯式指定型別：

```valkyrie
let big_int: i64 = 42
let precise_float: f32 = 3.14
let char_array: [char; 3] = ['a', 'b', 'c']
```

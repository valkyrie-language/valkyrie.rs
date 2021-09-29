# 控制流

Valkyrie 提供了豐富的控制流語句，用於控制程式的執行流程。

## 條件語句

### if 語句

```valkyrie
# 基本 if 語句
if condition {
    # 執行程式碼
}

# if-else 語句
if x > 0 {
    print("正數")
}
else {
    print("非正數")
}

# if-else if-else 鏈
if score >= 90 {
    grade = "A"
}
else if score >= 80 {
    grade = "B"
}
else if score >= 70 {
    grade = "C"
}
else {
    grade = "F"
}

# if 表達式（回傳值）
let result = if x > 0 { "positive" } else { "non-positive" }

# 多行 if 表達式
let message = if user.is_admin {
    "管理員使用者"
}
else if user.is_premium {
    "進階使用者"
}
else {
    "普通使用者"
}
```

### 條件表達式

```valkyrie
# 三元運算子風格
let max = if a > b { a } else { b }

# 鏈式條件
let status = if online { "在線" } else if busy { "忙碌" } else { "離線" }
```

## 迴圈語句

### loop 語句（無限迴圈）

```valkyrie
# 基本無限迴圈
loop {
    # 無限執行的程式碼
    if should_break {
        break
    }
}

# 帶標籤的迴圈
'outer: loop {
    'inner: loop {
        if condition1 {
            break 'outer  # 跳出外層迴圈
        }
        if condition2 {
            break 'inner  # 跳出內層迴圈
        }
    }
}

# 迴圈回傳值
let result = loop {
    let input = get_input()
    if input.is_valid() {
        break input.value()  # 回傳值
    }
}
```

### while 語句

```valkyrie
# 基本 while 迴圈
while condition {
    # 當條件為真時執行
    update_condition()
}

# 複雜條件
while x > 0 && y < 100 {
    x -= 1
    y += 2
}

# while let 模式比對
while let Some { value: item } = iterator.next() {
    process(item)
}

# 帶標籤的 while 迴圈
'search: while has_more_data() {
    let data = get_next_data()
    if data.is_target() {
        break 'search
    }
}
```

### until 語句

```valkyrie
# until 迴圈（當條件為假時執行）
until condition {
    # 當條件為假時執行
    update_condition()
}

# 等價於 while !condition
until x <= 0 {
    x -= 1
}

# until let 模式比對
until let None = optional_value {
    process(optional_value.unwrap())
    optional_value = get_next_optional()
}
```

### for 語句

```valkyrie
# 範圍迴圈
loop i in 0..<10 {
    print(i)  # 輸出 0 到 9
}

# 包含結束值的範圍
loop i in 0..=10 {
    print(i)  # 輸出 0 到 10
}

# 陣列疊代
let numbers = [1, 2, 3, 4, 5]
loop num in numbers {
    print(num)
}

# 帶索引的疊代
for (index, value) in numbers.enumerate() {
    print(f"索引 {index}: 值 {value}")
}

# 字串疊代
loop char in "hello".chars() {
    print(char)
}

# 物件屬性疊代
for (key, value) in object.entries() {
    print(f"{key}: {value}")
}

# 帶條件的 for 迴圈
loop item in collection where item.is_valid() {
    process(item)
}

# 嵌套迴圈
loop i in 0..<3 {
    loop j in 0..<3 {
        print(f"({i}, {j})")
    }
}
```

## 模式比對

### match 語句

Valkyrie 的 `match` 語句提供了強大的結構化模式比對能力。

```valkyrie
match value {
    case 1: print("一")
    case 2: print("二")
    case _: print("其他")
}
```

### 模式解構

```valkyrie
unite Option⟨T⟩ {
    Some { value: T }
    None
}

let result = Some { value: 42 }

match result {
    case Some { value }: print("值: ${value}")
    case None: print("空值")
}
```

### 解構賦值

```valkyrie
# 陣列解構
let [first, second, ..rest] = array  # 解構陣列到kvs
let [a, _, c] = [1, 2, 3]  # 忽略第二個元素

# 元組解構
let (x, y, z) = (1, 2, 3)
let (name, ..) = ("Alice", 25, "Engineer")  # 只取第一個

# 物件解構
let { name, age } = person
let { x: new_x, y: new_y } = point  # 重命名
let { name, ..rest } = user  # 解構dict到kvs
```

## 異常處理

### catch 語句（異常處理器）

```valkyrie
# 基本異常處理
catch {
    risky_operation()
} handle error {
    print(f"發生錯誤: {error}")
}

# 多種異常型別處理
catch {
    complex_operation()
} handle NetworkError(msg) {
    print(f"網路錯誤: {msg}")
} handle ValidationError(field) {
    print(f"驗證錯誤: {field}")
} handle error {
    print(f"未知錯誤: {error}")
}

# 帶資源管理的異常處理
using resource = acquire_resource() {
    catch {
        file_operation()
    } handle IOError(msg) {
        print("IO錯誤: ${msg}")
    }
}  # resource會自動清理

# 異常處理表達式
let result = catch {
    parse_number(input)
} handle ParseError(_) {
    0  # 預設值
}

# 嵌套異常處理
catch {
    catch {
        inner_operation()
    } handle InnerError(e) {
        handle_inner_error(e)
    }
    outer_operation()
} handle OuterError(e) {
    handle_outer_error(e)
}
```

### 異常傳播

```valkyrie
# 使用 ? 運算子傳播異常
micro process_file(path: string) -> Result⟨string, IOError⟩ {
    let content = read_file(path)?  # 如果失敗則提前回傳錯誤
    let processed = transform(content)?
    Fine { value: processed }
}

# 手動拋出異常
micro validate_age(age: i32) -> Result⟨unit, ValidationError⟩ {
    if age < 0 {
        throw ValidationError("年齡不能為負數")
    }
    if age > 150 {
        throw ValidationError("年齡不能超過150")
    }
    Fine { value: () }
}
```

## 控制流關鍵字

### break 和 continue

```valkyrie
# break 跳出迴圈
loop i in 0..<10 {
    if i == 5 {
        break  # 跳出迴圈
    }
    print(i)
}

# continue 跳過當前疊代
loop i in 0..<10 {
    if i % 2 == 0 {
        continue  # 跳過偶數
    }
    print(i)  # 只列印奇數
}

# 帶標籤的 break 和 continue
'outer: loop i in 0..<3 {
'inner: loop j in 0..<3 {
        if i == 1 && j == 1 {
            break 'outer  # 跳出外層迴圈
        }
        if j == 2 {
            continue 'outer  # 繼續外層迴圈的下一次疊代
        }
        print(f"({i}, {j})")
    }
}

# break 回傳值
let found = loop {
    let item = get_next_item()
    if item.is_target() {
        break Some { value: item }  # 回傳找到的項
    }
    if no_more_items() {
        break None  # 回傳空值
    }
}
```

### return 語句

```valkyrie
# 函式回傳
micro calculate(x: i32, y: i32) -> i32 {
    if x < 0 || y < 0 {
        return -1  # 提前回傳
    }
    x + y  # 隱式回傳
}

# 空回傳
micro log_message(msg: string) {
    if msg.is_empty() {
        return  # 提前回傳，無回傳值
    }
    print(msg)
}
```

# 匿名函數與閉包

## 匿名函數

匿名函數是沒有名稱的函數，可以直接在表達式中定義和使用。

### 基本語法

```valkyrie
# 基本匿名函數
let add = micro(x, y) { x + y }

# 單參數匿名函數
let square = micro(x) { x * x }

# 無參數匿名函數
let get_random = micro() { random() }
```

## 閉包

閉包是一種特殊的匿名函數，可以捕獲其定義環境中的變數。

### 閉包語法
閉包使用花括號 `{}` 定義，參數使用 `$` 或 `$n`（如 `$1`, `$2`）：

```valkyrie
# 單參數閉包
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map { $ * 2 }

# 多參數閉包
let pairs = [(1, 2), (3, 4), (5, 6)]
let sums = pairs.map { $1 + $2 }

# 無參數閉包
let lazy_value = { 42 }
```

### 參數自動註冊

閉包中的參數會自動註冊：

```valkyrie
# $ 等價於 $1，是第一個參數，$2 是第二個參數
let operation = { $ + $2 * 2 }

# $x 等價於 $.x 等價於 $1.x，是第一個參數的屬性
let user_name = users.map { $name }
```

## 尾隨閉包

當函數的最後一個參數是閉包時，可以使用尾隨閉包語法，省略括號：

```valkyrie
# 傳統呼叫方式
list.map(micro(x) { x * 2 })

# 尾隨閉包語法（完全等價）
list.map { $ * 2 }

# 多個參數時，只有最後一個可以使用尾隨語法
list.fold(0, micro(acc, item) { acc + item })
# 等價於
list.fold(0) { $1 + $2 }
```

### 複雜範例

```valkyrie
# 鏈式呼叫與尾隨閉包
let result = numbers
    .filter { $ > 0 }
    .map { $ * $ }
    .fold(0) { $1 + $2 }

# 嵌套閉包
let matrix = [[1, 2], [3, 4], [5, 6]]
let flattened = matrix
    .map { $map { $ * 2 } }
    .flatten()
```

## 閉包捕獲

閉包可以捕獲其定義環境中的變數：

```valkyrie
let multiplier = 10
let numbers = [1, 2, 3, 4, 5]

# 閉包捕獲外部變數 multiplier
let scaled = numbers.map { $ * multiplier }

# 捕獲外部變數
let counter = 0
let increment_counter = {
    counter += 1
    counter
}
```

## 高階函數範例

```valkyrie
# 自定義高階函數
micro apply_twice⟨T⟩(value: T, f: micro(T) -> T) -> T {
    f(f(value))
}

# 使用尾隨閉包
let result = apply_twice(5) { $ * 2 }  # 結果: 20

# 函數組合
micro compose⟨A, B, C⟩(f: micro(B) -> C, g: micro(A) -> B) -> micro(A) -> C {
    { f(g($)) }
}

let add_one = micro(x) { x + 1 }
let double = micro(x) { x * 2 }
let add_one_then_double = compose(double, add_one)
```

## 最佳實踐

1. **簡潔性**: 對於簡單操作，優先使用閉包而不是命名函數
2. **可讀性**: 複雜邏輯應該使用命名函數以提高可讀性
3. **尾隨閉包**: 當閉包是最後一個參數時，使用尾隨語法提高程式碼美觀度
4. **參數存取**: 使用 `$1`, `$2` 等存取參數，或者使用 `$x` 存取第一個參數的屬性

```valkyrie
# 好的實踐
users.filter { $is_active }
     .map { $name }
     .sort_by { $name.length() }

# 避免過度嵌套
let process_data = micro(data) {
    data.filter { $is_valid }
        .transform { $normalize() }
        .group_by { $category }
}
```

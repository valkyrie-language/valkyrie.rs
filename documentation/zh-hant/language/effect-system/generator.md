# 生成器

Valkyrie 的生成器是一種特殊的函數，可以通過 `yield` 關鍵字產生一系列值。生成器提供了一種惰性計算的方式，只在需要時計算下一個值，非常適合處理大量資料或無限序列。

## 基本生成器語法

### 簡單生成器

```valkyrie
# 基本生成器函數
micro count_up(max: i32) {
    let i = 0
    while i < max {
        yield i
        i += 1
    }
}

# 使用生成器
let counter = count_up(5)
loop value in counter {
    print(value)  # 輸出: 0, 1, 2, 3, 4
}
```

### 無限生成器

```valkyrie
# 斐波那契數列生成器
micro fibonacci() {
    let a = 0
    let b = 1
    loop {
        yield a
        let temp = a + b
        a = b
        b = temp
    }
}

# 獲取前10個斐波那契數
let fib = fibonacci()
loop i in 0..<10 {
    print(fib.next())  # 0, 1, 1, 2, 3, 5, 8, 13, 21, 34
}
```

### 帶返回值的生成器

```valkyrie
# 生成器可以有最終返回值
micro process_items(items: [utf8]) -> i32 {
    let count = 0
    loop item in items {
        if item.is_valid() {
            yield item.process()
            count += 1
        }
    }
    count  # 最終返回處理的項目數量
}

# 使用
let processor = process_items(["item1", "item2", "item3"])
loop result in processor {
    print("Processed: {result}")
}
let total_count = processor.return_value()  # 獲取最終返回值
```

## 生成器狀態管理

### 實現原理：代數效應

你可能會好奇，為什麼 `sequence` 作為一個普通函數，其尾隨閉包中的 `yield` 可以被「攔截」？這背後的核心機制是 Valkyrie 的 **代數效應 (Algebraic Effects)**。

1.  **Yield 是一個效應 (Effect)**：在 Valkyrie 中，`yield` 並不是一個綁定在函數定義上的硬編碼關鍵字。它本質上是一個「效應呼叫」，類似於一種可以恢復的異常。
2.  **處理器棧 (Handler Stack)**：Valkyrie 虛擬機維護著一個處理器棧。當執行到 `yield` 時，虛擬機會暫停當前執行流，並沿著呼叫棧向上尋找最近的匹配處理器（Handler）。
3.  **穿透能力**：由於效應是基於虛擬機棧管理的，它具有「穿透」普通函數和閉包的能力。即使 `sequence` 是一個普通函數，閉包是一個普通閉包，其中的 `yield` 也會一直向上冒泡，直到被 `sequence` 內部設置的 `handle` 塊捕獲。
4.  **恢復執行 (Resume)**：`sequence` 內部的處理器在捕獲到 `yield` 的值後，會同時獲得一個「延續 (Continuation)」。這使得 `sequence` 可以將值返回給迭代器，並在下次呼叫 `next()` 時，通過這個延續精確地恢復閉包的執行。

這種設計使得生成器不再局限於整個函數的轉化，而是可以作為一種局部的、可嵌套的控制流原語存在。

### 序列環境

除了將整個 `micro` 函數定義為生成器外，Valkyrie 還支援在普通函數中使用 `sequence` 環境來局部定義生成器。這允許你在不改變整個函數性質的情況下，產生一個惰性序列。

#### 局部生成器

使用 `sequence` 塊可以創建一個匿名的生成器物件：

```valkyrie
micro process_data(data: [i32]) {
    # 在普通函數中定義局部生成器
    let gen = sequence {
        loop item in data {
            if item > 0 {
                yield item * 2
            }
        }
    }
    
    # 使用局部生成器
    loop val in gen {
        print(val)
    }
}
```

#### 顯式型別宣告

你也可以為 `sequence` 環境顯式指定產生的元素型別（通過泛型參數）：

```valkyrie
let gen = sequence⟨utf8⟩ {
    yield "Hello"
    yield "World"
}
```

注意，`sequence` 並不是關鍵字，而是一個利用代數效應攔截 `yield` 的高階函數，因此它使用標準的泛型呼叫語法 `⟨T⟩`。

#### 表達式用法

`sequence` 環境是一個表達式，可以直接作為參數傳遞或返回：

```valkyrie
micro get_numbers() {
    return sequence {
        yield 1
        yield 2
        yield 3
    }
}
```

### 生成器生命週期

```valkyrie
# 生成器狀態列舉
unite GeneratorState {
    Created,     # 已建立但未開始
    Running,     # 正在執行
    Suspended,   # 已暫停（yield）
    Completed,   # 已完成
    Fail { error: any } # 發生錯誤
}

# 檢查生成器狀態
micro example_generator() {
    print("開始執行")
    yield "第一個值"
    print("繼續執行")
    yield "第二個值"
    print("執行完成")
}

let gen = example_generator()
print(gen.state())  # Created

let first = gen.next()
print(gen.state())  # Suspended
print(first)        # "第一個值"

let second = gen.next()
print(gen.state())  # Suspended
print(second)       # "第二個值"

gen.next()          # 完成執行
print(gen.state())  # Completed
```

### 生成器控制

```valkyrie
# 手動控制生成器執行
micro controlled_generator() {
    let mut value = 0
    loop {
        let input = yield value
        if input != null {
            value = input  # 接收外部輸入
        } else {
            value += 1     # 預設遞增
        }
    }
}

let gen = controlled_generator()
print(gen.next())        # 0
print(gen.send(10))      # 10 (發送值給生成器)
print(gen.next())        # 11
print(gen.send(100))     # 100
```

## 生成器管道

### 管道處理

```valkyrie
# 生成器管道處理
micro pipeline_stage1(input: Iterator⟨i32⟩) {
    loop value in input {
        yield value * 2  # 第一階段：乘以2
    }
}

micro pipeline_stage2(input: Iterator⟨i32⟩) {
    loop value in input {
        if value % 4 == 0 {
            yield value  # 第二階段：過濾4的倍數
        }
    }
}

micro pipeline_stage3(input: Iterator⟨i32⟩) {
    loop value in input {
        yield "Result: {value}"  # 第三階段：格式化
    }
}

# 構建管道
let numbers = [1, 2, 3, 4, 5, 6, 7, 8]
let stage1 = pipeline_stage1(numbers.iter())
let stage2 = pipeline_stage2(stage1)
let stage3 = pipeline_stage3(stage2)

loop result in stage3 {
    print(result)  # "Result: 4", "Result: 8", "Result: 12", "Result: 16"
}
```

### 組合生成器

```valkyrie
# 組合多個生成器
micro combine_generators(gen1: Generator⟨i32⟩, gen2: Generator⟨i32⟩) {
    # 交替產生兩個生成器的值
    loop {
        let val1 = gen1.next()
        let val2 = gen2.next()
        
        if val1.is_some() {
            yield val1.unwrap()
        }
        if val2.is_some() {
            yield val2.unwrap()
        }
        
        if val1.is_none() && val2.is_none() {
            break
        }
    }
}

let gen1 = count_up(3)  # 0, 1, 2
let gen2 = count_up(2)  # 0, 1
let combined = combine_generators(gen1, gen2)

loop value in combined {
    print(value)  # 0, 0, 1, 1, 2
}
```

## 高級生成器模式

### 惰性計算

```valkyrie
# 惰性計算素數
micro prime_generator() {
    let mut candidates = 2..
    let mut primes = []
    
    loop candidate in candidates {
        let is_prime = primes.all { candidate % $ != 0 }
        if is_prime {
            primes.push(candidate)
            yield candidate
        }
    }
}

# 獲取前10個素數
let primes = prime_generator()
loop i in 0..<10 {
    print(primes.next())  # 2, 3, 5, 7, 11, 13, 17, 19, 23, 29
}
```

### 檔案處理生成器

```valkyrie
# 逐行讀取檔案
micro read_lines(filename: utf8) {
    let file = open_file(filename)
    try {
        while !file.eof() {
            let line = file.read_line()
            if !line.is_empty() {
                yield line.trim()
            }
        }
    } finally {
        file.close()
    }
}

# 使用
loop line in read_lines("data.txt") {
    print("Line: {line}")
}
```

### 資料轉換生成器

```valkyrie
# 資料轉換管道
micro transform_data(data: Iterator<utf8>) {
    loop item in data {
        # 解析JSON
        let parsed = json_parse(item)
        if parsed.is_ok() {
            let obj = parsed.unwrap()
            
            # 驗證資料
            if obj.has_field("id") && obj.has_field("name") {
                # 轉換格式
                let transformed = {
                    id: obj.id,
                    name: obj.name.to_uppercase(),
                    timestamp: current_time()
                }
                yield transformed
            }
        }
    }
}
```

## 錯誤處理

### 生成器異常處理

```valkyrie
# 生成器中的異常處理
micro error_prone_generator() {
    try {
        yield "開始處理"
        
        let risky_operation = perform_risky_task()
        yield "風險操作完成"
        
        if risky_operation.is_error() {
            raise "操作失敗"
        }
        
        yield "處理成功"
    }
    .catch {
        case _:
            yield "發生錯誤: {error}"
            raise error  # 重新拋出異常
    }
}

# 使用帶錯誤處理的生成器
let gen = error_prone_generator()
try {
    loop status in gen {
        print(status)
    }
}
.catch {
    case _:
        print("生成器異常: {error}")
}
```

## 最佳實踐

### 1. 生成器設計原則

```valkyrie
# 保持生成器簡單和專注
micro good_generator(data: [utf8]) {
    loop item in data {
        if item.is_valid() {
            yield item.process()  # 只做一件事
        }
    }
}

# 避免在生成器中進行複雜的狀態管理
# 不好的例子：
micro bad_generator() {
    let mut complex_state = ComplexState::new()
    # ... 複雜的狀態邏輯
}
```

### 2. 資源管理

```valkyrie
# 確保資源正確釋放
micro file_processor(filename: utf8) {
    using file = open_file(filename) {
        while !file.eof() {
            let line = file.read_line()
            yield process_line(line)
        }
    }  # 檔案自動關閉
}
```

### 3. 效能考慮

```valkyrie
# 避免頻繁的小yield
# 不好的例子：
micro inefficient_generator(data: [i32]) {
    loop item in data {
        yield item  # 每個元素都yield
    }
}

# 好的例子：
micro efficient_generator(data: [i32]) {
    let mut batch = []
    loop item in data {
        batch.push(item)
        if batch.length >= 100 {
            yield batch  # 批量yield
            batch = []
        }
    }
    if !batch.is_empty() {
        yield batch  # 處理剩餘項目
    }
}
```

### 4. 測試生成器

```valkyrie
# 生成器測試策略
micro test_generator() {
    let gen = count_up(3)
    
    # 測試生成的值
    @assert_equal(gen.next(), Some(0))
    @assert_equal(gen.next(), Some(1))
    @assert_equal(gen.next(), Some(2))
    @assert_equal(gen.next(), None)
    
    # 測試狀態
    @assert_equal(gen.state(), GeneratorState::Completed)
}

# 生成器整合測試
micro test_pipeline() {
    let input = [1, 2, 3, 4]
    let pipeline = pipeline_stage1(input.iter())
    let results = pipeline.collect()
    
    @assert_equal(results, [2, 4, 6, 8])
}
```

### 5. 返回值限制

```valkyrie
# 生成器返回值不能是匿名類別
# 錯誤示例：
micro bad_generator() -> class { x: i32 } {  # 編譯錯誤
    yield 1
    class { x: 42 }  # 匿名類別作為返回值會導致型別推斷困難
}

# 正確示例：
class Result {
    x: i32
}

micro good_generator() -> Result {
    yield 1
    Result { x: 42 }  # 使用具名型別
}

# 或者使用型別別名
type GeneratorResult = class { x: i32 }

micro another_good_generator() -> GeneratorResult {
    yield 1
    GeneratorResult { x: 42 }
}
```

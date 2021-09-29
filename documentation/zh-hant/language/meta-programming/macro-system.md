# 宏系統 (Macro System)

## 概述

Valkyrie 提供了強大的宏系統，支援編譯時程式碼生成和元程式設計。宏系統分為兩個主要部分：

- **Macro (`@`)**: 編譯時函式呼叫，不捕捉後續參數
- **Annotation (`@`, `@`)**: 編譯時註解，會捕捉並作用於後續的 `class`、`micro` 等宣告

## Macro vs Annotation

### Macro (`@`)

Macro 使用 `@` 前綴，是編譯時的函式呼叫，不會捕捉後續的程式碼元素：

```valkyrie
# 編譯時常數計算
let FIBONACCI_10: i32 = evaluate(fibonacci(10))
let LOOKUP_TABLE: [i32; 256] = evaluate(generate_lookup_table())

# 環境變數獲取
let database_url: string = env("DATABASE_URL")

# 字串格式化
let message: string = f"Hello, {name}!"

# 向量建立
let numbers = [1, 2, 3, 4, 5]
let zeros = [0; 10]

# SQL 查詢
let query = @sql(
    "SELECT id, name FROM users WHERE active = $1",
    true
)
```

### Annotation (`@`)

Annotation 使用 `@` 前綴，會捕捉並作用於後續的宣告：

```valkyrie
# 測試註解
@test
micro test_addition() {
    @assert_eq(2 + 2, 4)
}

# 序列化註解
@derive(Serialize, Deserialize)
class User {
    name: string
    email: string
}

# 效能測試註解
@benchmark
micro fibonacci_benchmark() {
    fibonacci(30)
}

# 條件編譯註解
@cfg(feature = "debug")
micro debug_function() {
    print("Debug mode enabled")
}
```

## 常用 Macro

### 常用 Macro

```valkyrie
# 編譯時常數計算
let PI_SQUARED: f64 = evaluate(3.14159 * 3.14159)

# 編譯時檔案讀取
let config_content: string = compile_time_read_file("config.toml")

# 編譯時環境配置
@compile_time_env {
    memory_limit: "256MB",
    execution_timeout: "30s",
}
```


### 程式碼生成

```valkyrie
# 範本定義
@template {
    name: "crud_operations",
    params: [Entity: Type, Key: Type],
    body: {
        micro create(entity: Entity) -> Result⟨Key, Any⟩ {
            # 建立實體的通用邏輯
        }
        
        micro read(key: Key) -> Result⟨Entity, Any⟩ {
            # 讀取實體的通用邏輯
        }
        
        micro update(key: Key, entity: Entity) -> Result⟨unit, Any⟩ {
            # 更新實體的通用邏輯
        }
        
        micro delete(key: Key) -> Result⟨unit, Any⟩ {
            # 刪除實體的通用邏輯
        }
    }
}

# 範本實例化
@generate_code {
    crud_operations⟨User, UserId⟩
    crud_operations⟨Product, ProductId⟩
}
```

### 宏展開控制

```valkyrie
# 宏展開策略控制
@macro_expansion(strategy: "eager", max_depth: 100)
macro recursive_macro {
    # 遞迴宏定義
}
```

## 常用 Annotation

### 測試相關

```valkyrie
@test
micro test_user_creation() {
    let user = User("Alice", "alice@example.com")
    @assert_true(user.is_valid())
    @assert_eq(user.name, "Alice")
}

@test
@should_panic
micro test_invalid_email() {
    User("Bob", "invalid-email")
}
```

### 衍生註解

```valkyrie
@derive(Debug, Clone, PartialEq)
class Point {
    x: f64,
    y: f64,
}

@derive(Serialize, Deserialize)
class Config {
    database_url: string
    port: u16
}
```

### 條件編譯

```valkyrie
@cfg(target_os = "windows")
micro windows_specific_function() {
    # Windows 特定實作
}

@cfg(feature = "async")
class AsyncHandler {
    # 非同步處理器實作
}
```

## 自定義宏

### 宣告式宏

```valkyrie
macro vec_of {
    (#elem:expr; #n:expr) => {
        {
            let mut v = []
            loop _ in 0..#n {
                v.push(#elem)
            }
            v
        }
    }
    (#(#x:expr),+ #(,)?) => {
        @vec(#(#x),+)
    }
}
```

### 程序宏

```valkyrie
macro debug_print(args: TokenStream) -> TokenStream {
    if @cfg(debug_assertions) {
        @quote {
            print(#args)
        }
    } else {
        @quote {}
    }
}
```

## 最佳實踐

1. **明確區分用途**：
   - 使用 `@` 進行編譯時計算和程式碼生成
   - 使用 `@` 為宣告添加元資料和行為

2. **效能考量**：
   - 編譯時計算可以提高執行階段效能
   - 避免過度使用宏導致編譯時間過長

3. **可讀性**：
   - 為複雜宏添加文件註解
   - 使用有意義的宏名稱

4. **偵錯**：
   - 使用 `@macro_expansion` 控制宏展開
   - 利用編譯器的宏展開輸出進行偵錯

## 總結

Valkyrie 的宏系統提供了強大的元程式設計能力：

- **Macro (`@`)**: 編譯時函式，用於計算、生成和轉換
- **Annotation (`@`)**: 宣告註解，用於添加元資料和行為

正確使用這兩種機制可以大大提高程式碼的表達力和效能。

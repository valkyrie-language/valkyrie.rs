# Valkyrie 元編程架構

Valkyrie 提供了強大的元編程支援，允許在編譯時進行程式碼生成、變換和分析。透過整合在編譯器中的元編程系統，Valkyrie 能夠實作宏系統、編譯時計算、型別級編程等進階能力。

## 元編程架構概覽

### 元編程在編譯器中的定位

Valkyrie 的元編程系統深度整合在多層 IR 架構中，在不同層次提供相應的能力：

```
原始碼 + 元編程指令
         ↓
    AST + 宏展開
         ↓
    HIR + 編譯時計算
         ↓
    MIR + 程式碼優化
         ↓
    LIR + 平台特化
         ↓
    目標碼 + 執行階段支援
```

## 核心元編程特性

### [編譯時計算](./compile-time-computation.md)

**常量運算式求值**:
```valkyrie
# 編譯時常量計算
let FIBONACCI_10: i32 = evaluate(fibonacci(10))
let LOOKUP_TABLE: [i32; 256] = evaluate(generate_lookup_table())

# 編譯時字串處理
let CONFIG_KEY: string = evaluate(f"app.{env("BUILD_TARGET")}.version")
```

**編譯時函式執行**:
```valkyrie
# 標記為編譯時函式
@const_fn
micro fibonacci(n: i32) -> i32 {
    match n {
        case 0 | 1: n
        case _: fibonacci(n-1) + fibonacci(n-2)
    }
}

# 編譯時資料結構操作
@const_fn
micro build_state_machine() -> StateMachine {
    let mut sm = StateMachine()
    sm.add_state("start")
    sm.add_state("processing")
    sm.add_state("end")
    sm.add_transition("start", "process", "processing")
    sm.add_transition("processing", "finish", "end")
    sm
}
```

### [宏系統](./macro-system.md)

**宣告式宏**:
```valkyrie
# 模式比對宏
macro vec_of {
    (#elem:expr; #n:expr) => {
        {
            let mut v = []
            loop _ in 0..<#n {
                v.push(#elem)
            }
            v
        }
    }
    (#(#x:expr),+ #(,)?) => {
        @vec(#(#x),+)
    }
}

# 使用範例
let zeros = @vec_of(0; 10)
let numbers = @vec_of(1, 2, 3, 4, 5)
```

**過程宏**:
```valkyrie
# 自定義衍生宏
@derive(Serialize, Deserialize, Debug)
class User {
    id: u64,
    name: string,
    email: string,
}

# 屬性宏
@api_endpoint(method: "GET", path: "/users/{id}")
micro get_user(id: u64) -> Result⟨User, ApiError⟩ {
    # 自動生成路由註冊和參數驗證程式碼
    database::find_user(id)
}

# 函式式宏
let sql_query = @sql(
    "SELECT id, name, email FROM users WHERE active = $1",
    true
)
```

### [程式碼生成](./code-generation.md)

**基於範本的程式碼生成**:
```valkyrie
# 範本定義
@template {
    name: "crud_operations",
    params: [Entity: Type, Key: Type],
    body: {
        impl CrudOperations⟨{{Key}}⟩ for {{Entity}} {
            micro create(data: {{Entity}}) -> Result⟨{{Key}}, Any⟩ {
                # 生成建立邏輯
            }
            
            micro read(id: {{Key}}) -> Result⟨{{Entity}}, Any⟩ {
                # 生成讀取邏輯
            }
            
            micro update(id: {{Key}}, data: {{Entity}}) -> Result⟨unit, Any⟩ {
                # 生成更新邏輯
            }
            
            micro delete(id: {{Key}}) -> Result⟨unit, Any⟩ {
                # 生成刪除邏輯
            }
        }
    }
}

# 範本實例化
@generate_code {
    crud_operations<User, UserId>
    crud_operations<Product, ProductId>
    crud_operations<Order, OrderId>
}
```

**反射驅動的程式碼生成**:
```valkyrie
# 自動生成序列化程式碼
@auto_serialize
class Config {
    database_url: string,
    port: u16,
    debug: bool,
}

# 編譯時生成的程式碼
impl Serialize for Config {
    micro serialize(self) -> SerializedData {
        let mut data = SerializedData::new()
        data.insert("database_url", self.database_url)
        data.insert("port", self.port)
        data.insert("debug", self.debug)
        data
    }
}
```

### [型別級編程](./type-level-programming.md)

**型別級函式**:
```valkyrie
# 型別級計算
type Add(a: Nat, b: Nat) -> Nat {
    Add(Zero, b) = b,
    Add(Succ(a), b) = Succ(Add(a, b))
}

# 型別級列表操作
type Length(list: [T]) -> Nat {
    Length(Nil) = Zero,
    Length(Cons(_, tail)) = Succ(Length(tail))
}

# 編譯時型別驗證
micro safe_array_access⟨const N: usize, const I: usize⟩(arr: [i32; N]) -> i32 
where
    Assert⟨LessThan⟨I, N⟩⟩: True
{
    arr[I]  # 編譯時保證索引安全
}
```

**相依型別支援**:
```valkyrie
# 長度相依的向量型別
class Vector⟨T, const N: usize⟩ {
    data: [T; N],
}

impl⟨T, const N: usize⟩ Vector⟨T, N⟩ {
    micro push⟨const M: usize⟩(self, item: T) -> Vector⟨T, {N + 1}⟩ {
        # 型別級別保證長度正確性
    }
    
    micro concat⟨const M: usize⟩(self, other: Vector⟨T, M⟩) -> Vector⟨T, {N + M}⟩ {
        # 編譯時計算結果長度
    }
}
```

### [屬性系統](./attribute-system.md)

**註解驅動的程式碼變換**:
```valkyrie
# 效能監控註解
@monitor_performance
micro expensive_computation(data: [f64]) -> f64 {
    # 自動插入效能監控程式碼
    data.iter().map { $.powi(2) }.sum()
}

# 快取註解
@cache(ttl: "1h", key: "user_profile_{id}")
micro get_user_profile(id: UserId) -> UserProfile {
    # 自動生成快取邏輯
    database::load_user_profile(id)
}

# 驗證註解
@validate(email: "valid_email", age: "min:18,max:120")
class UserRegistration {
    email: string,
    age: u8,
    name: string,
}
```

**編譯時分析註解**:
```valkyrie
# 安全性分析
@security_analysis(check: "sql_injection,xss")
micro handle_user_input(input: string) -> string {
    # 編譯時靜態分析潛在安全問題
    sanitize_input(input)
}

# 記憶體安全註解
@memory_safe
micro process_buffer(buffer: mut [u8]) {
    # 編譯時驗證記憶體存取安全性
}
```

## 元編程執行模型

### **編譯時執行環境**

Valkyrie 提供了隔離的編譯時執行環境：

```valkyrie
# 編譯時環境配置
@compile_time_env {
    memory_limit: "256MB",
    execution_timeout: "30s",
    allowed_operations: ["file_read", "network_disabled", "system_disabled"]
}

# 編譯時資源管理
@const_fn
micro load_config_file() -> Config {
    let content = compile_time_read_file("config.toml")
    parse_toml(content)
}
```

### **宏展開策略**

```valkyrie
# 宏展開控制
@macro_expansion(strategy: "eager", max_depth: 100)
macro recursive_macro {
    # 宏定義
}

# 宏衛生性保證
macro hygienic_macro(var) {
    {
        let var = 42  # 不會與呼叫處的變數衝突
        var * 2
    }
}
```

### **程式碼生成快取**

```valkyrie
# 生成程式碼快取配置
@code_generation(cache: true, cache_key: "struct_hash")
@derive(Serialize)
class CachedStruct {
    # 結構體定義
}
```

## 跨語言元編程支援

### **統一的元編程介面**

Valkyrie 的元編程基礎設施可以支援多種語法的元編程介面：

```valkyrie
# Valkyrie 語言的宏
macro debug_print(#args...) {
    @cfg(debug_assertions)
    print("DEBUG: {}", format(#args...))
}

# 對應的 Python 風格宏（假設支援）
@macro
def debug_print(*args):
    if DEBUG:
        print(f"DEBUG: {format(*args)}")

# 對應的 JavaScript 風格宏（假設支援）
macro debugPrint(...args) {
    if (process.env.NODE_ENV === 'development') {
        console.log(`DEBUG: ${format(...args)}`);
    }
}
```

### **跨語言程式碼生成**

```valkyrie
# 介面定義
trait UserService {
    micro get_user(id: UserId) -> Result⟨User, Any⟩
    micro create_user(data: CreateUserRequest) -> Result⟨User, Any⟩
    micro update_user(id: UserId, data: UpdateUserRequest) -> Result⟨User, Any⟩
    micro delete_user(id: UserId) -> Result⟨unit, Any⟩
}

# 自動生成多語言繫結
@generate_bindings(languages: ["rust", "javascript", "python"])
class UserServiceBindings
```

## 效能和安全性

### **編譯時效能優化**

- **增量宏展開**: 只重新展開修改的宏
- **並行程式碼生成**: 多執行緒並行生成程式碼
- **智慧快取**: 基於相依圖的智慧快取策略
- **記憶體管理**: 高效的編譯時記憶體分配

### **安全性保證**

- **沙箱執行**: 編譯時程式碼在隔離環境中執行
- **資源限制**: 嚴格的記憶體和時間限制
- **權限控制**: 細粒度的操作權限管理
- **程式碼稽核**: 自動檢測潛在的安全問題

## 工具和除錯支援

### **元編程除錯器**

```valkyrie
# 宏展開除錯
@debug_macro_expansion
macro complex_macro {
    # 可以單步除錯宏展開過程
}

# 編譯時執行追蹤
@trace_const_eval
const RESULT: i32 = complex_computation()
```


## 條件編譯

Valkyrie 使用 staging 機制進行編譯期計算和條件編譯：

```valkyrie
# 編譯時條件
<# if DEBUG #>
    print("除錯模式")
<# else #>
    print("發布模式")
<# end if #>

# 編譯期值計算
<# x.value #>

# 平台特定程式碼
<# if PLATFORM == "windows" #>
    use windows_api
<# else if PLATFORM == "linux" #>
    use linux_api
<# else #>
    use generic_api
<# end if #>

# 複雜編譯期運算式
<# if feature_enabled && version >= "2.0" #>
    # 新功能程式碼
    advanced_feature()
<# end if #>
```

## 控制流最佳實踐

1. **優先使用運算式形式**：當控制流有傳回值時，使用運算式形式更簡潔
2. **合理使用標籤**：在巢狀迴圈中使用標籤提高程式碼可讀性
3. **異常處理要具體**：針對不同型別的異常進行具體處理
4. **避免深層巢狀**：使用提前傳回和守衛條件減少巢狀層次
5. **模式比對優於多重 if**：對於複雜條件判斷，使用 match 更清晰

### **程式碼生成視覺化**

- **宏展開樹**: 視覺化宏展開過程
- **程式碼生成圖**: 顯示程式碼生成的相依關係
- **效能分析**: 編譯時效能瓶頸分析
- **記憶體使用**: 編譯時記憶體使用情況

## 最佳實踐

### **宏設計原則**

1. **最小化原則**: 宏應該盡可能簡單和專注
2. **衛生性**: 避免意外的名稱衝突
3. **可除錯性**: 提供清晰的錯誤訊息
4. **效能考慮**: 避免過度的宏展開

### **編譯時計算指導**

1. **純函式**: 編譯時函式應該是純函式
2. **資源限制**: 注意記憶體和時間限制
3. **錯誤處理**: 提供清晰的編譯時錯誤訊息
4. **快取策略**: 合理使用編譯時快取

### **程式碼生成建議**

1. **範本化**: 使用範本而不是字串拼接
2. **型別安全**: 生成的程式碼應該是型別安全的
3. **可讀性**: 生成的程式碼應該是可讀的
4. **文件化**: 為生成的程式碼提供文件

## 總結

Valkyrie 的元編程系統提供了強大而安全的編譯時程式碼操作能力。透過統一的架構設計，它提供了包括以下內容在內的一致體驗：

1. **編譯時計算**: 高效的常量運算式求值和函式執行
2. **宏系統**: 宣告式和過程宏的統一支援
3. **程式碼生成**: 基於範本和反射的靈活程式碼生成
4. **型別級編程**: 強大的型別級計算和驗證
5. **屬性系統**: 註解驅動的程式碼變換和分析

這些特性使得開發者能夠編寫更加簡潔、安全和高效的程式碼，同時保持良好的開發體驗和除錯支援。
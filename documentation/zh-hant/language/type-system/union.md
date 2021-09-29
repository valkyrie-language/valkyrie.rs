# 聯合類型 (Unite Types)

聯合類型是 Valkyrie 中表示多種可能值的強大類型系統特性，使用顯式 `tag` 的 `unite` 定義。標準寫法是 `[tag(XXXKind)] unite XXX { }`，語言不再自動生成 tag 型別。

## 基本聯合類型

### 簡單聯合類型

```valkyrie
# 結果類型 - 表示操作可能成功或失敗
[tag(ResultKind)]
unite Result⟨T, E⟩ {
    Fine { value: T }
    Fail { error: E }
}

# 選項類型 - 表示值可能存在或不存在
[tag(OptionKind)]
unite Option⟨T⟩ {
    Some { value: T }
    None
}
```

### 複雜聯合類型

```valkyrie
# JSON 值類型
[tag(JsonValueKind)]
unite JsonValue {
    Null,
    Bool { value: bool },
    Number { value: f64 },
    String { value: string },
    Array { items: [JsonValue] },
    Object { fields: {string: JsonValue} }
}

# 表達式抽象語法樹
[tag(ExpressionKind)]
unite Expression {
    Literal { value: i32 },
    Variable { name: string },
    Binary {
        left: Expression,
        operator: string,
        right: Expression
    }
}
```

## 聯合類型的使用

### 模式匹配

```valkyrie
# 基本模式匹配
let result: Result⟨i32, string⟩ = Fine { value: 42 }
match result {
    case Fine { value }: print("成功: ${value}")
    case Fail { error }: print("失敗: ${error}")
}
```

# 嵌套模式匹配
let nested: Result⟨Option⟨i32⟩, string⟩ = Fine { value: Some { value: 42 } }
match nested {
    case Fine { value: Some { value } }: print("值: ${value}")
    case Fine { value: None }: print("無值")
    case Fail { error }: print("錯誤: ${error}")
}
```

### if let 表達式

```valkyrie
# 簡化的模式匹配
if let Fine { value } = result {
    print("成功獲得值: ${value}")
}

# 带 else 分支
if let Some { value } = option {
    process(value)
} else {
    print("選項為空")
}
```

## 聯合類型方法

### 關聯方法

```valkyrie
[tag(ResultKind)]
unite Result⟨T, E⟩ {
    Fine { value: T },
    Fail { error: E },
    
    # 檢查是否成功
    micro is_ok(self) -> bool {
        if let Fine { .. } = self {
            true
        } else {
            false
        }
    }
    
    # 檢查是否失敗
    micro is_err(self) -> bool {
        if let Fail { .. } = self {
            true
        } else {
            false
        }
    }
    
    # 獲取值（可能 panic）
    micro unwrap(self) -> T {
        if let Fine { value } = self {
            value
        } else {
            panic("Called unwrap on Fail")
        }
    }
    
    # 安全獲取值
    micro unwrap_or(self, default: T) -> T {
        if let Fine { value } = self {
            value
        } else {
            default
        }
    }
    
    # 映射成功值
    micro map⟨U⟩(self, f: micro(T) -> U) -> Result⟨U, E⟩ {
        if let Fine { value } = self {
            Fine { value: f(value) }
        } else if let Fail { error } = self {
            Fail { error }
        }
    }
    
    # 映射錯誤值
    micro map_err⟨F⟩(self, f: micro(E) -> F) -> Result⟨T, F⟩ {
        if let Fine { value } = self {
            Fine { value }
        } else if let Fail { error } = self {
            Fail { error: f(error) }
        }
    }
}
```

### Option 類型方法

```valkyrie
[tag(OptionKind)]
unite Option<T> {
    Some { value: T },
    None,
    
    # 檢查是否有值
    micro is_some(self) -> bool {
        if let Some { .. } = self {
            true
        } else {
            false
        }
    }
    
    # 檢查是否為空
    micro is_none(self) -> bool {
        if let None = self {
            true
        } else {
            false
        }
    }
    
    # 映射值
    micro map<U>(self, f: micro(T) -> U) -> Option<U> {
        if let Some { value } = self {
            Some { value: f(value) }
        } else {
            None
        }
    }
    
    # 過濾值
    micro filter(self, predicate: micro(T) -> bool) -> Option<T> {
        if let Some { value } = self {
            if predicate(value) {
                Some { value: value }
            } else {
                None
            }
        } else {
            None
        }
    }
}
```

## 高級特性

### 泛型聯合類型

```valkyrie
# 多參數泛型
[tag(EitherKind)]
unite Either<L, R> {
    Left { value: L },
    Right { value: R }
}

# 帶約束的泛型
[tag(ContainerKind)]
unite Container⟨T⟩ where T: Clone {
    Single { item: T },
    Multiple { items: [T] }
}
```

### 遞歸聯合類型

```valkyrie
# 鏈表
[tag(ListKind)]
unite List⟨T⟩ {
    Empty,
    Node {
        value: T,
        next: List⟨T⟩
    }
}

# 二叉樹
[tag(TreeKind)]
unite Tree⟨T⟩ {
    Leaf { value: T },
    Branch {
        left: Tree⟨T⟩,
        right: Tree⟨T⟩
    }
}
```

## 最佳實踐

### 1. 使用描述性的變體名稱

```valkyrie
# 好的命名
[tag(HttpResponseKind)]
unite HttpResponse {
    Success { data: String, status: u16 },
    ClientError { message: String, code: u16 },
    ServerError { message: String, code: u16 },
    NetworkError { reason: String }
}

# 避免過於簡單的命名
[tag(BadKind)]
unite Bad {
    A { x: i32 },
    B { y: String }
}
```

### 2. 合理使用欄位命名

```valkyrie
# 當只有一個欄位時，使用 value
[tag(OptionKind)]
unite Option<T> {
    Some { value: T },
    None
}

# 多個欄位時使用描述性名稱
[tag(PersonKind)]
unite Person {
    Student { name: String, grade: i32 },
    Teacher { name: String, subject: String }
}
```

### 3. 提供便利方法

```valkyrie
[tag(ValidationResultKind)]
unite ValidationResult<T> {
    Valid { data: T },
    Invalid { errors: [String] },
    
    # 便利方法
    micro is_valid(self) -> bool {
        matches!(self, Valid { .. })
    }
    
    micro get_errors(self) -> [String] {
        if let Invalid { errors } = self {
            errors
        } else {
            []
        }
    }
}
```

### 4. 錯誤處理模式

```valkyrie
# 使用 Result 進行錯誤處理
micro divide(a: f64, b: f64) -> Result<f64, String> {
    if b == 0.0 {
        Fail { error: "除零錯誤" }
    } else {
        Fine { value: a / b }
    }
}

# 鏈式錯誤處理
micro process_data(input: String) -> Result<ProcessedData, Error> {
    input
        .parse()
        .map_err { Error::ParseError($e) }?
        .validate()
        .map_err { Error::ValidationError($e) }?
        .transform()
        .map_err { Error::TransformError($e) }
}
```

聯合類型是 Valkyrie 類型系統的核心特性，它提供了類型安全的方式來處理多種可能的值，特別適合錯誤處理、狀態表示和數據建模等場景。

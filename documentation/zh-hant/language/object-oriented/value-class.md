# 值類別

值類別（Value Class）是一種輕量級的類別，用於封裝簡單值並提供型別安全。值類別在編譯時會盡可能內聯，避免物件分配的開銷。

## 定義值類別

### 基本語法

```valkyrie
# 使用 value 關鍵字定義值類別
value class UserId {
    value: i32
}

value class Email {
    value: string
}

value class Money {
    amount: i64
    currency: string
}
```

### 使用值類別

```valkyrie
# 建立值類別實例
let user_id = UserId { value: 42 }
let email = Email { value: "user@example.com" }
let price = Money { amount: 1000, currency: "USD" }

# 存取內部值
print(user_id.value)  # 42
print(email.value)    # "user@example.com"
```

## 型別安全

### 避免原始型別濫用

```valkyrie
# 沒有值類別時，容易混淆參數
micro process_order(user_id: i32, product_id: i32, quantity: i32) {
    # 可能錯誤地交換參數順序
}

# 使用值類別提供型別安全
micro process_order(user_id: UserId, product_id: ProductId, quantity: Quantity) {
    # 編譯器會檢查型別
}

let user = UserId { value: 1 }
let product = ProductId { value: 100 }
let qty = Quantity { value: 5 }

process_order(user, product, qty)  # 正確
# process_order(product, user, qty)  # 編譯錯誤！
```

## 內聯優化

### 編譯時優化

```valkyrie
# 值類別在編譯時會盡可能內聯
value class Meters {
    value: f64
}

let distance1 = Meters { value: 10.0 }
let distance2 = Meters { value: 5.0 }

# 運算會被優化為直接的數值運算
let total = distance1.value + distance2.value
```

### 避免裝箱

```valkyrie
# 值類別避免裝箱開銷
value class Point {
    x: f64
    y: f64
}

# Point 在記憶體中是連續的，沒有物件頭開銷
let points = [
    Point { x: 0, y: 0 },
    Point { x: 1, y: 1 },
    Point { x: 2, y: 2 }
]
```

## 實現 Trait

### 為值類別實現 Trait

```valkyrie
value class Meters {
    value: f64
}

impl Add for Meters {
    micro add(self, other: Meters) -> Meters {
        Meters { value: self.value + other.value }
    }
}

impl Display for Meters {
    micro display(self) -> string {
        "${self.value}m"
    }
}

# 使用
let d1 = Meters { value: 10.0 }
let d2 = Meters { value: 5.0 }
let total = d1 + d2  # Meters { value: 15.0 }
print(total)         # "15.0m"
```

### 衍生標準 Trait

```valkyrie
#[derive(Eq, Hash, Clone, Debug)]
value class UserId {
    value: i32
}
```

## 驗證

### 建構時驗證

```valkyrie
value class Email {
    value: string
    
    new(value: string) {
        if !value.contains("@") {
            raise "Invalid email format"
        }
        self.value = value
    }
}

# 安全建立
let valid_email = Email::new("user@example.com")  # OK
# let invalid = Email::new("invalid")  # 拋出異常
```

### 工廠方法

```valkyrie
value class Port {
    value: u16
    
    micro from_i32(value: i32) -> Result⟨Port, string⟩ {
        if value < 0 || value > 65535 {
            return Fail("Port must be between 0 and 65535")
        }
        Fine(Port { value: value as u16 })
    }
    
    micro http() -> Port {
        Port { value: 80 }
    }
    
    micro https() -> Port {
        Port { value: 443 }
    }
}
```

## 最佳實踐

### 1. 用於領域概念

```valkyrie
# 好的實踐：表示領域概念
value class UserId { value: i32 }
value class OrderId { value: string }
value class ProductCode { value: string }

# 避免：過度使用
value class Age { value: i32 }  # 可能不需要
```

### 2. 保持簡單

```valkyrie
# 好的實踐：簡單封裝
value class Temperature {
    value: f64  # 攝氏度
}

# 避免：複雜邏輯
value class ComplexValue {
    value: f64
    # 太多方法會增加內聯難度
}
```

### 3. 不變性

```valkyrie
# 值類別應該是不可變的
value class ImmutablePoint {
    x: f64
    y: f64
    
    # 返回新實例而不是修改
    micro translate(self, dx: f64, dy: f64) -> ImmutablePoint {
        ImmutablePoint { x: self.x + dx, y: self.y + dy }
    }
}
```

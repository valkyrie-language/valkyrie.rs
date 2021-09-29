# 屬性系統

屬性是類別成員的一種封裝形式，提供了對欄位的受控存取。Valkyrie 的屬性系統支援自定義 getter 和 setter，以及各種存取修飾符。

## 基本屬性

### 定義屬性

```valkyrie
class Person {
    # 基本屬性定義
    name: string
    age: i32
    
    # 帶預設值的屬性
    email: string = ""
    active: bool = true
}
```

### 屬性存取

```valkyrie
let person = Person {
    name: "Alice",
    age: 30
}

# 讀取屬性
print(person.name)  # "Alice"

# 設置屬性
person.age = 31
```

## Getter 和 Setter

### 自定義 Getter

```valkyrie
class Circle {
    radius: f64
    
    # 計算屬性（只讀）
    get area -> f64 {
        math.pi * self.radius * self.radius
    }
    
    get diameter -> f64 {
        2 * self.radius
    }
}
```

### 自定義 Setter

```valkyrie
class Temperature {
    mut celsius: f64
    
    # 設置時進行驗證
    set celsius(value: f64) {
        if value < -273.15 {
            raise "Temperature below absolute zero"
        }
        self.celsius = value
    }
    
    # 華氏溫度屬性
    get fahrenheit -> f64 {
        self.celsius * 9 / 5 + 32
    }
    
    set fahrenheit(value: f64) {
        self.celsius = (value - 32) * 5 / 9
    }
}
```

### 完整屬性定義

```valkyrie
class BankAccount {
    private mut _balance: f64 = 0
    
    # 完整屬性定義
    get balance -> f64 {
        self._balance
    }
    
    set balance(value: f64) {
        if value < 0 {
            raise "Balance cannot be negative"
        }
        let old_balance = self._balance
        self._balance = value
        self.on_balance_changed(old_balance, value)
    }
    
    private micro on_balance_changed(self, old: f64, new: f64) {
        print("Balance changed from ${old} to ${new}")
    }
}
```

## 存取修飾符

### 公開屬性

```valkyrie
class PublicExample {
    public name: string  # 預設就是公開的
}
```

### 私有屬性

```valkyrie
class PrivateExample {
    private _internal_state: i32
    
    # 通過公開方法存取私有屬性
    public micro get_state(self) -> i32 {
        self._internal_state
    }
}
```

### 保護屬性

```valkyrie
class ProtectedExample {
    protected internal_id: string
}

class Derived : ProtectedExample {
    micro access_internal(self) {
        print(self.internal_id)  # 子類可以存取
    }
}
```

## 延遲初始化屬性

### 懶載入屬性

```valkyrie
class ExpensiveResource {
    # 延遲初始化
    lazy heavy_data: HeavyData = {
        load_heavy_data()  # 首次存取時計算
    }
    
    # 另一種寫法
    private mut _cache: Cache? = null
    
    get cache -> Cache {
        if self._cache == null {
            self._cache = Cache::new()
        }
        self._cache
    }
}
```

## 觀察屬性

### 屬性變更通知

```valkyrie
class ObservableValue⟨T⟩ {
    private mut _value: T
    private mut observers: [(T, T) -> Unit] = []
    
    get value -> T {
        self._value
    }
    
    set value(new_value: T) {
        let old_value = self._value
        self._value = new_value
        loop observer in self.observers {
            observer(old_value, new_value)
        }
    }
    
    micro observe(mut self, callback: (T, T) -> Unit) {
        self.observers.push(callback)
    }
}
```

## 屬性委託

### 委託屬性

```valkyrie
class DelegatedProperty⟨T⟩ {
    private getter: { -> T }
    private setter: (T) -> Unit
    
    get value -> T {
        self.getter()
    }
    
    set value(v: T) {
        self.setter(v)
    }
    
    new(getter: { -> T }, setter: (T) -> Unit) {
        self.getter = getter
        self.setter = setter
    }
}
```

## 最佳實踐

### 1. 使用屬性封裝欄位

```valkyrie
# 好的實踐
class GoodExample {
    private _name: string
    
    get name -> string {
        self._name
    }
    
    set name(value: string) {
        self._name = value.trim()
    }
}

# 避免
class BadExample {
    mut name: string  # 直接暴露可變欄位
}
```

### 2. 計算屬性保持輕量

```valkyrie
class Rectangle {
    width: f64
    height: f64
    
    # 輕量計算屬性
    get area -> f64 {
        self.width * self.height
    }
    
    # 複雜計算應該使用方法
    micro complex_calculation(self) -> ComplexResult {
        # 耗時計算
    }
}
```

### 3. 屬性驗證

```valkyrie
class ValidatedProperty {
    private mut _email: string = ""
    
    get email -> string {
        self._email
    }
    
    set email(value: string) {
        if !value.contains("@") {
            raise "Invalid email format"
        }
        self._email = value
    }
}
```

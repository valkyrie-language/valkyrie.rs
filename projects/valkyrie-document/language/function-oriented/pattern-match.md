# 模式匹配 (Match)

Valkyrie 提供了强大的模式匹配功能，支持多种匹配模式和语法形式。

## 基本 Match 语法

### 标准 Match 语句

```valkyrie
# 基本模式匹配
match value {
    case 1: "one"
    case 2: "two"
    case 3: "three"
    else: "other"
}

# 范围匹配
match score {
    case 90..=100: "A"
    case 80..=89: "B"
    case 70..=79: "C"
    case 60..=69: "D"
    else: "F"
}

# 多值匹配
match day {
    case "Saturday" | "Sunday": "Weekend"
    case "Monday"..="Friday": "Weekday"
    else: "Invalid day"
}
```

### 表达式 Match 语法

```valkyrie
# 表达式形式的 match
let result = value.match {
    case 1: "one"
    case 2: "two"
    case 3: "three"
    else: "other"
}

# 链式调用
let processed = input
    .transform()
    .match {
        case Fine { value }: value * 2
        case Fail { error }: 0
    }
    .to_string()
```

## 解构匹配

### 元组解构

```valkyrie
match point {
    case (0, 0): "Origin"
    case (x, 0): "On X-axis at ${x}"
    case (0, y): "On Y-axis at ${y}"
    case (x, y): "Point at (${x}, ${y})"
}

# 嵌套元组
match nested {
    case ((a, b), c): "Nested: ${a}, ${b}, ${c}"
    case (x, (y, z)): "Other nested: ${x}, ${y}, ${z}"
    else: "No match"
}
```

### 数组解构

```valkyrie
match array {
    case []: "Empty array"
    case [x]: "Single element: ${x}"
    case [first, second]: "Two elements: ${first}, ${second}"
    case [head, ..tail]: "Head: ${head}, Tail length: ${tail.len()}"
    case [.., last]: "Last element: ${last}"
    case [first, .., last]: "First: ${first}, Last: ${last}"
}

# 固定长度匹配
match coordinates {
    case [x, y]: "2D point: (${x}, ${y})"
    case [x, y, z]: "3D point: (${x}, ${y}, ${z})"
    else: "Unsupported dimension"
}
```

### 对象解构

```valkyrie
match person {
    case { name: "Alice", age }: "Alice is ${age} years old"
    case { name, age: 18..=65 }: "${name} is working age"
    case { name, age, city: "Beijing" }: "${name} from Beijing, age ${age}"
    case { name, ..rest }: "Person ${name} with other fields"
    else: "Unknown person"
}

# 嵌套对象匹配
match user {
    case { profile: { name, email }, active: true }: 
        "Active user: ${name} (${email})"
    case { profile: { name }, active: false }: 
        "Inactive user: ${name}"
    else: "Invalid user"
}
```

## 联合类型匹配

### Result 类型匹配

```valkyrie
match operation_result {
    case Fine { value }: "Success: ${value}"
    case Fail { error }: "Error: ${error}"
}

# 嵌套 Result 匹配
match nested_result {
    case Fine { value: Fine { value } }: "Double success: ${value}"
    case Fine { value: Fail { error: inner_error } }: "Inner error: ${inner_error}"
    case Fail { error: outer_error }: "Outer error: ${outer_error}"
}
```

### Option 类型匹配

```valkyrie
match maybe_value {
    case Some(value): "Found: ${value}"
    case None: "Nothing found"
}

# 复杂 Option 匹配
match complex_option {
    case Some({ name, age }) if age >= 18: 
        "Adult: ${name}"
    case Some({ name, age }): 
        "Minor: ${name}"
    case None: "No person"
}
```

### 自定义联合类型匹配

```valkyrie
union JsonValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array([JsonValue]),
    Object({String: JsonValue})
}

match json_value {
    case Null: "null"
    case Bool(true): "true"
    case Bool(false): "false"
    case Number(n): "number: ${n}"
    case String(s): "string: ${s}"
    case Array(items): "array with ${items.len()} items"
    case Object(map): "object with ${map.len()} keys"
}
```

## 守卫条件

### 基本守卫

```valkyrie
match number {
    case x if x > 0: "positive"
    case x if x < 0: "negative"
    case 0: "zero"
    else: "not a number"
}

# 复杂守卫条件
match person {
    case { age, name } if age >= 65: "Senior: ${name}"
    case { age, name } if age >= 18: "Adult: ${name}"
    case { age, name } if age >= 13: "Teenager: ${name}"
    case { name, .. }: "Child: ${name}"
}
```

### 多重守卫

```valkyrie
match user {
    case { role: "admin", active: true } if user.permissions.contains("write"): 
        "Full access"
    case { role: "user", active: true } if user.last_login.is_recent(): 
        "User access"
    case { active: false }: "Account disabled"
    else: "No access"
}
```

## Extractor 模式和 Unapply

### Extractor 模式基础

Extractor 模式允许在模式匹配中使用自定义的解构逻辑，通过 `unapply` 方法实现。这使得可以使用类似构造函数的语法进行模式匹配。

```valkyrie
# 定义带有 unapply 方法的类
class User {
    name: String
    email: String
    age: i32
    
    new(name: String, email: String, age: i32) {
        self.name = name
        self.email = email
        self.age = age
    }
    
    # unapply 方法用于模式匹配中的解构
    unapply(user: User) -> Option<(String, String, i32)> {
        Some((user.name, user.email, user.age))
    }
}

# 使用 extractor 模式进行匹配
match user_data {
    case User(name, email, age) if age >= 18: 
        "Adult user: ${name} (${email})"
    case User(name, _, age): 
        "Minor user: ${name}, age ${age}"
    else: "Invalid user data"
}
```

### 复杂 Extractor 模式

```valkyrie
# 定义复杂的 extractor
class EmailAddress {
    local: String
    domain: String
    
    new(email: String) {
        let parts = email.split("@")
        if parts.len() == 2 {
            self.local = parts[0]
            self.domain = parts[1]
        } else {
            panic("Invalid email format")
        }
    }
    
    # 支持多种解构方式
    unapply(email: EmailAddress) -> Option<(String, String)> {
        Some((email.local, email.domain))
    }
    
    # 可以定义多个 unapply 重载
    unapply_full(email: EmailAddress) -> Option<String> {
        Some("${email.local}@${email.domain}")
    }
}

# 使用不同的 extractor 模式
match email {
    case EmailAddress(local, "gmail.com"): "Gmail user: ${local}"
    case EmailAddress(local, "outlook.com"): "Outlook user: ${local}"
    case EmailAddress(local, domain): "User ${local} from ${domain}"
    else: "Invalid email"
}
```

### 嵌套 Extractor 模式

```valkyrie
class Address {
    street: String
    city: String
    country: String
    
    unapply(addr: Address) -> Option<(String, String, String)> {
        Some((addr.street, addr.city, addr.country))
    }
}

class Person {
    name: String
    email: EmailAddress
    address: Address
    
    unapply(person: Person) -> Option<(String, EmailAddress, Address)> {
        Some((person.name, person.email, person.address))
    }
}

# 嵌套使用 extractor 模式
match person {
    case Person(name, EmailAddress(_, "gmail.com"), Address(_, "Beijing", "China")): 
        "Beijing Gmail user: ${name}"
    case Person(name, EmailAddress(local, domain), Address(_, city, _)): 
        "User ${name} (${local}@${domain}) from ${city}"
    else: "Unknown person"
}
```

### 条件 Extractor 模式

```valkyrie
class Range {
    start: i32
    end: i32
    
    # 条件性的 unapply
    unapply(value: i32, range: Range) -> Option<i32> {
        if value >= range.start && value <= range.end {
            Some(value)
        } else {
            None
        }
    }
}

# 使用条件 extractor
let valid_age = Range { start: 0, end: 120 }
let adult_age = Range { start: 18, end: 120 }

match age {
    case adult_age(a): "Adult age: ${a}"
    case valid_age(a): "Valid age: ${a}"
    else: "Invalid age"
}
```

### 自定义集合 Extractor

```valkyrie
class NonEmpty<T> {
    items: [T]
    
    new(items: [T]) {
        if items.is_empty() {
            panic("Cannot create NonEmpty with empty array")
        }
        self.items = items
    }
    
    unapply<T>(list: [T]) -> Option<(T, [T])> {
        if list.is_empty() {
            None
        } else {
            Some((list[0], list[1..]))
        }
    }
}

# 使用集合 extractor
match numbers {
    case NonEmpty(head, tail): "Head: ${head}, Tail length: ${tail.len()}"
    case []: "Empty list"
}
```

### 正则表达式 Extractor

```valkyrie
class Regex {
    pattern: String
    
    new(pattern: String) {
        self.pattern = pattern
    }
    
    unapply(text: String, regex: Regex) -> Option<[String]> {
        # 假设有正则表达式匹配功能
        regex.find_matches(text)
    }
}

# 使用正则表达式 extractor
let email_pattern = Regex::new(r"([a-zA-Z0-9._%+-]+)@([a-zA-Z0-9.-]+\.[a-zA-Z]{2,})")
let phone_pattern = Regex::new(r"(\d{3})-(\d{3})-(\d{4})")

match input {
    case email_pattern([_, local, domain]): "Email: ${local}@${domain}"
    case phone_pattern([_, area, exchange, number]): "Phone: (${area}) ${exchange}-${number}"
    else: "Unknown format"
}
```

### Extractor 最佳实践

```valkyrie
# 1. 保持 unapply 方法简单和高效
class Point {
    x: f64
    y: f64
    
    # 简单直接的 unapply
    unapply(point: Point) -> Option<(f64, f64)> {
        Some((point.x, point.y))
    }
    
    # 可以提供多个 unapply 变体
    unapply_polar(point: Point) -> Option<(f64, f64)> {
        let r = (point.x * point.x + point.y * point.y).sqrt()
        let theta = point.y.atan2(point.x)
        Some((r, theta))
    }
}

# 2. 使用有意义的变量名
match coordinate {
    case Point(x, y) if x == 0.0 && y == 0.0: "Origin"
    case Point(x, 0.0): "On X-axis at ${x}"
    case Point(0.0, y): "On Y-axis at ${y}"
    case Point(x, y): "Point at (${x}, ${y})"
}

# 3. 组合使用 extractor 和其他模式
match data {
    case [Point(0.0, 0.0), ..rest]: "Starts with origin, ${rest.len()} more points"
    case [Point(x, y), Point(x2, y2)] if x == x2: "Vertical line"
    case [Point(x, y), Point(x2, y2)] if y == y2: "Horizontal line"
    case points: "General path with ${points.len()} points"
}
```

## 变量绑定

### @ 绑定语法

```valkyrie
match value {
    case x @ 1..=10: "Small number: ${x}"
    case x @ 11..=100: "Medium number: ${x}"
    case large @ 101..: "Large number: ${large}"
    else: "Out of range"
}

# 复杂绑定
match data {
    case person @ { name, age } if age >= 18: 
        "Adult person: ${person}"
    case child @ { name, age }: 
        "Child: ${child}"
}
```

### 嵌套绑定

```valkyrie
match nested_structure {
    case { outer: inner @ { value, .. }, .. } if value > 0: 
        "Positive inner value: ${inner}"
    case { outer: inner @ { value, .. }, .. }: 
        "Non-positive inner value: ${inner}"
    else: "No match"
}
```

## 穷尽性检查

### 编译时检查

```valkyrie
# 编译器会检查是否覆盖所有情况
match boolean_value {
    case true: "yes"
    case false: "no"
    # 不需要 else，因为已经穷尽
}

# 联合类型的穷尽性
match result {
    case Fine { value }: handle_success(value)
    case Fail { error }: handle_error(error)
    # 编译器确保所有变体都被处理
}
```

### 不可达分支警告

```valkyrie
match number {
    case x if x > 0: "positive"
    case x if x >= 0: "non-negative"  # 警告：不可达
    case x: "negative"
}
```

## 最佳实践

### 性能优化

```valkyrie
# 将最常见的情况放在前面
match http_status {
    case 200: "OK"                    # 最常见
    case 404: "Not Found"            # 次常见
    case 500: "Internal Server Error" # 较少见
    case status: "Status: ${status}"  # 其他情况
}

# 避免复杂的守卫条件
match user {
    case { role: "admin" }: handle_admin(user)
    case { role: "user" }: handle_user(user)
    case other: handle_other(other)
}
```

### 可读性优化

```valkyrie
# 使用有意义的变量名
match request {
    case { method: "GET", path }: handle_get(path)
    case { method: "POST", path, body }: handle_post(path, body)
    case { method: unsupported_method, .. }: 
        error("Unsupported method: ${unsupported_method}")
}

# 适当使用注释
match complex_data {
    # 处理标准格式
    case { version: "1.0", data }: process_v1(data)
    # 处理新格式
    case { version: "2.0", data }: process_v2(data)
    # 向后兼容
    case legacy_data: migrate_and_process(legacy_data)
}
```

## 错误处理模式

### 链式错误处理

```valkyrie
let result = input
    .parse()
    .match {
        case Fine { value: parsed }: parsed
        case Fail { error }: return Fail { error: "Parse error: ${error}" }
    }
    .validate()
    .match {
        case Fine { value: validated }: validated
        case Fail { error }: return Fail { error: "Validation error: ${error}" }
    }
```

### 错误聚合

```valkyrie
match (result1, result2, result3) {
    case (Fine { value: a }, Fine { value: b }, Fine { value: c }): Fine { value: (a, b, c) }
    case (Fail { error: e }, _, _): Fail { error: e }
    case (_, Fail { error: e }, _): Fail { error: e }
    case (_, _, Fail { error: e }): Fail { error: e }
}
```
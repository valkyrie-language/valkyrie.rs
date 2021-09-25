# 模式匹配 (Match)

Valkyrie 提供了强大的模式匹配功能，支持多种匹配模式和语法形式。

## 基本 Match 语法

### 标准 Match 语句

```valkyrie
# 基本模式匹配
match value {
    with [branches];
    case 1: "one"
    case 2: "two"
    case 3: "three"
    else: "other"
}

# 范围匹配
match score {
    with [grade];
    case 90..=100: "A"
    case 80..=89: "B"
    case 70..=79: "C"
    case 60..=69: "D"
    else: "F"
}

# 多值匹配
match day {
    with [weekend_check];
    case "Saturday" | "Sunday": "Weekend"
    case "Monday"..="Friday": "Weekday"
    else: "Invalid day"
}
```

### 表达式 Match 语法

```valkyrie
# 表达式形式的 match
let result = value
    .match {
        case 1: "one"
        case 2: "two"
        case 3: "three"
        else: "other"
    }

# 链式调用
let processed = input
    .transform()
    .match {
        case Fine(value): value * 2
        case Fail(error): 0
    }
    .to_string()
```

## 解构匹配

### 元组解构

```valkyrie
match point {
    with [coordinate_check];
    case (0, 0): "Origin"
    case (x, 0): f"On X-axis at {x}"
    case (0, y): f"On Y-axis at {y}"
    case (x, y): f"Point at ({x}, {y})"
}

# 嵌套元组
match nested {
    with [nested_pattern];
    case ((a, b), c): f"Nested: {a}, {b}, {c}"
    case (x, (y, z)): f"Other nested: {x}, {y}, {z}"
    else: "No match"
}
```

### 数组解构

```valkyrie
match array {
    with [array_patterns];
    case []: "Empty array"
    case [x]: f"Single element: {x}"
    case [first, second]: f"Two elements: {first}, {second}"
    case [head, ..tail]: "Head: ${head}, Tail length: ${tail.len()}"
    case [.., last]: f"Last element: {last}"
    case [first, .., last]: f"First: {first}, Last: {last}"
}

# 固定长度匹配
match coordinates {
    with [dimension_check];
    case [x, y]: f"2D point: ({x}, {y})"
    case [x, y, z]: f"3D point: ({x}, {y}, {z})"
    else: "Unsupported dimension"
}
```

### 对象解构

```valkyrie
match person {
    with [person_patterns];
    case { name: "Alice", age }: f"Alice is {age} years old"
    case { name, age: 18..=65 }: f"{name} is working age"
    case { name, age, city: "Beijing" }: f"{name} from Beijing, age {age}"
    case { name, ..rest }: "Person ${name} with other fields"
    else: "Unknown person"
}

# 嵌套对象匹配
match user {
    with [user_validation];
    case { profile: { name, email }, active: true }: 
        f"Active user: {name} ({email})"
    case { profile: { name }, active: false }: 
        f"Inactive user: {name}"
    else: "Invalid user"
}
```

## 联合类型匹配

### Result 类型匹配

```valkyrie
match operation_result {
    with [result_handling];
    case Fine(value): "Success: ${value}"
    case Fail(error): "Error: ${error}"
}

# 嵌套 Result 匹配
match nested_result {
    with [nested_result_handling];
    case Fine(Fine(value)): "Double success: ${value}"
    case Fine(Fail(inner_error)): "Inner error: ${inner_error}"
    case Fail(outer_error): "Outer error: ${outer_error}"
}
```

### Option 类型匹配

```valkyrie
match maybe_value {
    with [option_handling];
    case Some(value): f"Found: {value}"
    case None: "Nothing found"
}

# 复杂 Option 匹配
match complex_option {
    with [complex_option_handling];
    case Some({ name, age }) if age >= 18: 
        f"Adult: {name}"
    case Some({ name, age }): 
        f"Minor: {name}"
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
    with [json_processing];
    case Null: "null"
    case Bool(true): "true"
    case Bool(false): "false"
    case Number(n): f"number: {n}"
    case String(s): f"string: {s}"
    case Array(items): f"array with {items.len()} items"
    case Object(map): f"object with {map.len()} keys"
}
```

## 守卫条件

### 基本守卫

```valkyrie
match number {
    with [number_classification];
    case x if x > 0: "positive"
    case x if x < 0: "negative"
    case 0: "zero"
    else: "not a number"
}

# 复杂守卫条件
match person {
    with [person_classification];
    case { age, name } if age >= 65: f"Senior: {name}"
    case { age, name } if age >= 18: f"Adult: {name}"
    case { age, name } if age >= 13: f"Teenager: {name}"
    case { name, .. }: f"Child: {name}"
}
```

### 多重守卫

```valkyrie
match user {
    with [user_access_control];
    case { role: "admin", active: true } if user.permissions.contains("write"): 
        "Full access"
    case { role: "user", active: true } if user.last_login.is_recent(): 
        "User access"
    case { active: false }: "Account disabled"
    else: "No access"
}
```

## 变量绑定

### @ 绑定语法

```valkyrie
match value {
    with [value_binding];
    case x @ 1..=10: f"Small number: {x}"
    case x @ 11..=100: f"Medium number: {x}"
    case large @ 101..: f"Large number: {large}"
    else: "Out of range"
}

# 复杂绑定
match data {
    with [complex_binding];
    case person @ { name, age } if age >= 18: 
        f"Adult person: {person}"
    case child @ { name, age }: 
        f"Child: {child}"
}
```

### 嵌套绑定

```valkyrie
match nested_structure {
    with [nested_binding];
    case { outer: inner @ { value, .. }, .. } if value > 0: 
        f"Positive inner value: {inner}"
    case { outer: inner @ { value, .. }, .. }: 
        f"Non-positive inner value: {inner}"
    else: "No match"
}
```

## 穷尽性检查

### 编译时检查

```valkyrie
# 编译器会检查是否覆盖所有情况
match boolean_value {
    with [boolean_check];
    case true: "yes"
    case false: "no"
    # 不需要 else，因为已经穷尽
}

# 联合类型的穷尽性
match result {
    with [result_exhaustive];
    case Fine(value): handle_success(value)
    case Fail(error): handle_error(error)
    # 编译器确保所有变体都被处理
}
```

### 不可达分支警告

```valkyrie
match number {
    with [unreachable_warning];
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
    with [status_handling];
    case 200: "OK"                    # 最常见
    case 404: "Not Found"            # 次常见
    case 500: "Internal Server Error" # 较少见
    case status: f"Status: {status}"  # 其他情况
}

# 避免复杂的守卫条件
match user {
    with [user_processing];
    case { role: "admin" }: handle_admin(user)
    case { role: "user" }: handle_user(user)
    case other: handle_other(other)
}
```

### 可读性优化

```valkyrie
# 使用有意义的变量名
match request {
    with [request_routing];
    case { method: "GET", path }: handle_get(path)
    case { method: "POST", path, body }: handle_post(path, body)
    case { method: unsupported_method, .. }: 
        error(f"Unsupported method: {unsupported_method}")
}

# 适当使用注释
match complex_data {
    with [data_processing];
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
        case Fine(parsed): parsed
        case Fail(error): return Fail("Parse error: ${error}")
    }
    .validate()
    .match {
        case Fine(validated): validated
        case Fail(error): return Fail("Validation error: ${error}")
    }
```

### 错误聚合

```valkyrie
match (result1, result2, result3) {
    with [multi_result_handling];
    case (Fine(a), Fine(b), Fine(c)): Fine((a, b, c))
    case (Fail(e), _, _): Fail(e)
    case (_, Fail(e), _): Fail(e)
    case (_, _, Fail(e)): Fail(e)
}
```
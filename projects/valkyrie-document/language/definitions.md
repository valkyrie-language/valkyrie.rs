# 定义

Valkyrie 提供了多种定义语法，用于声明变量、函数、类型和其他程序实体。

## 变量定义

### 基本变量定义

```valkyrie
# 不可变变量
let name = "Alice"
let age = 30
let is_active = true

# 可变变量
let mut counter = 0
let mut items = []

# 显式类型注解
let score: i32 = 95
let price: f64 = 29.99
let message: String = "Hello"

# 延迟初始化
let result: i32
if condition {
    result = 42
} else {
    result = 0
}
```

### 解构赋值

```valkyrie
# 数组解构
let [first, second, ..rest] = [1, 2, 3, 4, 5]  # 解构数组到kvs
let [x, _, z] = [10, 20, 30]  # 忽略中间值

# 元组解构
let (name, age, city) = ("Bob", 25, "Beijing")
let (a, ..) = (1, 2, 3, 4)  # 只取第一个

# 对象解构
let { name, age } = person
let { x: pos_x, y: pos_y } = point  # 重命名
let { name, ..other_fields } = user  # 解构dict到kvs
```

## 函数定义 (micro)

### 基本函数定义

```valkyrie
# 无参数函数
micro greet() {
    print("Hello, World!")
}

# 带参数函数
micro add(a: i32, b: i32) -> i32 {
    a + b
}

# 多参数函数
micro calculate(x: f64, y: f64, operation: String) -> f64 {
    if operation == "add" {
        x + y
    } else if operation == "subtract" {
        x - y
    } else if operation == "multiply" {
        x * y
    } else if operation == "divide" {
        x / y
    } else {
        0.0
    }
}

# 无返回值函数
micro log_message(message: String) {
    print("[LOG] ${message}")
}
```

### 函数参数

```valkyrie
# 默认参数
micro create_user(name: String, age: i32 = 18, active: bool = true) -> User {
    User { name, age, active }
}

# 可变参数
micro sum(numbers: ...i32) -> i32 {
    let mut total = 0
    for num in numbers {
        total += num
    }
    total
}

# 命名参数调用
let user = create_user(name: "Alice", active: false)
let result = sum(1, 2, 3, 4, 5)

# 引用参数
micro modify_array(arr: &mut [i32]) {
    for i in 0..<arr.len() {
        arr[i] *= 2
    }
}

# 泛型参数
micro identity<T>(value: T) -> T {
    value
}

micro map<T, U>(items: [T], transform: micro(T) -> U) -> [U] {
    let mut result = []
    for item in items {
        result.push(transform(item))
    }
    result
}
```

### 高阶函数

```valkyrie
# 函数作为参数
micro apply_operation(x: i32, y: i32, op: micro(i32, i32) -> i32) -> i32 {
    op(x, y)
}

# 返回函数
micro make_adder(n: i32) -> micro(i32) -> i32 {
    micro(x: i32) -> i32 {
        x + n
    }
}

# 闭包
let add_five = make_adder(5)
let result = add_five(10)  # 15

# 匿名函数
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map(micro(x) { x * 2 })
let filtered = numbers.filter(micro(x) { x % 2 == 0 })
```

# 匿名函数与闭包

## 匿名函数

匿名函数是没有名称的函数，可以直接在表达式中定义和使用。

### 基本语法

```valkyrie
# 基本匿名函数
let add = micro(x, y) { x + y }

# 单参数匿名函数
let square = micro(x) { x * x }

# 无参数匿名函数
let get_random = micro() { random() }
```

## 闭包

闭包是一种特殊的匿名函数，可以捕获其定义环境中的变量。

### 闭包语法
闭包使用花括号 `{}` 定义，参数使用 `%` 或 `%n`（如 `%1`, `%2`）：

```valkyrie
# 单参数闭包
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map { % * 2 }

# 多参数闭包
let pairs = [(1, 2), (3, 4), (5, 6)]
let sums = pairs.map { %1 + %2 }

# 无参数闭包
let lazy_value = { 42 }
```

### 参数自动注册

闭包中的参数会自动注册：

```valkyrie
# % 等价于 %1，是第一个参数，%2 是第二个参数
let operation = { % + %2 * 2 }

# %x 等价于 %.x 等价于 %1.x，是第一个参数的属性
let user_name = users.map { %name }
```

## 尾随闭包

当函数的最后一个参数是闭包时，可以使用尾随闭包语法，省略括号：

```valkyrie
# 传统调用方式
list.map(micro(x) { x * 2 })

# 尾随闭包语法（完全等价）
list.map { % * 2 }

# 多个参数时，只有最后一个可以使用尾随语法
list.fold(0, micro(acc, item) { acc + item })
# 等价于
list.fold(0) { %1 + %2 }
```

### 复杂示例

```valkyrie
# 链式调用与尾随闭包
let result = numbers
    .filter { % > 0 }
    .map { % * % }
    .fold(0) { %1 + %2 }

# 嵌套闭包
let matrix = [[1, 2], [3, 4], [5, 6]]
let flattened = matrix
    .map { %map { % * 2 } }
    .flatten()
```

## 闭包捕获

闭包可以捕获其定义环境中的变量：

```valkyrie
let multiplier = 10
let numbers = [1, 2, 3, 4, 5]

# 闭包捕获外部变量 multiplier
let scaled = numbers.map { % * multiplier }

# 捕获外部变量
let counter = 0
let increment_counter = {
    counter += 1
    counter
}
```

## 高阶函数示例

```valkyrie
# 自定义高阶函数
micro apply_twice⟨T⟩(value: T, f: micro(T) -> T) -> T {
    f(f(value))
}

# 使用尾随闭包
let result = apply_twice(5) { % * 2 }  # 结果: 20

# 函数组合
micro compose⟨A, B, C⟩(f: micro(B) -> C, g: micro(A) -> B) -> micro(A) -> C {
    { f(g(%)) }
}

let add_one = micro(x) { x + 1 }
let double = micro(x) { x * 2 }
let add_one_then_double = compose(double, add_one)
```

## 最佳实践

1. **简洁性**: 对于简单操作，优先使用闭包而不是命名函数
2. **可读性**: 复杂逻辑应该使用命名函数以提高可读性
3. **尾随闭包**: 当闭包是最后一个参数时，使用尾随语法提高代码美观度
4. **参数访问**: 使用 `%1`, `%2` 等访问参数，或者使用 `%x` 访问第一个参数的属性

```valkyrie
# 好的实践
users.filter { %is_active }
     .map { %name }
     .sort_by { %name.length() }

# 避免过度嵌套
let process_data = micro(data) {
    data.filter { %is_valid }
        .transform { %normalize() }
        .group_by { %category }
}
```
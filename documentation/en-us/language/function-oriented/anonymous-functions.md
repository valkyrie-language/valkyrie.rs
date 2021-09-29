# Anonymous Functions and Closures

## Anonymous Functions

Anonymous functions are functions without names that can be defined and used directly in expressions.

### Basic Syntax

```valkyrie
# Basic anonymous function
let add = micro(x, y) { x + y }

# Single parameter anonymous function
let square = micro(x) { x * x }

# No parameter anonymous function
let get_random = micro() { random() }
```

## Closures

Closures are special anonymous functions that can capture variables from their defining environment.

### Closure Syntax
Closures are defined using curly braces `{}`, with parameters using `$` or `$n` (like `$1`, `$2`):

```valkyrie
# Single parameter closure
let numbers = [1, 2, 3, 4, 5]
let doubled = numbers.map { $ * 2 }

# Multi-parameter closure
let pairs = [(1, 2), (3, 4), (5, 6)]
let sums = pairs.map { $1 + $2 }

# No parameter closure
let lazy_value = { 42 }
```

### Automatic Parameter Registration

Parameters in closures are automatically registered:

```valkyrie
# $ is equivalent to $1, the first parameter; $2 is the second parameter
let operation = { $ + $2 * 2 }

# $x is equivalent to $.x which is equivalent to $1.x, the property of the first parameter
let user_name = users.map { $name }
```

## Trailing Closures

When the last argument of a function is a closure, you can use trailing closure syntax, omitting the parentheses:

```valkyrie
# Traditional call style
list.map(micro(x) { x * 2 })

# Trailing closure syntax (completely equivalent)
list.map { $ * 2 }

# When there are multiple arguments, only the last one can use trailing syntax
list.fold(0, micro(acc, item) { acc + item })
# Equivalent to
list.fold(0) { $1 + $2 }
```

### Complex Examples

```valkyrie
# Chained calls with trailing closures
let result = numbers
    .filter { $ > 0 }
    .map { $ * $ }
    .fold(0) { $1 + $2 }

# Nested closures
let matrix = [[1, 2], [3, 4], [5, 6]]
let flattened = matrix
    .map { $map { $ * 2 } }
    .flatten()
```

## Closure Capture

Closures can capture variables from their defining environment:

```valkyrie
let multiplier = 10
let numbers = [1, 2, 3, 4, 5]

# Closure captures external variable multiplier
let scaled = numbers.map { $ * multiplier }

# Capture external variable
let counter = 0
let increment_counter = {
    counter += 1
    counter
}
```

## Higher-Order Function Examples

```valkyrie
# Custom higher-order function
micro apply_twice⟨T⟩(value: T, f: micro(T) -> T) -> T {
    f(f(value))
}

# Using trailing closure
let result = apply_twice(5) { $ * 2 }  # Result: 20

# Function composition
micro compose⟨A, B, C⟩(f: micro(B) -> C, g: micro(A) -> B) -> micro(A) -> C {
    { f(g($)) }
}

let add_one = micro(x) { x + 1 }
let double = micro(x) { x * 2 }
let add_one_then_double = compose(double, add_one)
```

## Best Practices

1. **Conciseness**: For simple operations, prefer closures over named functions
2. **Readability**: Complex logic should use named functions for better readability
3. **Trailing Closures**: When a closure is the last argument, use trailing syntax for cleaner code
4. **Parameter Access**: Use `$1`, `$2` etc. to access parameters, or `$x` to access properties of the first parameter

```valkyrie
# Good practice
users.filter { $is_active }
     .map { $name }
     .sort_by { $name.length() }

# Avoid over-nesting
let process_data = micro(data) {
    data.filter { $is_valid }
        .transform { $normalize() }
        .group_by { $category }
}
```

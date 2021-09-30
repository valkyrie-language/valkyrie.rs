# Control Flow

Valkyrie provides familiar control flow constructs with some modern enhancements for safety and expressiveness.

## Conditional: if / else

```valkyrie
if score >= 90 {
    grade = "A"
} else if score >= 80 {
    grade = "B"
} else if score >= 70 {
    grade = "C"
} else {
    grade = "F"
}
```

### if as Expression

In Valkyrie, `if` is an expression that returns a value:

```valkyrie
let grade = if score >= 90 { "A" }
             else if score >= 80 { "B" }
             else if score >= 70 { "C" }
             else { "F" }
```

### if let Pattern

```valkyrie
if let Some(value) = maybe_value {
    print("Got: {value}")
} else {
    print("Nothing")
}
```

## Loops

### loop

Infinite loop, must exit with `break` or `return`:

```valkyrie
let mut count = 0
loop {
    count += 1
    if count > 10 {
        break
    }
}
```

### while

```valkyrie
while condition {
    # Do something
}
```

### for in

Iterate over collections:

```valkyrie
for item in collection {
    print(item)
}

for i in 0..10 {
    print(i)
}

for (index, item) in collection.enumerate() {
    print("{index}: {item}")
}
```

### Loop Control

- `break`: Exit the loop immediately
- `continue`: Skip to next iteration
- `break value`: Exit with a value (for loop expressions)

```valkyrie
let result = loop {
    let input = get_input()
    if input == "quit" {
        break "Exited"
    }
    if input == "success" {
        break "Success"
    }
}
```

## Pattern Matching: match

Valkyrie's `match` is a powerful pattern matching expression:

### Basic Matching

```valkyrie
match value {
    case 0 => "zero"
    case 1 => "one"
    case 2 | 3 | 4 => "small"
    case n if n < 10 => "single digit"
    case _ => "other"
}
```

### Destructuring

```valkyrie
match point {
    case Point { x: 0, y: 0 } => "origin"
    case Point { x, y: 0 } => "on x-axis at {x}"
    case Point { x: 0, y } => "on y-axis at {y}"
    case Point { x, y } => "at ({x}, {y})"
}
```

### Enum Matching

```valkyrie
match result {
    case Fine { value } => print("Success: {value}")
    case Fail { error } => print("Error: {error}")
}
```

### Guard Clauses

```valkyrie
match number {
    case n if n < 0 => "negative"
    case 0 => "zero"
    case n if n > 0 => "positive"
}
```

## Error Handling: try / catch

Valkyrie uses structured error handling:

```valkyrie
try {
    let data = read_file(path)?
    let parsed = parse_json(data)?
    process(parsed)
}
.catch {
    case IoError(e):
        print("IO error: {e}")
    case ParseError(e):
        print("Parse error: {e}")
    case _:
        print("Unknown error")
}
```

### The ? Operator

The `?` operator propagates errors:

```valkyrie
micro process_file(path: string) -> Result<Data, Error> {
    let content = read_file(path)?      # Propagates IoError
    let parsed = parse_json(content)?   # Propagates ParseError
    Fine(validate(parsed)?)
}
```

## Exception Handling: raise / handle

For exceptional situations that shouldn't be part of normal control flow:

```valkyrie
micro dangerous_operation() {
    if critical_failure {
        raise CriticalError("System failure")
    }
}

micro caller() {
    handle {
        dangerous_operation()
    }
    .case CriticalError(e) {
        emergency_shutdown()
    }
}
```

## Early Return

```valkyrie
micro find_item(id: i32) -> Option<Item> {
    if id < 0 {
        return None
    }
    
    for item in database {
        if item.id == id {
            return Some(item)
        }
    }
    
    None
}
```

## Best Practices

1. **Prefer expressions**: Use `if` and `match` as expressions when possible
2. **Exhaustive matching**: Always handle all enum variants
3. **Early returns**: Use guard clauses and early returns to reduce nesting
4. **Error propagation**: Use `?` for clean error handling

---
**Related Sections**:
- [Union Types](../type-system/union.md) - Enum types for pattern matching
- [Error Handling](../patterns/index.md) - Error handling patterns

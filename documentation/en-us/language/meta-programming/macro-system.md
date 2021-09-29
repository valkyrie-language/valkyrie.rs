# Macro System

## Overview

Valkyrie provides a powerful macro system that supports compile-time code generation and meta-programming. The macro system is divided into two main parts:

- **Macro (`@`)**: Compile-time function calls, don't capture subsequent arguments
- **Annotation (`@`)**: Compile-time annotations, capture and act on subsequent `class`, `micro` etc. declarations

## Macro vs Annotation

### Macro (`@`)

Macros use the `@` prefix and are compile-time function calls that don't capture subsequent code elements:

```valkyrie
# Compile-time constant computation
let FIBONACCI_10: i32 = evaluate(fibonacci(10))
let LOOKUP_TABLE: [i32; 256] = evaluate(generate_lookup_table())

# Environment variable retrieval
let database_url: string = env("DATABASE_URL")

# String formatting
let message: string = f"Hello, {name}!"

# Vector creation
let numbers = [1, 2, 3, 4, 5]
let zeros = [0; 10]

# SQL query
let query = @sql(
    "SELECT id, name FROM users WHERE active = $1",
    true
)
```

### Annotation (`@`)

Annotations use the `@` prefix and capture and act on subsequent declarations:

```valkyrie
# Test annotation
@test
micro test_addition() {
    @assert_eq(2 + 2, 4)
}

# Serialization annotation
@derive(Serialize, Deserialize)
class User {
    name: string
    email: string
}

# Benchmark annotation
@benchmark
micro fibonacci_benchmark() {
    fibonacci(30)
}

# Conditional compilation annotation
@cfg(feature = "debug")
micro debug_function() {
    print("Debug mode enabled")
}
```

## Common Macros

### Common Macros

```valkyrie
# Compile-time constant computation
let PI_SQUARED: f64 = evaluate(3.14159 * 3.14159)

# Compile-time file reading
let config_content: string = compile_time_read_file("config.toml")

# Compile-time environment configuration
@compile_time_env {
    memory_limit: "256MB",
    execution_timeout: "30s",
}
```


### Code Generation

```valkyrie
# Template definition
@template {
    name: "crud_operations",
    params: [Entity: Type, Key: Type],
    body: {
        micro create(entity: Entity) -> Result⟨Key, Any⟩ {
            # Generic logic for creating entity
        }
        
        micro read(key: Key) -> Result⟨Entity, Any⟩ {
            # Generic logic for reading entity
        }
        
        micro update(key: Key, entity: Entity) -> Result⟨unit, Any⟩ {
            # Generic logic for updating entity
        }
        
        micro delete(key: Key) -> Result⟨unit, Any⟩ {
            # Generic logic for deleting entity
        }
    }
}

# Template instantiation
@generate_code {
    crud_operations⟨User, UserId⟩
    crud_operations⟨Product, ProductId⟩
}
```

### Macro Expansion Control

```valkyrie
# Macro expansion strategy control
@macro_expansion(strategy: "eager", max_depth: 100)
macro recursive_macro {
    # Recursive macro definition
}
```

## Common Annotations

### Test Related

```valkyrie
@test
micro test_user_creation() {
    let user = User("Alice", "alice@example.com")
    @assert_true(user.is_valid())
    @assert_eq(user.name, "Alice")
}

@test
@should_panic
micro test_invalid_email() {
    User("Bob", "invalid-email")
}
```

### Derive Annotations

```valkyrie
@derive(Debug, Clone, PartialEq)
class Point {
    x: f64,
    y: f64,
}

@derive(Serialize, Deserialize)
class Config {
    database_url: string
    port: u16
}
```

### Conditional Compilation

```valkyrie
@cfg(target_os = "windows")
micro windows_specific_function() {
    # Windows-specific implementation
}

@cfg(feature = "async")
class AsyncHandler {
    # Async handler implementation
}
```

## Custom Macros

### Declarative Macros

```valkyrie
macro vec_of {
    (#elem:expr; #n:expr) => {
        {
            let mut v = []
            for _ in 0..#n {
                v.push(#elem)
            }
            v
        }
    }
    (#(#x:expr),+ #(,)?) => {
        @vec(#(#x),+)
    }
}
```

### Procedural Macros

```valkyrie
macro debug_print(args: TokenStream) -> TokenStream {
    if cfg(debug_assertions) {
        @quote {
            print(#args)
        }
    } else {
        @quote {}
    }
}
```

## Best Practices

1. **Clearly Distinguish Purpose**:
   - Use `@` for compile-time computation and code generation
   - Use `@` to add metadata and behavior to declarations

2. **Performance Considerations**:
   - Compile-time computation can improve runtime performance
   - Avoid excessive macro use leading to long compile times

3. **Readability**:
   - Add documentation comments for complex macros
   - Use meaningful macro names

4. **Debugging**:
   - Use `@macro_expansion` to control macro expansion
   - Leverage compiler's macro expansion output for debugging

## Summary

Valkyrie's macro system provides powerful meta-programming capabilities:

- **Macro (`@`)**: Compile-time functions for computation, generation, and transformation
- **Annotation (`@`)**: Declaration annotations for adding metadata and behavior

Correctly using these two mechanisms can greatly improve code expressiveness and performance.

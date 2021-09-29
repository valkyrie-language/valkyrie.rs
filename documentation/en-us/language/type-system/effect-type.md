# Effect Types

In Valkyrie, a function's type includes not only the types of its input and output values but also the **effects** produced during its execution. The effect system allows the compiler to statically track side effects such as IO operations, non-local jumps, and state changes.

## Core Syntax

Effects follow the return type, separated by the `/` symbol:

```valkyrie
# Pure function: produces no effects (default)
micro add(a: i32, b: i32) -> i32 {
    a + b
}

# Function with an IO effect
micro print_hello() -> Unit / IO {
    print("Hello")
}

# Function with multiple effects
micro process() -> i32 / IO + Error {
    # ...
}

## Common Built-in Effects

| Effect | Description |
| :--- | :--- |
| **`IO`** | Performs input/output operations (file, network, console). |
| **`Error`** | May throw exceptions or errors. |
| **`Async`** | Asynchronous execution. |
| **`State`** | Accesses or modifies external global/closure state. |
| **`NonDet`** | Non-deterministic computation. |

---

## Propagation and Elimination of Effects

### 1. Automatic Propagation
If a function calls another function with an effect, that effect automatically propagates to the current function.

```valkyrie
micro outer() -> Unit / IO {
    print_hello() # Produces an IO effect, must be declared in the signature
}
```

### 2. Effect Handlers
You can capture and eliminate effects through effect handlers, converting them into concrete values or another effect.

```valkyrie
micro main() -> Unit {
    # The handle block can eliminate effects
    handle {
        run_app()
    } case IO {
        # Handle IO requests
    }
}
```

---

## Why Effect Types?

1. **Visibility**: You can tell at a glance from the function signature whether it is safe, whether it modifies global state, or whether it makes network requests.
2. **Decoupling**: Logic code only declares the required effects, while the specific implementation (e.g., saving to a file or a database) is decided by higher-level handlers.
3. **Concurrency Safety**: The compiler can prohibit functions with `IO` or blocking effects from executing in certain concurrency contexts (like a render loop).

---

## Advanced Application: Dependency Injection

The effect system provides an extremely elegant way to perform dependency injection: the logic layer declares effects, and the environment layer provides handlers.

### Scenario: Testable Database Operations
```valkyrie
# Declare a custom effect
effect Database {
    micro get_user(id: u64) -> User
}

# Business logic: No need to care how the database is implemented
micro process_user(id: u64) -> Unit / Database + IO {
    let user = Database::get_user(id)
    print("User: {}", user.name)
}

# Production environment handler
micro run_prod() {
    handle {
        process_user(1)
    } case Database::get_user(id) {
        # Actual SQL query
        sql_exec("SELECT * FROM users WHERE id = {}", id)
    }
}

# Test environment handler (Mock)
micro run_test() {
    handle {
        process_user(1)
    } case Database::get_user(id) {
        # Return mock data
        User { id, name: "Mock User" }
    }
}
```

---

**Previous**: [Linear Types](./linear-types.md) | **Next**: [Type System (Index)](./index.md)

- Explore the [Effect System](../effect-system/index.md) to learn how to define custom effects and handlers.
- Learn how [Algebraic Data Types](./algebraic-data-types.md) work together with the effect system.

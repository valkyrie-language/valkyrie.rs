# Row Types and Row Polymorphism

Row types are the core mechanism in Valkyrie for handling structured data (records and variants). They allow us to perform polymorphic operations on objects with specific fields without losing type information.

## What is a Row?

A row is an unordered collection of fields (mappings from names to types). In Valkyrie, record types are essentially rows wrapped under a label.

```valkyrie
# A simple record
let p = { x: 1.0, y: 2.0 }
```

## Row Polymorphism

Row polymorphism allows functions to accept records that contain "at least certain fields." This is achieved through **Row Variables**.

### 1. Open Rows
Use the `, ...R` syntax to represent "the remaining fields." This is consistent with the object spread syntax in pattern matching.

```valkyrie
# Accepts any record that contains at least an x field
micro get_x⟨R⟩(record: { x: f64, ...R }) -> f64 {
    record.x
}

let p2 = { x: 1.0, y: 2.0 }
let p3 = { x: 3.0, y: 4.0, z: 5.0 }

get_x(p2) # Success, R is { y: f64 }
get_x(p3) # Success, R is { y: f64, z: f64 }
```

### 2. Extensions Preserving Type Information
A key advantage of row polymorphism is that it preserves all field information from the original record, which is very useful when a function returns.

```valkyrie
# Add a tag and return all other fields of the original record
micro tag⟨R⟩(record: { ...R }) -> { tagged: bool, ...R } {
    { tagged: true, ...record }
}

let original = { name: "Valkyrie", version: 1 }
let extended = tag(original) 
# extended's type is { tagged: bool, name: utf8, version: i32 }
```

---

## Variant Rows

Row types are also applicable to `unite` (sum types).

```valkyrie
# Variant row polymorphism
micro handle_error⟨R⟩(res: unite { Fail(utf8), ...R }) {
    match res {
        case Fail(msg): print("Error: {}", msg)
        case _: pass # Handle other cases in R
    }
}
```

---

## Extensive Rows

Extensive rows allow us to combine or override existing rows.

### 1. Row Combination
```valkyrie
type Named = { name: utf8 }
type Aged = { age: i32 }

# Combine two rows
type Person = { ...Named, ...Aged }
```

### 2. Row Overriding
When duplicate fields appear in a combination, the last one defined usually takes precedence (the specific behavior depends on the context).

---

## Row Types and Subtyping Relationships

Row types provide a more flexible way of subtyping than traditional inheritance.

- **Depth Subtyping**: `{ x: { a: i32 } }` is a subtype of `{ x: { } }`.
- **Width Subtyping**: `{ x: i32, y: i32 }` is a subtype of `{ x: i32 }`.

In Valkyrie, structural subtyping is essentially an automatic implicit conversion of row types.

---

## Syntax Consistency and Design

Valkyrie has abandoned the traditional `| R` syntax in favor of `, ...R`. This design decision aims to improve the overall consistency of the language:

### 1. Eliminating Ambiguity
The previous `|` symbol shared symbols with Union Types within records, which was prone to parsing ambiguity. By using `...`, row variables are visually isolated from union types:
```valkyrie
# Clear and unambiguous
type UnionField = { x: i32 | utf8 } 
type RowExtension = { x: i32, ...R }
```

### 2. Unification with Value Level
The `...` syntax is completely consistent with Valkyrie's spread syntax in pattern matching and object updates:
- **`..`** is used for spreading or ignoring ordered collections (like arrays).
- **`...`** is used for spreading unordered collections (like objects and row types).

```valkyrie
# Pattern matching (object destructuring)
match user {
    case { name, ...rest }: print(name)
}

# Pattern matching (array destructuring)
match list {
    case [head, ..tail]: print(head)
}

# Object update
let new_user = { name: "New", ...old_user }
```
Using the same symbols in type declarations significantly reduces the cognitive burden on developers.

---

## Advanced Application: Middleware and Context

Using row types, one can implement extensible data structures similar to plugin systems, allowing middleware or modules to add metadata without breaking existing type contracts.

### Scenario: Middleware Context
```valkyrie
# Base context
type BaseContext = {
    request_id: u64,
}

# Authentication middleware: adds user properties while preserving original row R
micro auth_middleware⟨R⟩(ctx: { ...BaseContext, ...R }) -> { user_id: u64, ...BaseContext, ...R } {
    { user_id: 123, ...ctx }
}

# Logging middleware: requires only that at least request_id is included
micro log_request⟨R⟩(ctx: { request_id: u64, ...R }) {
    print("Processing request: {}", ctx.request_id)
}

micro main() {
    let ctx = { request_id: 1 }
    let authed_ctx = auth_middleware(ctx)
    
    log_request(authed_ctx) # Success: contains request_id
    print("User: {}", authed_ctx.user_id)
}
```

---

**Previous**: [Associated Types](./associated-types.md) | **Next**: [Intersections and Unions](./intersection-union.md)

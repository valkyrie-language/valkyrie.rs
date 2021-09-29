# Generic Programming

Generics allow you to write code that can handle multiple types without having to rewrite the logic for each type. Valkyrie's generic system combines static parameterization with powerful constraint mechanisms.

---

## Generic Parameters

Generic parameters are declared using mathematical angle brackets `⟨ ⟩`:

```valkyrie
# Generic structure
structure Box⟨T⟩ {
    item: T
}

# Generic function
micro identity⟨T⟩(value: T) -> T {
    value
}

# Multiple generic parameters
type Pair⟨K, V⟩ = {
    key: K,
    value: V,
}
```

## Default Generic Parameters

You can provide default values for generic parameters.

```valkyrie
structure Map⟨K, V, S = DefaultHasher⟩ {
    # ...
}
```

---

## Generic Constraints

You can constrain the behaviors that generic parameters must possess through Traits.

### 1. Inline Constraints
```valkyrie
micro print_item⟨T: Display⟩(item: T) {
    print(item.fmt())
}
```

### 2. Where Clauses
For complex constraints, it is recommended to use the `where` clause to keep the code clean.
```valkyrie
micro process_data⟨T, U⟩(t: T, u: U) 
where
    T: Display + Clone,
    U: IntoIterator⟨Item = T⟩
{
    # ...
}
```

## Specialization

Valkyrie supports providing more optimized generic implementations for specific types.

```valkyrie
imply⟨T⟩ Box⟨T⟩ {
    micro describe(self) -> utf8 { "A generic box" }
}

# Provide a specialized implementation for Box⟨i32⟩
imply Box⟨i32⟩ {
    micro describe(self) -> utf8 { "A box containing an integer" }
}
```

## Universal vs. Existential Quantification

Valkyrie's type system distinguishes between two main forms of quantification:

- **Universal Quantification (∀)**:
  - Form: `micro func⟨T⟩(item: T)`
  - Meaning: The caller decides what `T` is; the function must be valid for **all** `T` that satisfy the constraints.
- **Existential Quantification (∃)**:
  - Form: `let item: Display = ...` (Trait Object)
  - Meaning: The implementer decides what `T` is; the caller only knows it satisfies the `Display` trait but doesn't know the concrete type.
  - See also: [Witness Tables in the Trait System](../object-oriented/trait-system.md#底层原理见证表-witness-table).

---

## Advanced Application: Phantom Types

Phantom types are a pattern where a generic parameter is used in the definition but not actually used in the structure's fields. They are commonly used to track object states at compile time.

### Scenario: Type-Safe Web Requests
```valkyrie
# Define state markers
structure Unvalidated {}
structure Validated {}

# Request structure includes a phantom type parameter S
structure Request⟨S⟩ {
    url: utf8,
    body: utf8,
}

# Only unvalidated requests can be validated
micro validate(req: Request⟨Unvalidated⟩) -> Request⟨Validated⟩ {
    # Perform validation logic...
    Request⟨Validated⟩ { url: req.url, body: req.body }
}

# Only validated requests can be sent
micro send(req: Request⟨Validated⟩) -> Unit / IO {
    # Send the request...
}

micro main() {
    let req = Request⟨Unvalidated⟩ { url: "...", body: "..." }
    
    # send(req) # Compile error: expected Request⟨Validated⟩, got Request⟨Unvalidated⟩
    
    let valid_req = validate(req)
    send(valid_req) # Compile pass
}
```

---

**Previous**: [Pointers and References](./pointer-type.md) | **Next**: [Associated Types](./associated-types.md)

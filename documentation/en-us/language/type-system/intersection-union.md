# Intersection and Union Types

Valkyrie's type system supports algebraic type composition. Through intersection and union operations, you can build expressive composite types that precisely describe data structures.

## Union Types (`unite`)

Union types represent a value that can be **one of** multiple types. Valkyrie uses explicitly tagged `unite` definitions for named closed variant families. The standard form is `[tag(XXXKind)] unite XXX { }`.

### 1. Named Union (`unite`)
This is the most common form, using explicit tags (Variants) to distinguish different states.

```valkyrie
[tag(ShapeKind)]
unite Shape {
    Circle { radius: f32 },
    Rectangle { width: f32, height: f32 },
    Point
}
```

### 2. Anonymous Unions
In some temporary scenarios, you can use the `|` symbol to combine types:

```valkyrie
# Variable can be i32 or utf8
let data: i32 | utf8 = 42
```

> **Design Note**: Valkyrie strictly distinguishes between **state overlay** and **property extension**. The `|` symbol is now specifically used for union types, representing an "or" relationship; while row extension in records uses the `, ...R` syntax, which is semantically consistent with object spreading in pattern matching. See [Row Types and Polymorphism](./row-types.md#syntax-consistency-and-design).

### 3. Semantic Characteristics
- **Exclusivity**: At any moment, a union type's value can only belong to one of its defined variants.
- **Exhaustiveness Checking**: The compiler enforces handling of all possible branches in `match` expressions.

---

## Intersection Types

Intersection types represent a value that must **simultaneously satisfy** multiple type constraints. Valkyrie uses the `&` symbol to express intersection.

### 1. Structural Intersection
Intersection types are commonly used to require a type to implement multiple traits:

```valkyrie
# Variable must implement both Display and Clone traits
micro process_data(item: Display & Clone) {
    print(item.fmt())
    let _ = item.clone()
}
```

### 2. Semantic Characteristics
- **Capability Overlay**: Intersection types have the sum of all methods and properties from their constituent members.
- **Multiple Constraints**: It's logically equivalent to `T: TraitA + TraitB` in generic constraints, but can be used directly as an independent type.

---

## Physical Layout and Optimization

The Valkyrie compiler performs deep physical optimization on these composite types:

1. **Union Type Compression (Tag Stripping)**:
   - For special unions like `Option⟨ref T⟩`, the compiler uses "niche optimization" to eliminate tags, making its physical size equal to the raw pointer.
   - For `unite` with mutually exclusive fields, the compiler minimizes memory usage through memory overlay techniques.

2. **Intersection Type Flattening**:
   - Intersection types at the底层 are typically handled as a cluster of "fat pointers" pointing to multiple trait vtables, ensuring zero-cost abstraction during polymorphic calls.

## Use Cases

- **Union Types**: State machine modeling, error handling (Result), optional values (Option), polymorphic heterogeneous containers.
- **Intersection Types**: Plugin systems, dependency injection, multi-trait combination constraints, fine-grained permission control.

---

**Previous**: [Row Types and Polymorphism](./row-types.md) | **Next**: [Variance and Subtyping](./polarity-type.md)

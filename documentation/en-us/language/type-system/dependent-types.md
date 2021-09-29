# Dependent Types

Dependent types are among the most powerful features in a type system, allowing **types to depend on values**. In Valkyrie, this means you can directly reference runtime or compile-time values in type signatures, enabling extremely high-precision type constraints.

## Core Manifestations

### 1. Const Generics
This is the most common form of dependent types, where type parameters can be concrete values as well as types.

```valkyrie
# Array length is part of the type
structure Vector⟨T, N: usize⟩ {
    data: array⟨T, N⟩
}

# Two Vectors of different lengths are different types
let v1: Vector⟨f32, 3⟩ = Vector::new([1.0, 2.0, 3.0])
let v2: Vector⟨f32, 4⟩ = Vector::new([1.0, 2.0, 3.0, 4.0])
```

### 2. Refinement Types
Refinement types limit the range of values for an existing type through predicates.

```valkyrie
# Define a positive integer type
type PositiveInt = i32 where { $ > 0 }

# Define a non-empty list
type NonEmptyList⟨T⟩ = [T] where { $.length > 0 }

micro first⟨T⟩(list: NonEmptyList⟨T⟩) -> T {
    list[0] # No need to return an Option here, as the type guarantees the list is not empty
}
```

---

## Dependent Functions

A function's return type can depend on the value of its input parameters.

```valkyrie
# Returns an array of a specific size based on the input length
micro create_array(n: usize) -> array⟨i32, n⟩ {
    @uninitialized()
}

# Here n determines the concrete type of the return value
let arr = create_array(5) # Type is array⟨i32, 5⟩
```

---

## Why Dependent Types?

1. **Eliminate Boundary Checks**: Guarantees through types that indices will never be out of bounds, allowing for the safe removal of bounds checks at runtime.
2. **Formal Verification**: Verifies complex mathematical properties or business logic at compile time (e.g., the transfer amount must be less than the balance).
3. **Precision Modeling**: Describes highly structured data protocols, such as ensuring a network packet's size field matches the subsequent data length.

---

## Limitations and Challenges

While powerful, dependent types also bring the following challenges:
- **Compile-time Burden**: The compiler needs to perform more complex logical reasoning.
- **Undecidability**: Certain complex predicates may lead to the compiler being unable to determine if types match.
- **Syntactic Complexity**: Requires more granular code annotations.

---

**Previous**: [Type-Level Programming](./type-level.md) | **Next**: [Linear Types](./linear-types.md)

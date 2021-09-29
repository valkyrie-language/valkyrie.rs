# Variance and Polarity

In the theory of Algebraic Subtyping, variance is not an isolated set of rules but a direct manifestation of **Type Polarity**. Understanding polarity not only helps us master covariance and contravariance but also reveals the essence of the "polar types" within the type system.

## Type Polarity

Polarity describes the position of a type parameter within a constructed type and the direction in which it affects the subtyping relationship.

1. **Positive Polarity (+)**: Corresponds to **Covariance**. The direction of the subtyping relationship is the same as that of the parameters.
   - If `Sub <: Super`, then `F⟨+Sub⟩ <: F⟨+Super⟩`.
   - **Semantics**: Represents the "output" or "source" of data.

2. **Negative Polarity (-)**: Corresponds to **Contravariance**. The direction of the subtyping relationship is reversed.
   - If `Sub <: Super`, then `F⟨-Super⟩ <: F⟨-Sub⟩`.
   - **Semantics**: Represents the "consumption" or "sink" of data.

3. **Non-polar / Invariant**: Corresponds to **Invariance**.
   - Requires an exact match; there is no subtyping relationship.
   - **Semantics**: Represents a "two-way flow" of data (both reading and writing).

---

## Polar Types

Algebraic subtyping systems have two ultimate "poles" that form the top and bottom of the Type Lattice.

### 1. Top Type (⊤): `any`
- **Polar Position**: The supertype of all types.
- **Semantics**: Represents "any possible value."
- **Variance Behavior**: Provides the least information in a positive polarity position (output) and is most demanding in a negative polarity position (input).

### 2. Bottom Type (⊥): `never`
- **Polar Position**: The subtype of all types.
- **Semantics**: Represents "impossible to happen" or an "empty set."
- **Variance Behavior**: Can be assigned to any type in a positive polarity position (output) because it never actually produces a value, and in a negative polarity position (input), it indicates that the function cannot be called.

---

## Structural Subtyping

For Record types, if `A` contains all fields of `B`, then `A` is a subtype of `B`. This is supported at the low level by the [Row Types and Polymorphism](./row-types.md) mechanism.
```valkyrie
type Point2D = { x: f64, y: f64 }
type Point3D = { x: f64, y: f64, z: f64 }

# Point3D <: Point2D
let p2: Point2D = Point3D { x: 1, y: 2, z: 3 }
```

---

## Type Conversions

Valkyrie distinguishes between implicit conversions based on subtyping relationships and explicit conversions.

### 1. Upcasting
Conversions from a subtype to a supertype (e.g., from `Dog` to `Animal`, or from `i32` to `any`) are typically implicit because they are type-safe.

### 2. Explicit Casting
Use `@cast` or specific methods for explicit conversions, often used in scenarios where there is a risk of information loss (e.g., converting a float to an integer).
```valkyrie
let a: f64 = 1.5
let b: i32 = @cast(a) # Explicit truncation conversion
```

### 3. Raw Pointer Casting
For low-level operations, you can use `@pointer_cast` for unsafe pointer type reinterpretation.

---

## Algebraic Application of Polarity: Function Types

Functions are the most classic stage for polarity reversal. For a function type `micro(P) -> R`:

- **Return type `R` is in a positive polarity position**: It is the output of the function.
- **Parameter type `P` is in a negative polarity position**: It is the consumption of the function.

Therefore, a function `f1` is a subtype of `f2` if and only if:
`f1.Input` is a supertype of `f2.Input` (contravariant) **AND** `f1.Output` is a subtype of `f2.Output` (covariant).

```valkyrie
# Example of polarity annotation
trait Function⟨-In, +Out⟩ {
    micro call(arg: In) -> Out
}
```

---

## Containers and Mutability

### 1. Read-only Containers (Positive Polarity)
In Valkyrie, read-only containers (like `[T]`) are treated as producers of T; therefore, T is in a positive polarity position and exhibits covariance.

### 2. Mutable Containers (Invariance)
When a generic parameter `T` appears in both an input position (`push(T)`) and an output position (`get() -> T`), its polarities cancel each other out, resulting in **Invariance**.

---

## Variance Annotations

When defining generic types, you can use polarity symbols to explicitly declare variance:

- **`+T`**: Explicitly declared as covariant (positive polarity).
- **`-T`**: Explicitly declared as contravariant (negative polarity).

```valkyrie
# Producer (Positive Polarity)
trait Producer⟨+T⟩ {
    micro produce() -> T
}

# Consumer (Negative Polarity)
trait Consumer⟨-T⟩ {
    micro consume(item: T)
}
```

---

## Summary

- **Covariance (+)** is upward-moving, productive polarity.
- **Contravariance (-)** is downward-moving, consumptive polarity.
- **Polar types (`any`/`never`)** define the boundaries of subtyping relationships.

---

**Previous**: [Intersections and Unions](./intersection-union.md) | **Next**: [Type Functions](./type-function.md)

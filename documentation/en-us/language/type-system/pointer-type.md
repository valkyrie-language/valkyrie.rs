# Pointers and References

Valkyrie is a memory-safe language, but in low-level development, high-performance computing, and Foreign Function Interface (FFI) scenarios, direct manipulation of memory addresses is essential. Valkyrie provides a unified symbolic system for handling raw pointers and safe references.

To match Valkyrie's symbolic aesthetics, we use **circles** to represent safety and **diamonds** to represent the underlying system:
- **Circle**: Safe References, managed automatically by the compiler's lifecycle management.
- **Diamond**: Raw Pointers, representing low-level, dangerous operations.

---

## References and Modifiers

Valkyrie introduces a modifier mechanism to simplify common reference operations, keeping the code safe while making it more concise.

### Parameter Modifiers

Valkyrie's function parameters are passed by **read-only reference (ref)** by default, which greatly reduces syntactic noise:

| Syntax | Description |
| :--- | :--- |
| `data: [f64]` | **Default behavior**. Passed as a read-only reference. |
| `mut data: [f64]` | Passed as a mutable reference. |
| `own data: [f64]` | Transfer ownership (Move). |

### Type Symbols

While parameters are references by default, symbols are still needed to explicitly indicate permissions when defining complex data structures (such as structure fields or nested containers):

- **Circle**: Explicit reference types, such as `mut T` (mutable field).
- **Diamond**: Raw pointer types, such as `◇T`, `◆T`.

## Dereferencing

Valkyrie further simplifies the use of references by automatically handling dereferencing based on the type:

### Automatic Dereferencing

For `class` types (reference types), accessing fields or performing assignments will **automatically dereference**, requiring no explicit symbol:

```valkyrie
class Variable { value: f64 }

micro update(mut var: Variable) {
    var = Variable(10.0)  # Automatically dereferences and updates the pointed object
}
```

### Suffix Dereference

For `structure` (value type) or raw pointers, when you need to explicitly emphasize dereferencing behavior, use symmetric suffix symbols:

| Permission | Symbol | Example |
| :--- | :--- | :--- |
| **Read-only** | `.◇` | `ptr.◇` (Read-only read) |
| **Mutable** | `.◆` | `ptr.◆ = val` (Mutable write) |

> **Note**: Dereferencing a raw pointer must be performed within an `unsafe` block.

---

---

## Deep Comparison: Null vs. Option

In Valkyrie, while both `Option` and `null` are used to handle "missing" cases, they have fundamental differences in semantic depth and physical manifestation.

### 1. Physical Essence vs. Logical Container
- **`null` (Null Pointer)**: A **physical concept**. It represents the memory address `0` and is specifically used for raw pointers (`◇T`, `◆T`). In an `unsafe` environment, `null` is a legal (though dangerous) pointer value.
- **`Option⟨T⟩` (Optional Value)**: A **logical concept**. It is an Algebraic Data Type (ADT) used to wrap any type `T` in safe code.

### 2. Nestability: Flat vs. Hierarchical
This is the most significant difference between the two:

- **`Option` is nestable**:
  `Option⟨Option⟨i32⟩⟩` has a clear logical hierarchy.
  - `Some(Some(1))`: Value exists inside.
  - `Some(None)`: The outer container exists, but the inner value is missing.
  - `None`: The outermost container does not exist.
  
- **`null` is not nestable**:
  The null value of a raw pointer is **flat**. Since `null` is merely a physical address `0`, you cannot distinguish between a "pointer to null" and the "null pointer itself."
  - For `◇(◇T)` (a pointer to a pointer), if the innermost address is `0`, it is a `null`.
  - There is no physical representation like `Some(null)` that preserves the hierarchical structure.

### 3. Memory Layout Optimization (Niche Optimization)
To balance performance and safety, the Valkyrie compiler performs physical "null pointer optimization" on `Option`:
- For safe references `ref T` or `class` objects, the physical size of `Option⟨ref T⟩` is exactly the same as `ref T`. The compiler exploits the fact that valid references are never `0`, mapping `None` directly to the physical `null`.
- This optimization allows you to enjoy the logical rigor of `Option` (nestable, type-safe) while achieving physical efficiency consistent with C-style raw pointers.

## Migration Guide

### Syntax Comparison Table

| Feature | C | Rust | Valkyrie (Modifier/Symbol) |
| :--- | :--- | :--- | :--- |
| **Read-only Reference** | `const T&` | `&T` | `ref T` |
| **Mutable Reference** | `T&` | `&mut T` | `mut T` |
| **Read-only Pointer** | `const T*` | `*const T` | `◇T` |
| **Mutable Pointer** | `T*` | `*mut T` | `◆T` |
| **Dereference** | `*ptr` | `*ptr` | `ptr.◇` / `ptr.◆` |

### Core Differences

1. **Suffix Chain Calling**: `ptr.◇.field` is more in line with modern chain programming habits than `(*ptr).field`.
2. **Permission Symmetry**: When you see `p.◆ = val`, you know not only that it is dereferencing, but also that it is an operation with "solid/mutable" permissions.
3. **Visual Safety Zone**:
   - Modifiers `mut` / `ref`: Indicate safety; the compiler checks borrowing rules.
   - Diamonds `◇` / `◆`: Indicate danger; must appear in an `unsafe` environment.

### Optional Mutable Pointer

```valkyrie
# Represents a potentially non-existent but, if present, modifiable block of memory
let buffer: ◆u8? = None
}

---

**Previous**: [Algebraic Data Types (ADT)](./algebraic-data-types.md) | **Next**: [Generic Programming](./generics.md)

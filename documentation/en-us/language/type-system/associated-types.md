# Associated Types

Associated types are an advanced abstraction mechanism in the Valkyrie type system that allow you to declare a placeholder type within a `trait` definition. The specific choice of this type is decided by the implementer of the `trait`.

## Core Concepts

Associated types bind a "type placeholder" to a `trait`. Compared to generic parameters, associated types emphasize a **functional mapping relationship**: for every concrete type that implements the `trait`, there is a uniquely determined output for the associated type.

### Basic Syntax

Use the `type` keyword within a `trait` to declare an associated type:

```valkyrie
trait Iterator {
    type Item
    
    # The next method returns this associated type
    micro next(mut self) -> Option⟨Self::Item⟩
}
```

Specify the concrete type when implementing the `trait`:

```valkyrie
imply [i32]: Iterator {
    type Item = i32
    
    micro next(mut self) -> Option⟨i32⟩ {
        # ... implementation details
    }
}
```

---

## Associated Types vs. Generic Parameters

Choosing between associated types and generic parameters is a key decision when designing abstract interfaces.

### 1. Uniqueness Constraint (One Implementation per Type)
- **Associated Types**: A type can only provide **one** implementation for a given `trait`. For example, `[i32]` can only have one `Iterator` implementation, and its `Item` must be `i32`.
- **Generic Parameters**: A type can provide **multiple** implementations for the same `trait`. For example, a `Data` type can simultaneously implement `Convert⟨i32⟩` and `Convert⟨utf8⟩`.

### 2. Syntactic Conciseness
Using associated types can significantly reduce the generic boilerplate in function signatures.

**Using generic parameters (verbose)**:
```valkyrie
micro process⟨I, T⟩(iter: I) where I: Iterator⟨T⟩ { ... }
```

**Using associated types (concise)**:
```valkyrie
micro process⟨I: Iterator⟩(iter: I) {
    # Access the associated type via double colons
    let first: I::Item = iter.next()?
}
```

## Advanced Usage

### 1. Associated Types with Constraints
You can add trait bounds to the associated type itself:

```valkyrie
trait Container {
    type Element: Display + Clone
    
    micro get(self, index: usize) -> Self::Element
}
```

### 2. Default Values for Associated Types
You can provide a default type when defining the `trait`:

```valkyrie
trait Logger {
    type Output = utf8
    micro log(self, msg: utf8) -> Self::Output
}
```

### 3. GATs (Generic Associated Types)
Valkyrie supports Generic Associated Types, allowing associated types themselves to carry generic parameters, expressing more complex type mapping relationships:

```valkyrie
trait Iterable {
    # The associated type itself is generic
    type Collection⟨T⟩
    
    # Use GATs to define transformation operations
    micro map⟨T, U⟩(self: Self:::Collection⟨T⟩, f: micro(T) -> U) -> Self:::Collection⟨U⟩
}
```

---

**Previous**: [Generic Programming](./generics.md) | **Next**: [Row Types and Polymorphism](./row-types.md)

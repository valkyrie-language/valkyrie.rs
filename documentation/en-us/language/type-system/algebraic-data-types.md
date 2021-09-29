# Algebraic Data Types (ADT)

Valkyrie's type system is built on the solid foundation of Algebraic Data Types (ADT). By combining **Product Types** and **Sum Types**, you can build domain models that precisely map to your business logic.

---

## Product Types

Product types are so named because the space of possible values for the type is the **Cartesian product** of the value spaces of all its members. In Valkyrie, product types manifest as combinations of data.

### 1. Structures and Classes (Structure & Class)
The most common product type, combining multiple named fields.

```valkyrie
structure Point {
    x: f64,
    y: f64,
}
```
The state space of `Point` = space of `f64` × space of `f64`.

### 2. Tuples
Anonymous, ordered product types.

```valkyrie
let color: (u8, u8, u8) = (255, 0, 0)
```

---

## Sum Types

Sum types are so named because the space of possible values for the type is the **logical sum** of the value spaces of all its branches. In Valkyrie, sum types manifest as mutually exclusive choices of state.

### 1. Union Types (`unite`)
Valkyrie uses explicitly tagged `unite` definitions for named sum types. The standard form is `[tag(XXXKind)] unite XXX { }`, and the tag type is no longer generated automatically.

```valkyrie
[tag(LoadingStateKind)]
unite LoadingState {
    Idle,
    Loading(f32),      # Carries progress data
    Success(utf8),     # Carries success result
    Failure(ErrorCode) # Carries error code
}
```
The state space of `LoadingState` = `Idle` + `f32` + `utf8` + `ErrorCode`.

### 2. Enums
When all branches of a sum type carry no additional data, it degenerates into a traditional enum.

```valkyrie
[tag(DirectionKind)]
unite Direction { North, South, East, West }
```

---

## Pattern Matching: The Natural Partner of ADTs

The power of sum types lies in the compiler's ability to ensure you have handled every possible case.

```valkyrie
micro process(state: LoadingState) {
    match state {
        case Idle: print("Waiting...")
        case Loading(p): print("Loading: {}%", p * 100)
        case Success(data): print("Success: {}", data)
        case Failure(e): print("Error: {}", e)
    }
}
```

---

## Recursive Algebraic Data Types

ADTs can be recursive, making them ideal for describing tree-like or chain-like structures:

```valkyrie
[tag(ListKind)]
unite List⟨T⟩ {
    Empty,
    Node(T, List⟨T⟩)
}

[tag(JSONKind)]
unite JSON {
    Null,
    Bool(bool),
    Number(f64),
    String(utf8),
    Array([JSON]),
    Object({ utf8: JSON })
}
```

---

## Physical Layout Optimization

The Valkyrie compiler performs extreme optimizations on ADTs:
- **Non-zero Optimization**: `Option⟨ref T⟩` takes no extra space.
- **Tag Compression**: For `unite` with few branches, the tag usually occupies only 1 byte or even less.
- **Field Overlay**: Data from different branches share the same space in physical memory.

---

## Generalized Algebraic Data Types (GADT)

Generalized Algebraic Data Types (GADT) allow you to explicitly specify the concrete type constructed by a branch when defining a `unite`. This breaks the restriction in traditional ADTs that "all branches must have the same type parameters."

### The Pain Point: Loss of Type Information
In a normal ADT, even if you construct a `Literal(1.0)`, its type is just a broad `Expr⟨T⟩`. When writing an interpreter, you have to use pattern matching or type casting again to determine what `T` actually is.

### Valkyrie's Solution: Constructor Signatures
Valkyrie allows specifying a return type for each branch, thereby locking in the type parameter at construction.

```valkyrie
[tag(ExprKind)]
unite Expr⟨T⟩ {
    # Explicitly specify return type, locking T as f64
    Literal(f64): Expr⟨f64⟩
    
    # Lock T as bool
    IsZero(Expr⟨f64⟩): Expr⟨bool⟩
    
    # Recursive definition: result type T is determined by sub-expressions
    If(Expr⟨bool⟩, Expr⟨T⟩, Expr⟨T⟩): Expr⟨T⟩
}

# Static type checking:
let ok: Expr⟨f64⟩ = If(IsZero(Literal(0.0)), Literal(1.0), Literal(2.0))

# Interpreter logic:
# Since the type is locked at construction, we don't need extra type casting
micro eval⟨T⟩(expr: Expr⟨T⟩) -> T {
    match expr {
        case Literal(v): v
        case IsZero(e):  eval(e) == 0.0
        case If(c, t, e): if eval(c) { eval(t) } else { eval(e) }
    }
}
```

---

## Advanced Application: Final Tagless Paradigm

Compared to traditional recursive ADTs, Final Tagless is a more advanced abstraction pattern. It defines the semantics of a DSL through Traits, achieving highly extensible operational logic without the need for intermediate data structures.

```valkyrie
# Define DSL semantic interface
trait Expr⟨F⟩ {
    micro literal(val: f64) -> F
    micro add(left: F, right: F) -> F
    micro mul(left: F, right: F) -> F
}

# Implementation 1: Direct evaluation interpreter
imply Evaluator: Expr⟨f64⟩ {
    micro literal(val) -> f64 { val }
    micro add(l, r) -> f64 { l + r }
    micro mul(l, r) -> f64 { l * r }
}

# Implementation 2: Formatted printing interpreter
imply Printer: Expr⟨utf8⟩ {
    micro literal(val) -> utf8 { val.to_utf8() }
    micro add(l, r) -> utf8 { "({} + {})".format(l, r) }
    micro mul(l, r) -> utf8 { "({} * {})".format(l, r) }
}

# Use generic functions to write business logic
micro program⟨F, E: Expr⟨F⟩⟩(e: E) -> F {
    e.add(e.literal(1.0), e.mul(e.literal(2.0), e.literal(3.0)))
}
```

**Advantages**:
- **Extensibility**: New interpreters can be added at any time without modifying the original DSL definition.
- **Performance**: No memory allocation overhead for intermediate ADTs; logic expands directly into operations of the target type.
- **Type Safety**: Ensures all operations comply with defined semantics at compile time.

---

## Advanced Application: Newtype

By creating a wrapper for an existing type, you can improve code safety without increasing runtime overhead, preventing the confusion of data that is logically different but of the same type.

```valkyrie
structure UserId(u64)
structure OrderId(u64)

micro fetch_order(user: UserId, order: OrderId) {
    # Business logic...
}

micro main() {
    let u = UserId(1)
    let o = OrderId(100)
    
    # fetch_order(o, u) # Compile error: type mismatch
    fetch_order(u, o)   # Compile pass
}

---

**Previous**: [Type System (Index)](./index.md) | **Next**: [Pointers and References](./pointer-type.md)

- Learn how to use anonymous sum types and intersection types in [Intersection and Union Types](intersection-union.md).
- Explore how [Associated Types](associated-types.md) provide flexible type mapping for ADTs.


# Type-Level Programming

Valkyrie's type system is not just a static checking tool, but also a compile-time computation engine. Type-level programming allows you to perform logical reasoning, data transformation, and protocol verification at the compilation stage.

## Saying Goodbye to "Type Gymnastics"

In languages like TypeScript, type-level programming often involves complex recursive conditional types, jokingly referred to as "type gymnastics." Valkyrie's design philosophy is: **Type-level programming should not be gymnastics; it should be normal programming.**

### Pain Point Analysis: Why does TypeScript need "writing twice"?

In TypeScript, the type system and the runtime expression system are **completely isolated**:
- **Runtime system**: The interpreter/JIT runs JavaScript.
- **Type system**: The compiler runs a Domain-Specific Language (DSL) based on "structural pattern matching" and "recursive ternary operators."

This leads to logic duplication. If you write an `isEmpty(list)` function for runtime judgment, and you also want to constrain `NonEmptyList<T>` at the type level, you must implement it again using two completely different syntaxes.

### Valkyrie's Solution: Unified Logic Model

Valkyrie eliminates this duplication in two ways:

#### 1. Syntactic Unity
Whether in `micro` (runtime) or `mezzo` (compile-time), you use the same `match`, `if`, `map`, and `filter`. This means the logic is **one set** in your mental model, only the timing of execution differs.

#### 2. Cross-level Reuse
Through `@const_fn` and `@evaluate`, the same logic can travel directly between the two worlds.

```valkyrie
# Logic definition: write once
@const_fn
micro validate_age(age: i32) -> bool {
    age >= 0 && age <= 150
}

# Runtime: call directly
let ok = validate_age(25)

# Compile-time: used as type constraint or constant
let VALID_DEFAULT: i32 = evaluate(if validate_age(20) { 20 } else { 0 })

# Type-level: reuse in mezzo
mezzo ValidAge⟨A: i32⟩ -> type {
    if evaluate(validate_age(A)) { A } else { never }
}
```

## Core Foundations

### 1. Literal Types
In Valkyrie, literals (such as `42`, `"hello"`, `true`) can exist as independent types. This is known as Singleton Types.

```valkyrie
# The type of x is not just i32, but also the literal type 42
let x: 42 = 42

# Error: type mismatch
# let y: 42 = 43
```

### 2. Type Functions (Mezzo Functions)
Functions defined using `mezzo` run at compile time, accepting and returning types or constants.

```valkyrie
mezzo Add⟨N: int, M: int⟩ -> int {
    N + M
}

# Using type-level computation
let buffer: array⟨u8, Add⟨10, 20⟩⟩ = uninitialized()
```

## Type-Level Lists and Tuples

You can operate on type lists just like runtime lists.

```valkyrie
mezzo Head⟨List⟩ {
    match List {
        case (H, ..): H
        case _: never
    }
}

# Head⟨(i32, utf8, bool)⟩ will evaluate to i32
type First = Head⟨(i32, utf8, bool)⟩
```

## Use Cases

### 1. Dimensional Analysis
Ensuring physical units (like meters, seconds) remain consistent in calculations.

```valkyrie
type Quantity⟨Value, Unit⟩ = {
    value: Value
}

type Meter = { length: 1 }
type Second = { time: 1 }

# Define unit multiplication
mezzo MulUnit⟨U1, U2⟩ {
    { 
        length: U1::length + U2::length,
        time: U1::time + U2::time 
    }
}

micro multiply⟨V, U1, U2⟩(a: Quantity⟨V, U1⟩, b: Quantity⟨V, U2⟩) -> Quantity⟨V, MulUnit⟨U1, U2⟩⟩ {
    Quantity { value: a.value * b.value }
}
```

### 2. Static Assertions and Proofs
Leveraging the type system to prove properties of the code.

```valkyrie
trait IsTrue {}
imply true: IsTrue {}

# If condition is not true, compilation will fail
micro static_assert⟨condition: bool⟩() where condition: IsTrue {}

# Example
static_assert⟨Add⟨2, 2⟩ == 4⟩() # Compilation succeeds
# static_assert⟨2 + 2 == 5⟩()   # Compilation fails
```

---

## Advanced Application: Peano Arithmetic

Simulating natural numbers and their operations at the type level is the foundation for understanding recursive type proofs.

```valkyrie
# Define natural number structure
unite Nat {
    Zero,
    Succ(Nat),
}

# Type-level addition
mezzo Add⟨N: Nat, M: Nat⟩ -> Nat {
    match N {
        case Zero: M
        case Succ(N1): Succ(Add⟨N1, M⟩)
    }
}

# Proof: 1 + 1 = 2
type One = Succ(Zero)
type Two = Succ(Succ(Zero))
static_assert⟨Add⟨One, One⟩ == Two⟩()
```

---

**Previous Page**: [Higher-Kinded Types (HKT)](./higher-kinded-types.md) | **Next Page**: [Dependent Types](./dependent-types.md)

# Type Functions

Type functions are a powerful feature in the Valkyrie language used for computation at the type level. Defined using the `mezzo` keyword, type functions allow for the manipulation and transformation of types during compilation.

## Root Cause Analysis: Why Does TypeScript Require "Writing Twice"?

The root cause of why TypeScript requires writing things twice lies in **The Type-Value Gap**:

1. **Language Disunity**: TypeScript actually consists of two completely different languages: JavaScript at runtime and a Type DSL at compile time. They have different syntaxes, different control flows (JS uses `if/match`, while types use nested ternaries), and different execution engines.
2. **Types Are Not Values**: In TS, types are erased at runtime; they are not "first-class citizens." You cannot pass a type like you pass a string, nor can you use the same function to handle both `1` and `number`.

### Valkyrie's Solution: Unified Evaluation Engine

Valkyrie's secret to eliminating repetition is: **Types are Values**.

In Valkyrie, the compiler does not distinguish between a "type-level language" and a "value-level language." It has only one unified syntax and evaluation engine. `mezzo` functions and `@const_fn` simply run the same logic at different stages (compile-time vs. runtime).

#### Example: True Logic Reuse (DRY)

Instead of using ambiguous constructs like `any`, we leverage Valkyrie's unite to let a single logic directly describe "transformation rules."

```valkyrie
# Logic definition: Describes a "square or length" transformation rule
# This logic can act on both concrete values and the types themselves
# Because in Valkyrie, a type is also a kind of value
@const_fn
micro transform⟨T⟩(input: T) {
    match input {
        # When input is a concrete value
        case i: i32: i * i
        case s: utf8: s.count()
        
        # When input is the type itself
        # This eliminates the need to "write twice": logic branches are defined in one place
        case i32: i32
        case utf8: i32
        
        case _: @error("Unsupported")
    }
}

# 1. Runtime: Process data
let r1 = transform(10)      # 100

# 2. Compile-time: Reuse transform directly in type signatures
# Note: Here we write the function call directly in the return type position
micro process_data⟨T: i32 | utf8⟩(val: T) -> transform(T) {
    transform(val)
}

# 3. Static verification
type R = transform(i32) # R is i32
```

**Why is this true DRY?**
- **Unified Syntax**: You don't need to learn how to write `If<T>` using nested ternary operators.
- **Single Logic**: If you decide that `utf8` should return `bool` instead of a length, you only need to modify the code in one place: `transform`.
- **Stage Transparency**: The Valkyrie compiler automatically decides whether this logic runs at compile-time (to determine the type) or at runtime (to process data).

---

## Core Mechanism: Mezzo vs. @const_fn

```valkyrie
mezzo FunctionName(param: Type) -> ReturnType {
    # Type function body
}
```

## Logic Branches and Matching

Type functions can use `if` and `match` for logical branching.

### 1. Conditional Selection (If-Else)
```valkyrie
mezzo ConditionalType⟨T, U⟩(condition: bool) -> Type {
    if condition { T } else { U }
}
```

### 2. Type Pattern Matching (Match)
```valkyrie
mezzo MapType(input: Type) -> Type {
    match input {
        case i32: i64
        case f32: f64
        case _: input
    }
}
```

---

## Recursion and Characteristics

Type functions support recursive definitions, making it possible to process nested structures like tuple lists.

### 1. Recursive Type Functions
```valkyrie
mezzo Flatten⟨T⟩(input: T) -> Type {
    match input {
        case (Head, Tail): Flatten⟨Tail⟩
        case _: input
    }
}
```

### 2. Core Characteristics
- **Compile-time Execution**: Type functions are fully expanded during the compilation phase and produce no runtime overhead.
- **Purity**: Type functions must be pure functions and cannot produce any side effects.
- **Determinism**: The same input must produce the same output type.
- **Recursion Depth**: The compiler limits the recursion depth to prevent infinite loops at compile-time.

---

## Advanced Application: Type Mapping

Using type functions, one can implement complex type transformations, which is useful when processing heterogeneous lists or automatically generating API bindings.

### Scenario: Automatic Result Wrapping
```valkyrie
mezzo ToResult(T: Type) -> Type {
    unite { Ok(T), Err(utf8) }
}

# Usage Example
type SafeInt = ToResult(i32) 
# Equivalent to unite { Ok(i32), Err(utf8) }
```

---

## Use Cases

1. **Type Verification**: Verifying that a type satisfies specific conditions at compile-time.
2. **Type Transformation**: Automatically deriving and converting related types.
3. **Generic Constraints**: Adding complex type constraints to generic parameters.
4. **Metaprogramming**: Implementing advanced compile-time code generation.

---

**Previous**: [Variance and Subtyping](./polarity-type.md) | **Next**: [Higher-Kinded Types (HKT)](./higher-kinded-types.md)

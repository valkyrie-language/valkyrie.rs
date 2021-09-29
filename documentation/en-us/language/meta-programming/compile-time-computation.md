# Compile-time Computation

Valkyrie allows code execution during compilation, achieving zero-cost abstraction and highly flexible code generation.

## Constant Expressions (evaluate)

Use the `evaluate` compiler built-in directive to force the compiler to compute expression values at compile time.

```valkyrie
# Compile-time Fibonacci sequence calculation
let FIB_20: i32 = evaluate(fibonacci(20))

# Compile-time string formatting
let VERSION: string = evaluate(f"v{1}.{0}.{5}")
```

## Compile-time Functions (@const_fn)

Only functions marked with `@const_fn` can be safely executed at compile time. These functions must be pure functions (no side effects).

```valkyrie
@const_fn
micro square(n: i32) -> i32 {
    n * n
}

# Valid call
let X: i32 = evaluate(square(10))
```

## Compile-time Reflection

The compiler provides a series of built-in directives to get type information or environment information:

- `type_of(expr)`: Get the type of an expression.
- `name_of(sym)`: Get the name string of a symbol.
- `env("VAR_NAME")`: Read environment variables from the compile environment.
- `is_defined(sym)`: Check if a symbol is defined.

## External Resource Embedding

You can read external files at compile time and embed their content into the generated binary:

```valkyrie
# Embed text file
let SHADER_SOURCE: string = evaluate(read_file("src/shaders/basic.glsl"))

# Embed binary file
let ICON_DATA: [u8] = evaluate(read_bytes("assets/icon.png"))
```

## Why Use Compile-time Computation?

1. **Performance**: Shift runtime overhead to compile time.
2. **Validation**: Catch invalid configurations or parameters at compile stage.
3. **Flexibility**: Generate different code paths based on environment parameters (like dev/prod mode).

---
**Related Sections**:
- [Macro System](./macro-system.md) - More advanced code generation tools
- [Type Functions](../type-system/type-function.md) - Type-level compile-time computation

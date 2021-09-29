# CLR (.NET) Backend Evaluation and Proposal

CLR (Common Language Runtime) is the runtime environment for the .NET platform. Similar to JVM, CLR uses a stack-based instruction set (CIL - Common Intermediate Language).

## 1. Compilation Pipeline

Since CLR is also a stack-based virtual machine, its compilation pipeline is recommended to follow the JVM backend implementation, skipping the register allocation stage.

- **Recommended Path**: `Source -> AST -> HIR -> CFG -> CIL (Common Intermediate Language)`
- **Core Logic**:
    - **CFG Linearization**: Arrange CFG blocks in sequential order.
    - **Stack Instruction Mapping**: Directly generate `ldloc`, `stloc`, `add`, `call`, and other CIL instructions from CFG expressions.
    - **Metadata Generation**: Use `System.Reflection.Emit` style libraries (or directly manipulate PE format) to generate metadata for assemblies, classes, and methods.

## 2. Technology Selection

### Option A: Static Assembly Generation (AOT-like)
Directly generate PE format binary files (.dll or .exe) compliant with ECMA-335 standard.
- **Advantages**: No compiler required at runtime, good performance.
- **Tools**: 
    - [dnlib](https://github.com/0xd4d/dnlib) (C# library, may require FFI)
    - Rust alternatives to [Kestrel](https://github.com/jbevain/cecil) (Cecil) or directly generate binary streams.

### Option B: Dynamic Generation (JIT-like)
Use reflection to emit instructions at runtime.
- **Advantages**: Simple implementation, suitable for scripting scenarios.
- **Disadvantages**: Depends on .NET runtime.

## 3. Differences from JVM

1. **Value Types (Structure)**: CLR natively supports custom value types (ValueType), which is more powerful than JVM's current implementation (Project Valhalla is still in progress). Valkyrie's `structure` can be directly mapped to CLR's `valuetype`.
2. **Generics**: CLR generics are specialized, with type information preserved at runtime. This allows Valkyrie to implement more efficient generic code.
3. **Tail Call**: CIL explicitly supports the `tail.` prefix, which is very suitable for functional programming language optimization.

## 4. Valkyrie Feature Handling

### Traits and Interfaces
- **Mapping Scheme**: Valkyrie's `trait` can be perfectly mapped to CLR's `interface`.
- **Default Implementation**: CLR now supports default method implementations for interfaces, which aligns with Valkyrie's trait default implementations.

### Algebraic Effects
- **Core Logic**: Valkyrie's `raise` and `yield` are governed by the AE mechanism.
- **Exception Mapping**: In non-`resume` scenarios, `raise` is equivalent to CLR's `throw` instruction. Need to generate corresponding `Exception` subclasses for different effects.
- **Continuation Support**: Since CLR does not support delimited continuations, for `resume` scenarios, functions need to be rewritten as state machines (similar to C#'s `async` or `yield return`), lifting local variables to class fields and managing execution flow resumption through state codes.

### FFI and External Imports (`@import`)
- **CLR-specific Marker**: Import markers targeting `target: clr` are directly mapped to .NET assembly metadata.
- **Calling Convention**: Use the `call` instruction to invoke fully qualified method names (e.g., `[mscorlib]System.Console::WriteLine`).
- **Independence**: The CLR backend's FFI path is completely independent from WASM/WASI, without needing to consider WASM target calling conventions or marshalling shims.

### Memory Management
- **Managed Heap**: Directly utilize CLR's efficient generational GC.
- **Finalizers**: Valkyrie's finalization logic can be mapped to the `IDisposable` pattern.

## 5. Implementation Progress and Plan

### Current Status
Text-based CIL (Common Intermediate Language) instruction emission has been implemented, with integrated `ilasm` automated compilation process.
- [x] Basic type mapping (`int32`, `int64`, `bool`, `utf8`, etc.)
- [x] Structure (`structure`) mapping to `valuetype`
- [x] Namespace support (Namespace mapping)
- [x] Algebraic Data Type (ADT) basic mapping
- [x] Tail Call Optimization
- [x] Complex projection path loading and storing (`ldfld`, `stfld`, `ldelema`, etc.)
- [x] `ilasm` automation integration and EXE generation
- [x] `main` function as program entry point (`.entrypoint`)
- [x] ADT variant constructor calls (Parameterized constructor for ADT)
- [ ] Full algebraic effect support (Full AE support with state machine)
- [ ] CLR-specific FFI imports (Target-specific FFI for CLR)

### Short-term Plan
1. **AE Exception Mapping**: Implement automatic conversion from effects to exceptions in non-resume scenarios.
2. **CLR FFI Validation**: Implement backend support for `@import(target: clr, ...)`, enabling successful calls to .NET BCL methods.
3. **Testing Framework**: Improve integration testing, supporting automatic execution of generated `.exe` and output comparison.
4. **Arrays and Slices**: Deep testing of array and slice operation edge cases.

### Long-term Plan
1. **Generic Support**: Implement Valkyrie generics using CLR native generics.
2. **Exceptions and Algebraic Effects**: Explore mapping between `try-catch` and Valkyrie effect system.
3. **Binary Generation**: Consider introducing a PE generator to eliminate dependency on `ilasm`.

# JVM Backend Maintenance Guide

The JVM backend is responsible for compiling Valkyrie to Java class file format.

## Compilation Pipeline

`Source -> AST -> HIR -> CFG -> JVM Bytecode`

## Design Considerations

### Stack Machine Architecture
JVM is a stack-based virtual machine. Using register-based LIR would introduce redundant `iload`/`istore` instructions. Therefore, the JVM backend skips the SSA and LIR stages and generates code directly from **CFG** (Control Flow Graph).

### Linearization
CFG basic blocks are linearized into a flat instruction stream. Jump instructions (`Goto`, `SwitchInt`) are handled by calculating relative offsets during a second pass (label fixup).

### Type System
Valkyrie types map to JVM descriptors:
- `i32` -> `I`
- `i64` -> `J`
- `f32` -> `F`
- `f64` -> `D`
- `string` -> `Ljava/lang/String;`
- `bool` -> `Z`
- `unit` -> `V`
- `class/unite` -> `Lpath/to/Class;`
- `Pointer` -> `J` (currently mapped to 64-bit integer)

### Expression Generation
- **Arithmetic Operations**: Directly mapped to `iadd`, `ladd`, `fadd`, `dadd`, and `irem`, `lrem`, `frem`, `drem`, etc.
- **Comparison Operations**:
    - For `i32`: Use `if_icmp<cond>` to jump and push 0 or 1.
    - For `i64/f32/f64`: Use `lcmp/fcmpl/dcmpl` instructions, followed by `if<cond>` instructions to generate boolean values.
    - For object types (class, unite, string): Use `if_acmp<cond>` to implement reference equality comparison.
- **Bitwise and Logical Operations**:
    - `And`, `Or`, `Xor` mapped to `iand/land`, `ior/lor`, `ixor/lxor`.
    - `Shl`, `Shr` mapped to `ishl/lshl`, `ishr/lshr`.
- **Unary Operations**:
    - `Neg` mapped to `ineg/lneg/fneg/dneg`.
    - `Not`: For `bool` and `i32`, use `iconst_1/iconst_m1` with `ixor`; for `i64`, use `-1L` from constant pool with `lxor`.

### Methods and Invocation
- Each Valkyrie function is emitted as a `static` method in a class.
- **Name Mangling**:
    - Global functions keep their original names.
    - Class/Unity methods: `ClassName$MethodName`.
    - Trait methods: `TraitName$MethodName`.
    - Impl methods: `[TraitName$]TargetName_MethodName`.
- Method calls (`Call`) currently use `invokestatic`. The backend automatically retrieves the callee's signature from `UIR` or `CfgProgram` to generate correct descriptors.
- For dynamic calls or function pointers, the backend uses `java/lang/invoke/MethodHandle`'s `invoke` method.

### Generic Support
- **Type Erasure**: All generic types are erased to `Ljava/lang/Object;` in descriptors.
- **Signature Attribute**: To preserve generic information, the backend generates JVM `Signature` attributes for generic functions and fields.
- **Function Types**: Valkyrie's function types map to `Ljava/lang/invoke/MethodHandle;`.

### Algebraic Effects
- Valkyrie's `raise` is treated as an Effect, governed by the AE mechanism.
- **Raise Implementation**: In non-resume scenarios, equivalent to Java exceptions, mapped to `athrow` instruction.
- **Handlers**: Implemented through JVM `Code` attribute's `exception_table` for `PushHandler`/`PopHandler`.
- When entering a Handler block, the backend automatically initializes `current_stack` to 1 to match JVM specification (Effect object will be pushed to stack top).
- **Resume**: Currently only non-resume paths are supported; full Continuation support is planned for future implementation through stack frame capture or bytecode rewriting.

### Control Flow Optimization
- **Jump Instructions**:
    - Default to `goto_w` (4-byte offset) to support jumps in super-large basic blocks.
    - `SwitchInt` automatically selects `tableswitch` or `lookupswitch` based on jump range and sparsity, both using 4-byte offsets.

## Current Progress

- [x] Basic Class file structure generation
- [x] Constant pool management (UTF8, Integer, Float, Long, Double, Class, String, FieldRef, MethodRef, NameAndType, MethodHandle, MethodType)
- [x] Basic type mapping (bool, i32, i64, f32, f64, string, unit)
- [x] Arithmetic operations (Add, Sub, Mul, Div, Rem) support for i32, i64, f32, f64
- [x] Bitwise and logical operations (And, Or, Xor, Shl, Shr)
- [x] Unary operations (Neg, Not)
- [x] Comparison operations (Eq, Ne, Lt, Le, Gt, Ge) support for i32, i64, f32, f64
- [x] Basic instantiation and field access for Class and Unity (limited to 1 level)
- [x] Basic array support (newarray, anewarray, iastore/iaload, etc.)
- [x] Static method calls (invokestatic)
- [x] Basic algebraic effects (Raise, Handlers)
- [x] Basic generic Signature support

## Roadmap

### Short-term
1. **Code Refactoring**: Refactor `emitter.rs` to reduce redundancy in bytecode emission logic.
2. **Debugging Support**: Implement `LineNumberTable` attribute for source code line number mapping.
3. **Field Access Enhancement**: Support multi-level depth field/index access.
4. **Method Call Completion**: Support `invokevirtual` and `invokeinterface`.

### Mid-term
1. **Closure Support**: Implement Lambda and closures using `invokedynamic`.
2. **Performance Optimization**: Implement smarter stack balance optimization to reduce unnecessary `dup` and `pop`.
3. **Reflection and Introspection**: Support Valkyrie runtime reflection.

### Long-term
1. **Incremental Compilation**: Support class file-based incremental compilation.
2. **Native Interoperability**: Optimize interaction performance with Java standard library.

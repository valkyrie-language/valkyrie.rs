# Lifetime & Memory Management

Valkyrie employs a layered memory management system designed to balance development experience (UX) with fine-grained low-level control. Valkyrie provides garbage collection (GC) by default, while also supporting explicit memory management for high-performance or embedded scenarios.

## Core Pillars

Valkyrie's memory management system is built on the following core concepts:

1. **[AIFD Lifecycle Model](lifecycle.md)**: Rigorously divides object lifecycle into four distinct phases: Allocate, Initiate, Finalize, and Delocate.
2. **[Scope and Static Analysis](scope.md)**: Explains how Valkyrie uses deterministic scopes and deep control flow analysis to achieve fully automatic, intelligent injection of lifecycle functions.
3. **[Reference Types (Class)](class.md)**: The default choice for application-level development, providing zero mental burden memory safety, powered by a high-performance garbage collection (GC) engine.
4. **[Value Types (Structure)](structure.md)**: Provides extreme memory control for performance-sensitive scenarios, supporting inline data storage.
5. **[Allocator](allocator.md)**: Provides fine-grained memory control for low-level development, allowing precise orchestration of physical allocation and deallocation in the AIFD model.
6. **[Foreign Objects](foreign-objects.md)**: Defines how to rigorously manage object lifecycles across language boundaries when interoperating with C/C++/Rust.

---

## Quick Navigation

- **Explore the complete journey of objects from birth to death?** See [AIFD Lifecycle](lifecycle.md).
- **Investigate how the compiler determines destruction timing?** See [Scope and Auto-insertion](scope.md).
- **Deeply understand how reference types work?** See [Reference Types (Class)](class.md).
- **Pursuing ultimate performance or doing low-level system development?** See [Value Types (Structure)](structure.md).
- **Need to customize memory allocation behavior?** See [Allocator](allocator.md).
- **Need seamless integration with existing C/Rust libraries?** See [Foreign Objects Interoperation](foreign-objects.md).

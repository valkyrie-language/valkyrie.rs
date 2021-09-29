# Reference Types (Class)

Valkyrie uses the most advanced garbage collection (GC) mechanism for memory management by default. For most application-level code, GC provides zero-burden memory safety, excellent development experience, and highly automated resource reclamation capabilities.

## Type Declaration: `class`

In Valkyrie, types defined with the `class` keyword are **reference types**. This means variables store references to instances on the heap, not the object data itself.

```valkyrie
class Person {
    name: utf8,
    age: i32,
}

# Instantiation: Default allocation on managed heap
let p1 = Person { name: "Valkyrie", age: 1 }
```

### Core Features

1. **Fully Automatic Lifecycle Management**: The complete lifecycle of objects is automatically tracked by a high-performance GC engine, allowing developers to fully focus on business logic without worrying about memory release.
2. **Default Reference Sharing Semantics**: When assigning or passing `class` instances, the system defaults to **`ref`** mode for sharing references (i.e., passing pointers). This design fundamentally avoids the "implicit shared mutability" pitfall common in traditional languages.
3. **Automatic Cycle Resolution**: GC can precisely identify and safely reclaim object clusters with complex cyclic reference relationships, ensuring the system maintains excellent stability when handling large-scale associated data.

```valkyrie
let p1 = Person { name: "Valkyrie", age: 1 }
let p2 = p1  # Automatically shares reference in ref mode; p1 and p2 point to the same physical memory instance
```

---

## Ownership Model: Sharing and Interior Mutability

Under the AIFD model, `class` defaults to **Shared Reference** semantics. To ensure memory safety under this sharing mode, Valkyrie introduces the following mechanisms:

### 1. Default Immutability and `ref`
When assigning or passing `class` instances, the default is `ref` mode. This means multiple variables can simultaneously point to the same instance, but they only have read-only access by default.

### 2. Reactive Containers (Signal)
For scenarios requiring cross-component state synchronization, Valkyrie advocates using **[Signal](../reactive-programming/signal.md)**. It encapsulates data in containers with observer pattern capabilities, managing state flow through explicit reactive bindings rather than hidden pointer operations.

### 3. Interior Mutability (Cell/Atom)
For specific scenarios, you can use `Cell` (single-thread safe) or `Atom` (cross-thread safe) to modify internal data while holding a read-only reference.

**Core Philosophy: Safe by default, explicit sharing.**

---

## GC and AIFD Model Co-evolution

For managed objects, the Valkyrie runtime automatically orchestrates AIFD lifecycle phases through a rigorous mechanism:

- **Allocate**: GC instantly allocates memory from the managed heap space or dedicated **Thread Local Allocation Buffer (TLAB)**.
- **Initiate**: Compiler automatically injects instructions to immediately execute the object's `initiate` method on successfully allocated memory.
- **Finalize**: When GC engine confirms through reachability analysis that an object is no longer used, it schedules and asynchronously executes its `finalize` cleanup logic.
- **Delocate**: After all finalization operations complete successfully, GC formally reclaims the corresponding physical memory block and marks it as available.

### Alloy: Perfect Interweaving of Ownership System and GC

Valkyrie's type system deeply integrates cutting-edge results from the **Alloy** research paper (*Garbage Collection for Rust: The Finalizer Frontier*), achieving seamless integration of static ownership checking and dynamic garbage collection.

1. **Static Finalization Analysis**: The compiler deeply analyzes `Finalize` implementation details. If it determines finalization logic produces no external side effects (pure memory operations), GC can employ parallelized or delayed execution strategies, significantly reducing system pause times (Stop-the-world).
2. **Ownership Integration Architecture**: GC is not a replacement for ownership, but a powerful supplement. Managed objects can safely hold references to manually managed objects, and vice versa. The compiler ensures absolute safety of such cross-boundary references through rigorous lifecycle contracts.
3. **Deterministic Resource Reclamation**: Although memory reclamation is asynchronous, for critical system resources implementing `Finalize` (like network Sockets, file handles), Valkyrie triggers their finalization logic with highest priority after objects become invalid.

---

## Reference Types vs Value Types Deep Comparison

| Core Dimension | class (Reference Type) | structure (Value Type) |
| :--- | :--- | :--- |
| **Memory Physical Layout** | Default allocation on managed heap, accessed via pointer | Default allocation on stack or inline storage in parent structure |
| **Default Assignment Semantics** | **Reference Sharing (Shared)** | **Ownership Move** |
| **Clone/Copy Behavior** | No explicit copy needed, share same instance | Must explicitly call `.copy()` or `.clone()` |
| **Lifecycle Driver** | Automatically tracked and reclaimed by background GC engine | Follows strict lexical scope rules (RAII) |
| **Typical Use Cases** | Business entities with unique identity, complex state machines | Lightweight data containers, math vectors, performance-sensitive components |

---

## Low-level Physical Optimizations for Reference Types

Although `class` provides a highly abstract programming model, the Valkyrie compiler employs a series of advanced optimization techniques to minimize runtime overhead:

### 1. Escape Analysis
The compiler continuously tracks object lifecycles. If it can statically prove that a `class` instance doesn't escape the current function, it will **automatically optimize it from heap allocation to stack allocation**, or even directly decompose and map to CPU registers, completely eliminating GC pressure.

### 2. Deep Devirtualization
Using context awareness and type flow analysis, the compiler can rewrite originally expensive dynamic dispatch (virtual function lookup) into direct function calls. This not only eliminates jump overhead but also opens huge space for subsequent **Function Inlining**.

### 3. Pointer Compression
In 64-bit environments, if heap memory requirements are within 32GB, the compiler will use 32-bit compressed pointers instead of 64-bit absolute addresses. This significantly reduces object size in memory and doubles CPU cache (L1/L2 Cache) utilization.

### 4. Thread Local Allocation (TLAB)
Each execution thread has a private fast allocation buffer. When instantiating objects, only simple pointer bumping is needed, without any global lock contention, with speed almost equivalent to raw stack operations.

### 5. Static Finalization Shortcut
If the `finalize` method is determined to be "trivial finalization" (no external side effects), GC will directly skip the finalization queue during scan reclamation, greatly shortening garbage collection cycles.

### 6. Write Barrier Elimination
In generational GC architecture, the compiler can intelligently identify and remove redundant write barrier instructions (e.g., initial property assignments to newly created objects), further reducing the cost of maintaining object reference relationships.

---

## Next Steps

Reference types provide us with an excellent development experience. However, in pursuit of ultimate performance or hardware-level control scenarios, we need to leverage **[Value Types (Structure)](structure.md)** to achieve more precise memory layout and ownership management.

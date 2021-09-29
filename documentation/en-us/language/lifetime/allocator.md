# Allocator

`Allocator` is the core abstraction of Valkyrie's manual memory management mechanism, specifically responsible for the **A (Allocate)** and **D (Delocate)** phases in the AIFD model.

## Allocator Interface Definition

The `Allocator` interface directly abstracts low-level memory allocation and reclamation behaviors, providing developers with fine-grained control over memory layout.

```valkyrie
trait Allocator {
    # A: Allocate memory
    # Request raw memory space according to the specified layout.
    # Returns: raw pointer to the start of allocated memory, or None if allocation fails.
    micro allocate(self, layout: Layout) -> Option⟨◆u8⟩

    # D: Return memory
    # Release and return previously allocated memory space to the allocator.
    # Parameter: ptr must be a pointer previously returned by allocate, and layout must match.
    micro delocate(self, ptr: ◆u8, layout: Layout)
}
```

### Runtime Implementation: Fat Pointer with VTable

In the Valkyrie runtime, when the `Allocator` interface is passed as a parameter or variable, it is typically implemented as a **Fat Pointer**. This structure consists of two parts:
1. **Data Pointer**: Points to the allocator instance's private state data.
2. **Virtual Table (vtable)**: Contains the concrete implementation addresses of `allocate` and `delocate`.

This design ensures that even in complex dynamic dispatch scenarios, memory operation overhead remains extremely low and deterministic.

## Explicit Resource Orchestration (RAII)

In high-performance scenarios where garbage collection (GC) uncertainty overhead needs to be avoided, developers can use explicit containers like `Scoped` or `Box`, combined with custom allocators to manually orchestrate the complete AIFD lifecycle of objects.

### Core Application: Arena Allocator (ArenaAllocator)

`ArenaAllocator` is well-suited for batch processing tasks with highly consistent lifecycles. It allows continuous fast memory allocation during task execution and releases the entire memory pool in a single operation when the task completes, greatly improving throughput.

```valkyrie
micro heavy_task(arena: ArenaAllocator) {
    # Allocate memory in specified arena and initialize object in place (triggers A and I phases)
    let buffer = Scoped::new_in⟨BigData⟩(args, arena)
    
    # ... Execute intensive business computation ...
    
}
# Automatically triggers destruction when leaving scope:
# 1. Automatically calls object's finalization logic (F)
# 2. Returns physical memory uniformly to the system through arena interface (D)
```

## Best Practice Guidelines

1. **Pass Over Hold**: Try to avoid storing allocator references in long-lived structures. Prefer passing allocators as context parameters in methods that perform memory operations.
2. **Design for Interfaces**: When writing generic high-performance libraries, accept the generic `Allocator` interface. This allows library users to freely inject the most appropriate memory allocation strategy based on their actual environment (e.g., embedded, high-performance servers).
3. **Alignment and Layout**: When customizing allocation logic, strictly follow `Layout` alignment requirements to avoid performance degradation or illegal access on certain hardware architectures.

---

## Next Steps

Having mastered allocators, you now possess the core capability to build high-performance systems. Finally, let's look at how to handle **[Foreign Objects](foreign-objects.md)** that are not directly managed by Valkyrie, completing the final piece of cross-language memory management.

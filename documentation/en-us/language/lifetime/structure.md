# Value Types (Structure)

`structure` is the cornerstone of building high-performance systems in Valkyrie, representing rigorous **Value Semantics**. It uses inline storage in memory and follows strict move semantics, making it core to achieving deterministic resource management and hardware-level performance optimization.

## Memory Layout: Inline and Locality

`structure` is allocated on the stack by default, with its biggest physical characteristic being **Inline** storage. When a structure is a field of another structure, it's directly embedded in the parent structure's memory layout, completely eliminating additional pointer indirection overhead.

```valkyrie
structure Vec2 { x: f32, y: f32 }

structure Rect {
    origin: Vec2, # Directly inline in Rect memory block
    size: Vec2,   # Directly inline in Rect memory block
}
```

This layout scheme is extremely CPU cache (Cache) friendly. By ensuring data **spatial locality**, the compiler can significantly improve cache hit rates, extracting ultimate hardware performance.

---

## Ownership and Ownership Transfer

`structure`'s core runtime logic is **Move Semantics**. Different ownership qualifiers determine how underlying data flows:

### 1. Default and `own`: Complete Ownership Transfer

When you assign a `structure` to a new variable or pass it in `own` mode, the underlying data undergoes **Physical Bitwise Copy**, while the original variable is immediately logically marked invalid (inaccessible).

#### Necessity of Physical Copy
Although logically only one owner exists at a time, in physical implementation, `structure`'s location is fixed (at a specific stack slot). When ownership transfers to a new variable or crosses stack frames, data must be moved to a new physical address to maintain inline characteristics.

- **Logical Semantics**: Ownership transfer (Move), original variable lifecycle ends.
- **Physical Effect**: Executes efficient `memcpy` to move data to new Slot.

```valkyrie
let v1 = Vec2 { x: 1.0, y: 1.0 }
let v2 = v1 # Logical move, physical transfer; v1 no longer usable after this
```

### 2. `ref` and `mut`: Zero-copy Borrowing
Borrowing operations don't touch ownership. In underlying implementation, the compiler only passes the structure's physical memory address (pointer).

```valkyrie
# Only passes address, no data movement involved
micro distance(ref a: Vec2, ref b: Vec2) -> f32 { ... }
```

### 3. Explicit Copy: `.copy()` and `.clone()`
If you need to preserve the original object and produce an independent copy, you must express this through explicit calls:
- **`.copy()`**: Executes simple bitwise copy. Only applicable to "trivial" structures not holding complex external resources (like raw pointers, handles).
- **`.clone()`**: Triggers user-defined deep copy logic, for handling resource replication with complex ownership relationships.

---

## AIFD Orchestration: Custom Behavior

Developers can precisely intervene in `structure`'s critical phases by implementing specific lifecycle functions:

- **`initiate(mut self)`**: Triggered immediately after A phase (allocation). Commonly used to establish initial invariants or validate state.
- **`finalize(mut self)`**: Triggered before D phase (deallocation). For `structure`, this is **physically deterministic** (i.e., executes immediately when leaving scope).

```valkyrie
structure TempFile {
    path: utf8,
    
    # Use RAII to ensure deterministic resource cleanup
    finalize(mut self) {
        # When leaving scope, ensure physical file is removed from disk
        std::fs::remove(self.path)
    }
}
```

---

## Hardware-level Physical Optimizations (Black Magic)

To achieve ultimate runtime efficiency, the Valkyrie compiler applies multiple deep physical-level optimizations to `structure` while strictly maintaining logical semantics:

### 1. Move Elision
Although `move` logically involves copying, the compiler eliminates 90%+ of actual overhead through these techniques:
- **In-place Construction**: Through **NRVO (Named Return Value Optimization)**, the compiler directly generates objects at the target variable's physical location, completely skipping temporary variable creation and movement.
- **Register Mapping**: For small structures (like `Vec2`), they're typically allocated directly to CPU registers. At this point "move" is just logical renaming, producing no memory I/O.
- **Tail Call Forwarding**: When a variable is passed to a downstream function as its last use, the compiler attempts to let the new function reuse the current memory Slot.

### 2. Layout Reordering
To align memory and reduce size, the compiler automatically optimizes field physical order:
- **Eliminate Gaps (Padding)**: By arranging fields in descending size order, squeezes out padding space generated for memory alignment.
- **Space Compression**: A non-aligned structure originally occupying 24 bytes might only need 16 bytes after optimization, significantly improving cache line utilization.

### 3. Scalar Replacement (SROA)
If a structure is only used locally and doesn't escape, the compiler "decomposes" it into independent scalar variables:
- **Registerization**: Structure fields are separately mapped to different CPU registers, completely eliminating memory access.
- **Decoupling**: Once decomposed, traditional optimizations like dead code elimination can target individual fields.

### 4. Zero-Size Types (ZST)
`structure` without any fields occupies no physical space at runtime (0 Bytes).
- **Zero-cost Abstraction**: You can define massive ZSTs to strengthen type safety or express state machine logic without any performance burden on generated machine code.

### 5. Transparent Wrapping
Single-field wrapper classes (like `UserId(u64)`) are completely equivalent to underlying types at ABI level.
- **Seamless Wrapping**: Directly maps to primitive types in FFI calls, achieving zero-overhead semantic strengthening.

### 6. Niche Optimization
The compiler uses illegal bit patterns in fields (like non-null pointer's 0 value) to store `Option` tags.
- **Memory Burden Reduction**: Makes `Option<ref T>` size completely consistent with `ref T`, achieving zero-space-loss error handling.

### 7. SIMD Auto-vectorization
For numerically aligned structures (like `Vec4`), the compiler uses SSE/AVX instruction sets for single-instruction multiple-data parallelism:
- **Throughput Surge**: Processes 4 or more floating-point operations at once, particularly suitable for graphics, physics engines, and high-performance scientific computing scenarios.

---

## Best Practices: Core Decision Recommendations

- **Default to `structure`**: Suitable for math vectors, configuration snapshots, small state machines, and all lightweight data models pursuing ultimate performance.
- **Embrace Inlining**: Reduce unnecessary heap allocation by properly composing `structure`, building cache-friendly data structures.
- **Scrutinize Move Costs**: For huge structures (like those containing large fixed arrays), frequent `own` passing may bring physical copy overhead; prefer `ref` borrowing in such cases.

---

## Next Steps

Value types give us ultimate control over memory layout. To go further, we can manually take over physical allocation and deallocation processes in the AIFD model through the **[Allocator](allocator.md)** interface.

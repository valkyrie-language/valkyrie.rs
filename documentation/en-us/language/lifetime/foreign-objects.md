# Foreign Objects

When Valkyrie deeply interacts (FFI) with external languages like C, C++, or Rust, rigorous and precise management of foreign object lifecycles is key to ensuring system stability, preventing memory leaks, and maintaining high-performance execution.

## FFI and AIFD Model Deep Mapping

Since foreign objects are typically managed by their host language (like C heap or Rust allocator) and not directly controlled by Valkyrie's garbage collection (GC) mechanism, developers must manually map their lifecycles explicitly into the AIFD model.

Particularly important is the mapping of the **F (Finalize)** phase. This ensures that when the Valkyrie-side wrapper object becomes invalid, the underlying foreign resources (like raw pointers, system handles, network sockets, etc.) can be deterministically released.

### Practical Example: Safely Wrapping C++ Objects

```valkyrie
# Import external C++ native interfaces
@import(c, "lib", "obj_new")
micro obj_new() -> ◆u8

@import(c, "lib", "obj_delete")
micro obj_delete(ptr: ◆u8)

# Valkyrie safe wrapper class
class ExternalWrapper {
    ptr: ◆u8
    
    # State initialization (I): Request external resource at construction
    initiate(mut self) {
        unsafe { 
            self.ptr = obj_new() 
        }
    }
    
    # Logic finalization (F): Ensure external resource is safely released before object destruction
    finalize(mut self) {
        if self.ptr != null {
            unsafe { 
                obj_delete(self.ptr) 
                self.ptr = null # Defensive programming: prevent double free
            }
        }
    }
}
```

## Safe Wrapper Patterns (Safe Guards)

To provide better development experience (UX) and reduce error risk, it's recommended to use the following patterns to leverage `Finalize` for automatic foreign resource management:

1. **Object Handle Pattern**: Wrap external raw pointers in Valkyrie's `class`. This way, the foreign resource's existence is tightly bound to the Valkyrie object's lifecycle, with cleanup logic indirectly driven by GC.
2. **RAII Guardians**: Automatically trigger external language destructors or resource reclamation routines through the `finalize` method, achieving deterministic, automated cleanup similar to Rust RAII.

## Core Memory Safety Guidelines

- **Null-Safety**: In `finalize` implementations, always validate the validity of external pointers. This not only prevents double free but also enhances program robustness in exceptional situations.
- **Memory Ownership and Pinning**: If you need to pass Valkyrie-managed memory addresses (like array buffers) to external languages for asynchronous use, you must ensure the memory won't be moved by GC during the external call. Use `Pin` mechanism to lock object location.

---

## Low-level Physical Optimizations for Foreign Objects

To achieve native-level execution efficiency for cross-language calls, the Valkyrie compiler performs deep targeted optimizations on FFI interaction paths:

### 1. Zero-copy Data Exchange
Valkyrie's `structure` types are strictly compatible with standard C in memory layout.
- **Pointer Pass-through**: When passing complex structures, the compiler directly passes raw memory pointers, completely eliminating expensive data copying.
- **Constant Overhead**: Even when handling giant vertex buffers containing millions of elements, cross-language passing performance overhead remains at $O(1)$ level.

### 2. Fast-path FFI
The compiler precisely orchestrates call sequences according to the target platform's ABI (like System V or Windows x64).
- **Register Direct Pass**: Lightweight foreign objects (like resource handles, short vectors) are passed directly through CPU registers, completely bypassing slow stack memory.
- **Built-in Instruction Replacement**: For standard library hotspots like math operations (e.g., `sin`, `sqrt`), the compiler attempts to directly replace them with equivalent hardware CPU instructions.

### 3. Finalizer Batching
Releasing foreign objects typically involves expensive cross-language context switching costs.
- **Delayed Coalescing Strategy**: GC intelligently aggregates multiple pending foreign finalization logic and triggers them in batches during a single scan cycle.
- **Pipeline Optimization**: This approach greatly reduces pipeline flush losses from frequent CPU switching between Valkyrie runtime and external runtime.

### 4. Wrapper Stripping and Inlining
- **Locality Optimization**: If a foreign wrapper instance doesn't escape the current function scope, the compiler attempts to strip the wrapper layer, directly operate on raw pointers, and inline all cleanup code.
- **Stack Promotion**: The `class` instance wrapping the pointer itself may be optimized to stack allocation, completely eliminating heap allocation overhead.

### 5. Cross-language Whole Program Optimization (LTO)
When external libraries are integrated via static linking, Valkyrie supports enabling cross-language whole program analysis:
- **Boundary Inlining**: The compiler can break language barriers and directly inline hot code from external libraries into Valkyrie call sites.
- **Dead Code Elimination (DCE)**: Precisely identify and remove uncalled redundant logic from external libraries during link stage, greatly compressing binary size.

---

## More References

- **Return to Overview**: Back to [Lifetime & Memory Management](index.md) home page.
- **Advanced Topic**: Deeply understand Valkyrie's [FFI Detailed Guide](../module-system/foreign-function.md).
